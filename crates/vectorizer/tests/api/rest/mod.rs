//! REST API Tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod batch_insert_real;
pub mod batch_ops_real;
#[cfg(test)]
pub mod dashboard_spa;
#[cfg(test)]
pub mod encryption;
#[cfg(test)]
pub mod encryption_complete;
#[cfg(test)]
pub mod encryption_extended;
#[cfg(test)]
pub mod file_upload;
pub mod force_save_real;
#[cfg(test)]
pub mod graph_integration;
pub mod hub_endpoints;
pub mod hub_integration_live;
pub mod integration;
pub mod vector_search_real;
#[cfg(test)]
pub mod workspace;
