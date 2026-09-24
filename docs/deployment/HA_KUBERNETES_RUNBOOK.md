# HA on Kubernetes — End-to-End Runbook

This is the step-by-step playbook for deploying Vectorizer in High-Availability
mode (Raft consensus + leader/follower replication) on a Kubernetes cluster
with `ghcr.io/hivellm/vectorizer:3.8.0`. It starts with an empty namespace and
ends with a 3-pod Raft cluster that survives leader kills and rolling updates.

The neighbouring docs (`CLUSTER.md`, `KUBERNETES.md`, `users/configuration/CLUSTER.md`)
cover the architecture and reference configuration. This file is the
operational sequence.

> **Use 3.8.0 or later for HA.** From 3.8.0 every write path replicates, a
> full sync makes a follower an exact copy of the leader, and Raft state is
> persisted so a pod restarted on its own rejoins as a follower — which is
> what makes plain rolling updates safe. Coming from ≤ 3.7.2? Read
> [Upgrading](#10-rolling-updates-and-upgrades) first: that upgrade needs one
> restart of all pods together.

---

## 1. Prerequisites

| Requirement | Notes |
|---|---|
| Kubernetes 1.20+ | Tested on k3s and standard kubeadm clusters |
| `kubectl` configured for the target cluster | `kubectl get nodes` must succeed |
| A `StorageClass` that supports `ReadWriteOnce` | k3s `local-path`, AWS `gp3`, GKE `standard-rwo`, etc. |
| Pull access to `ghcr.io/hivellm/vectorizer` | The package is public — no `imagePullSecrets` needed |
| 3 pods worth of capacity | Each pod requests 1 vCPU / 1 Gi RAM minimum (4 Gi / 100 Gi recommended) |

Images:

| Tag | Base | Embeddings |
|---|---|---|
| `ghcr.io/hivellm/vectorizer:3.8.0` | `scratch`, no shell, non-root (UID 65532) | BM25 only (default) |
| `ghcr.io/hivellm/vectorizer:3.8.0-fastembed` | Debian, ONNX Runtime, non-root (UID 65532) | BM25 + fastembed dense/multilingual models ([§11](#11-multilingual-embeddings-optional)) |

Always pin an exact tag — `:latest` floats and makes rollouts
irreproducible. GHCR tags are unprefixed (`3.8.0`, not `v3.8.0`). Docker Hub
(`hivehub/vectorizer`) may carry the same tags as a mirror; GHCR is the
primary registry.

A 3-node Raft cluster needs **majority quorum (2 of 3)** to commit writes.
Two healthy pods means writes succeed. One healthy pod means the cluster
cannot elect a leader and accepts no writes until quorum returns.

---

## 2. Namespace

```bash
NS=vectorizer-ha
kubectl create namespace "$NS"
```

No pull secret is required. Only if you run a **private fork** of the image
on GHCR, create one (PAT with `read:packages`) and add
`imagePullSecrets: [{ name: ghcr-credentials }]` to the pod spec:

```bash
kubectl create secret docker-registry ghcr-credentials -n "$NS" \
  --docker-server=ghcr.io --docker-username='GH_USER' --docker-password='GH_PAT'
```

---

## 3. Application secret (JWT + admin credentials)

The server **refuses to bind to `0.0.0.0` without authentication enabled**
— a hard guard in the bootstrap path, not a config warning. HA pods must bind
to `0.0.0.0` so their peers can reach them, so a JWT secret and an admin
user/password are mandatory. Every pod must share the same JWT secret so a
token issued by one pod is valid on the others.

```bash
JWT="$(openssl rand -hex 64)"            # 128-char hex secret
ADM_PASS="$(openssl rand -base64 24 | tr -d '=+/')"

kubectl create secret generic vectorizer-credentials \
  --namespace "$NS" \
  --from-literal=VECTORIZER_USERNAME='admin' \
  --from-literal=VECTORIZER_PASSWORD="$ADM_PASS" \
  --from-literal=VECTORIZER_API_KEY="$JWT"
```

`VECTORIZER_API_KEY` is the JWT signing secret; the StatefulSet maps it to
`VECTORIZER_JWT_SECRET`.

> **Save `$ADM_PASS` somewhere safe** — it's the only credential that lets
> you reach gated routes (`/collections`, `/insert_texts`, etc.) while you
> set up dashboards, API keys, or HiveHub.

---

## 4. Services

The Raft transport (gRPC on port 15003) and replication transport (TCP on
port 7001) both rely on per-pod DNS names —
`<pod-name>.<service-name>.<ns>.svc.cluster.local`. That only works behind a
*headless* `Service` (`clusterIP: None`). A second, ordinary `ClusterIP`
Service gives clients one readiness-gated address for reads.

```yaml
# deploy/k8s/service-ha.yaml
apiVersion: v1
kind: Service
metadata:
  name: vectorizer-headless
  namespace: vectorizer-ha
  labels:
    app: vectorizer
spec:
  clusterIP: None                # headless — required for per-pod DNS
  publishNotReadyAddresses: true # peers must resolve before readiness passes
  selector:
    app: vectorizer
  ports:
    - { name: rpc,         port: 15503, targetPort: 15503 }
    - { name: http,        port: 15002, targetPort: 15002 }
    - { name: grpc,        port: 15003, targetPort: 15003 }
    - { name: replication, port: 7001,  targetPort: 7001  }
---
apiVersion: v1
kind: Service
metadata:
  name: vectorizer
  namespace: vectorizer-ha
  labels:
    app: vectorizer
spec:
  type: ClusterIP                # reads; writes must reach the leader (§8)
  selector:
    app: vectorizer
  ports:
    - { name: rpc,  port: 15503, targetPort: 15503 }
    - { name: http, port: 15002, targetPort: 15002 }
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
# deploy/k8s/configmap-ha.yaml
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
      # Not applied to the console log filter — set RUST_LOG in the
      # StatefulSet env instead (Step 6).
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
      # alphabetically-first id is the bootstrap node — the only pod that
      # calls openraft `initialize_cluster`. The others wait for the
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
# deploy/k8s/statefulset-ha.yaml
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
  podManagementPolicy: Parallel    # all pods come up together; Raft needs peers to elect
  selector:
    matchLabels:
      app: vectorizer
  template:
    metadata:
      labels:
        app: vectorizer
    spec:
      securityContext:
        fsGroup: 65532             # the image runs as UID/GID 65532; makes the PVC writable
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
          image: ghcr.io/hivellm/vectorizer:3.8.0
          imagePullPolicy: IfNotPresent
          ports:
            - { name: rpc,         containerPort: 15503 }
            - { name: http,        containerPort: 15002 }
            - { name: grpc,        containerPort: 15003 }
            - { name: replication, containerPort: 7001  }
          env:
            # Without RUST_LOG only WARN lines are printed, which hides the
            # LEADER/FOLLOWER transitions this runbook relies on.
            - name: RUST_LOG
              value: "info"
            - name: HOSTNAME
              valueFrom:
                fieldRef: { fieldPath: metadata.name }
            - name: POD_IP
              valueFrom:
                fieldRef: { fieldPath: status.podIP }
            - name: VECTORIZER_SERVICE_NAME
              value: "vectorizer-headless.vectorizer-ha.svc.cluster.local"
            # Pin the data dir to the PVC. Collections, auth files and the
            # Raft log/vote/snapshot (<data_dir>/raft/) all live here.
            # See "Data directory pitfall" below.
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
            httpGet: { path: /ready, port: http }
            initialDelaySeconds: 5
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

Notes on the manifest:

- **Probes.** `/health` and `/ready` are anonymous on purpose —
  auth-protected probes would deadlock against the JWT issuer. `/health`
  answers 200 as soon as the HTTP server is up, even while collections are
  still loading, so it is the liveness probe. `/ready` answers 503 (with
  `Retry-After`) until the startup collection load completes and 200 after,
  so a pod only receives Service traffic — and a rolling update only moves
  to the next pod — once it can actually serve its data.
- **No shell.** The default image is `FROM scratch` and runs as non-root
  (UID 65532); `kubectl exec ... sh` does not work. Use `kubectl logs`, the
  REST API, or a throwaway pod that mounts the PVC
  ([Data directory pitfall](#data-directory-pitfall)).
- **Volume permissions.** `fsGroup: 65532` makes the PVC writable by the
  image user. If your storage driver ignores `fsGroup`, or the PVC holds
  files written as root by an older deployment, run the pod as root instead
  (`securityContext: { runAsUser: 0 }`).

Apply everything in order (namespace and secret from Steps 2–3 first):

```bash
kubectl apply -f deploy/k8s/service-ha.yaml
kubectl apply -f deploy/k8s/configmap-ha.yaml
kubectl apply -f deploy/k8s/statefulset-ha.yaml
```

---

## 7. Validate the cluster came up

A successful first boot looks like this in the pod logs:

```text
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

These lines are logged at INFO, so they only appear with `RUST_LOG=info`.

**A leader should be elected within ~10 seconds of the third pod starting.**
If you see `No leader elected – node entering Candidate state` repeating for
more than 30 seconds across all three pods, jump to
[Troubleshooting](#12-troubleshooting).

Quick checks:

```bash
NS=vectorizer-ha

# All three pods Ready
kubectl get pods -n "$NS" -l app=vectorizer

# Who is leader? (last role transition in each pod's log)
roles() {
  for p in vectorizer-0 vectorizer-1 vectorizer-2; do
    role=$(kubectl logs "$p" -n "$NS" 2>/dev/null \
      | grep -E 'This node is now (the )?(LEADER|FOLLOWER)' \
      | tail -1 | grep -oE 'LEADER|FOLLOWER')
    echo "$p ${role:-UNKNOWN}"
  done
}
roles
```

You should see exactly one `LEADER` and two `FOLLOWER`.

If a pod's log has been rotated, ask a follower instead: any write sent to
a follower is answered with `307` and the leader's address, even without a
token (the leader itself answers `401`):

```bash
kubectl port-forward -n "$NS" pod/vectorizer-1 18003:15002 >/dev/null &
PF=$!; sleep 2
curl -s -X POST http://127.0.0.1:18003/collections
# {"redirect":"write operations must go to leader","leader_url":"http://vectorizer-0.vectorizer-headless.vectorizer-ha.svc.cluster.local:15002"}
kill "$PF"
```

Do **not** use these to find the leader: `GET /api/v1/cluster/leader` and
`GET /api/v1/cluster/role` currently return a fixed `"standalone"`
placeholder even in HA mode, and `GET /replication/stats` does not reflect
Raft HA state either.

---

## 8. How clients write to the cluster

- **Reads** (GET, and read-only POSTs such as search, scroll, recommend,
  count, GraphQL) are served by any pod. Point readers at the `vectorizer`
  ClusterIP Service. Replication is asynchronous, so a read on a follower
  right after a write may briefly miss it.
- **Writes** must reach the leader. A follower does **not** forward or proxy
  a write; it answers:

  ```text
  HTTP/1.1 307 Temporary Redirect
  Location: http://vectorizer-0.vectorizer-headless.vectorizer-ha.svc.cluster.local:15002/<original path>
  X-Vectorizer-Leader: http://vectorizer-0.vectorizer-headless.vectorizer-ha.svc.cluster.local:15002
  X-Vectorizer-Role: follower

  {"redirect":"write operations must go to leader","leader_url":"http://vectorizer-0.vectorizer-headless.vectorizer-ha.svc.cluster.local:15002"}
  ```

  `leader_url` is an in-cluster DNS name, so only clients running inside the
  cluster can follow it. Most HTTP clients (curl `-L`, Python `requests`,
  Go `net/http`, `fetch`) drop the `Authorization` header when a redirect
  changes host, so a blindly followed 307 reaches the leader unauthenticated
  and gets `401`. Re-send the request to `leader_url` with the same headers
  and body (`curl --location-trusted` does this).

Pick one of:

1. **Client-side leader handling** — send writes to the ClusterIP Service;
   on `307`, re-send to `leader_url` with the token and cache it until the
   next `307` or connection error.
2. **A leader-routing layer** — a small in-cluster gateway/proxy that sends
   writes to the current leader (learned from a follower's `307`) and reads
   to the Service.

Every write path replicates — REST, RPC, MCP, GraphQL and native gRPC:
inserts, updates, vector deletes, collection create/delete/rename, and file
uploads. **Known gap:** Qdrant-compatible gRPC *points* writes (upsert,
delete, payload changes over the Qdrant gRPC API) are not replicated yet —
use REST or RPC for writes in HA mode.

---

## 9. Smoke tests

### Replication

```bash
NS=vectorizer-ha
ADM_PASS=$(kubectl get secret vectorizer-credentials -n "$NS" \
  -o jsonpath='{.data.VECTORIZER_PASSWORD}' | base64 -d)

# Write to the LEADER (roles() from Step 7).
LEADER=$(roles | awk '$2=="LEADER"{print $1}')
echo "leader = $LEADER"
kubectl port-forward -n "$NS" "pod/$LEADER" 18002:15002 >/dev/null &
PF=$!; sleep 3

TOKEN=$(curl -sS -X POST http://127.0.0.1:18002/auth/login \
  -H 'content-type: application/json' \
  -d "{\"username\":\"admin\",\"password\":\"$ADM_PASS\"}" \
  | jq -r .access_token)

# Dimension MUST match the embedder's output (default BM25 = 512 dim).
curl -sS -X POST http://127.0.0.1:18002/collections \
  -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"name":"smoke","dimension":512}'

curl -sS -X POST http://127.0.0.1:18002/insert_texts \
  -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"collection":"smoke","texts":[
        {"id":"a","text":"replication smoke test 1","metadata":{}},
        {"id":"b","text":"replication smoke test 2","metadata":{}}
       ]}'
kill "$PF"

# Read the count back from every pod.
port=18010
for p in vectorizer-0 vectorizer-1 vectorizer-2; do
  kubectl port-forward -n "$NS" "pod/$p" "$port:15002" >/dev/null &
  PF=$!; sleep 2
  vc=$(curl -sS -H "Authorization: Bearer $TOKEN" \
    "http://127.0.0.1:$port/collections/smoke" | jq -r .vector_count)
  echo "$p: vector_count=$vc"
  kill "$PF"; port=$((port+1))
done
```

Expected: `vector_count=2` on all three pods within seconds of the insert.
Deleting a vector or the collection through the leader must likewise show up
on every pod.

### Failover

```bash
NS=vectorizer-ha
LEADER=$(roles | awk '$2=="LEADER"{print $1}')
echo "current leader = $LEADER"

# Kill the leader.
kubectl delete pod "$LEADER" -n "$NS" --grace-period=2

# Watch the pod come back (Ctrl-C once it is 1/1 Running).
kubectl get pods -n "$NS" -l app=vectorizer -w

roles
```

A new leader should be elected within ~10 s. End state is again exactly one
`LEADER` and two `FOLLOWER`, with the killed pod back as a follower. A pod
resuming persisted Raft state holds its own elections for 10 s so its peers
can re-resolve its DNS name — it will not immediately take leadership back.

---

## 10. Rolling updates and upgrades

### Raft state on disk

Each pod keeps its Raft log, vote and snapshot in `<data_dir>/raft/` —
`/data/data/raft/` with the StatefulSet above, i.e. on the PVC. A pod
restarted on its own (rolling restart, eviction, crash, node drain) rejoins
as a follower. When a follower starts — or reconnects after falling out of
the leader's replication log window — it gets a full sync: it becomes an
exact copy of the leader's collections and vectors, and the sync waits
until both nodes finished loading their collections from disk.

### From 3.8.0 onwards: plain rolling updates

```bash
kubectl set image sts/vectorizer -n "$NS" vectorizer=ghcr.io/hivellm/vectorizer:<new-tag>
kubectl rollout status sts/vectorizer -n "$NS"
```

The StatefulSet replaces one pod at a time and waits for `/ready` before
moving on, so quorum (2 of 3) is kept throughout. Configuration changes
(`kubectl apply -f configmap-ha.yaml` + `kubectl rollout restart sts/vectorizer`)
work the same way.

### Upgrading from ≤ 3.7.2 to 3.8.0: restart all pods together once

Before 3.8.0 Raft state was kept in memory, so a follower restarted on its
own never rejoined, and followers only become exact copies of the leader
through a full sync from a 3.8.0 leader. Restart the whole cluster at once
for this one upgrade:

```bash
NS=vectorizer-ha   # your namespace

# 1. Stop the StatefulSet from rolling pods one by one.
kubectl patch sts vectorizer -n "$NS" -p '{"spec":{"updateStrategy":{"type":"OnDelete"}}}'

# 2. Move to the 3.8.0 image and INFO logs. Also switch the readiness probe
#    to /ready and drop imagePullSecrets (GHCR is public now) — with
#    `kubectl edit sts/vectorizer` or by re-applying your manifest.
kubectl set image sts/vectorizer -n "$NS" vectorizer=ghcr.io/hivellm/vectorizer:3.8.0
kubectl set env sts/vectorizer -n "$NS" RUST_LOG=info

# 3. Delete every pod at once; they come back together on 3.8.0 and the
#    followers full-sync from the new leader.
kubectl delete pod -n "$NS" -l app=vectorizer
kubectl wait pod -n "$NS" -l app=vectorizer --for=condition=Ready --timeout=15m

# 4. Back to rolling updates for every later release.
kubectl patch sts vectorizer -n "$NS" -p '{"spec":{"updateStrategy":{"type":"RollingUpdate"}}}'
```

Scaling to 0 and back to 3 achieves the same. Then run the checks in Steps 7
and 9. Deployments whose data was written by a root container should keep
`runAsUser: 0` (or fix ownership first) when moving to the non-root image.

### Older releases

- **3.0.x ≤ 3.0.10** had Raft bootstrap bugs in the binary (split init,
  leader address resolution, forced re-elections) — do not try to fix them
  with config. If the cluster will not elect after upgrading, the persisted
  Raft state is poisoned: reset it as described below.
- **v2.x configs** are compatible, but check two things first: every
  `cluster.servers[].id` must equal the real pod hostname (a mismatch shows
  up as `DNS resolution for '...' failed`), and the JWT secret from Step 3
  must exist — v3 hard-fails at startup without it when binding `0.0.0.0`.

### Resetting Raft state

To reset only the consensus state (a poisoned log, or a cluster whose
membership you changed) without touching collection data: scale to 0, delete
`<data_dir>/raft/` from every PVC with a maintenance pod (see
[Data directory pitfall](#data-directory-pitfall)), and scale back to 3. The
bootstrap pod re-initialises the cluster.

```bash
kubectl scale sts vectorizer -n "$NS" --replicas=0
for i in 0 1 2; do pvc_run "$i" 'rm -rf /data/data/raft'; done
kubectl scale sts vectorizer -n "$NS" --replicas=3
```

The same reset is the right call when turning an existing single-node data
directory into an HA cluster: the standalone node has no Raft membership to
reconcile against.

---

## 11. Multilingual embeddings (optional)

The default image is BM25-only. For dense and multilingual models —
including synonym and cross-language matches ("automóvel" finds "carro") —
run the `-fastembed` image and register the model next to BM25:

```yaml
# statefulset-ha.yaml
image: ghcr.io/hivellm/vectorizer:3.8.0-fastembed

# configmap-ha.yaml, top level of config-template.yml
embedding:
  model: "bm25"                      # default for collections created without a provider
  additional_models:
    - "fastembed:multilingual-e5-small"
```

Then create collections that use it (through the leader):

```bash
curl -sS -X POST http://127.0.0.1:18002/collections \
  -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"name":"docs_pt","dimension":384,"embedding_provider":"fastembed:multilingual-e5-small"}'
```

- Text inserted into and searched in that collection is embedded with the
  E5 model; the `passage:` / `query:` prefixes are added automatically.
- Existing BM25 collections keep working unchanged. A collection's model is
  fixed at creation: to move one to E5, create a new collection and
  re-insert its texts.
- Every pod needs the model (followers embed search queries locally), which
  the shared ConfigMap takes care of. The model is stored under
  `<data_dir>/fastembed` on the PVC; the PVC mounted at `/data` hides
  anything baked into the image there, so each pod downloads the model on
  its first boot and needs outbound HTTPS access for it.

Full details: [Embedding Providers Guide](../users/guides/EMBEDDINGS.md).

---

## Data directory pitfall

`vectorizer-core::paths::data_dir()` — the function the server uses to
locate `vectorizer.vecdb`, snapshots, the auth files, the Raft state and the
fastembed model cache — resolves in this order:

1. `$VECTORIZER_DATA_DIR` if set and non-empty.
2. `dirs::data_dir().join("vectorizer")` — per-OS user data directory
   (`~/.local/share/vectorizer/` on Linux).
3. `./data` — relative to the current working directory.

The image sets `VECTORIZER_DATA_DIR=/data`; the StatefulSet overrides it to
`/data/data` to match deployments that predate that default. Whatever value
you use, it **must** be inside the PVC mount — otherwise every restart looks
like a fresh first-time setup, and a pod that loses its Raft state behaves
like a brand-new member. Deploys built from `deploy/k8s/statefulset-ha.yaml`
before v3.0.13 dropped this env var: the data was still on the PVC at
`/data/data/`, just read from the wrong path.

### Maintenance pod for PVC work

The image has no shell, so file work on a PVC goes through a throwaway
busybox pod. Scale the StatefulSet to 0 first so the `ReadWriteOnce` volume
is free:

```bash
# pvc_run <ordinal> '<shell command>' — runs the command with the PVC
# data-vectorizer-<ordinal> mounted at /data. No double quotes in the command.
pvc_run() {
  kubectl apply -n "$NS" -f - <<EOF
apiVersion: v1
kind: Pod
metadata: { name: pvc-maint-$1 }
spec:
  restartPolicy: Never
  containers:
    - name: maint
      image: busybox:1.36
      command: ["sh", "-c", "$2"]
      volumeMounts: [{ name: data, mountPath: /data }]
  volumes:
    - name: data
      persistentVolumeClaim: { claimName: data-vectorizer-$1 }
EOF
  kubectl wait "pod/pvc-maint-$1" -n "$NS" \
    --for=jsonpath='{.status.phase}'=Succeeded --timeout=5m
  kubectl logs "pvc-maint-$1" -n "$NS"
  kubectl delete pod "pvc-maint-$1" -n "$NS"
}
```

### Recovering data lost to the data-dir trap

If the API reports zero collections but the PVC holds
`/data/data/vectorizer.vecdb`:

```bash
# 1. Point the server at the PVC.
kubectl set env sts/vectorizer -n "$NS" VECTORIZER_DATA_DIR=/data/data

# 2. The auth files on the PVC were written with a different JWT secret /
#    admin password. Move them aside so the server recreates them from the
#    current Secret on next boot.
kubectl scale sts vectorizer -n "$NS" --replicas=0
for i in 0 1 2; do
  pvc_run "$i" 'cd /data/data && mv auth.enc auth.enc.bak; mv .auth.key .auth.key.bak; ls -la'
done

# 3. Start again (all pods together) and check the collection list.
kubectl scale sts vectorizer -n "$NS" --replicas=3
kubectl wait pod -n "$NS" -l app=vectorizer --for=condition=Ready --timeout=15m
```

Log in through the leader as in Step 9 and compare `GET /collections` with
what was on the PVC. The `.bak` files stay on the PVC in case you need to
roll the auth state back; remove them once the new admin login works.

---

## 12. Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `ImagePullBackOff: not found` for `:vX.Y.Z` | GHCR tags are unprefixed | Use `:3.8.0` (no `v`). |
| `ImagePullBackOff: unauthorized` on `ghcr.io/hivellm/vectorizer` | Stale `imagePullSecrets` with an expired PAT | The image is public — remove `imagePullSecrets` (or refresh the secret for a private fork). |
| `kubectl exec ... sh` fails with `executable file not found` | The default image has no shell | Use `kubectl logs`, the REST API, or the maintenance pod above. |
| `Permission denied (os error 13)` under `/data` | PVC not writable by UID 65532 | Set `securityContext.fsGroup: 65532`, or `runAsUser: 0` for volumes written by root. |
| No `LEADER` / `FOLLOWER` lines in the logs | Without `RUST_LOG` only WARN is printed; `logging.level` in config.yml does not change that | Set `RUST_LOG=info` in the StatefulSet env. |
| Pod Running but not Ready for a while after start | `/ready` returns 503 until the startup collection load completes | Expected on large data sets; it turns Ready once loading is done. |
| Writes fail with `307` or `401` | Write sent to a follower; the redirect was not followed, or was followed without the `Authorization` header | Send writes to the leader — see [§8](#8-how-clients-write-to-the-cluster). |
| Writes through the Qdrant gRPC API are missing on followers | Qdrant-compatible gRPC points writes are not replicated yet | Use REST or RPC for writes in HA mode. |
| `Cannot bind to 0.0.0.0 without authentication enabled` | `auth.enabled: false` plus a public bind | Set `VECTORIZER_AUTH_ENABLED=true` and provide a JWT secret. |
| Crashloop with `auth: missing field jwt_secret at line N` | YAML config is missing `auth.jwt_secret` *and* env override is unset | Set `jwt_secret: "anything"` in the ConfigMap or set `VECTORIZER_JWT_SECRET`. |
| `Address already in use (os error 98)` on port 7001 | Two replicas trying to bind on the same node + hostNetwork | Don't use `hostNetwork: true`. Each pod owns port 7001 inside its own netns. |
| `MasterNode failed: IO error: Address in use (os error 98)` on the leader, followers stop receiving writes | Pod regained leadership without restarting; the previous term's master still held port 7001 (release ≤ 3.7.1) | Upgrade to 3.8.0. |
| `DNS resolution for '<id>.<svc>...' failed: Name or service not known` | `cluster.servers[].id` doesn't match the real pod hostname | Edit the ConfigMap so the ids are exactly the StatefulSet pod names. |
| `No leader elected – node entering Candidate state` for >30 s on all pods | All three pods called `initialize_cluster` (release ≤ 3.0.9), or ids don't match hostnames | Upgrade to 3.8.0 and check the `cluster.servers` ids. |
| Leader rotates every ~10 s on every pod | Forced election retry loop (release ≤ 3.0.10) | Upgrade to 3.8.0. |
| Followers log `Leader address not found after retries` | `resolve_leader_addr` falling back to the empty state-machine map (release ≤ 3.0.8) | Upgrade to 3.8.0. |
| Pods never become Ready at first boot | Peers can't resolve each other before readiness | Confirm the headless Service has `publishNotReadyAddresses: true` and `podManagementPolicy: Parallel`. |
| `vector_count` lags between pods after writes | Heartbeat interval too low for the cluster size, or replication TCP throttled | Bump `replication.heartbeat_interval_secs` to 10 and check pod CPU limits. |
| Different `vector_count` on the pods long after a write | Follower fell out of the leader's replication log window | On 3.8.0, restart the lagging follower; its full sync makes it an exact copy of the leader. On ≤ 3.7.2, upgrade (see the next rows). |
| A pod restarted on its own logs `current_state=Learner` forever and stops receiving writes | Raft log and vote were kept in memory (release ≤ 3.7.2) | Upgrade to 3.8.0 with one all-pods restart ([§10](#10-rolling-updates-and-upgrades)). |
| Pods disagree on a collection's contents even right after restarting them together — e.g. one lists every vector twice | Full sync ran before the startup load finished and replicas kept their stale copy; loading also duplicated repeated ids (release ≤ 3.7.2) | Upgrade to 3.8.0 with one all-pods restart; each follower becomes a copy of the leader. |
| A vector or collection deleted through the leader is still on the followers | Only inserts and collection creates were replicated (release ≤ 3.7.2) | Upgrade to 3.8.0 with one all-pods restart to resync. |
| `/api/v1/cluster/leader` says `standalone` in an HA cluster | Those endpoints return a fixed placeholder | Find the leader from the logs or a follower's `307` ([§7](#7-validate-the-cluster-came-up)). |

When in doubt, the **single most useful diagnostic** is to set
`RUST_LOG=info,openraft=debug,vectorizer::cluster=debug` in the env list,
restart the pods, and grep the logs for `current_leader`,
`AppendEntries`, and `vote=`. Real Raft progress will be visible
immediately; if those three strings never appear, the cluster never even
started electing.

---

## Reference manifests

The repository ships ready-to-apply copies of the manifests in this guide
(namespace `vectorizer-ha`, StatefulSet `vectorizer`):

- [`deploy/k8s/service-ha.yaml`](../../deploy/k8s/service-ha.yaml) — headless + client Services
- [`deploy/k8s/configmap-ha.yaml`](../../deploy/k8s/configmap-ha.yaml)
- [`deploy/k8s/statefulset-ha.yaml`](../../deploy/k8s/statefulset-ha.yaml)

If you rename the namespace, StatefulSet or headless Service, update the
`dns_name`, every `cluster.servers[]` entry, and `VECTORIZER_SERVICE_NAME`
to match. The Helm chart in `deploy/helm/vectorizer/` does not render the
Raft cluster configuration; use these manifests for HA.
