## 1. Implementation
- [ ] 1.1 Review openraft alpha.22 -> alpha.30 changelog for API/behavior changes
- [ ] 1.2 Bump both `=` pins (openraft + openraft-memstore) to alpha.30 in vectorizer + vectorizer-server; refresh Cargo.lock
- [ ] 1.3 Adapt HA/consensus code to any API changes; cargo check + clippy clean

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 2.1 Update or create documentation covering the implementation
- [ ] 2.2 Run tests/integration/cluster_ha.rs against a live multi-node cluster and confirm HA works
- [ ] 2.3 Run tests and confirm they pass
