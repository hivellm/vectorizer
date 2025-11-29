//! GraphQL HTTP handlers for Axum integration
//!
//! This module provides HTTP handlers for the GraphQL API.

use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::response::{Html, IntoResponse};

use crate::api::graphql::VectorizerSchema;

/// GraphQL state containing the schema
#[derive(Clone)]
pub struct GraphQLState {
    pub schema: VectorizerSchema,
}

/// Handle GraphQL queries/mutations
pub async fn graphql_handler(
    State(state): State<GraphQLState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
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
