# Proposal: phase4_implement-batch-processor-active-operations

## Why

`BatchOperationManager::get_active_operations` at `src/batch/operations.rs:234` returns an empty `HashMap<String, BatchStatus>` unconditionally, with a marker saying the real implementation lives in `BatchProcessor`. The function exists in the public API of the batch module, so callers (status endpoints, admin dashboard) see "no active operations" even under heavy load — giving them no observability into in-flight batch work.

## What Changes

1. Add an `active_operations: Arc<DashMap<String, BatchStatus>>` to `BatchProcessor`.
2. On `execute_operation`, insert a `BatchStatus::Running` entry keyed by the operation id; update to `Completed`/`Failed` on exit.
3. Expose a `BatchProcessor::active_operations()` accessor and plumb it through `BatchOperationManager::get_active_operations`.
4. Add a TTL so completed/failed statuses are cleaned up after a configurable window (default 5 min) instead of leaking forever.

## Impact

- Affected specs: none; this is an internal observability gap.
- Affected code: `src/batch/processor.rs`, `src/batch/operations.rs`, anything that exposes batch status over REST/MCP.
- Breaking change: NO.
- User benefit: clients and operators can actually see what the batch processor is doing.
