# Dashboard - Administrative Interface

**Version**: 0.3.0  
**Status**: ✅ Production Ready  
**Priority**: 🟡 P0 (Critical for Quantization)  
**Last Updated**: 2025-10-01

---

## Overview

Web-based administrative interface accessible from localhost for managing collections, vectors, API keys, and system monitoring.

### Security Model

- **Localhost Only**: Binds exclusively to 127.0.0.1
- **Authentication**: User login with role-based access
- **Read-Only Operations**: Cannot modify vector data
- **Management Operations**: API keys and collection management

---

## Features

### 1. API Key Management

**Create API Key**:
- Generate secure random keys
- Custom names and descriptions
- One-time display (not stored)

**List API Keys**:
- Masked display (first/last 4 chars + asterisks)
- Metadata: name, description, created, last used
- Usage statistics

**Delete API Key**:
- Immediate revocation
- Connection termination
- Audit log entry

### 2. Collection Management

**List Collections**:
- Name, dimension, vector count
- Index type (HNSW, IVF, etc.)
- Quantization settings
- Size and status

**Collection Details**:
- Comprehensive metadata
- Performance metrics
- Configuration inspection

### 3. Quantization Metrics (NEW - P0)

**Real-time Monitoring**:
- **Memory compression ratio**: Live 4x reduction display
- **Quality metrics**: MAP/Recall@K with baseline
- **Quantization method**: SQ-8bit/PQ/Binary status
- **Performance impact**: Search latency comparison
- **Collection-level metrics**: Per-collection settings

```rust
pub struct QuantizationMetrics {
    pub compression_ratio: f32,        // 4.0x for SQ-8bit
    pub quality_improvement: f32,      // +8.9% MAP
    pub active_method: QuantizationType,
    pub memory_saved_mb: u64,
    pub search_latency_ms: f32,
}
```

### 4. System Monitoring

**Real-time Metrics**:
- Memory usage
- CPU utilization
- Active connections
- Request throughput

**Collection Stats**:
- Vector counts
- Index health
- Query performance
- Cache hit rates

---

## Authentication & Authorization

### User Roles

**Admin**:
- Full access
- Create/delete users
- Manage API keys
- System configuration

**ReadWrite**:
- Modify collections
- Manage own API keys
- View all metrics

**ReadOnly**:
- View only
- No modification rights

### Login Flow

```http
POST /dashboard/api/login
{
  "username": "admin",
  "password": "secure_password"
}

Response:
Set-Cookie: session_id=...; HttpOnly; Secure
{
  "success": true,
  "user": {...},
  "role": "Admin"
}
```

---

## Endpoints

### Authentication
- `POST /dashboard/api/login` - User login
- `POST /dashboard/api/logout` - User logout
- `GET /dashboard/api/session` - Session validation

### API Keys
- `GET /dashboard/api-keys` - List keys
- `POST /dashboard/api-keys` - Create key
- `DELETE /dashboard/api-keys/{id}` - Delete key
- `GET /dashboard/api-keys/stats` - Usage statistics

### Collections
- `GET /dashboard/collections` - List collections
- `GET /dashboard/collections/{name}` - Collection details
- `GET /dashboard/collections/{name}/metrics` - Performance metrics

### System
- `GET /dashboard/stats` - System statistics
- `GET /dashboard/health` - Health check
- `GET /dashboard/quantization` - Quantization metrics

---

## Technical Stack

- **Backend**: Integrated into Rust server
- **Frontend**: Vanilla HTML/CSS/JavaScript
- **Communication**: Direct function calls (same process)
- **Port**: Server port + 1 (e.g., 15002 if server on 15001)

---

## Performance Metrics

**Key Metrics**:
- Dashboard response time: <50ms
- Real-time updates: 1-second refresh
- Memory overhead: <10MB
- Authentication overhead: <5ms per request

---

**Status**: ✅ Production Ready with Quantization Support  
**Maintained by**: HiveLLM Team
