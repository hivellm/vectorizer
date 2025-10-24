# Replication Capability - Spec Delta

**Change ID**: `improve-production-readiness`  
**Capability**: replication  
**Type**: MODIFIED

---

## ADDED Requirements

### Requirement: Replication Statistics Collection
The system SHALL collect and expose comprehensive replication statistics for operational monitoring.

#### Scenario: Master node exposes stats
- **GIVEN** a vectorizer instance running as master with 2 connected replicas
- **WHEN** the `/api/v1/replication/status` endpoint is called
- **THEN** the response SHALL include:
  - Total bytes sent to all replicas
  - Number of connected replicas
  - Operations pending replication
  - Last snapshot size

#### Scenario: Replica node exposes stats
- **GIVEN** a vectorizer instance running as replica connected to master
- **WHEN** the `/api/v1/replication/status` endpoint is called
- **THEN** the response SHALL include:
  - Replication lag in milliseconds
  - Total bytes received from master
  - Last successful sync timestamp
  - Operations pending application

#### Scenario: Disconnected replica shows status
- **GIVEN** a replica that has lost connection to master
- **WHEN** stats are retrieved
- **THEN** the lag SHALL continue increasing
- **AND** the last_sync timestamp SHALL remain unchanged
- **AND** the status SHALL indicate "Disconnected"

---

### Requirement: Replica Health Monitoring
The system SHALL track health status of all connected replicas.

#### Scenario: Healthy replica tracking
- **GIVEN** a master node with connected replicas
- **WHEN** a replica sends heartbeat every 10 seconds
- **THEN** the replica SHALL be marked as "Connected"
- **AND** the last_heartbeat timestamp SHALL be updated
- **AND** the lag_ms SHALL be less than 50ms

#### Scenario: Lagging replica detection
- **GIVEN** a replica with network issues
- **WHEN** replication lag exceeds 1000ms
- **THEN** the replica status SHALL change to "Lagging"
- **AND** an alert MAY be logged
- **AND** operations SHALL continue but with warning

#### Scenario: Dead replica detection
- **GIVEN** a replica that has stopped sending heartbeats
- **WHEN** no heartbeat received for 60 seconds
- **THEN** the replica status SHALL change to "Disconnected"
- **AND** the replica SHALL be removed from active list
- **AND** master SHALL continue operating with remaining replicas

---

### Requirement: Replica List Retrieval
The system SHALL provide a complete list of all replicas with their current status.

#### Scenario: List all replicas
- **GIVEN** a master node with 3 replicas (2 connected, 1 disconnected)
- **WHEN** `/api/v1/replication/replicas` is called
- **THEN** the response SHALL include all 3 replicas
- **AND** each replica SHALL have: id, host, port, status, lag_ms, last_heartbeat
- **AND** connected replicas SHALL show current lag
- **AND** disconnected replicas SHALL show last known state

---

## MODIFIED Requirements

### Requirement: Replication Status Endpoint
The system SHALL expose replication status via REST API **with complete statistics**.

**Changes**: Added stats and replicas fields to response structure

#### Scenario: Status includes statistics (UPDATED)
- **GIVEN** a vectorizer instance with replication enabled
- **WHEN** GET `/api/v1/replication/status` is called
- **THEN** the response SHALL include:
  - `role`: "master" or "replica"
  - `enabled`: true/false
  - `stats`: ReplicationStats object (NEW)
  - `replicas`: Array of ReplicaInfo objects for master (NEW)

#### Scenario: Stats update in real-time (NEW)
- **GIVEN** an active replication connection
- **WHEN** vectors are inserted on master
- **THEN** stats SHALL update immediately
- **AND** bytes_sent SHALL increase
- **AND** operations_pending MAY temporarily increase
- **AND** lag_ms on replica SHALL reflect delay

---

## Requirements NOT Changed

The following existing requirements remain unchanged:
- Full Sync Process
- Partial Sync Process
- Automatic Failover
- Snapshot Creation and Verification
- Replication Log Management
- Connection Management

---

## API Changes

### Response Structure (BREAKING)

**Before**:
```json
{
  "role": "master",
  "enabled": true,
  "stats": null,
  "replicas": null
}
```

**After (v1.2.0)**:
```json
{
  "role": "master",
  "enabled": true,
  "stats": {
    "role": "master",
    "lag_ms": 0,
    "bytes_sent": 1048576,
    "bytes_received": 0,
    "last_sync": "2024-10-24T12:00:00Z",
    "operations_pending": 0,
    "snapshot_size": 524288,
    "connected_replicas": 2
  },
  "replicas": [
    {
      "id": "replica-1",
      "host": "192.168.1.10",
      "port": 6381,
      "status": "Connected",
      "lag_ms": 5,
      "last_heartbeat": "2024-10-24T12:00:00Z",
      "operations_synced": 1000
    }
  ]
}
```

### New Data Types

```rust
pub struct ReplicationStats {
    pub role: ReplicationRole,
    pub lag_ms: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_sync: SystemTime,
    pub operations_pending: usize,
    pub snapshot_size: usize,
    pub connected_replicas: Option<usize>,
}

pub struct ReplicaInfo {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub status: ReplicaStatus,
    pub lag_ms: u64,
    pub last_heartbeat: SystemTime,
    pub operations_synced: u64,
}

pub enum ReplicaStatus {
    Connected,
    Syncing,
    Lagging,
    Disconnected,
}
```

---

## Performance Requirements

### Requirement: Stats Collection Overhead
Stats collection SHALL NOT degrade performance by more than 1%.

#### Scenario: Stats collection performance
- **GIVEN** a master node handling 1000 operations/second
- **WHEN** stats collection is enabled
- **THEN** throughput SHALL remain >= 990 operations/second
- **AND** latency SHALL increase by <= 0.1ms
- **AND** memory overhead SHALL be <= 1MB

---

## Migration Guide

### For SDK Developers

**Backwards Compatibility**: Old SDKs will continue to work
- `stats` will be populated instead of `null`
- `replicas` will be populated instead of `null`
- Existing fields unchanged

**Recommended Update**:
```typescript
// Old SDK
interface ReplicationStatus {
  role: string;
  enabled: boolean;
  stats: null;
  replicas: null;
}

// New SDK (v1.2.0+)
interface ReplicationStatus {
  role: string;
  enabled: boolean;
  stats: ReplicationStats | null;      // NEW
  replicas: ReplicaInfo[] | null;      // NEW
}
```

### For API Users

No breaking changes for basic usage:
- `role` and `enabled` fields remain unchanged
- New fields are additive
- Null checks recommended for `stats` and `replicas`

---

## Testing Requirements

### New Tests Required
- Stats collection on master node
- Stats collection on replica node
- Replica health tracking
- Replica list retrieval
- Stats update in real-time
- Stats accuracy under load
- Stats persistence across restarts

### Integration Tests
- Full replication cycle with stats verification
- Failover scenario with stats tracking
- Multiple replicas with different lag times
- Disconnected replica handling

---

## Documentation Updates

### Required Documentation
- Update `docs/REPLICATION.md` with stats examples
- Add monitoring guide using stats
- Update API reference with new response structure
- Add Grafana dashboard template using stats
- Update client SDK documentation

---

**Spec Delta Status**: Complete  
**Review Status**: Pending  
**Implementation Status**: Not started

