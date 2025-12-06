//! Dashboard SPA Routing Integration Tests
//!
//! These tests verify that the dashboard SPA routing works correctly:
#![allow(unused_imports, clippy::uninlined_format_args)]
//! - Static files are served with proper cache headers
//! - SPA routes return index.html with 200 status
//! - API routes are not affected by SPA fallback

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    /// Test that the dashboard root returns index.html
    #[tokio::test]
    async fn test_dashboard_root_returns_index_html() {
        // This test requires a running server, skip if not available
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        match client.get("http://localhost:15002/dashboard/").send().await {
            Ok(response) => {
                assert_eq!(response.status(), 200);
                let content_type = response
                    .headers()
                    .get("content-type")
                    .map(|v| v.to_str().unwrap_or(""))
                    .unwrap_or("");
                assert!(
                    content_type.contains("text/html"),
                    "Expected text/html, got {}",
                    content_type
                );
            }
            Err(_) => {
                // Server not running, skip test
                println!("Skipping test: server not running on port 15002");
            }
        }
    }

    /// Test that SPA routes return index.html (not 404)
    #[tokio::test]
    async fn test_spa_routes_return_index_html() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let spa_routes = vec![
            "/dashboard/collections",
            "/dashboard/search",
            "/dashboard/settings",
            "/dashboard/vectors",
            "/dashboard/graph",
            "/dashboard/file-watcher",
            "/dashboard/logs",
            "/dashboard/connections",
            "/dashboard/backups",
            "/dashboard/workspace",
            "/dashboard/collections/test/vectors",
            "/dashboard/deeply/nested/route/that/does/not/exist",
        ];

        for route in spa_routes {
            match client
                .get(format!("http://localhost:15002{}", route))
                .send()
                .await
            {
                Ok(response) => {
                    assert_eq!(
                        response.status(),
                        200,
                        "Route {} should return 200, got {}",
                        route,
                        response.status()
                    );

                    let body = response.text().await.unwrap_or_default();
                    assert!(
                        body.contains("<!doctype html>") || body.contains("<!DOCTYPE html>"),
                        "Route {} should return HTML, got: {}...",
                        route,
                        &body[..100.min(body.len())]
                    );
                }
                Err(_) => {
                    println!("Skipping test: server not running on port 15002");
                    return;
                }
            }
        }
    }

    /// Test that static assets are served correctly (not index.html)
    #[tokio::test]
    async fn test_static_assets_served_correctly() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        // Test favicon
        match client
            .get("http://localhost:15002/dashboard/favicon.ico")
            .send()
            .await
        {
            Ok(response) => {
                assert_eq!(response.status(), 200);
                let content_type = response
                    .headers()
                    .get("content-type")
                    .map(|v| v.to_str().unwrap_or(""))
                    .unwrap_or("");
                // favicon can be image/x-icon, image/vnd.microsoft.icon, or application/octet-stream
                assert!(
                    content_type.contains("icon")
                        || content_type.contains("octet-stream")
                        || content_type.contains("image"),
                    "Expected icon content type, got {}",
                    content_type
                );
            }
            Err(_) => {
                println!("Skipping test: server not running on port 15002");
            }
        }
    }

    /// Test that assets have proper cache headers
    #[tokio::test]
    async fn test_assets_have_cache_headers() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        // First, get the index.html to find an asset URL
        match client.get("http://localhost:15002/dashboard/").send().await {
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();

                // Find a JS asset URL in the HTML
                if let Some(start) = body.find("/dashboard/assets/js/") {
                    let end = body[start..].find('"').unwrap_or(50);
                    let asset_path = &body[start..start + end];

                    // Request the asset and check cache headers
                    if let Ok(asset_response) = client
                        .get(format!("http://localhost:15002{}", asset_path))
                        .send()
                        .await
                    {
                        assert_eq!(asset_response.status(), 200);

                        let cache_control = asset_response
                            .headers()
                            .get("cache-control")
                            .map(|v| v.to_str().unwrap_or(""))
                            .unwrap_or("");

                        assert!(
                            cache_control.contains("max-age=31536000"),
                            "Assets should have 1 year max-age, got: {}",
                            cache_control
                        );
                        assert!(
                            cache_control.contains("immutable"),
                            "Assets should be immutable, got: {}",
                            cache_control
                        );
                    }
                }
            }
            Err(_) => {
                println!("Skipping test: server not running on port 15002");
            }
        }
    }

    /// Test that SPA routes have no-cache headers
    #[tokio::test]
    async fn test_spa_routes_have_no_cache_headers() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let routes = vec!["/dashboard/", "/dashboard/collections", "/dashboard/search"];

        for route in routes {
            match client
                .get(format!("http://localhost:15002{}", route))
                .send()
                .await
            {
                Ok(response) => {
                    let cache_control = response
                        .headers()
                        .get("cache-control")
                        .map(|v| v.to_str().unwrap_or(""))
                        .unwrap_or("");

                    assert!(
                        cache_control.contains("no-cache") || cache_control.contains("no-store"),
                        "Route {} should have no-cache, got: {}",
                        route,
                        cache_control
                    );
                }
                Err(_) => {
                    println!("Skipping test: server not running on port 15002");
                    return;
                }
            }
        }
    }

    /// Test that API routes are not affected by dashboard fallback
    #[tokio::test]
    async fn test_api_routes_not_affected() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        // Health endpoint should return JSON, not HTML
        match client.get("http://localhost:15002/health").send().await {
            Ok(response) => {
                assert_eq!(response.status(), 200);
                let content_type = response
                    .headers()
                    .get("content-type")
                    .map(|v| v.to_str().unwrap_or(""))
                    .unwrap_or("");
                assert!(
                    content_type.contains("application/json"),
                    "Health endpoint should return JSON, got {}",
                    content_type
                );
            }
            Err(_) => {
                println!("Skipping test: server not running on port 15002");
            }
        }

        // Collections API endpoint should return JSON
        match client
            .get("http://localhost:15002/collections")
            .send()
            .await
        {
            Ok(response) => {
                // Should return 200 or 401 (if auth required), not redirect to dashboard
                assert!(
                    response.status() == 200 || response.status() == 401,
                    "Collections API should return 200 or 401, got {}",
                    response.status()
                );
            }
            Err(_) => {
                println!("Skipping test: server not running on port 15002");
            }
        }
    }

    /// Test edge cases: long URLs
    #[tokio::test]
    async fn test_long_urls() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        // Very long path
        let long_segment = "a".repeat(200);
        let long_url = format!(
            "http://localhost:15002/dashboard/{}/{}/{}",
            long_segment, long_segment, long_segment
        );

        match client.get(&long_url).send().await {
            Ok(response) => {
                // Should return 200 with index.html (SPA handles the route)
                // or 414 URI Too Long (which is also acceptable)
                assert!(
                    response.status() == 200 || response.status() == 414,
                    "Long URL should return 200 or 414, got {}",
                    response.status()
                );
            }
            Err(_) => {
                println!("Skipping test: server not running on port 15002");
            }
        }
    }

    /// Test edge cases: special characters in URLs
    #[tokio::test]
    async fn test_special_characters_in_urls() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let special_routes = vec![
            "/dashboard/search?q=hello%20world",
            "/dashboard/collections#section",
            "/dashboard/test%2Fpath",
        ];

        for route in special_routes {
            match client
                .get(format!("http://localhost:15002{}", route))
                .send()
                .await
            {
                Ok(response) => {
                    // Should return 200 (SPA handles routing)
                    // or 400 if the URL is malformed
                    assert!(
                        response.status() == 200 || response.status() == 400,
                        "Route {} should return 200 or 400, got {}",
                        route,
                        response.status()
                    );
                }
                Err(_) => {
                    println!("Skipping test: server not running on port 15002");
                    return;
                }
            }
        }
    }

    /// Test trailing slash handling
    #[tokio::test]
    async fn test_trailing_slash_handling() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let routes = vec![
            ("/dashboard", "/dashboard/"),
            ("/dashboard/collections", "/dashboard/collections/"),
        ];

        for (without_slash, with_slash) in routes {
            let resp1 = client
                .get(format!("http://localhost:15002{}", without_slash))
                .send()
                .await;
            let resp2 = client
                .get(format!("http://localhost:15002{}", with_slash))
                .send()
                .await;

            match (resp1, resp2) {
                (Ok(r1), Ok(r2)) => {
                    // Both should work (return 200)
                    assert!(
                        r1.status() == 200 || r1.status() == 301 || r1.status() == 308,
                        "Route {} should work, got {}",
                        without_slash,
                        r1.status()
                    );
                    assert_eq!(
                        r2.status(),
                        200,
                        "Route {} should return 200, got {}",
                        with_slash,
                        r2.status()
                    );
                }
                _ => {
                    println!("Skipping test: server not running on port 15002");
                    return;
                }
            }
        }
    }
}
