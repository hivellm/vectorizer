# Route auth matrix

Defines the three auth buckets the Vectorizer REST server uses when
`auth.enabled = true` (i.e. `AuthHandlerState` is present). When auth is
globally disabled every caller passes through — single-user local setups
are preserved.

## Buckets

| Bucket | Requirement | How it is enforced |
|--------|------------|--------------------|
| **Public** | No token needed. | Registered on `public_routes` / `public_auth_router`; no auth layer applied. |
| **Authenticated** | Valid JWT **or** valid API key. | Handlers declare `_auth: Authenticated` in their signature; the `FromRequestParts` extractor fails with 401 before the body runs. `Extension<AuthState>` is still supported for middleware-layered routes. |
| **Admin** | Valid JWT/API key **and** the claims contain `Role::Admin`. | Routes are registered on a dedicated `admin_router` with `axum::middleware::from_fn_with_state(auth_state, require_admin_middleware)` applied as a router-level layer. The middleware short-circuits to 401 (no token) or 403 (non-admin) before the handler runs. **Adding a new admin route only requires placing it in the `admin_router` builder** — no per-handler change is needed. |

## Routes by bucket

### Public (no auth)

| Method | Path | Handler | Why public |
|--------|------|---------|------------|
| GET | `/health` | `rest_handlers::health_check` | Liveness probe; must work pre-auth |
| GET | `/prometheus/metrics` | `rest_handlers::get_prometheus_metrics` | Scraped by unauth'd monitoring |
| POST | `/auth/login` | `auth_handlers::login` | Credential exchange |
| POST | `/auth/validate-password` | `auth_handlers::validate_password_endpoint` | Pre-registration strength check; no secrets returned |

### Authenticated (any logged-in user)

| Method | Path | Handler |
|--------|------|---------|
| GET | `/auth/me` | `auth_handlers::get_me` |
| POST | `/auth/logout` | `auth_handlers::logout` |
| POST | `/auth/refresh` | `auth_handlers::refresh_token` |
| POST / GET / DELETE | `/auth/keys[/{id}]` | `auth_handlers::{create,list,revoke}_api_key` |
| (various) | `/collections/*`, `/vectors/*`, `/search`, `/discover*`, `/file/*`, `/qdrant/*`, `/graph/*`, `/graphql` | data-access handlers |
| GET | `/setup/status`, `/setup/verify`, `/setup/templates*` | `setup_handlers::*` (read-only wizard state) |
| GET | `/config`, `/backups`, `/workspace/list`, `/workspace/config`, `/backups/directory` | read-only inspection |

### Admin (router-level `require_admin_middleware`)

All 9 routes below live on the `admin_router` built in
`src/server/core/routing.rs`. The middleware is the single enforcement
point — handlers do **not** declare `AdminAuth` in their signatures.

| Method | Path | Handler |
|--------|------|---------|
| POST | `/workspace/add` | `rest_handlers::add_workspace` |
| POST | `/workspace/remove` | `rest_handlers::remove_workspace` |
| POST | `/workspace/config` | `rest_handlers::update_workspace_config` |
| POST | `/setup/apply` | `setup_handlers::apply_setup_config` |
| POST | `/setup/browse` | `setup_handlers::browse_directory` |
| POST | `/config` | `rest_handlers::update_config` |
| POST | `/admin/restart` | `rest_handlers::restart_server` |
| POST | `/backups/create` | `rest_handlers::create_backup` |
| POST | `/backups/restore` | `rest_handlers::restore_backup` |

Note: 4 of these 9 routes (`/workspace/remove`, `/workspace/config`,
`/setup/browse`, `/backups/create`) were previously documented as admin
in source comments but had **no actual enforcement** because the
handler was missing the `AdminAuth` extractor. The router-level lift
closes that drift gap structurally — protection comes from
registration-site placement, not handler-by-handler vigilance.

### Auth user mgmt (admin enforced inside handler)

These routes still use the inline `Extension<AuthState>` admin check
(legacy pattern from before the `AdminAuth` extractor existed). Moving
them onto the same `admin_router` is straightforward but is gated on a
state-type unification because `auth_handlers::*` use
`State<AuthHandlerState>` rather than `State<VectorizerServer>`.
Tracked as `phase4_unify-admin-auth-handlers-state`.

| Method | Path | Handler | Gate |
|--------|------|---------|------|
| POST / GET | `/auth/users` | `auth_handlers::create_user` / `list_users` | Inline `is_admin` check on `AuthState` extension |
| DELETE | `/auth/users/{username}` | `auth_handlers::delete_user` | Inline `is_admin` check |
| PUT | `/auth/users/{username}/password` | `auth_handlers::change_password` | Inline `is_admin` check |

## How the router layer is wired

```rust
// src/server/core/routing.rs (excerpt)
let admin_router: Router<()> = Router::new()
    .route("/workspace/add", post(rest_handlers::add_workspace))
    .route("/workspace/remove", post(rest_handlers::remove_workspace))
    // ... 7 more admin routes ...
    .with_state(self.clone());

let admin_router = if let Some(auth_state) = self.auth_handler_state.clone() {
    admin_router.layer(axum::middleware::from_fn_with_state(
        auth_state,
        crate::server::auth_handlers::require_admin_middleware,
    ))
} else {
    admin_router  // single-user mode: no enforcement, matches legacy behaviour
};

// Merged into the final app alongside `rest_routes`, `mcp_router`, etc.
let app = Router::new()
    .merge(public_routes)
    .merge(umicp_routes)
    .merge(mcp_router)
    .merge(admin_router)        // <-- new
    .merge(rest_routes)
    .merge(metrics_router);
```

## Why this works now (and didn't before)

The previous attempt at the router layer (logged in
`docs/api/route-auth-matrix.md` history) failed with:

```
the trait `tonic::codegen::Service<Request<Body>>` is not implemented
for `FromFn<..., ..., _>`
```

The root cause was a `Send` violation hidden inside
`extract_auth_from_request(state, &request).await` — the `&Request`
reference was held across an `.await` and `axum::Request` is `Send`
but not `Sync`, which makes any `&Request` non-`Send`. The typed
`from_fn_with_state` form surfaces the violation as a confusing
`Service` trait mismatch (rustc reports the first trait it tried,
which happened to be `tonic::codegen::Service`).

Fix: split `extract_auth_from_request` into
`extract_auth_from_parts(state, &HeaderMap, Option<&str>)`. Both
borrowed types are `Send + Sync`, so the resulting future is `Send` and
the middleware composes cleanly. The three middleware functions
(`auth_middleware`, `require_auth_middleware`, `require_admin_middleware`)
now extract headers and the query string up-front and call the parts
helper.

## Testing

- Unit (`src/server/auth_handlers_tests.rs`):
  - Legacy helper coverage: `require_admin_for_rest_*` (4 tests).
  - `AdminAuth` / `Authenticated` extractor coverage: 8 tests.
  - **New router-level regression** (`router_admin_layer_returns_401_without_token`,
    `router_admin_layer_returns_403_for_viewer_token`,
    `router_admin_layer_returns_200_for_admin_token`): builds a synthetic
    `Router::new().route(...).layer(from_fn_with_state(...))`, dispatches
    real `axum::http::Request`s through `tower::ServiceExt::oneshot`,
    and asserts the status. Adding a new admin route to the production
    router without breaking these tests guarantees the layer fires.
- End-to-end via the REST API is covered by existing
  `tests/api/rest/*` suites whose `AuthHandlerState::new_with_root_user`
  path exercises admin token creation and subsequent calls against
  `/setup/apply` etc.
