## 1. Implementation
- [x] 1.1 Review openraft alpha.22 -> alpha.30 changelog for API/behavior changes — one breaking change (`SnapshotData` moved off `RaftTypeConfig` in alpha.29) plus consensus fixes; no GitHub release notes for most tags, reviewed via the tag-to-tag commit range
- [x] 1.2 Bump both `=` pins (openraft + openraft-memstore) to alpha.30 in vectorizer + vectorizer-server; refresh Cargo.lock — also pinned openraft-macros/-rt/-rt-tokio to alpha.30 in the lock, which Cargo had floated to alpha.32
- [x] 1.3 Adapt HA/consensus code to any API changes; cargo check + clippy clean — `type SnapshotData` on the state machine, snapshot builder and network v2 impls; two-parameter snapshot aliases; one `ClusterSnapshotData` alias keeps them in step

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [x] 2.1 Update or create documentation covering the implementation — CHANGELOG entry with the API change and the fixes, pin rationale comments refreshed in both Cargo.tomls, stale `=0.10.0-alpha.17` in README corrected
- [x] 2.2 Write tests covering the new behavior — ran tests/integration/cluster_ha.rs against a live multi-node cluster and confirmed HA works: a new three-node test on real gRPC sockets covers election, replication to both followers, leader killed, new leader elected, and pre- and post-failover writes present; 5/5 consecutive runs
- [x] 2.3 Run tests and confirm they pass — 21/21 cluster_ha, 113/113 cluster, 926 passed in the vectorizer suite, 235 in vectorizer-server, clippy clean; the single remaining failure (`prometheus_counter_increments_on_every_cache_get`) reproduces at HEAD without this bump
