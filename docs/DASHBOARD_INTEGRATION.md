# Dashboard Integration Guide

This document describes how the new Vite + React + TypeScript dashboard is integrated with the Rust server.

## Architecture

### Frontend (Dashboard)
- **Location**: `dashboard/`
- **Build Output**: `dashboard/dist/`
- **Base Path**: `/dashboard/` (production)
- **Tech Stack**: Vite, React 19, TypeScript, Tailwind CSS, React Router

### Backend (Rust Server)
- **Location**: `src/server/mod.rs`
- **Serving**: Static files from `dashboard/dist/`
- **SPA Routing**: Fallback handler for client-side routing

## Server Integration

### Static File Serving

The server serves dashboard static files using Axum's `ServeDir`:

```rust
.nest_service("/dashboard", ServeDir::new("dashboard/dist"))
```

This serves:
- `/dashboard/index.html` - Main HTML file
- `/dashboard/assets/*` - CSS, JS, images, etc.

### SPA Routing Fallback

For client-side routing (React Router), a fallback handler serves `index.html` for all `/dashboard/*` routes that:
- Don't match static assets (`/dashboard/assets/*`)
- Don't match API routes (`/api/*`, `/collections`, etc.)

```rust
async fn dashboard_fallback(uri: Request<Body>) -> impl IntoResponse {
    // Only handle dashboard routes
    if !path.starts_with("/dashboard/") {
        return 404;
    }
    
    // Don't serve index.html for static assets
    if path.starts_with("/dashboard/assets/") {
        return 404;
    }
    
    // Serve index.html for SPA routing
    serve_file("dashboard/dist/index.html")
}
```

## Build Process

### Development
```bash
cd dashboard
npm run dev
# Dashboard available at http://localhost:5173/
```

### Production Build
```bash
cd dashboard
npm run build
# Output: dashboard/dist/
```

### Testing Integration
```bash
# Run test script
./scripts/test-dashboard.sh

# Build and run server
cargo build --release
./target/release/vectorizer

# Access dashboard
# http://localhost:15002/dashboard/
```

## Route Priority

Routes are matched in this order:

1. **UMICP routes** (`/umicp/*`) - Most specific
2. **MCP routes** (`/mcp`)
3. **REST API routes** (`/api/*`, `/collections`, `/health`, etc.)
4. **Metrics routes** (`/prometheus/metrics`)
5. **Dashboard static files** (`/dashboard/assets/*`) - Handled by ServeDir
6. **Dashboard SPA fallback** (`/dashboard/*`) - Serves index.html for React Router

## API Endpoints Used by Dashboard

The dashboard uses these REST API endpoints:

### Collections
- `GET /collections` - List all collections
- `POST /collections` - Create collection
- `GET /collections/{name}` - Get collection details
- `DELETE /collections/{name}` - Delete collection

### Vectors
- `GET /collections/{name}/vectors` - List vectors
- `GET /collections/{name}/vectors/{id}` - Get vector details
- `PUT /collections/{name}/vectors/{id}` - Update vector

### Search
- `POST /collections/{name}/search` - Search vectors
- `POST /collections/{name}/hybrid-search` - Hybrid search

### File Watcher
- `GET /api/workspace/config` - Get file watcher config
- `POST /api/workspace/config` - Update file watcher config

### Workspace
- `GET /api/workspace/list` - List workspace projects
- `POST /api/workspace/add` - Add workspace project
- `POST /api/workspace/remove` - Remove workspace project

### Configuration
- `GET /api/config` - Get server configuration
- `POST /api/config` - Update server configuration

### Logs
- `GET /api/logs` - Get server logs

### Backups
- `GET /api/backups` - List backups
- `POST /api/backups/create` - Create backup
- `POST /api/backups/restore` - Restore backup

## Troubleshooting

### Dashboard Not Loading

1. **Check if dashboard is built**:
   ```bash
   ls dashboard/dist/index.html
   ```

2. **Rebuild dashboard**:
   ```bash
   cd dashboard && npm run build
   ```

3. **Check server logs** for errors serving dashboard files

### 404 Errors for Dashboard Routes

- Verify base path is `/dashboard/` in `dashboard/dist/index.html`
- Check that fallback handler is correctly configured
- Ensure static assets are being served by `ServeDir`

### API Endpoints Not Working

- Verify API routes are registered before dashboard routes
- Check CORS configuration (should be permissive)
- Check server logs for API errors

### Build Errors

- Run `npm run build` in `dashboard/` directory
- Check TypeScript errors: `npm run build` shows TS errors
- Verify all dependencies are installed: `npm install`

## Development Workflow

1. **Frontend Development**:
   ```bash
   cd dashboard
   npm run dev
   # Edit files in dashboard/src/
   # Hot reload available at http://localhost:5173/
   ```

2. **Backend Development**:
   ```bash
   cargo run
   # Server runs at http://localhost:15002/
   ```

3. **Integration Testing**:
   ```bash
   # Build dashboard
   cd dashboard && npm run build && cd ..
   
   # Run server
   cargo run
   
   # Test dashboard
   curl http://localhost:15002/dashboard/
   ```

## Production Deployment

1. **Build Dashboard**:
   ```bash
   cd dashboard
   npm ci  # Install dependencies
   npm run build
   cd ..
   ```

2. **Build Server**:
   ```bash
   cargo build --release
   ```

3. **Verify Integration**:
   ```bash
   ./scripts/test-dashboard.sh
   ```

4. **Run Server**:
   ```bash
   ./target/release/vectorizer
   ```

5. **Access Dashboard**:
   - URL: `http://your-server:15002/dashboard/`
   - The dashboard will be served as static files
   - All routes under `/dashboard/` will work with React Router

## File Structure

```
vectorizer/
├── dashboard/
│   ├── src/              # React source code
│   ├── dist/             # Production build output
│   ├── package.json
│   └── vite.config.ts
├── src/
│   └── server/
│       └── mod.rs        # Server with dashboard integration
└── scripts/
    └── test-dashboard.sh # Integration test script
```

## Notes

- Dashboard is built separately from the Rust server
- Dashboard build must be run before server can serve it
- Base path `/dashboard/` is hardcoded in production build
- Development mode uses `/` base path (Vite dev server)
- All API calls from dashboard use relative paths (handled by React Router)

