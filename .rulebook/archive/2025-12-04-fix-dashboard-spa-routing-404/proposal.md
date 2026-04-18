# Proposal: fix-dashboard-spa-routing-404

## Why

The Dashboard has a critical routing bug that affects user experience:

**Problem**: When users navigate to any dashboard route (e.g., `/collections`, `/search`, `/settings`) and press F5 (refresh), the server returns **404 Not Found**.

**Root Cause**: 
- Dashboard is a SPA (Single Page Application) using React Router
- React Router handles routing client-side in JavaScript
- When user refreshes, browser sends request to server for that path
- Server tries to find a file at that path (e.g., `/collections`)
- File doesn't exist → Server returns 404
- React Router never loads → User sees error page

**Example of Bug**:
```
1. User navigates to http://localhost:3000/collections (works ✅)
2. User presses F5 to refresh
3. Browser sends: GET /collections HTTP/1.1
4. Server looks for file: ./dashboard/dist/collections
5. File not found → Returns 404 ❌
6. User sees: "404 Not Found" instead of dashboard
```

**Impact**:
- Poor user experience (can't refresh pages)
- Broken bookmarks (direct links to routes fail)
- Broken browser back/forward navigation
- Appears as broken application to users

**Standard SPA Solution**:
- Server should serve `index.html` for all non-static routes
- This allows React Router to handle routing client-side
- Known as "HTML5 History API" or "SPA fallback routing"

## What Changes

### 1. Implement SPA Fallback Routing

Configure server to serve `index.html` for all non-static routes:

**Routes that should serve index.html**:
- `/` → index.html (already works)
- `/collections` → index.html (currently 404)
- `/search` → index.html (currently 404)
- `/settings` → index.html (currently 404)
- Any other `/path` → index.html (if not a file)

**Routes that should serve files**:
- `/assets/*` → Static files (JS, CSS, images)
- `/api/*` → API endpoints (NOT index.html)
- `/prometheus/*` → Metrics (NOT index.html)

### 2. Update Axum Router Configuration

Modify dashboard server to add fallback handler:

```rust
// Current (broken)
.nest_service("/", ServeDir::new("dashboard/dist"))

// Fixed (with fallback)
.nest_service("/", ServeDir::new("dashboard/dist"))
.fallback(spa_fallback_handler)

async fn spa_fallback_handler() -> impl IntoResponse {
    // Serve index.html for all non-file routes
    ServeFile::new("dashboard/dist/index.html")
}
```

### 3. Add Route Priority Logic

Implement proper route priority:
1. **First**: Try exact file match (assets, favicon, etc.)
2. **Second**: Check if it's an API route → pass to API handlers
3. **Third**: Serve index.html (SPA fallback)

### 4. Handle Special Cases

- `/api/*` routes must NOT fallback to index.html
- `/prometheus/*` must NOT fallback to index.html
- Static assets (`.js`, `.css`, `.png`, etc.) serve files
- Everything else → index.html

### 5. Add Development vs Production Handling

**Development mode**:
- Vite dev server handles routing automatically
- No changes needed

**Production mode**:
- Rust server must handle SPA routing
- Apply fallback logic

### 6. Update Build Process

Ensure build process creates correct structure:
```
dashboard/dist/
├── index.html           # Fallback for all routes
├── assets/
│   ├── index-abc123.js  # Hashed filenames
│   └── index-def456.css
└── favicon.ico
```

## Impact

### Affected Specs
- `docs/specs/DASHBOARD.md` - TO CREATE: Dashboard server specification

### Affected Code
- `src/server/mod.rs` - Dashboard router configuration
- `src/server/dashboard.rs` - TO CREATE: Dedicated dashboard server module
- `dashboard/vite.config.ts` - Ensure correct build output
- `dashboard/index.html` - Verify base path configuration

### Breaking Change
**NO** - This is a bug fix, improves existing behavior

**Before** (buggy):
- ✅ Navigate to `/collections` via UI → Works
- ❌ Refresh on `/collections` → 404 Error
- ❌ Direct link to `/collections` → 404 Error
- ❌ Browser back button → Sometimes 404

**After** (fixed):
- ✅ Navigate to `/collections` via UI → Works
- ✅ Refresh on `/collections` → Works
- ✅ Direct link to `/collections` → Works
- ✅ Browser back button → Always works

### User Benefit

**Immediate Benefits**:
- ✅ Can refresh any dashboard page without errors
- ✅ Direct links to dashboard routes work
- ✅ Bookmarks work correctly
- ✅ Browser navigation (back/forward) works reliably
- ✅ Better perceived reliability

**Technical Benefits**:
- ✅ Follows SPA best practices
- ✅ Matches behavior of modern web apps (React, Vue, Angular)
- ✅ Enables sharing of specific dashboard URLs
- ✅ Better SEO (if public dashboard in future)

**Example Use Cases Fixed**:
- User bookmarks `/collections` → Can access directly ✅
- User shares link `/search?q=test` → Recipient can open ✅
- User refreshes during workflow → Doesn't lose context ✅
- Browser crashes and restores tabs → All tabs work ✅
