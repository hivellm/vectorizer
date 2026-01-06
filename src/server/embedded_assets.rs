//! Embedded Dashboard Assets
//!
//! This module embeds the dashboard static files into the binary using rust-embed.
//! This allows distributing a single binary without external dependencies.

use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use rust_embed::Embed;

/// Embedded dashboard assets from dashboard/dist
#[derive(Embed)]
#[folder = "dashboard/dist"]
#[prefix = ""]
pub struct DashboardAssets;

/// Serve an embedded file by path - returns an owned response
fn serve_file(path: &str) -> Response<Body> {
    // Remove leading slash if present
    let path = path.strip_prefix('/').unwrap_or(path);

    // Try to get the file
    match DashboardAssets::get(path) {
        Some(content) => {
            // Determine content type
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            // Build response with appropriate headers
            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref());

            // Add cache headers based on file type
            if path.starts_with("assets/") {
                // Fingerprinted assets: cache for 1 year
                builder = builder.header(
                    header::CACHE_CONTROL,
                    "public, max-age=31536000, immutable",
                );
            } else if path == "index.html" || path.is_empty() {
                // HTML: no cache
                builder = builder.header(
                    header::CACHE_CONTROL,
                    "no-cache, no-store, must-revalidate",
                );
            } else {
                // Other files: cache for 1 hour
                builder = builder.header(header::CACHE_CONTROL, "public, max-age=3600");
            }

            builder
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => {
            // File not found - for SPA routing, return index.html
            if !path.contains('.') || path.ends_with(".html") {
                // This is likely a route, serve index.html
                if let Some(index) = DashboardAssets::get("index.html") {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                        .body(Body::from(index.data.into_owned()))
                        .unwrap();
                }
            }

            // Return 404
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from("Not Found"))
                .unwrap()
        }
    }
}

/// Handler for dashboard routes
pub async fn dashboard_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    serve_file(&path)
}

/// Handler for dashboard root
pub async fn dashboard_root_handler() -> Response<Body> {
    serve_file("index.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_html_exists() {
        let index = DashboardAssets::get("index.html");
        assert!(index.is_some(), "index.html should be embedded");
    }

    #[test]
    fn test_assets_exist() {
        // Check that at least some assets exist
        let files: Vec<_> = DashboardAssets::iter().collect();
        assert!(!files.is_empty(), "Dashboard assets should not be empty");

        // Check for expected asset directories
        let has_js = files.iter().any(|f| f.contains("assets/js/"));
        let has_css = files.iter().any(|f| f.contains("assets/css/"));
        assert!(has_js, "Should have JS assets");
        assert!(has_css, "Should have CSS assets");
    }

    #[test]
    fn test_serve_file_returns_correct_content_type() {
        let response = serve_file("index.html");
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let content_type = response.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("text/html"));
    }

    #[test]
    fn test_spa_fallback() {
        // Non-existent route should return index.html
        let response = serve_file("some/spa/route");
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
