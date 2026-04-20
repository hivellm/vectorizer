//! GraphQL HTTP handlers for Axum integration
//!
//! This module provides HTTP handlers for the GraphQL API.

use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse};

use crate::api::graphql::VectorizerSchema;
use crate::hub::auth::TenantContext;

/// GraphQL state containing the schema
#[derive(Clone)]
pub struct GraphQLState {
    pub schema: VectorizerSchema,
}

/// Handle GraphQL queries/mutations
///
/// Supports multi-tenant authentication via headers:
/// - x-hivehub-service: Service API key (bypasses user auth)
/// - x-hivehub-user-id: User/tenant ID for context
pub async fn graphql_handler(
    State(state): State<GraphQLState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // Extract tenant context from headers (if present)
    let tenant_context = extract_tenant_context(&headers);

    // Build request with tenant context
    let mut graphql_req = req.into_inner();

    if let Some(ctx) = tenant_context {
        graphql_req = graphql_req.data(ctx);
    }

    state.schema.execute(graphql_req).await.into()
}

/// Extract tenant context from request headers
fn extract_tenant_context(headers: &HeaderMap) -> Option<TenantContext> {
    // Check for service header (internal service-to-service calls)
    let has_service_header = headers
        .get("x-hivehub-service")
        .and_then(|v| v.to_str().ok())
        .is_some();

    // Get user ID if present
    let user_id = headers
        .get("x-hivehub-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // If we have both service header and user ID, create tenant context
    if has_service_header {
        if let Some(uid) = user_id {
            return Some(TenantContext {
                tenant_id: uid.clone(),
                tenant_name: format!("GraphQL User {}", uid),
                api_key_id: "graphql-service".to_string(),
                permissions: vec![], // GraphQL uses context, not permission checks
                rate_limits: None,
                validated_at: chrono::Utc::now(),
                is_test: false,
            });
        }
    }

    None
}

/// Serve the GraphQL Playground (GraphiQL)
pub async fn graphql_playground() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .title("Vectorizer GraphQL Playground")
            .finish(),
    )
}
