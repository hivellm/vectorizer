# Dashboard SPA Routing Fix - Technical Design

## Problem Analysis

### Why SPAs Need Fallback Routing

Single Page Applications (SPAs) like our React dashboard use **client-side routing**:

```
Traditional Multi-Page App (MPA):
  /collections → Server serves collections.html
  /search      → Server serves search.html
  /settings    → Server serves settings.html

Single Page App (SPA):
  /collections → Server serves index.html → React Router → Collections component
  /search      → Server serves index.html → React Router → Search component  
  /settings    → Server serves index.html → React Router → Settings component
```

**The Problem**:
When user refreshes or accesses directly, browser asks server for the file at that path. Server doesn't have a file called "collections" → returns 404.

**The Solution**:
Server should serve `index.html` for all non-file routes, letting React Router handle the routing.

## Implementation

### Option 1: Axum Fallback Handler (Recommended)

```rust
// src/server/dashboard.rs (NEW or modified)

use axum::{
    Router,
    response::{IntoResponse, Response},
    http::{StatusCode, Request, header, HeaderValue},
    body::Body,
};
use tower_http::services::{ServeDir, ServeFile};

/// Create dashboard router with SPA fallback support
pub fn create_dashboard_router(dist_path: &str) -> Router {
    Router::new()
        // Serve static files from dist/ directory
        .nest_service("/", ServeDir::new(dist_path))
        // Fallback to index.html for SPA routes
        .fallback(spa_fallback_handler)
}

/// Fallback handler for SPA routing
async fn spa_fallback_handler() -> impl IntoResponse {
    let index_path = "dashboard/dist/index.html";
    
    match tokio::fs::read(index_path).await {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                // CRITICAL: Don't cache index.html
                .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                .header(header::PRAGMA, "no-cache")
                .header(header::EXPIRES, "0")
                .body(Body::from(content))
                .unwrap()
        }
        Err(e) => {
            tracing::error!("Failed to read index.html: {}", e);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Dashboard not found"))
                .unwrap()
        }
    }
}
```

### Option 2: Manual Route Filtering (More Control)

```rust
async fn spa_fallback_handler(req: Request<Body>) -> impl IntoResponse {
    let path = req.uri().path();
    
    // Don't serve index.html for API routes
    if path.starts_with("/api/") || path.starts_with("/prometheus/") {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }
    
    // Don't serve index.html for specific file extensions
    if path.ends_with(".js") || path.ends_with(".css") || 
       path.ends_with(".png") || path.ends_with(".ico") ||
       path.ends_with(".svg") || path.ends_with(".woff2") {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }
    
    // Serve index.html for everything else
    serve_index_html().await
}
```

### Option 3: Nginx Reverse Proxy (Production Deployment)

For production deployments behind nginx:

```nginx
server {
    listen 80;
    server_name vectorizer.example.com;
    
    root /var/www/vectorizer/dashboard/dist;
    index index.html;
    
    # API routes - proxy to backend
    location /api/ {
        proxy_pass http://localhost:15002;
    }
    
    location /prometheus/ {
        proxy_pass http://localhost:15002;
    }
    
    # Static assets - serve with long cache
    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # SPA fallback - serve index.html for all other routes
    location / {
        try_files $uri $uri/ /index.html;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }
}
```

## Router Integration

### Current Server Structure (src/server/mod.rs)

```rust
// Current implementation (BEFORE fix)
let app = Router::new()
    // API routes
    .route("/api/collections", post(create_collection))
    .route("/api/collections/:name/search", post(search_collection))
    // ... more API routes
    
    // Prometheus metrics
    .route("/prometheus/metrics", get(metrics_handler))
    
    // Dashboard (BROKEN - no fallback)
    .nest_service("/", ServeDir::new("dashboard/dist"));
```

### Fixed Server Structure (AFTER)

```rust
let app = Router::new()
    // IMPORTANT: API routes MUST come before dashboard
    // This ensures /api/* routes are matched first
    .route("/api/collections", post(create_collection))
    .route("/api/collections/:name/search", post(search_collection))
    // ... more API routes
    
    // Prometheus metrics (before dashboard)
    .route("/prometheus/metrics", get(metrics_handler))
    
    // Dashboard with SPA fallback (FIXED)
    .nest_service("/", ServeDir::new("dashboard/dist"))
    .fallback(spa_fallback_handler);  // <-- ADD THIS
```

**Critical**: API routes MUST be registered before dashboard to avoid being caught by fallback.

## Request Flow Diagram

```
HTTP Request
    │
    ▼
┌─────────────────────────────────────┐
│  Does path start with /api/ ?       │
│  Yes → Route to API handler         │
│  No  → Continue                     │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Does path start with /prometheus/? │
│  Yes → Route to metrics handler     │
│  No  → Continue                     │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Does file exist in dist/ ?         │
│  Yes → Serve file (assets, favicon)│
│  No  → Continue                     │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Fallback: Serve index.html         │
│  (SPA routing)                      │
└─────────────────────────────────────┘
```

## Caching Strategy

### index.html (SPA entry point)
```
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
Expires: 0

Why: Ensures users always get latest app version
```

### Hashed Assets (index-abc123.js)
```
Cache-Control: public, max-age=31536000, immutable

Why: Vite generates new hashes on changes, safe to cache forever
```

### Other Assets (favicon.ico, etc.)
```
Cache-Control: public, max-age=86400

Why: Reasonable 24-hour cache, not critical if stale
```

## Vite Configuration

Ensure Vite is configured correctly for SPA routing:

**dashboard/vite.config.ts**:
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  
  // Base path (use '/' for root deployment)
  base: '/',
  
  // Build output
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    
    // Generate hashed filenames for cache busting
    rollupOptions: {
      output: {
        entryFileNames: 'assets/[name]-[hash].js',
        chunkFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash].[ext]',
      },
    },
  },
  
  // Dev server (not needed for this bug, but good to have)
  server: {
    port: 3000,
    // Vite dev server handles SPA routing automatically
  },
});
```

## React Router Configuration

Verify React Router is using BrowserRouter:

**dashboard/src/App.tsx**:
```typescript
import { BrowserRouter, Routes, Route } from 'react-router-dom';

function App() {
  return (
    <BrowserRouter>  {/* NOT HashRouter! */}
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/collections" element={<Collections />} />
        <Route path="/search" element={<Search />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="*" element={<NotFound />} />  {/* Client-side 404 */}
      </Routes>
    </BrowserRouter>
  );
}
```

**Important**: Use `BrowserRouter`, NOT `HashRouter`. HashRouter would use URLs like `/#/collections` which is a workaround we don't need.

## Testing

### Manual Testing Steps

```bash
# 1. Build dashboard
cd dashboard && npm run build

# 2. Start server
cd .. && cargo run

# 3. Test scenarios
# Open http://localhost:3000/collections
# Press F5 → Should show collections page (not 404)

# Open http://localhost:3000/search
# Press F5 → Should show search page (not 404)

# Direct access: Paste http://localhost:3000/settings in browser
# Should show settings page (not 404)
```

### Automated Testing

```rust
#[tokio::test]
async fn test_spa_fallback_serves_index_html() {
    let app = create_test_app();
    
    let response = app
        .oneshot(Request::builder()
            .uri("/collections")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    
    // Verify it's HTML (contains <!DOCTYPE html>)
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<div id=\"root\">"));
}

#[tokio::test]
async fn test_api_routes_not_affected() {
    let app = create_test_app();
    
    let response = app
        .oneshot(Request::builder()
            .uri("/api/collections")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    // API should return JSON, not HTML
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}
```

## Common Pitfalls

### ❌ Wrong: Using HashRouter
```typescript
// DON'T DO THIS
<HashRouter>  {/* URLs become /#/collections */}
  <Routes>...</Routes>
</HashRouter>
```

### ✅ Correct: Using BrowserRouter
```typescript
// DO THIS
<BrowserRouter>  {/* Clean URLs: /collections */}
  <Routes>...</Routes>
</BrowserRouter>
```

### ❌ Wrong: Fallback Before API Routes
```rust
// DON'T DO THIS - fallback catches everything
Router::new()
    .fallback(spa_fallback_handler)  // ❌ This catches /api/* too!
    .route("/api/collections", get(list_collections))
```

### ✅ Correct: API Routes Before Fallback
```rust
// DO THIS - API routes registered first
Router::new()
    .route("/api/collections", get(list_collections))  // ✅ Matched first
    .fallback(spa_fallback_handler)  // ✅ Only for unmatched routes
```

## Rollback Plan

If issues occur after implementing:

1. **Immediate**: Comment out `.fallback()` line
2. **Short-term**: Document routes that need manual access
3. **Long-term**: Fix implementation and re-deploy

No data loss risk - this is purely a routing change.

