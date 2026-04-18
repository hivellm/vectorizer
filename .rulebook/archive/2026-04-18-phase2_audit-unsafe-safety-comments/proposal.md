# Proposal: phase2_audit-unsafe-safety-comments

## Why

`/.rulebook/specs/RUST.md` mandates: "No `unsafe` without a `// SAFETY:` comment explaining why the invariants hold." Audit found **19 `unsafe` blocks in `src/` without SAFETY comments**:

- `src/embedding/cache.rs:334` — `Mmap::map()`
- `src/embedding/candle_models.rs:135,296` — `VarBuilder::from_mmaped_safetensors()`
- `src/bin/vectorizer-cli.rs:240` — unsafe `Command` execution
- `src/config/enhanced_config.rs:723` — `env::set_var()` (test)
- `src/storage/mmap.rs:101` — `MmapOptions`
- plus ~14 other sites

Each missing comment is a latent UB hazard: readers can't verify the invariant, refactors might silently invalidate it, and new unsafe can't be added without a reviewer spotting it. For a vector database with mmap-backed persistence, this is a critical correctness issue — not cosmetic.

## What Changes

For each unsafe block:
1. **Document the invariant** in a `// SAFETY:` comment above the `unsafe { }` block, referencing the concrete preconditions (file is exclusive, pointer is aligned, lifetime bound, etc.).
2. **Or**: wrap in a safe abstraction (e.g., use `memmap2::MmapOptions::map()` return type directly, use `bytemuck` for pod casts) and delete the unsafe.
3. **Or**: if the invariant can't be locally satisfied, mark the enclosing function `unsafe fn` and push the SAFETY comment up to the caller — this makes the contract visible.

Finally: enable clippy lint `clippy::undocumented_unsafe_blocks` at project level in `clippy.toml` and make it deny in CI.

## Impact

- Affected specs: `/.rulebook/specs/RUST.md`
- Affected code: 19 unsafe sites across `src/embedding/`, `src/storage/`, `src/bin/`, `src/config/`, `src/models/`, `src/cluster/`
- Breaking change: NO (internal)
- User benefit: auditable unsafe surface; prevents future UB; aligns with project rules.
