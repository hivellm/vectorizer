# Proposal: phase6_major-dep-migrations

## Why

`phase5_refresh-all-dependencies` completed a first-pass safe
refresh of every manifest, but a number of major-version bumps were
held back because absorbing them means touching application code,
not just version strings. Each hold-back is a real migration: API
shape changes, config-format changes, runtime-model changes. They
need their own scoped work so the diff stays readable and any
regression is surgically revertable per-migration.

This task is the umbrella tracker for those migrations. Each line
in "What Changes" is a standalone sub-task of work; they run
independently and in whatever order makes sense for the release
train.

## What Changes

Per-ecosystem migrations, each to be done individually (create a
sub-task when starting work):

### Rust
- `rand 0.9 → 0.10` (blocked by openraft alpha-17 using
  `random_range`; either wait for openraft 0.11 or contribute a patch
  upstream).
- `bincode 2 → 3` (serialisation format tweaks; survey call sites in
  `vectorizer-core::codec` + persistence).
- `rmcp 0.10 → 1.5` (MCP stable line; API has moved).
- `reqwest 0.12 → 0.13` (TLS + middleware changes; audit SDK +
  server usage).
- `hmac 0.12 → 0.13` + `sha2 0.10 → 0.11` (RustCrypto family;
  cross-crate dep graph).
- `arrow/parquet 57 → 58` (Apache Arrow ecosystem — rarely
  drop-in).
- `zip 6 → 8` (zip crate's major rewrites).
- `tantivy 0.25 → 0.26` (BM25 backend API changes).
- `hf-hub 0.4 → 0.5`.
- `sysinfo 0.37 → 0.38`.
- `opentelemetry-prometheus 0.29 → 0.31` (tied to opentelemetry
  family version matching).
- `ort 2.0.0-rc.11 → 2.0.0-rc.12` (blocked by fastembed pinning to
  rc.11; upstream fastembed bump required).

### TypeScript SDK
- `eslint 8 → 9/10` — requires migrating `.eslintrc.js` to flat
  `eslint.config.js`.
- `vitest 2 → 4` — config format changes.
- `@types/node 24 → 25` — match the Node version we pin in CI first.

### GUI
- `typescript 5 → 6`.
- `vite 7 → 8`.
- `vue-router 4 → 5` — API migration.
- `uuid 13 → 14`.
- `electron 39 → 41` — verify `electron-builder` compat + app-signing
  impact.

### Dashboard
- `react 18 → 19` (+ `react-dom`, `@types/react`, `@types/react-dom`)
  — concurrent-features migration.
- `react-router 6 → 7` (+ `react-router-dom`) — loader-based API.
- `@vitejs/plugin-react 4 → 6`.
- `eslint 9 → 10` (+ `@eslint/js`).
- `typescript 5 → 6`.
- `vite 7 → 8`.
- `jsdom 27 → 29`.
- `tailwind-merge 2 → 3`.
- `@types/node 24 → 25`.

## Impact

- Affected specs: none (dependency metadata + call-site adjustments
  only).
- Affected code: varies per sub-task — some are Cargo.toml-only after
  upstream un-blocks, others require call-site edits.
- Breaking change: NO to downstream consumers when done carefully.
- User benefit: closes remaining CVE gaps and keeps us on the
  supported upstream line of each ecosystem.
