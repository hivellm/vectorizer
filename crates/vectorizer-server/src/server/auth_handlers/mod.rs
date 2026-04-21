//! Authentication REST API handlers.
//!
//! The original `auth_handlers.rs` grew to ~1800 lines covering seven
//! distinct concerns. It is now split into sibling files so that each
//! concern is reviewable in isolation:
//!
//! - [`types`] — request / response / error DTOs shared by handlers
//! - [`state`] — `AuthHandlerState`, `UserRecord`, rate-limit bookkeeping,
//!   persistence bootstrap, and the one-time first-run credentials helper
//! - [`public`] — unauthenticated endpoints (`/auth/login`,
//!   `/auth/validate-password`)
//! - [`authenticated`] — logged-in endpoints (`/auth/me`, `/auth/logout`,
//!   `/auth/refresh`, `/auth/keys…`)
//! - [`admin`] — admin-gated user management endpoints
//!   (`/auth/users…`, `/auth/users/{u}/password`)
//! - [`middleware`] — the five extractor/guard middlewares and the
//!   shared header-parsing helper
//!
//! The public surface is preserved verbatim via `pub use`: every handler
//! name, struct, and middleware that `src/server/mod.rs` used to see on
//! `auth_handlers::X` is still available at exactly that path.

mod admin;
mod authenticated;
mod extractors;
mod middleware;
mod public;
mod state;
mod types;

// Import names the test submodule expects to pick up via `use super::*;`.
// These are exactly what the pre-refactor `auth_handlers.rs` exposed to
// its inline test module — keep this list in sync with the tests file
// (`src/server/auth_handlers_tests.rs`).
#[allow(unused_imports)]
use std::sync::Arc;

pub use admin::{change_password, create_user, delete_user, list_users};
pub use authenticated::{
    create_api_key, get_me, list_api_keys, logout, refresh_token, revoke_api_key,
};
pub use extractors::{AdminAuth, Authenticated};
pub use middleware::{
    auth_middleware, require_admin_for_rest, require_admin_from_headers, require_admin_middleware,
    require_auth_middleware,
};
pub use public::{login, validate_password_endpoint};
// Internal items that tests reach for via `use super::*`. The outer world
// doesn't see these — the visibility is scoped to the crate.
pub(crate) use state::persist_first_run_credentials;
pub use state::{AuthHandlerState, UserRecord};
pub use types::{
    ApiKeyInfo, AuthErrorResponse, ChangePasswordRequest, CreateApiKeyRequest,
    CreateApiKeyResponse, CreateUserRequest, CreateUserResponse, ListApiKeysResponse,
    ListUsersResponse, LoginRequest, LoginResponse, LogoutResponse, RefreshTokenRequest,
    RefreshTokenResponse, UserInfo, ValidatePasswordRequest, ValidatePasswordResponse,
};
#[allow(unused_imports)]
use vectorizer::auth::roles::Role;

#[cfg(test)]
#[path = "../auth_handlers_tests.rs"]
mod tests;
