# HA on Kubernetes — End-to-End Runbook

This is the step-by-step playbook for deploying Vectorizer in High-Availability
mode (Raft consensus + master/replica replication) on a Kubernetes cluster.
It covers the exact configuration that was validated end-to-end on a 3-node
k3s cluster against `ghcr.io/hivellm/vectorizer:3.0.11`, including the gotchas
that cost the v3.0.0 → v3.0.10 release line three iterations to land.

The neighbouring docs (`CLUSTER.md`, `KUBERNETES.md`, `users/configuration/CLUSTER.md`)
cover the architecture and reference configuration. This file is the
operational sequence: **start with an empty namespace, end with a Raft
cluster that survives leader kills and rolling restarts.**

> **Minimum version:** `3.0.11`. Earlier 3.0.x releases hit one of three
> bugs that make HA mode unstable in Kubernetes (split-init, address
> resolution, election sabotage — see [What changed in 3.0.11](#what-changed-in-3011)).

---

## 1. Prerequisites

| Requirement | Notes |
|---|---|
| Kubernetes 1.20+ | Tested on k3s 1.34.5 and standard kubeadm clusters |
| `kubectl` configured for the target cluster | `kubectl get nodes` must succeed |
| A `StorageClass` that supports `ReadWriteOnce` | k3s `local-path`, AWS `gp3`, GKE `standard-rwo`, etc. |
| Pull access to `ghcr.io/hivellm/vectorizer` | If private, create an `imagePullSecret` (Step 2) |
| 3 pods worth of capacity | Each pod requests 1 vCPU / 1 Gi RAM minimum (4 Gi / 100 Gi recommended) |

A 3-node Raft cluster needs **majority quorum (2 of 3)** to commit writes.
Two healthy pods means writes succeed. One healthy pod means the cluster
is read-only until quorum returns.

---

## 2. Namespace and image-pull secret

```bash
NS=vectorizer-ha
kubectl create ns "$NS"
```

If `ghcr.io/hivellm/vectorizer` is private for your account (it is for the
HiveLLM org), create a pull secret. Replace `GH_USER` and `GH_PAT` with a
GitHub username and a Personal Access Token that has `read:packages`:

```bash
kubectl create secret docker-registry ghcr-credentials \
  --namespace "$NS" \
  --docker-server=ghcr.io \
  --docker-username='GH_USER' \
  --docker-password='GH_PAT'
```

The StatefulSet in Step 5 references this secret name; if you call it
something else, update `imagePullSecrets[0].name` accordingly.

---

## 3. Application secret (JWT + admin credentials)

The v3 server **refuses to bind to `0.0.0.0` without authentication enabled**
— this is a hard guard inside the bootstrap path, not a config warning. The
HA manifests bind to `0.0.0.0` because pods need to be reachable from
sibling pods, so a JWT secret and an admin user/password are mandatory.

```bash
JWT="$(openssl rand -hex 64)"            # 128-char hex secret
ADM_PASS="$(openssl rand -base64 24 | tr -d '=+/')"

kubectl create secret generic vectorizer-credentials \
  --namespace "$NS" \
  --from-literal=VECTORIZER_USERNAME='admin' \
  --from-literal=VECTORIZER_PASSWORD="$ADM_PASS" \
  --from-literal=VECTORIZER_API_KEY="$JWT"
```

> **Save `$ADM_PASS` somewhere safe** — it's the only credential that lets
> you reach gated routes (`/collections`, `/insert_texts`, etc.) while you
> set up dashboards, API keys, or HiveHub.

---

## 4. Headless Service

The Raft transport (`gRPC` on port 15003) and replication transport (TCP on
port 7001) both rely on per-pod DNS names — `<pod-name>.<service-name>.<ns>.svc.cluster.local`.
That only works behind a *headless* `Service` (`clusterIP: None`).

```yaml
apiVersion: v1
kind: Service
metadata:
  name: vectorizer-headless
  namespace: vectorizer-ha
  labels:
    app: vectorizer
spec:
  clusterIP: None                # headless — required for per-pod DNS
  publishNotReadyAddresses: true # peers must resolve before /health passes
  selector:
    app: vectorizer
  ports:
    - { name: rpc,         port: 15503, targetPort: 15503 }
    - { name: http,        port: 15002, targetPort: 15002 }
    - { name: grpc,        port: 15003, targetPort: 15003 }
    - { name: replication, port: 7001,  targetPort: 7001  }
```

`publishNotReadyAddresses: true` is **load-bearing**: pods need to reach
their peers' gRPC endpoint to elect a leader, which they do *before* their
own readiness probe passes. Without this flag, the headless DNS only
returns Ready pods, and the cluster deadlocks at startup.

---

## 5. ConfigMap (Raft + auth template)

The ConfigMap is a **template**: the placeholder `__NODE_ID__` gets replaced
with each pod's hostname by an init container in the StatefulSet (Step 6).
The `cluster.servers` list **must contain entries whose `id` matches the
pod hostname literally** — `vectorizer-0` here means a pod named exactly
`vectorizer-0`.

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: vectorizer-ha-config
  namespace: vectorizer-ha
data:
  config-template.yml: |
    server:
      host: "0.0.0.0"
      port: 15002
      mcp_port: 15002

    file_watcher:
      enabled: false      # MUST be false in cluster mode

    logging:
      level: "info"
      format: "json"
      log_requests: false
      log_responses: false
      log_errors: true

    auth:
      enabled: true
      # jwt_secret is overridden by VECTORIZER_JWT_SECRET env var (Step 6).
      jwt_secret: "placeholder-overridden-by-env-VECTORIZER_JWT_SECRET"
      jwt_expiration: 3600
      api_key_length: 32
      rate_limit_per_minute: 1000
      rate_limit_per_hour: 100000

    cluster:
      enabled: true
      # __NODE_ID__ is replaced per-pod by the init container with $HOSTNAME.
      node_id: "__NODE_ID__"
      discovery: "dns"
      dns_name: "vectorizer-headless.vectorizer-ha.svc.cluster.local"
      dns_resolve_interval: 30
      dns_grpc_port: 15003
      timeout_ms: 5000
      retry_count: 3
      # Each `id` MUST match a pod hostname literally. The
      # alphabetically-first id is the bootstrap node — that's the only pod
      # that calls openraft `initialize_cluster`. The others wait for the
      # bootstrap pod to propagate the membership log entry. Misaligning
      # these ids with the pod hostnames is the #1 cause of "No leader
      # elected" in production.
      servers:
        - id: "vectorizer-0"
          address: "vectorizer-0.vectorizer-headless.vectorizer-ha.svc.cluster.local"
          grpc_port: 15003
        - id: "vectorizer-1"
          address: "vectorizer-1.vectorizer-headless.vectorizer-ha.svc.cluster.local"
          grpc_port: 15003
        - id: "vectorizer-2"
          address: "vectorizer-2.vectorizer-headless.vectorizer-ha.svc.cluster.local"
          grpc_port: 15003
      memory:
        max_cache_memory_bytes: 1073741824   # 1 GiB
        enforce_mmap_storage: true
        disable_file_watcher: true
        cache_warning_threshold: 80
        strict_validation: true

    replication:
      enabled: true
      # role is set automatically by HaManager based on Raft leadership.
      bind_address: "0.0.0.0:7001"
      heartbeat_interval_secs: 5
      replica_timeout_secs: 30
      log_size: 1000000
      reconnect_interval_secs: 5

    api:
      grpc:
        enabled: true
        port: 15003
```

---

## 6. StatefulSet

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: vectorizer
  namespace: vectorizer-ha
  labels:
    app: vectorizer
spec:
  serviceName: vectorizer-headless
  replicas: 3
  podManagementPolicy: Parallel    # all pods come up together; Raft tolerates this
  selector:
    matchLabels:
      app: vectorizer
  template:
    metadata:
      labels:
        app: vectorizer
    spec:
      imagePullSecrets:
        - name: ghcr-credentials
      initContainers:
        # Materialise the per-pod config from the template ConfigMap.
        # `sed` substitutes the literal placeholder `__NODE_ID__` with
        # the pod's hostname (provided by the StatefulSet ordinal).
        - name: config-selector
          image: busybox:1.36
          command: ["sh", "-c"]
          args:
            - |
              echo "Setting node_id to $HOSTNAME"
              sed "s/__NODE_ID__/$HOSTNAME/g" /configs/config-template.yml > /active-config/config.yml
              cat /active-config/config.yml
          volumeMounts:
            - { name: config-templates, mountPath: /configs }
            - { name: active-config,    mountPath: /active-config }
      containers:
        - name: vectorizer
          # Pin to an exact tag — `:latest` floats and breaks rollouts.
          # GHCR tags are *unprefixed*: use `3.0.11`, NOT `v3.0.11`.
          image: ghcr.io/hivellm/vectorizer:3.0.11
          imagePullPolicy: IfNotPresent
          ports:
            - { name: rpc,         containerPort: 15503 }
            - { name: http,        containerPort: 15002 }
            - { name: grpc,        containerPort: 15003 }
            - { name: replication, containerPort: 7001  }
          env:
            - name: HOSTNAME
              valueFrom:
                fieldRef: { fieldPath: metadata.name }
            - name: POD_IP
              valueFrom:
                fieldRef: { fieldPath: status.podIP }
            - name: VECTORIZER_SERVICE_NAME
              value: "vectorizer-headless.vectorizer-ha.svc.cluster.local"
            # Pin the data dir to the PVC mount; otherwise the server
            # falls back to `~/.local/share/vectorizer/` which is
            # *outside* the PVC and survives nothing. See "Data
            # directory pitfall" further down.
            - name: VECTORIZER_DATA_DIR
              value: "/data/data"
            - name: VECTORIZER_AUTH_ENABLED
              value: "true"
            - name: VECTORIZER_ADMIN_USERNAME
              valueFrom:
                secretKeyRef: { name: vectorizer-credentials, key: VECTORIZER_USERNAME }
            - name: VECTORIZER_ADMIN_PASSWORD
              valueFrom:
                secretKeyRef: { name: vectorizer-credentials, key: VECTORIZER_PASSWORD }
            - name: VECTORIZER_JWT_SECRET
              valueFrom:
                secretKeyRef: { name: vectorizer-credentials, key: VECTORIZER_API_KEY }
          volumeMounts:
            - { name: data,          mountPath: /data }
            - { name: active-config, mountPath: /vectorizer/config.yml, subPath: config.yml }
          resources:
            requests: { cpu: "1",   memory: "1Gi" }
            limits:   { cpu: "4",   memory: "4Gi" }
          livenessProbe:
            httpGet: { path: /health, port: http }
            initialDelaySeconds: 30
            periodSeconds: 10
            failureThreshold: 3
          readinessProbe:
            httpGet: { path: /health, port: http }
            initialDelaySeconds: 10
            periodSeconds: 5
            failureThreshold: 3
      volumes:
        - name: config-templates
          configMap: { name: vectorizer-ha-config }
        - name: active-config
          emptyDir: {}
  volumeClaimTemplates:
    - metadata: { name: data, labels: { app: vectorizer } }
      spec:
        accessModes: ["ReadWriteOnce"]
        resources:
          requests: { storage: 20Gi }
```

`/health` is anonymous on purpose — auth-protected probes would deadlock the
readiness gate against the JWT issuer. `/api/status` exists too but
requires a token, so it's not appropriate as a probe path.

Apply everything in order:

```bash
kubectl apply -f service-headless.yaml
kubectl apply -f configmap-ha.yaml
kubectl apply -f statefulset.yaml
```

---

## 7. Validate the cluster came up

A successful first boot looks like this in the pod logs:

```bash
# vectorizer-0 (the bootstrap pod)
🗳️  Calling initialize_cluster with 3 members (this node is the bootstrap node)
✅ Raft cluster initialized successfully
🔭 Raft watcher started — monitoring leadership changes
👑 This node became LEADER — starting MasterNode
This node is now the LEADER (id=...)

# vectorizer-1 / vectorizer-2 (followers)
⏸️  Skipping initialize_cluster — waiting for bootstrap node to propagate membership
🔭 Raft watcher started
📡 Following new leader   leader_addr=vectorizer-0.vectorizer-headless...
This node is now FOLLOWER
ReplicaNode started (connecting to leader at vectorizer-0...:7001)
```

**A leader should be elected within ~10 seconds of the third pod becoming
ready.** If you see `No leader elected — node entering Candidate state`
repeating for more than 30 seconds across all three pods, jump to
[Troubleshooting](#troubleshooting).

Quick checks:

```bash
NS=vectorizer-ha

# All three pods Ready
kubectl get pods -n "$NS" -l app=vectorizer

# Quick "who is leader?"
for p in vectorizer-0 vectorizer-1 vectorizer-2; do
  role=$(kubectl logs "$p" -n "$NS" 2>&1 \
    | grep -E 'This node became LEADER|This node is now FOLLOWER' \
    | tail -1 | grep -oE 'LEADER|FOLLOWER')
  echo "$p: ${role:-UNKNOWN}"
done
```

You should see exactly one `LEADER` and two `FOLLOWER`.

---

## 8. Functional smoke test (replication)

```bash
NS=vectorizer-ha
ADM_PASS=$(kubectl get secret vectorizer-credentials -n "$NS" \
  -o jsonpath='{.data.VECTORIZER_PASSWORD}' | base64 -d)

# Port-forward a follower so we can exercise the API from your laptop.
# (The HTTP API works on every pod, not just the leader — followers
#  serve reads locally and forward writes to the leader.)
kubectl port-forward -n "$NS" pod/vectorizer-1 18002:15002 &
PF=$!
sleep 3

TOKEN=$(curl -sS -X POST http://127.0.0.1:18002/auth/login \
  -H 'content-type: application/json' \
  -d "{\"username\":\"admin\",\"password\":\"$ADM_PASS\"}" \
  | jq -r .access_token)

# Create a collection and insert 50 vectors via text.
# Dimension MUST match the embedder's output (default BM25 = 512 dim).
curl -sS -X POST http://127.0.0.1:18002/collections \
  -H "Authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"name":"smoke","dimension":512}'

curl -sS -X POST http://127.0.0.1:18002/insert_texts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'content-type: application/json' \
  -d '{"collection":"smoke","texts":[
        {"id":"a","text":"replication smoke test 1","metadata":{}},
        {"id":"b","text":"replication smoke test 2","metadata":{}}
       ]}'

kill "$PF" 2>/dev/null

# Confirm the count is identical on every pod.
kubectl run dbg --image=alpine:3.20 -n "$NS" --restart=Never --rm -i --command -- sh -c "
  apk add --no-cache curl jq >/dev/null 2>&1
  for h in vectorizer-0 vectorizer-1 vectorizer-2; do
    vc=\$(curl -sS -H 'Authorization: Bearer $TOKEN' \
      http://\$h.vectorizer-headless.$NS.svc.cluster.local:15002/collections/smoke \
      | jq -r .vector_count)
    echo \"\$h: vector_count=\$vc\"
  done
"
```

Expected: `vector_count=2` on all three pods within seconds of the insert.

---

## 9. Failover smoke test

```bash
NS=vectorizer-ha
LEADER=$(for p in vectorizer-0 vectorizer-1 vectorizer-2; do
  role=$(kubectl logs "$p" -n "$NS" 2>&1 \
    | grep -E 'This node (became|is now the) LEADER|This node is now FOLLOWER' \
    | tail -1 | grep -oE 'LEADER|FOLLOWER')
  [ "$role" = "LEADER" ] && { echo "$p"; break; }
done)
echo "current leader = $LEADER"

# Kill the leader.
kubectl delete pod "$LEADER" -n "$NS" --grace-period=2

# Watch for the new leader. Should appear within ~10s.
kubectl get pods -n "$NS" -l app=vectorizer -w
# (Ctrl-C once the killed pod is back to 1/1 Running.)

# Confirm the cluster is consistent again.
for p in vectorizer-0 vectorizer-1 vectorizer-2; do
  role=$(kubectl logs "$p" -n "$NS" --tail=200 2>&1 \
    | grep -E 'This node (became|is now the) LEADER|This node is now FOLLOWER' \
    | tail -1 | grep -oE 'LEADER|FOLLOWER')
  echo "$p: $role"
done
```

End state should again be exactly one `LEADER` and two `FOLLOWER`, with the
killed pod returning as a follower.

For a heavier validation (writes during the failover, count integrity,
rolling restart), see the load-test script in
[`docs/runbooks/PROCEDURES.md`](../runbooks/PROCEDURES.md#ha-load-test).

---

## Data directory pitfall (must read before deploying)

`vectorizer-core::paths::data_dir()` — the function the server uses to
locate `vectorizer.vecdb`, snapshots, and the auth files — resolves in
this order:

1. `$VECTORIZER_DATA_DIR` if set and non-empty.
2. `dirs::data_dir().join("vectorizer")` — per-OS user data directory
   (`~/.local/share/vectorizer/` on Linux, `~/Library/Application
   Support/vectorizer/` on macOS).
3. `./data` — relative to the current working directory.

A standard StatefulSet template mounts the PVC at `/data` and the
server runs out of `/vectorizer/`. Without the env override, the
default Linux path wins — `~/.local/share/vectorizer/` — which is
**inside the container's writable layer, not on the PVC**. Every
restart looks like a fresh first-time-setup even when
`vectorizer.vecdb` is sitting unread on the PVC at `/data/data/`.

The Step 6 manifest above sets `VECTORIZER_DATA_DIR=/data/data`
explicitly for this reason. Deploys built off the old `k8s/statefulset-ha.yaml`
(prior to the v3.0.13 release) silently dropped this env var and lost
visibility into pre-existing data — the data was still on disk, just
read from the wrong path. If you suspect this happened to a running
deployment, see [Recovering data lost to the data-dir trap](#recovering-data-lost-to-the-data-dir-trap)
below.

### Recovering data lost to the data-dir trap

If the API reports zero collections but `kubectl exec <pod> -- ls /data/data`
shows a `vectorizer.vecdb` file:

```bash
NS=...           # whatever your namespace is
STS=...          # StatefulSet name (e.g. ermes-vectorizer)

# 1. Add the env var.
kubectl set env sts/$STS -n $NS VECTORIZER_DATA_DIR=/data/data

# 2. The auth.enc on the PVC was written by an older deploy with a
#    different VECTORIZER_JWT_SECRET / VECTORIZER_ADMIN_PASSWORD, so
#    the current secret won't be able to log in. Back it up and let
#    the server recreate it from the current env on next boot.
for p in ${STS}-0 ${STS}-1 ${STS}-2; do
  kubectl exec "$p" -n "$NS" -- sh -c '
    cp /data/data/auth.enc /data/data/auth.enc.bak-$(date +%Y-%m-%d)
    cp /data/data/.auth.key /data/data/.auth.key.bak-$(date +%Y-%m-%d)
    rm /data/data/auth.enc /data/data/.auth.key
  '
done

# 3. Rolling restart: vectorizer.vecdb gets loaded; auth files get
#    recreated from VECTORIZER_ADMIN_PASSWORD + VECTORIZER_JWT_SECRET.
kubectl rollout restart sts/$STS -n $NS
kubectl rollout status sts/$STS -n $NS

# 4. Validate. The collection list should match what was on the PVC.
ADM_PASS=$(kubectl get secret <your-credentials-secret> -n $NS \
  -o jsonpath='{.data.VECTORIZER_PASSWORD}' | base64 -d)
kubectl port-forward -n $NS pod/${STS}-0 18002:15002 &
TOKEN=$(curl -sS -X POST http://127.0.0.1:18002/auth/login \
  -H 'content-type: application/json' \
  -d "{\"username\":\"admin\",\"password\":\"$ADM_PASS\"}" | jq -r .access_token)
curl -sS -H "Authorization: Bearer $TOKEN" http://127.0.0.1:18002/collections \
  | jq '{total: (.collections | length), total_vectors: ([.collections[].vector_count] | add)}'
```

The `.bak` files stay on the PVC in case you need to roll the auth
state back; delete them once you've confirmed the new admin login
works.

## What changed in 3.0.11

If you're upgrading from an earlier 3.0.x release, three Raft bugs were
fixed across 3.0.9 → 3.0.11. Each was caught only by live testing on a
real K8s cluster:

| Version | Fix |
|---|---|
| **3.0.9** | `resolve_leader_addr` reads from openraft membership instead of the post-bootstrap `AddNode` state machine. Followers can route to the leader the moment one is elected, instead of waiting for an `AddNode` proposal that often never lands. |
| **3.0.10** | Only the lowest-ordinal pod calls `initialize_cluster`. The previous "every pod calls initialize, others get 'already initialized'" pattern produced N divergent term-1 logs (each pod self-voted) and Vote RPCs were rejected on log-mismatch grounds forever. |
| **3.0.11** | The post-bootstrap `AddNode` retry loop no longer calls `raft().trigger().elect()` unconditionally every 10 seconds — it only fires when no leader is visible after a 30 s warm-up. The previous unconditional trigger was kicking stable leaders out before they could renew their lease, causing leadership to rotate forever. |

If you're stuck on 3.0.x where x ≤ 10, **do not try to roll a config fix** —
the bugs are in the binary. Bump the image tag, do a rolling restart, and
the symptoms disappear. PVCs from a poisoned 3.0.x cluster *can* persist
the bad Raft state; if the cluster won't elect after the upgrade, wipe the
PVCs (Step 11) and let the cluster reinitialize.

---

## 10. Migration from v2.x

The v2.x configuration format is compatible with v3, but v2 clusters in
production today commonly have one of two issues:

1. **Wrong `node_id` / `cluster.servers[].id`** — usually missing the namespace
   prefix (e.g. config says `vectorizer-0` but the pod is actually
   `myapp-vectorizer-0`). Symptom: `DNS resolution for '...' failed: failed to
   lookup address information`. Fix the ConfigMap to match the real pod
   hostnames, then restart.
2. **Missing JWT secret** when the StatefulSet binds `0.0.0.0`. v2 was more
   permissive; v3 hard-fails at startup. Add the secret described in Step 3
   and the env vars in Step 6.

For the actual upgrade:

```bash
NS=ermes   # or whatever your real namespace is

# 1. Patch the ConfigMap to match v3 expectations (Step 5 above).
kubectl edit cm vectorizer-ha-config -n "$NS"

# 2. Bump the image tag.
kubectl set image sts/vectorizer -n "$NS" vectorizer=ghcr.io/hivellm/vectorizer:3.0.11

# 3. Make sure the credentials secret exists with all three keys
#    (Step 3); add it if not.

# 4. Rolling restart so the new image and config take effect.
kubectl rollout restart sts/vectorizer -n "$NS"
kubectl rollout status sts/vectorizer -n "$NS"

# 5. Validate (Steps 7-9).
```

If the cluster won't elect after the upgrade — `current_leader` keeps
flipping between pods, or every pod stays in Candidate — **the persisted
Raft logs are poisoned** (term/vote written by an earlier broken release).
Wipe and let it rebuild from scratch:

```bash
# DESTRUCTIVE: deletes all collection data on the cluster.
# Take a backup first if the data is worth anything.
kubectl delete sts vectorizer -n "$NS" --cascade=foreground
kubectl delete pvc -n "$NS" -l app=vectorizer
kubectl apply -f statefulset.yaml      # redeploy
```

A poisoned-PVC reset is also the right call when migrating *to* HA mode
from a single-node v3 deployment: the standalone-mode log has no
membership entry, so adding cluster mode on top of an existing data
directory leaves the new HA cluster trying to reconcile against a log it
can't make sense of.

---

## 11. Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `ImagePullBackOff: not found` for `:vX.Y.Z` | GHCR tags are unprefixed | Use `:3.0.11` (no `v`). |
| `Cannot bind to 0.0.0.0 without authentication enabled` | `auth.enabled: false` plus a public bind | Set `VECTORIZER_AUTH_ENABLED=true` and provide a JWT secret. |
| Crashloop with `auth: missing field jwt_secret at line N` | YAML config is missing `auth.jwt_secret` *and* env override is unset | Set `jwt_secret: "anything"` in the ConfigMap or set `VECTORIZER_JWT_SECRET`. |
| `Address already in use (os error 98)` on port 7001 | Two replicas trying to bind on the same node + hostNetwork | Don't use `hostNetwork: true`. Each pod owns port 7001 inside its own netns. |
| `DNS resolution for '<id>.<svc>...' failed: Name or service not known` | `cluster.servers[].id` doesn't match the real pod hostname | Edit the ConfigMap so the ids are exactly the StatefulSet pod names. |
| `No leader elected — node entering Candidate state` for >30 s on all pods | All three pods called `initialize_cluster` (release ≤ 3.0.9) | Upgrade to ≥ 3.0.10. If already on ≥ 3.0.10, check the `cluster.servers` ids match hostnames. |
| Leader rotates every ~10 s on every pod | Forced election retry loop (release ≤ 3.0.10) | Upgrade to ≥ 3.0.11. |
| Followers log `Leader address not found after retries` | `resolve_leader_addr` falling back to the empty state-machine map (release ≤ 3.0.8) | Upgrade to ≥ 3.0.9. |
| `/health` returns 503 / pod stays NotReady | `cluster.servers` references peers that don't resolve via DNS | Confirm the headless service has `publishNotReadyAddresses: true` and that all pods exist. |
| `vector_count` lags between pods after writes | Heartbeat interval too low for the cluster size, or replication TCP throttled | Bump `replication.heartbeat_interval_secs` to 10 and check pod CPU limits. |
| Different `vector_count` on the three pods days after a write | Replica fell behind and the master's WAL window already rolled past it | Restart the lagging follower; a fresh full sync brings it back in line. |

When in doubt, the **single most useful diagnostic** is to bump
`RUST_LOG=info,openraft=debug,vectorizer::cluster=debug` in the env list,
restart the pods, and grep the logs for `current_leader`,
`AppendEntries`, and `vote=`. Real Raft progress will be visible
immediately; if those three strings never appear, the cluster never even
started electing.

---

## Reference manifests

The repository ships ready-to-edit copies of the manifests in this guide:

- [`k8s/configmap-ha.yaml`](../../k8s/configmap-ha.yaml)
- [`k8s/statefulset-ha.yaml`](../../k8s/statefulset-ha.yaml)
- [`k8s/service.yaml`](../../k8s/service.yaml)
- [`helm/vectorizer/`](../../helm/vectorizer/) — Helm chart that wraps the
  same shapes; set `cluster.enabled=true` in your values overlay to land
  the HA topology described here.

Adjust the namespace, service name, and `cluster.servers` ids for your
deployment before applying.
