//! Service-specific config sections owned by `config`, not by the
//! services themselves.
//!
//! `phase41_architecture-decoupling` §2 inverted the `config ->
//! {auth,hub,cluster}` dependency edge: these sub-structs used to be
//! defined in their respective service modules and imported by
//! `config::vectorizer`. They now live here as plain serde data types;
//! `auth`, `hub`, and `cluster` import them back and re-export them
//! under their original paths (`crate::auth::AuthConfig`,
//! `crate::hub::HubConfig`, `crate::cluster::ClusterConfig`, ...) so
//! every existing call site keeps compiling unchanged. Business logic
//! (validation, manager construction, etc.) stays in the owning
//! service module as inherent `impl` blocks on the re-exported type.

pub mod auth;
pub mod cluster;
pub mod hub;
