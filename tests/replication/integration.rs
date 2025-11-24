//! Replication Integration Tests

// Each file is a separate module to avoid import conflicts
#[path = "integration_basic.rs"]
mod integration_basic;

#[path = "comprehensive.rs"]
mod comprehensive;

#[path = "api.rs"]
mod api;
