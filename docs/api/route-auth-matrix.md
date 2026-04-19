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
| **Admin** | Valid JWT/API key **and** the claims contain `Role::Admin`. | Handlers declare `_admin: AdminAuth` in their signature; the extractor fails with 401 (no token) or 403 (non-admin) before the body runs. Legacy `require_admin_for_rest(&state.auth_handler_state, &headers)` helper is still exported for routes that cannot take an extractor. |

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
| GET | `/config`, `/backups`, `/workspace/list`, `/workspace/config` | read-only inspection |

### Admin (role=admin enforced inside handler)

| Method | Path | Handler | Gate |
|--------|------|---------|------|
| POST | `/setup/apply` | `setup_handlers::apply_setup_config` | `AdminAuth` extractor |
| POST | `/workspace/add` | `rest_handlers::add_workspace` | `AdminAuth` extractor |
| POST | `/config` | `rest_handlers::update_config` | `AdminAuth` extractor |
| POST | `/admin/restart` | `rest_handlers::restart_server` | `AdminAuth` extractor |
| POST | `/backups/restore` | `rest_handlers::restore_backup` | `AdminAuth` extractor |

### Pending admin hardening (tracked)

Routes that are operationally admin-sensitive but whose gate is
not yet wired (return type refactor required or signature constraints):

| Method | Path | Follow-up task |
|--------|------|----------------|
| POST | `/workspace/remove` | `phase4_gate-workspace-admin-routes` |
| POST | `/workspace/config` | `phase4_gate-workspace-admin-routes` |
| POST | `/setup/browse` | `TASK(phase4_gate-setup-browse-as-admin)` in the source |
| POST | `/backups/create` | `phase4_gate-workspace-admin-routes` |
| POST / GET / DELETE / PUT | `/auth/users*` | `phase4_gate-auth-users-admin-routes` |

## Why not router-level middleware?

The natural axum pattern is a dedicated `Router::new().layer(from_fn_with_state(
state, require_admin_middleware))`. We attempted this and hit a type-system
wall: the admin bucket spans two state types — `AuthHandlerState` (for
`auth_handlers::*`) and `VectorizerServer` (for `rest_handlers::*` +
`setup_handlers::*`) — and axum can't infer a single layer service that
unifies both. Splitting into two sub-routers each with its own state
still failed because the `FromFn<..., _>: Service<_>` bound won't match
once `.with_state(...)` has been applied before `.layer(...)`.

Rather than accept a half-baked cross-state workaround, we moved the
admin gate into each handler. It adds two lines per handler, compiles
cleanly, is independently testable, and reads linearly next to the
business logic it protects.

Follow-up `phase4_router-layer-admin-middleware` revisits this once
either (a) the handler state types are unified, or (b) a cleaner
`.route_layer` path is found.

## Extractor pattern

Admin-gated handlers declare the extractor as their first parameter:

```rust
pub async fn restart_server(
    _admin: crate::server::auth_handlers::AdminAuth,
    State(_state): State<VectorizerServer>,
) -> Result<Json<Value>, ErrorResponse> {
    // handler body runs only after 401/403 checks pass
}
```

The extractor clones `state.auth_handler_state` out of the typed
`State<VectorizerServer>` and runs the same admin check that
`require_admin_for_rest` uses. Failure returns the same `ErrorResponse`
shape as the legacy helper, so existing REST callers see identical
401/403 bodies.

There is a companion `Authenticated` extractor that accepts any
logged-in user regardless of role. Both extractors yield
`Option<AuthState>` — `None` when auth is globally disabled (legacy
no-auth mode), `Some` when a token was validated.

## Testing

- Unit: `src/server/auth_handlers_tests.rs` covers both the legacy
  `require_admin_for_rest` helper and the `AdminAuth` / `Authenticated`
  extractors across the four input states (auth disabled, no header,
  non-admin token, admin token).
- End-to-end via the REST API is covered by existing
  `tests/api/rest/*` suites whose `AuthHandlerState::new_with_root_user`
  path exercises admin token creation and subsequent calls against
  `/setup/apply` etc.
