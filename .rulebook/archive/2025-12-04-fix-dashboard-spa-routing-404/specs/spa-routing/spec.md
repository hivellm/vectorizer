# SPA Routing Fix Specification

## ADDED Requirements

### Requirement: SPA Fallback Routing
The dashboard server SHALL serve index.html for all non-file routes to support client-side routing.

##### Scenario: Refresh on Dashboard Route
Given a user navigates to /collections via the UI
When the user presses F5 (refresh)
Then the server MUST serve index.html
And React Router MUST handle the /collections route client-side
And the user MUST see the collections page (not 404)

##### Scenario: Direct URL Access
Given a user enters URL http://localhost:3000/search directly in browser
When the browser requests the page
Then the server MUST serve index.html
And React Router MUST navigate to /search route
And the user MUST see the search page (not 404)

##### Scenario: Browser Back Button
Given a user navigates: Dashboard → Collections → Search
When the user presses browser back button
Then the browser MUST request previous URL
And the server MUST serve index.html
And React Router MUST restore previous route
And the user MUST see the correct page (not 404)

### Requirement: Route Priority
The server SHALL apply route priority to distinguish between files, API endpoints, and SPA routes.

##### Scenario: Static Asset Request
Given a request for /assets/index-abc123.js
When the server processes the request
Then the server MUST serve the JavaScript file
And the server MUST NOT serve index.html
And the server MUST set correct Content-Type: application/javascript

##### Scenario: API Endpoint Request
Given a request for /api/collections
When the server processes the request
Then the server MUST route to API handler
And the server MUST NOT serve index.html
And the server MUST return JSON response

##### Scenario: Prometheus Metrics Request
Given a request for /prometheus/metrics
When the server processes the request
Then the server MUST route to metrics handler
And the server MUST NOT serve index.html
And the server MUST return Prometheus text format

##### Scenario: Favicon Request
Given a request for /favicon.ico
When the server processes the request
Then the server MUST serve the favicon file
And the server MUST NOT serve index.html

##### Scenario: Non-Existent SPA Route
Given a request for /nonexistent-route
When the server processes the request
Then the server MUST serve index.html
And React Router MUST display 404 page (client-side)
And the HTTP response code MUST be 200 (not 404)

### Requirement: Query Parameters and Fragments
The server SHALL preserve query parameters and URL fragments in SPA fallback.

##### Scenario: Route with Query Parameters
Given a request for /search?q=test&limit=10
When the server serves index.html
Then React Router MUST receive the full URL
And React Router MUST parse query parameters correctly
And the search page MUST use the query parameters

##### Scenario: Route with URL Fragment
Given a request for /collections#section-2
When the server serves index.html
Then the browser MUST preserve the #section-2 fragment
And React Router MUST handle the fragment correctly

### Requirement: Caching Headers
The server SHALL set appropriate caching headers for SPA fallback and static assets.

##### Scenario: index.html Caching
Given a request that serves index.html (fallback)
When the server sends the response
Then the response MUST include header "Cache-Control: no-cache, no-store, must-revalidate"
And the response MUST include header "Pragma: no-cache"
And the response MUST include header "Expires: 0"
And this ensures fresh index.html on every request

##### Scenario: Static Asset Caching
Given a request for /assets/index-abc123.js (hashed filename)
When the server sends the response
Then the response MUST include header "Cache-Control: public, max-age=31536000, immutable"
And this enables long-term caching for hashed assets

## MODIFIED Requirements

### Requirement: Dashboard Static File Serving
**BEFORE**: Server used simple directory serving without fallback handling.

**AFTER**: Server SHALL use layered routing with SPA fallback support.

##### Delta: Router Configuration
```rust
// BEFORE (broken - returns 404 on refresh)
let dashboard_router = Router::new()
    .nest_service("/", ServeDir::new("dashboard/dist"));

// AFTER (fixed - SPA fallback)
let dashboard_router = Router::new()
    .nest_service("/", ServeDir::new("dashboard/dist"))
    .fallback(spa_fallback_handler);

async fn spa_fallback_handler() -> impl IntoResponse {
    match ServeFile::new("dashboard/dist/index.html").oneshot(Request::new(Body::empty())).await {
        Ok(response) => {
            // Add no-cache headers
            let mut response = response.into_response();
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-cache, no-store, must-revalidate"),
            );
            response
        }
        Err(_) => {
            (StatusCode::NOT_FOUND, "Dashboard not found").into_response()
        }
    }
}
```

## Performance Requirements

### Latency
- SPA fallback MUST respond within 1ms
- Static file serving MUST respond within 5ms
- No performance regression vs current implementation

### Caching
- index.html MUST NOT be cached (always fresh)
- Static assets (with hashed names) MUST be cached for 1 year
- Browser MUST revalidate index.html on every navigation

## Testing Requirements

### Manual Testing Checklist
- [ ] Navigate to /collections via UI, press F5 → Works
- [ ] Direct access to /search → Works
- [ ] Bookmark /settings, open bookmark → Works
- [ ] Navigate back/forward → Works
- [ ] API routes still work (/api/collections)
- [ ] Static assets load correctly (/assets/*.js)
- [ ] Metrics endpoint works (/prometheus/metrics)

### Automated Testing
- [ ] Integration test: GET /collections returns 200 with index.html
- [ ] Integration test: GET /api/collections returns JSON (not HTML)
- [ ] Integration test: GET /assets/file.js returns JS file
- [ ] Integration test: Verify caching headers
- [ ] E2E test: Browser refresh on SPA route

