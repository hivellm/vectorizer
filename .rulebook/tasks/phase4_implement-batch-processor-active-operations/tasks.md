## 1. Tracking state

- [ ] 1.1 Add `active_operations: Arc<DashMap<String, BatchStatus>>` to `BatchProcessor`.
- [ ] 1.2 Update `execute_operation` to insert `Running` before dispatch and set the terminal status on return.

## 2. Accessor wiring

- [ ] 2.1 Add `pub fn active_operations(&self) -> HashMap<String, BatchStatus>` on `BatchProcessor`.
- [ ] 2.2 Replace the empty `HashMap::new()` at `src/batch/operations.rs:234` with a delegation to the processor accessor.

## 3. TTL sweep

- [ ] 3.1 Spawn a background task that removes completed/failed entries older than 5 minutes.
- [ ] 3.2 Make the TTL configurable via `BatchConfig`.

## 4. Tests

- [ ] 4.1 Start a long-running batch; assert `active_operations()` shows `Running`.
- [ ] 4.2 Complete the batch; assert the entry flips to `Completed`.
- [ ] 4.3 Advance time past the TTL; assert the entry is swept.

## 5. Tail (mandatory)

- [ ] 5.1 Update batch module docs with the new observability.
- [ ] 5.2 Tests above cover the new behavior.
- [ ] 5.3 Run `cargo test --all-features` and confirm pass.
