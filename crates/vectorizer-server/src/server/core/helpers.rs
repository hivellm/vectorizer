//! Shared helper functions used by the bootstrap/routing/grpc modules.
//!
//! - [`extract_auth_credentials`] / [`check_mcp_auth_with_credentials`]
//!   — header parsing + validation used by the global auth middleware
//!   and (historically) the MCP entrypoint.
//! - [`security_headers_middleware`] — adds standard security headers
//!   (CSP, X-Frame-Options, etc.) to every response.
//! - [`get_file_watcher_metrics`] — the `/metrics` REST handler that
//!   exposes File Watcher metrics to the dashboard.

use std::sync::Arc;

use axum::extract::State;
use axum::response::Json;

use crate::server::ServerState;
use vectorizer::file_watcher::FileWatcherMetrics;

/// Extract auth credentials from request headers (sync part)
/// Returns (Option<jwt_token>, Option<api_key>)
pub(super) fn extract_auth_credentials(
    req: &axum::extract::Request,
) -> (Option<String>, Option<String>) {
    use axum::http::header::AUTHORIZATION;

    let mut jwt_token: Option<String> = None;
    let mut api_key: Option<String> = None;

    // Try to get authorization header
    if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            // Check for Bearer token (JWT)
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                jwt_token = Some(token.to_string());
            } else {
                // Try as direct API key
                api_key = Some(auth_str.to_string());
            }
        }
    }

    // Check for X-API-Key header (if no API key found yet)
    if api_key.is_none() {
        if let Some(api_key_header) = req.headers().get("X-API-Key") {
            if let Ok(key) = api_key_header.to_str() {
                api_key = Some(key.to_string());
            }
        }
    }

    // Check for API key in query parameters (if no API key found yet)
    if api_key.is_none() {
        if let Some(query) = req.uri().query() {
            for param in query.split('&') {
                if let Some(key) = param.strip_prefix("api_key=") {
                    api_key = Some(key.to_string());
                    break;
                }
            }
        }
    }

    (jwt_token, api_key)
}

/// Check authentication for MCP/UMICP requests in production mode
/// Returns true if authentication is valid, false otherwise.
///
/// Currently unused — the MCP auth guard is commented out in
/// [`super::routing::create_mcp_router`] — but kept because the guard
/// is planned to come back once the `.route_layer` shape for MCP is
/// finalized.
#[allow(dead_code)]
pub(super) async fn check_mcp_auth_with_credentials(
    jwt_token: Option<String>,
    api_key: Option<String>,
    auth_manager: &std::sync::Arc<vectorizer::auth::AuthManager>,
) -> bool {
    // Try JWT first
    if let Some(token) = jwt_token {
        if auth_manager.validate_jwt(&token).is_ok() {
            return true;
        }
    }

    // Try API key
    if let Some(key) = api_key {
        if auth_manager.validate_api_key(&key).await.is_ok() {
            return true;
        }
    }

    false
}

/// Security headers middleware
///
/// Adds standard security headers to all responses:
/// - X-Content-Type-Options: nosniff — prevents MIME type sniffing
/// - X-Frame-Options: SAMEORIGIN — limits clickjacking while allowing
///   dashboard framing
/// - X-XSS-Protection: 1; mode=block — XSS filter (legacy browsers)
/// - Content-Security-Policy: allows Monaco Editor CDN resources
///   (scripts, styles, workers, source maps) for the embedded dashboard
/// - Referrer-Policy: strict-origin-when-cross-origin
/// - Permissions-Policy: disables geolocation / camera / microphone /
///   payment APIs
// Every `.parse().unwrap()` below converts a static `&'static str` literal
// into `HeaderValue`; a malformed value would be a code-edit bug surfaced
// at the first request, not a runtime condition. Function-scoped allow
// keeps `unwrap_used = "deny"` enforceable everywhere else
// (phase4_enforce-no-unwrap-policy).
#[allow(clippy::unwrap_used)]
pub(super) async fn security_headers_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    // Prevent MIME type sniffing
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    // Prevent clickjacking (allow framing for dashboard)
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        "SAMEORIGIN".parse().unwrap(),
    );

    // XSS protection for legacy browsers
    headers.insert(
        axum::http::HeaderName::from_static("x-xss-protection"),
        "1; mode=block".parse().unwrap(),
    );

    // Content Security Policy — Monaco Editor requires:
    // - script-src: CDN for editor scripts
    // - style-src: CDN for editor styles
    // - connect-src: CDN for source maps
    // - worker-src: blob: and CDN for web workers
    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; style-src-elem 'self' 'unsafe-inline' https://cdn.jsdelivr.net; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' ws: wss: https://cdn.jsdelivr.net; worker-src 'self' blob: https://cdn.jsdelivr.net".parse().unwrap(),
    );

    headers.insert(
        axum::http::header::REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    headers.insert(
        axum::http::HeaderName::from_static("permissions-policy"),
        "geolocation=(), microphone=(), camera=(), payment=()"
            .parse()
            .unwrap(),
    );

    response
}

/// Get File Watcher metrics endpoint
pub async fn get_file_watcher_metrics(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<FileWatcherMetrics>, crate::server::error_middleware::ErrorResponse> {
    // Get the file watcher system from the state
    let watcher_lock = state.file_watcher_system.lock().await;

    if let Some(watcher_system) = watcher_lock.as_ref() {
        let metrics = watcher_system.get_metrics().await;
        return Ok(Json(metrics));
    }

    // Return empty/default metrics if File Watcher is not available
    use std::collections::HashMap;

    use vectorizer::file_watcher::metrics::*;

    let default_metrics = FileWatcherMetrics {
        timing: TimingMetrics {
            avg_file_processing_ms: 0.0,
            avg_discovery_ms: 0.0,
            avg_sync_ms: 0.0,
            uptime_seconds: 0,
            last_activity: None,
            peak_processing_ms: 0,
        },
        files: FileMetrics {
            total_files_processed: 0,
            files_processed_success: 0,
            files_processed_error: 0,
            files_skipped: 0,
            files_in_progress: 0,
            files_discovered: 0,
            files_removed: 0,
            files_indexed_realtime: 0,
        },
        system: SystemMetrics {
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            thread_count: 0,
            active_file_handles: 0,
            disk_io_ops_per_sec: 0,
            network_io_bytes_per_sec: 0,
        },
        network: NetworkMetrics {
            total_api_requests: 0,
            successful_api_requests: 0,
            failed_api_requests: 0,
            avg_api_response_ms: 0.0,
            peak_api_response_ms: 0,
            active_connections: 0,
        },
        status: StatusMetrics {
            total_errors: 0,
            errors_by_type: HashMap::new(),
            current_status: "initializing".to_string(),
            last_error: None,
            health_score: 0,
            restart_count: 0,
        },
        collections: HashMap::new(),
    };

    Ok(Json(default_metrics))
}
