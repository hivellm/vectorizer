//! Authentication middleware for HiveHub integration
//!
//! Provides Axum middleware for extracting and validating API keys,
//! and attaching tenant context to requests.
//!
//! Note: In cluster mode, the Vectorizer runs locally and the HiveHub
//! communicates directly with it. Token validation is not required
//! at the Vectorizer level - the Hub handles all authentication.

use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace, warn};

use super::HubConfig;
use super::auth::{HubAuth, TenantContext, TenantPermission};
use super::quota::{QuotaManager, QuotaType};
use crate::error::VectorizerError;

/// Header name for API key
const AUTHORIZATION_HEADER: &str = "authorization";
const BEARER_PREFIX: &str = "Bearer ";

/// API key header alternatives
const X_API_KEY_HEADER: &str = "x-api-key";

/// Internal service header (used by HiveHub to identify itself)
const X_HIVEHUB_SERVICE_HEADER: &str = "x-hivehub-service";

/// Internal user ID header (used by HiveHub to identify the user)
const X_HIVEHUB_USER_ID_HEADER: &str = "x-hivehub-user-id";

/// Error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl AuthErrorResponse {
    pub fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Extension type for storing tenant context in request
#[derive(Clone, Debug)]
pub struct RequestTenantContext(pub TenantContext);

/// HiveHub authentication middleware state
#[derive(Clone)]
pub struct HubAuthMiddleware {
    auth: Arc<HubAuth>,
    quota: Arc<QuotaManager>,
    config: HubConfig,
}

impl HubAuthMiddleware {
    /// Create a new HubAuthMiddleware
    pub fn new(auth: Arc<HubAuth>, quota: Arc<QuotaManager>, config: HubConfig) -> Self {
        Self {
            auth,
            quota,
            config,
        }
    }

    /// Extract API key from request headers
    pub fn extract_api_key(req: &Request<Body>) -> Option<String> {
        // Try Authorization: Bearer <key>
        if let Some(auth_header) = req.headers().get(AUTHORIZATION_HEADER) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(key) = auth_str.strip_prefix(BEARER_PREFIX) {
                    return Some(key.to_string());
                }
            }
        }

        // Try X-API-Key: <key>
        if let Some(api_key_header) = req.headers().get(X_API_KEY_HEADER) {
            if let Ok(key) = api_key_header.to_str() {
                return Some(key.to_string());
            }
        }

        None
    }

    /// Check if request is from HiveHub service (internal request)
    pub fn is_internal_request(req: &Request<Body>) -> bool {
        req.headers().contains_key(X_HIVEHUB_SERVICE_HEADER)
    }

    /// Extract user ID from internal request headers
    pub fn extract_internal_user_id(req: &Request<Body>) -> Option<String> {
        req.headers()
            .get(X_HIVEHUB_USER_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Create an error response
    pub fn error_response(status: StatusCode, error: AuthErrorResponse) -> Response {
        let body = Json(error);
        (status, body).into_response()
    }
}

/// Axum middleware function for HiveHub authentication
///
/// This middleware is optional - in cluster mode, the Vectorizer
/// runs locally and the HiveHub handles all authentication.
/// Internal requests from HiveHub bypass authentication.
pub async fn hub_auth_middleware(
    State(middleware): State<HubAuthMiddleware>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Skip authentication if not enabled
    if !middleware.config.enabled {
        return next.run(req).await;
    }

    // Skip authentication for internal HiveHub service requests
    // but extract user_id if provided
    if HubAuthMiddleware::is_internal_request(&req) {
        trace!("Internal HiveHub service request - skipping authentication");

        // If user_id is provided, create a tenant context for it
        if let Some(user_id) = HubAuthMiddleware::extract_internal_user_id(&req) {
            debug!("Internal request with user_id: {}", user_id);
            let internal_context = TenantContext {
                tenant_id: user_id.clone(),
                tenant_name: format!("Internal user {}", user_id),
                api_key_id: "internal".to_string(),
                permissions: vec![TenantPermission::Admin],
                rate_limits: None,
                validated_at: chrono::Utc::now(),
                is_test: false,
            };
            req.extensions_mut()
                .insert(RequestTenantContext(internal_context));
        }

        return next.run(req).await;
    }

    // Extract API key
    let api_key = match HubAuthMiddleware::extract_api_key(&req) {
        Some(key) => key,
        None => {
            warn!("Request missing API key");
            return HubAuthMiddleware::error_response(
                StatusCode::UNAUTHORIZED,
                AuthErrorResponse::new("API key required", "AUTH_MISSING_KEY"),
            );
        }
    };

    // Validate API key
    let tenant_context = match middleware.auth.validate_api_key(&api_key).await {
        Ok(context) => context,
        Err(e) => match &e {
            VectorizerError::AuthenticationError(msg) => {
                warn!("Authentication failed: {}", msg);
                return HubAuthMiddleware::error_response(
                    StatusCode::UNAUTHORIZED,
                    AuthErrorResponse::new(msg, "AUTH_INVALID_KEY"),
                );
            }
            VectorizerError::RateLimitExceeded { limit_type, limit } => {
                warn!("Rate limit exceeded: {} (limit: {})", limit_type, limit);
                return HubAuthMiddleware::error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    AuthErrorResponse::new("Too many authentication attempts", "AUTH_RATE_LIMIT"),
                );
            }
            _ => {
                error!("Unexpected authentication error: {}", e);
                return HubAuthMiddleware::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AuthErrorResponse::new("Authentication service unavailable", "AUTH_ERROR"),
                );
            }
        },
    };

    debug!(
        "Authenticated tenant: {} ({})",
        tenant_context.tenant_name, tenant_context.tenant_id
    );

    // Check rate limit quota
    match middleware
        .quota
        .check_quota(&tenant_context.tenant_id, QuotaType::RequestsPerMinute, 1)
        .await
    {
        Ok(true) => {}
        Ok(false) | Err(_) => {
            warn!(
                "Rate limit exceeded for tenant {}",
                tenant_context.tenant_id
            );
            return HubAuthMiddleware::error_response(
                StatusCode::TOO_MANY_REQUESTS,
                AuthErrorResponse::new("Rate limit exceeded", "RATE_LIMIT_EXCEEDED")
                    .with_details("Please try again later"),
            );
        }
    }

    // Attach tenant context to request
    req.extensions_mut()
        .insert(RequestTenantContext(tenant_context));

    // Continue to the handler
    next.run(req).await
}

/// Middleware function to check specific permissions
pub fn require_permission(
    permission: TenantPermission,
    req: &Request<Body>,
) -> Result<&TenantContext, Response> {
    let tenant_ctx = req
        .extensions()
        .get::<RequestTenantContext>()
        .map(|ctx| &ctx.0);

    match tenant_ctx {
        Some(ctx) if ctx.has_permission(permission) => Ok(ctx),
        Some(ctx) => {
            warn!(
                "Permission denied for tenant {}: required {:?}, has {:?}",
                ctx.tenant_id, permission, ctx.permissions
            );
            Err(HubAuthMiddleware::error_response(
                StatusCode::FORBIDDEN,
                AuthErrorResponse::new(format!("Permission required: {}", permission), "FORBIDDEN")
                    .with_details(format!(
                        "Your API key does not have {} permission",
                        permission
                    )),
            ))
        }
        None => {
            error!("No tenant context in request - authentication middleware not applied?");
            Err(HubAuthMiddleware::error_response(
                StatusCode::UNAUTHORIZED,
                AuthErrorResponse::new("Authentication required", "AUTH_REQUIRED"),
            ))
        }
    }
}

/// Extract tenant context from request extensions
pub fn extract_tenant_context(req: &Request<Body>) -> Option<&TenantContext> {
    req.extensions()
        .get::<RequestTenantContext>()
        .map(|ctx| &ctx.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_api_key_bearer() {
        let req = Request::builder()
            .header(
                AUTHORIZATION_HEADER,
                "Bearer hh_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6",
            )
            .body(Body::empty())
            .unwrap();

        let key = HubAuthMiddleware::extract_api_key(&req);
        assert_eq!(
            key,
            Some("hh_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".to_string())
        );
    }

    #[test]
    fn test_extract_api_key_x_api_key() {
        let req = Request::builder()
            .header(X_API_KEY_HEADER, "hh_test_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6")
            .body(Body::empty())
            .unwrap();

        let key = HubAuthMiddleware::extract_api_key(&req);
        assert_eq!(
            key,
            Some("hh_test_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".to_string())
        );
    }

    #[test]
    fn test_extract_api_key_none() {
        let req = Request::builder().body(Body::empty()).unwrap();
        let key = HubAuthMiddleware::extract_api_key(&req);
        assert!(key.is_none());
    }

    #[test]
    fn test_is_internal_request() {
        let internal_req = Request::builder()
            .header(X_HIVEHUB_SERVICE_HEADER, "vectorizer-manager")
            .body(Body::empty())
            .unwrap();

        assert!(HubAuthMiddleware::is_internal_request(&internal_req));

        let external_req = Request::builder().body(Body::empty()).unwrap();
        assert!(!HubAuthMiddleware::is_internal_request(&external_req));
    }

    #[test]
    fn test_auth_error_response() {
        let error = AuthErrorResponse::new("Test error", "TEST_CODE");
        assert_eq!(error.error, "Test error");
        assert_eq!(error.code, "TEST_CODE");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_auth_error_response_with_details() {
        let error =
            AuthErrorResponse::new("Test error", "TEST_CODE").with_details("Additional info");
        assert_eq!(error.error, "Test error");
        assert_eq!(error.code, "TEST_CODE");
        assert_eq!(error.details, Some("Additional info".to_string()));
    }
}
