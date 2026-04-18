## 1. Reproduction

- [ ] 1.1 Run `cargo test --test all_tests replication::integration_basic::test_replica_full_sync_on_connect -- --ignored --nocapture` and capture the output.
- [ ] 1.2 Identify the exact point where the replica expects the snapshot but doesn't receive it.

## 2. Fix

- [ ] 2.1 Trace the master-side `send_snapshot` path end-to-end.
- [ ] 2.2 Trace the replica-side `receive_snapshot` handler.
- [ ] 2.3 Root-cause the missing hand-off and patch the offending module.
- [ ] 2.4 Remove `#[ignore]` from every test that passes after the fix.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `docs/specs/REPLICATION.md` documenting the snapshot-sync protocol with sequence diagrams.
- [ ] 3.2 12 newly-active tests ARE the regression guard.
- [ ] 3.3 Run `cargo test --test all_tests replication::integration_basic` and confirm all pass without `--ignored`.

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
