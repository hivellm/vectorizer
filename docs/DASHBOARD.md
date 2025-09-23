# Dashboard Technical Documentation

## Overview

The Vectorizer Dashboard is a web-based administrative interface accessible only from `localhost` (127.0.0.1) that provides comprehensive management capabilities for the Vectorizer server. It serves as a secure, user-friendly alternative to CLI operations for managing collections, vectors, API keys, and server monitoring.

## Architecture

### Security Model
- **Localhost Only**: Dashboard binds exclusively to 127.0.0.1
- **No Authentication**: Relies on localhost access restriction for security
- **Read-Only Operations**: Cannot modify vector data or perform searches
- **Management Operations**: API key creation/deletion and collection management

### Technical Stack
- **Backend**: Integrated into Rust server using Actix-web or similar
- **Frontend**: Vanilla HTML/CSS/JavaScript (no external dependencies)
- **Communication**: Direct function calls (same process as server)
- **Data Access**: Read-only access to server state

## Core Features

### 1. API Key Management

#### Create API Key
```
Interface: POST /dashboard/api-keys
Purpose: Generate new API keys with custom names and descriptions
Parameters:
- name: string (required) - Human-readable identifier
- description: string (optional) - Purpose or usage description
Returns: Generated API key (displayed once, not stored)

Security Notes:
- Keys are cryptographically secure random strings
- Keys are hashed before storage
- Original key shown only during creation
```

#### List API Keys
```
Interface: GET /dashboard/api-keys
Purpose: Display all active API keys with metadata
Display Fields:
- Key ID (masked): Shows first/last 4 characters + asterisks
- Name: Human-readable identifier
- Description: Usage description
- Created: Timestamp of creation
- Last Used: Timestamp of last API call
- Usage Count: Number of API operations performed

Security Notes:
- Actual API keys never displayed in full
- Metadata visible for management purposes
```

#### Delete API Key
```
Interface: DELETE /dashboard/api-keys/{key_id}
Purpose: Immediately revoke an API key
Effects:
- Key marked as revoked in database
- All future API calls rejected
- Existing connections terminated
- Audit log entry created

Confirmation: Requires explicit confirmation to prevent accidental deletion
```

#### Key Statistics
```
Interface: GET /dashboard/api-keys/stats
Purpose: Display usage statistics per API key
Metrics:
- Total API calls
- Average response time
- Error rate
- Memory usage
- Bandwidth consumption
```

### 2. Collection Management

#### List Collections
```
Interface: GET /dashboard/collections
Purpose: Display all collections with metadata
Display Fields:
- Name: Collection identifier
- Dimension: Vector dimensionality
- Vector Count: Number of stored vectors
- Index Type: HNSW, IVF, etc.
- Quantization: PQ/SQ/Binary settings
- Created: Creation timestamp
- Size: Disk/memory usage
- Status: Active/Inactive
```

#### Collection Details
```
Interface: GET /dashboard/collections/{name}
Purpose: Detailed view of collection configuration
Display Sections:
- Configuration: dimension, metric, index parameters, compression settings
- Statistics: vector count, memory usage, performance metrics
- Compression Stats: compression ratio, saved space, threshold effectiveness
- Recent Operations: Last 10 insertions/searches with timestamps
- Health Status: Index integrity, memory usage alerts
```

#### Create Collection
```
Interface: POST /dashboard/collections
Purpose: Create new collection with custom configuration
Parameters:
- name: string (required)
- dimension: number (required)
- metric: enum [cosine, euclidean, dot_product]
- quantization: object (optional)
  - type: enum [pq, sq, binary]
  - parameters: type-specific config
- index: object (required)
  - type: enum [hnsw, ivf]
  - parameters: type-specific config

Validation: Server-side validation of all parameters
```

#### Delete Collection
```
Interface: DELETE /dashboard/collections/{name}
Purpose: Permanently remove collection and all vectors
Effects:
- All vectors deleted from storage
- Index files removed
- Configuration cleaned up
- Audit log entry created

Confirmation: Requires explicit confirmation with collection name typing
```

### 3. Vector Management

#### Browse Vectors
```
Interface: GET /dashboard/collections/{name}/vectors
Purpose: Paginated list of vectors in collection
Display Fields:
- Vector ID: Unique identifier
- Preview: First 5 dimensions + "..." (not full vector)
- Payload: Associated metadata (truncated if large)
- Distance: Not applicable (browsing view)
- Added: Timestamp of insertion

Pagination: 50 vectors per page with navigation
Sorting: By ID, timestamp, or payload fields
```

#### Vector Details
```
Interface: GET /dashboard/collections/{name}/vectors/{id}
Purpose: Detailed view of single vector
Display Sections:
- Vector Data: Full vector array (scrollable)
- Payload: Complete metadata object
- Metadata: ID, timestamps, source info
- Similar Vectors: Top 5 most similar vectors (preview)

Note: Full vector data displayed for inspection only
```

#### Edit Vector Payload
```
Interface: PUT /dashboard/collections/{name}/vectors/{id}/payload
Purpose: Modify associated metadata without changing vector
Allowed Operations:
- Add new metadata fields
- Modify existing values
- Remove metadata fields

Restrictions:
- Vector data itself cannot be modified
- ID cannot be changed
- Operation logged in audit trail
```

#### Delete Vector
```
Interface: DELETE /dashboard/collections/{name}/vectors/{id}
Purpose: Remove single vector from collection
Effects:
- Vector removed from index
- Payload data deleted
- Index rebuilt incrementally
- Audit log entry created

Note: Operation may affect search quality temporarily
```

### 4. Search Interface (Preview)

#### Text Search Demo
```
Interface: POST /dashboard/search/preview
Purpose: Demonstrate search functionality without full results
Parameters:
- collection: string (required)
- query: string (required) - Text to search for
- limit: number (default: 5) - Number of results to preview

Process:
1. Text sent to embedding engine
2. Vector generated using collection's embedding model
3. Similarity search performed
4. Top results displayed (metadata only, no full vectors)

Display: Results show ID, score, truncated payload
Security: No full vector data exposed in dashboard
```

#### Vector Search Demo
```
Interface: POST /dashboard/search/vector-preview
Purpose: Search using pre-existing vector
Parameters:
- collection: string (required)
- vector_id: string (required) - ID of vector to use as query
- limit: number (default: 5)

Process:
1. Retrieve specified vector from collection
2. Perform similarity search
3. Display results

Use Case: Test search quality for known vectors
```

### 5. Server Monitoring

#### System Metrics
```
Interface: GET /dashboard/metrics
Purpose: Real-time server performance metrics
Display Fields:
- CPU Usage: Percentage and core breakdown
- Memory Usage: RAM consumption with breakdown
- Disk Usage: Storage used by collections
- Network I/O: Bandwidth consumption
- Active Connections: Current client connections
- Queue Length: Pending operations
```

#### Collection Performance
```
Interface: GET /dashboard/collections/{name}/performance
Purpose: Performance metrics specific to collection
Metrics:
- Average search latency
- Index build time
- Memory usage breakdown
- Cache hit rate
- Quantization compression ratio
- Payload compression stats (ratio, saved space, CPU overhead)
```

#### Audit Logs
```
Interface: GET /dashboard/audit
Purpose: View recent operations and events
Log Types:
- API key operations (create/delete)
- Collection operations (create/delete/modify)
- Vector operations (insert/delete/update)
- Search operations (with API key reference)
- System events (startup, shutdown, errors)

Filtering: By time range, operation type, API key
Retention: Configurable log retention period
```

## UI/UX Design

### Layout Structure
```
┌─────────────────────────────────────────────────┐
│ Header: Server Status | Quick Actions           │
├─────────────────────────────────────────────────┤
│ Navigation: Keys | Collections | Monitoring     │
├─────────────────────────────────────────────────┤
│ Main Content Area                               │
│                                                 │
│ [Dynamic content based on navigation]          │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Responsive Design
- **Desktop**: Full layout with side navigation
- **Tablet**: Collapsible navigation, adjusted spacing
- **Mobile**: Single-column layout, simplified views

### Accessibility
- **Keyboard Navigation**: All controls accessible via keyboard
- **Screen Reader**: Proper ARIA labels and semantic HTML
- **High Contrast**: Sufficient color contrast ratios
- **Focus Indicators**: Clear focus states for all interactive elements

## Security Considerations

### Network Security
- **Localhost Binding**: Dashboard only accessible from 127.0.0.1
- **No External Access**: Cannot be exposed to external networks
- **Process Isolation**: Runs in same process as server (trusted)

### Data Protection
- **Masked Sensitive Data**: API keys, full vectors never displayed
- **Read-Only Operations**: Cannot modify vector data through dashboard
- **Audit Trail**: All operations logged with timestamps and API key references

### Operational Security
- **No Authentication**: Relies on localhost access control
- **Session Management**: No sessions (stateless interface)
- **CSRF Protection**: Not applicable (localhost only)
- **Input Validation**: Server-side validation for all inputs

## API Endpoints

### Dashboard-Specific Endpoints
All endpoints prefixed with `/dashboard` and only accessible from localhost.

```
GET    /dashboard                    # Dashboard home page
GET    /dashboard/api-keys          # List API keys
POST   /dashboard/api-keys          # Create API key
DELETE /dashboard/api-keys/{id}     # Delete API key
GET    /dashboard/api-keys/stats    # Key usage statistics

GET    /dashboard/collections       # List collections
POST   /dashboard/collections       # Create collection
GET    /dashboard/collections/{name} # Collection details
DELETE /dashboard/collections/{name} # Delete collection

GET    /dashboard/collections/{name}/vectors # Browse vectors
GET    /dashboard/collections/{name}/vectors/{id} # Vector details
PUT    /dashboard/collections/{name}/vectors/{id}/payload # Edit payload
DELETE /dashboard/collections/{name}/vectors/{id} # Delete vector

POST   /dashboard/search/preview    # Text search demo
POST   /dashboard/search/vector-preview # Vector search demo

GET    /dashboard/metrics           # System metrics
GET    /dashboard/collections/{name}/performance # Collection metrics
GET    /dashboard/audit             # Audit logs
```

### Response Formats
- **HTML**: Full pages for navigation
- **JSON**: Data for dynamic updates
- **Plain Text**: Simple responses for actions

## Limitations and Best Practices

### Functional Limitations
1. **No Vector Modification**: Vector data cannot be edited (by design)
2. **Read-Only Searches**: Search results show metadata only
3. **Localhost Only**: Cannot be accessed remotely
4. **No Bulk Operations**: Operations limited to single items
5. **No Export Features**: Data cannot be exported through dashboard

### Performance Considerations
1. **Pagination Required**: Large collections must be paginated
2. **Lazy Loading**: Vector details loaded on demand
3. **Caching**: Metrics cached to reduce server load
4. **Resource Limits**: Dashboard operations throttled to prevent impact

### Security Best Practices
1. **Physical Access**: Ensure console access is restricted
2. **Network Isolation**: Server should be on isolated network segment
3. **Regular Audits**: Monitor dashboard access logs
4. **Key Rotation**: Regularly rotate API keys through dashboard
5. **Backup**: Regular database backups before major operations

## Configuration

### Dashboard Settings
```toml
[dashboard]
enabled = true                    # Enable/disable dashboard
bind_address = "127.0.0.1"       # Always localhost
port = 3000                      # Dashboard port
theme = "auto"                   # light/dark/auto
locale = "en"                    # Interface language
audit_retention_days = 30        # Log retention period
```

### Server Integration
```rust
// Dashboard routes integrated into main server
fn configure_dashboard_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dashboard")
            .route("", web::get().to(dashboard_home))
            .route("/api-keys", web::get().to(list_api_keys))
            .route("/api-keys", web::post().to(create_api_key))
            // ... additional routes
    );
}
```

## Development Guidelines

### Frontend Development
- **Vanilla JS**: No framework dependencies
- **Modular Code**: Separate files for each feature
- **Progressive Enhancement**: Works without JavaScript
- **Error Handling**: Graceful degradation on errors

### Backend Integration
- **Shared Types**: Reuse server types where possible
- **Consistent Error Handling**: Use server's error types
- **Performance Monitoring**: Track dashboard operation performance
- **Security Auditing**: Regular security reviews of dashboard code

## Troubleshooting

### Common Issues
1. **Dashboard Not Accessible**: Check if server started with dashboard enabled
2. **Port Conflicts**: Ensure port 3000 is available
3. **Performance Issues**: Check server resources and collection sizes
4. **Display Problems**: Clear browser cache, check for JavaScript errors

### Debug Mode
Enable debug logging for dashboard operations:
```bash
RUST_LOG=vectorizer::dashboard=debug vectorizer server
```

### Compression Issues
1. **High CPU Usage**: Check compression thresholds - payloads <1KB don't compress
2. **Slow API Responses**: Monitor compression ratios - very large payloads may cause delays
3. **Storage Not Reducing**: Verify LZ4 library is properly installed and configured
4. **Decompression Errors**: Check for corrupted compressed data in storage

### Health Checks
Dashboard provides health endpoints for monitoring:
```
GET /dashboard/health     # Basic health check
GET /dashboard/health/detailed # Detailed server status
```

---

This dashboard provides a comprehensive yet secure management interface for Vectorizer server operations, ensuring administrators can effectively manage the system while maintaining data security and operational integrity.
