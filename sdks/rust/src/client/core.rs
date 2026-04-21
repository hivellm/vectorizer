//! Server-status surface: `health_check`.
//!
//! Lives in its own module because it doesn't fit any of the
//! domain-specific surfaces (collections / vectors / search / ...)
//! and likely grows to include `/metrics`, `/stats`, and similar
//! observability endpoints in future releases.

use super::VectorizerClient;
use crate::error::{Result, VectorizerError};
use crate::models::*;

impl VectorizerClient {
    /// Check server health.
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.make_request("GET", "/health", None).await?;
        let health: HealthStatus = serde_json::from_str(&response).map_err(|e| {
            VectorizerError::server(format!("Failed to parse health check response: {e}"))
        })?;
        Ok(health)
    }
}
