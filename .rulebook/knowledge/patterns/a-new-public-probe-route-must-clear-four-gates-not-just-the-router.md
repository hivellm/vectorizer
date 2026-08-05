# A new public probe route must clear four gates, not just the router

**Category**: rest
**Tags**: rest, routing, middleware, readiness, healthcheck, ha

## Description

Adding `GET /ready` to `public_routes` in `core/routing.rs` looks sufficient and is not. `/health` — the model to copy — is exempted in **three separate allow-lists**, and a route that joins only the router 401s the moment auth is enabled:

- `routing.rs` auth middleware — an explicit `path == "/health" || ...` chain.
- `crates/vectorizer/src/hub/middleware.rs` — `path.starts_with("/health")`, applies when HiveHub is active.
- `crates/vectorizer/src/hub/request_signing.rs` — `SigningConfig::exempt_paths`, or a probe carrying no signature is rejected.

A fourth gate is different in kind and easy to miss because it is not about auth: the **HA middleware** in `routing.rs` redirects requests to the leader. A readiness probe must be exempted there, because the answer describes *this* node — proxying it lets a follower still loading its catalog answer with the leader's "ready" and take traffic it cannot serve. Liveness/readiness/metrics all want local answers; anything cluster-wide does not.

Grep `"/health"` across `crates/` before adding any probe route; each hit is a gate to consider. `rest_auth_enforcement.rs::public_routes_stay_anonymous_with_auth_enabled` is the test that actually exercises the first three — add the new path there or nothing covers them.

Related, on the same endpoint family: **`/health` must stay liveness**. The Dockerfile HEALTHCHECK probes it with `--start-period=40s --interval=30s --retries=3`, so making it fail during startup warm-up marks a container unhealthy after ~2 minutes — and the deployments slow enough to hit that are exactly the large ones that need warm-up. The orchestrator would restart them in a loop. Readiness belongs on its own route; a test should pin the 200-during-warm-up behaviour so the "improvement" of folding readiness into `status` cannot land quietly.

## When to Use

Adding any probe/public endpoint (readiness, liveness, metrics, discovery) to the REST surface, or debugging why a route registered in `public_routes` still returns 401.
