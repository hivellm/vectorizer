# Master -> Replica Replication Architecture

This document describes how master nodes push state to replicas in
Vectorizer, how the initial snapshot handshake works, and the
invariants that the apply loop relies on. The goal is to make the
failure modes that caused `phase4_fix-replication-snapshot-sync` hard
to reproduce visible up front.

## Files

- `src/replication/master.rs` - `MasterNode` (accept TCP, per-replica
  command channel, heartbeat loop).
- `src/replication/replica.rs` - `ReplicaNode` (connect, sync, apply
  loop, ACK back to master).
- `src/replication/sync.rs` - snapshot create / apply (serialization
  of collections + vectors + payload).

## Connection handshake

When a replica starts, it connects to the master over plain TCP and
sends a single `u64` (length-prefixed, big-endian) carrying its last
known offset. Master reads that offset and branches:

1. `replica_offset == 0` OR the requested offset is no longer in the
   in-memory replication log -> **full sync**.
   Master builds a `SnapshotData` from the live `VectorStore`
   (`create_snapshot`), wraps it in
   `ReplicationCommand::FullSync { snapshot_data, offset }`, and sends
   it on the same TCP stream.
2. Otherwise -> **partial sync**: master sends
   `ReplicationCommand::PartialSync { from_offset, operations }` with
   every queued operation after `replica_offset`.

After the sync command, master **splits** the stream into read/write
halves (`OwnedReadHalf` / `OwnedWriteHalf`) so the ACK reader task
doesn't serialize behind the command sender task. See `I1` below.

## Live replication

Write-paths (`REST collections / insert / qdrant vector_handlers`) call
`master.replicate(op)` which:

1. Appends the op to the durable replication log (returns the assigned
   offset).
2. Sends `ReplicationMessage::Operation(op)` on the master's global
   `mpsc::UnboundedSender<ReplicationMessage>`.

A single spawned replication task (`start_replication_task`) receives
those messages, reads `replication_log.current_offset()`, wraps each op
in `ReplicationOperation { offset, timestamp, operation }`, and fan-outs
to every connected replica's per-replica channel. Each replica's
connection loop dequeues commands and writes them on its TCP write
half.

Heartbeats are sent by a separate spawned interval task at
`ReplicationConfig::heartbeat_interval` to keep replicas alive and to
let `wait_for_replicas` wake up.

## ACKs and write concern

Replicas call `send_ack` after every applied `Operation`, writing a
`ReplicationCommand::Ack { replica_id, offset }` frame back on the
*same* TCP stream. On the master side the ACK reader task (spawned
from `handle_replica`) reads the read half, updates
`confirmed_offsets`, and notifies `ack_notify`. `wait_for_replicas`
uses `ack_notify` to implement `WriteConcern::Count(n)` timeouts.

## Snapshot wire format

`create_snapshot`:

1. Walks every collection in `VectorStore`, dumping name + dimension +
   metric + `(id, Vec<f32>, Option<Vec<u8>>) payload JSON bytes` per
   vector into a `SnapshotData` struct.
2. Serializes with `crate::codec` (bincode v1).
3. Prefixes a `SnapshotMetadata { offset, total_vectors, checksum }`
   where `checksum = crc32fast::hash(data)`.

`apply_snapshot` verifies the checksum, then:

1. Deserializes `SnapshotData`.
2. For each collection: `delete_collection` (best-effort) + recreate
   with `StorageType::Memory` + `insert` the vectors. Payloads are
   reconstructed from their JSON bytes via
   `serde_json::from_slice(..).unwrap_or_default()`.

## Invariants

The replication path breaks if any of these is violated:

- **I1.** Master uses split read/write halves for the live phase so
  the command sender (write) and the ACK reader (read) don't serialize
  on each other. Keeping a single `TcpStream` mutably borrowed across
  both tasks would deadlock under load.
- **I2.** Replica sends its current offset in the *first* frame. The
  master uses that single `u64` to decide full vs. partial sync; if
  the offset framing is wrong, the replica will hang on the master's
  expected-read and the master will hang on the replica's missing
  offset.
- **I3.** Global `replication_log` offsets grow monotonically and are
  assigned *before* dispatch. The per-replica channel must receive
  operations in append order; reordering breaks the replica's
  `state.offset` tracking and any subsequent partial sync.
- **I4.** Full-sync snapshot serialization uses `crate::codec` (bincode
  v1). This is **positional** - adding fields to `Vector`,
  `SnapshotData`, etc. without `#[serde(default)]` breaks existing
  snapshots. See the `document_id` change in
  `phase4_add-document-id-to-vector` for why `skip_serializing_if`
  must not be used on these types.
- **I5.** `apply_snapshot` wipes and recreates collections on the
  replica. Anything that held a `Collection` handle across a full
  sync becomes stale; callers must re-query after sync completes.
- **I6.** Vector-level mutations (`VectorStore::delete`, etc.) on
  CPU/Sharded collections must take a *shared* DashMap shard ref
  (`get_collection`) and rely on the inner type's interior mutability,
  *not* an exclusive `get_collection_mut` ref. Anything that requires
  exclusive shard access while a long-lived shared ref is alive (a
  user holding `get_collection`, a paginated read, the apply loop on
  the previous op) deadlocks until that ref is dropped — and on the
  replica that drop only happens when the test panics or the read
  consumer finishes. The `HiveGpu` variant still needs `&mut self`
  until its `vector_count` becomes atomic, and falls back to the
  exclusive path explicitly.

## Known limitations

- `ReplicationConfig` carries `wal_enabled` + `wal_dir` but the replica
  side doesn't log applied operations to its own WAL, so a replica
  restart without a surviving master state file loses anything beyond
  the most recent snapshot.
