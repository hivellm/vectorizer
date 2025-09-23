//! Authentication middleware for Axum
//! 
//! Provides middleware for JWT and API key authentication

use crate::auth::{AuthManager, UserClaims};
// Result type is used in function signatures
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
};
use serde::Serialize;
use std::sync::Arc;

/// Authentication state for request context
#[derive(Debug, Clone)]
pub struct AuthState {
    /// User claims from authentication
    pub user_claims: UserClaims,
    /// Whether the request was authenticated
    pub authenticated: bool,
}

/// Authentication middleware
pub struct AuthMiddleware;

impl AuthMiddleware {
    /// Create authentication middleware
    pub fn new() -> Self {
        Self
    }

    /// Extract authentication information from request
    pub async fn extract_auth(
        State(auth_manager): State<Arc<AuthManager>>,
        request: Request,
        next: Next,
    ) -> std::result::Result<Response, StatusCode> {
        let auth_state = Self::authenticate_request(&auth_manager, &request).await;
        
        // Add auth state to request extensions
        let mut request = request;
        request.extensions_mut().insert(auth_state);
        
        Ok(next.run(request).await)
    }

    /// Authenticate a request using JWT or API key
    async fn authenticate_request(
        auth_manager: &AuthManager,
        request: &Request,
    ) -> AuthState {
        // Try to get authorization header
        if let Some(auth_header) = request.headers().get(AUTHORIZATION)
            && let Ok(auth_str) = auth_header.to_str() {
            // Check for Bearer token (JWT)
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = auth_manager.validate_jwt(token) {
                    return AuthState {
                        user_claims: claims,
                        authenticated: true,
                    };
                }
            }
            
            // Check for API key (direct token)
            if let Ok(claims) = auth_manager.validate_api_key(auth_str).await {
                return AuthState {
                    user_claims: claims,
                    authenticated: true,
                };
            }
        }

        // Check for API key in query parameters
        if let Some(query) = request.uri().query() {
            for param in query.split('&') {
                if let Some(api_key) = param.strip_prefix("api_key=") {
                    if let Ok(claims) = auth_manager.validate_api_key(api_key).await {
                        return AuthState {
                            user_claims: claims,
                            authenticated: true,
                        };
                    }
                }
            }
        }

        // No authentication found
        AuthState {
            user_claims: UserClaims {
                user_id: "anonymous".to_string(),
                username: "anonymous".to_string(),
                roles: vec![],
                iat: 0,
                exp: 0,
            },
            authenticated: false,
        }
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self { Self::new() }
}

/// Require authentication middleware
pub struct RequireAuthMiddleware;

impl RequireAuthMiddleware {
    /// Require authentication for protected routes
    pub async fn require_auth(
        State(auth_manager): State<Arc<AuthManager>>,
        request: Request,
        next: Next,
    ) -> Response {
        let auth_state = AuthMiddleware::authenticate_request(&auth_manager, &request).await;
        
        if !auth_state.authenticated {
            return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
        }
        
        // Add auth state to request extensions
        let mut request = request;
        request.extensions_mut().insert(auth_state);
        
        next.run(request).await
    }
}

/// Require specific role middleware
pub struct RequireRoleMiddleware;

impl RequireRoleMiddleware {
    /// Require specific role for access
    pub async fn require_role(
        State(auth_manager): State<Arc<AuthManager>>,
        State(required_role): State<crate::auth::roles::Role>,
        request: Request,
        next: Next,
    ) -> Response {
        let auth_state = AuthMiddleware::authenticate_request(&auth_manager, &request).await;
        
        if !auth_state.authenticated {
            return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
        }
        
        if !auth_state.user_claims.roles.contains(&required_role) {
            return (StatusCode::FORBIDDEN, "Insufficient permissions").into_response();
        }
        
        // Add auth state to request extensions
        let mut request = request;
        request.extensions_mut().insert(auth_state);
        
        next.run(request).await
    }
}

/// Require specific permission middleware
pub struct RequirePermissionMiddleware;

impl RequirePermissionMiddleware {
    /// Require specific permission for access
    pub async fn require_permission(
        State(auth_manager): State<Arc<AuthManager>>,
        State(required_permission): State<crate::auth::roles::Permission>,
        request: Request,
        next: Next,
    ) -> Response {
        let auth_state = AuthMiddleware::authenticate_request(&auth_manager, &request).await;
        
        if !auth_state.authenticated {
            return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
        }
        
        let has_permission = auth_state.user_claims.roles.iter()
            .any(|role| role.has_permission(&required_permission));
        
        if !has_permission {
            return (StatusCode::FORBIDDEN, "Insufficient permissions").into_response();
        }
        
        // Add auth state to request extensions
        let mut request = request;
        request.extensions_mut().insert(auth_state);
        
        next.run(request).await
    }
}

/// Rate limiting middleware
pub struct RateLimitMiddleware;

impl RateLimitMiddleware {
    /// Apply rate limiting to requests
    pub async fn rate_limit(
        State(_auth_manager): State<Arc<AuthManager>>,
        request: Request,
        next: Next,
    ) -> Response {
        // For now, just pass through (rate limiting configured in AuthManager)
        // In future, inspect headers/IP/etc.
        next.run(request).await
    }
}

/// Authentication error response
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
}

impl AuthErrorResponse {
    /// Create an authentication error response
    pub fn unauthorized() -> Self {
        Self {
            error: "Unauthorized".to_string(),
            message: "Authentication required".to_string(),
            code: 401,
        }
    }

    /// Create a forbidden error response
    pub fn forbidden() -> Self {
        Self {
            error: "Forbidden".to_string(),
            message: "Insufficient permissions".to_string(),
            code: 403,
        }
    }

    /// Create a rate limit error response
    pub fn rate_limited() -> Self {
        Self {
            error: "Rate Limited".to_string(),
            message: "Too many requests".to_string(),
            code: 429,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthConfig, AuthManager};
    use crate::auth::roles::{Role, Permission};
    use axum::{
        body::Body,
        http::Request,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_auth_middleware_jwt() {
        let config = AuthConfig::default();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        
        let token = auth_manager
            .generate_jwt("user123", "testuser", vec![Role::User])
            .unwrap();
        
        let request = Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        
        let auth_state = AuthMiddleware::authenticate_request(&auth_manager, &request).await;
        
        assert!(auth_state.authenticated);
        assert_eq!(auth_state.user_claims.user_id, "user123");
        assert_eq!(auth_state.user_claims.username, "testuser");
    }

    #[tokio::test]
    async fn test_auth_middleware_api_key() {
        let config = AuthConfig::default();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        
        let (api_key, _) = auth_manager
            .create_api_key("user123", "test_key", vec![Permission::Read], None)
            .await
            .unwrap();
        
        let request = Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, api_key)
            .body(Body::empty())
            .unwrap();
        
        let auth_state = AuthMiddleware::authenticate_request(&auth_manager, &request).await;
        
        assert!(auth_state.authenticated);
        assert_eq!(auth_state.user_claims.user_id, "user123");
    }

    #[tokio::test]
    async fn test_auth_middleware_no_auth() {
        let config = AuthConfig::default();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        
        let auth_state = AuthMiddleware::authenticate_request(&auth_manager, &request).await;
        
        assert!(!auth_state.authenticated);
        assert_eq!(auth_state.user_claims.user_id, "anonymous");
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_token() {
        let config = AuthConfig::default();
        let auth_manager = Arc::new(AuthManager::new(config).unwrap());
        
        let request = Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();
        
        let auth_state = AuthMiddleware::authenticate_request(&auth_manager, &request).await;
        
        assert!(!auth_state.authenticated);
        assert_eq!(auth_state.user_claims.user_id, "anonymous");
    }

    #[test]
    fn test_auth_error_responses() {
        let unauthorized = AuthErrorResponse::unauthorized();
        assert_eq!(unauthorized.code, 401);
        assert_eq!(unauthorized.error, "Unauthorized");

        let forbidden = AuthErrorResponse::forbidden();
        assert_eq!(forbidden.code, 403);
        assert_eq!(forbidden.error, "Forbidden");

        let rate_limited = AuthErrorResponse::rate_limited();
        assert_eq!(rate_limited.code, 429);
        assert_eq!(rate_limited.error, "Rate Limited");
    }
}
