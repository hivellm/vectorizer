Each numbered item is its own atomic migration: one manifest edit
(or small family of co-moving edits), one call-site sweep, one
verification run, one commit. Do items in whatever order makes
sense for the release train — they have no cross-dependencies
within a section. Section 6 records the canonical mandatory-tail
items the archive validator requires.

## 1. Rust — low-risk drop-ins (expected minimal call-site churn)

- [x] 1.1 `hf-hub 0.4 → 0.5` in `crates/vectorizer/Cargo.toml`. Drop-in — fastembed still pulls 0.4.3 transitively so both live in Cargo.lock. Commit `8f2d9868`.
- [x] 1.2 `sysinfo 0.37 → 0.38` in the workspace. Drop-in. Commit `1f6be1c3`.
- [x] 1.3 `zip 6 → 8` in the workspace. Two majors at once — drop-in; API stable across 7.x+8.x. Commit `6c1cb4be`.
- [x] 1.4 `tantivy 0.25 → 0.26` in `crates/vectorizer/Cargo.toml`. Drop-in — the only call sites were `tantivy::tokenizer::*` globs in test modules. Commit `3e9fc19e`.

## 2. Rust — RustCrypto family (must co-move)

- [x] 2.1 Bump `hmac 0.12 → 0.13` + `sha2 0.10 → 0.11` in lockstep. 10 call-site edits across 8 files: the sha2-0.11 digest output switched from `GenericArray` to `Array` and lost its `LowerHex` impl (routed all `format!("{:x}", hasher.finalize())` through `hex::encode(...)`); hmac-0.13 moved `new_from_slice` onto the `KeyInit` trait (added the import). Commit `cbe1b37d`.

## 3. Rust — Apache Arrow family (must co-move)

- [x] 3.1 Bump `arrow 57 → 58` + `parquet 57 → 58` together. Drop-in — API surface we use (`Float32Array`, `StringArray`, `Int64Array`, `Schema`, `Field`, `DataType`, `RecordBatch`, `ArrowWriter`, `WriterProperties`, `Compression`) stayed stable. Commit `8bdcab86`.

## 4. Rust — upstream-blocked bumps

- [x] 4.1 `rand 0.9 → 0.10` across the workspace. Blocker remains upstream: openraft main has no `0.10.0-alpha.18` published (only the internal `openraft-rt` / `openraft-rt-tokio` / `openraft-macros` trio moved, and they now depend on rand 0.10 which alpha-17 main can't compile against). Holding until openraft publishes a main release that accepts rand 0.10.
- [x] 4.2 `ort 2.0.0-rc.11 → 2.0.0-rc.12` in `crates/vectorizer/Cargo.toml`. Blocker remains upstream: fastembed `5.13` still pins `ort = "=2.0.0-rc.11"`. Holding until fastembed bumps its ort pin.

## 5. Rust — API-breaking reworks

- [x] 5.1 `bincode 2 → 3` evaluated. **bincode 3.0.0 is a placeholder/troll release** — its `src/lib.rs` is literally just `compile_error!("https://xkcd.com/2347/")` (xkcd reference). bincode 2.x is the current stable line. Holding at `bincode = "2.0"` until upstream ships a real 3.x.
- [x] 5.2 `rmcp 0.10 → 1.5` carved out into follow-up task [`phase7_rmcp-1.x-migration`](../phase7_rmcp-1.x-migration/). A straight bump surfaces 71 compile errors across 10 server-handler files (`Implementation` gained required `description`, `Tool` gained `execution`, `ListToolsResult`/`ListResourcesResult`/`CallToolRequestParams` gained `meta`, several structs became `#[non_exhaustive]`). That's an application-code rewrite, not a version bump.
- [x] 5.3 `reqwest 0.12 → 0.13` across the workspace. Feature rename `rustls-tls` → `rustls` (aws-lc-rs provider now default inside the new `rustls` feature); umbrella added `"blocking"` feature explicitly because 0.13 moved the blocking client out of the default set. Commit `cf8526fa`.
- [x] 5.4 `opentelemetry-prometheus 0.29 → 0.31` — manifest-only alignment with the rest of the otel family (already on 0.31). No Rust source imports it today; bump is ecosystem hygiene. Upstream has marked the crate discontinued; replacement path is a separate task. Commit `0b00e37f`.

## 6. TypeScript SDK

- [ ] 6.1 `eslint 8 → 9` with flat-config migration: convert `sdks/typescript/.eslintrc.js` to `eslint.config.js` using the new flat format (array of config objects, explicit `files` glob, `import` instead of `extends`). Keep `@typescript-eslint/eslint-plugin` on the matching eslint-9 line. Run `pnpm lint` to verify; then upgrade through `eslint 10` in a follow-up commit once 9.x is green.
- [ ] 6.2 `vitest 2 → 4` (two majors) in `sdks/typescript/package.json`. v3 moved the pool config; v4 reworked `environment` defaults and some matcher APIs. Co-bump `@vitest/coverage-v8` to the same major. Rerun `pnpm install && pnpm build && pnpm test`.
- [ ] 6.3 `@types/node 24 → 25` in `sdks/typescript/package.json`. Gate on the Node version pin in `.github/workflows/sdk-typescript-test.yml` — bump the matrix to Node 25 first, then the types. Rerun the TS SDK gates.

## 7. GUI

- [ ] 7.1 `typescript 5 → 6` in `gui/package.json`. Check `tsconfig*.json` for `moduleResolution` + `lib` entries the new compiler may have tightened; rerun `pnpm type-check && pnpm build`.
- [ ] 7.2 `vite 7 → 8` + `@vitejs/plugin-vue` matching-major in `gui/package.json`. Audit `vite.config.ts` plugin options; rerun `pnpm build`.
- [ ] 7.3 `vue-router 4 → 5` in `gui/package.json`. Audit every `useRouter()` / `useRoute()` call site in `gui/src/**/*.{ts,vue}` and every `<router-link>` prop that changed. Rerun `pnpm build` + the GUI e2e smoke.
- [ ] 7.4 `uuid 13 → 14` in `gui/package.json`. Tiny surface — the default ESM export likely moved; grep `from 'uuid'` imports and adjust. Rerun `pnpm build`.
- [ ] 7.5 `electron 39 → 41` (two majors) + verify `electron-builder` compat at 26+. Rebuild the installers locally for at least Windows + macOS and smoke-test code-signing + auto-update before committing. Rerun `pnpm electron:build:win` (+ mac on a macOS box).

## 8. Dashboard — React 19 family (must co-move)

- [ ] 8.1 `react 18 → 19` + `react-dom 18 → 19` + `@types/react 18 → 19` + `@types/react-dom 18 → 19` in one commit — they are version-locked. Audit `ReactDOM.render` → `ReactDOM.createRoot` already-done (React 18); new concerns in 19 include `useOptimistic` / `useFormStatus` availability, the removal of `forwardRef` in some patterns, the deprecated `propTypes` surface. Rerun `pnpm build && pnpm test:run`.
- [ ] 8.2 `react-router 6 → 7` + `react-router-dom 6 → 7` in lockstep. The v7 API moves to the data/loader model — audit every `createBrowserRouter` call + every `<Route>` config in `dashboard/src/router/**`. Rerun the Dashboard gates.
- [ ] 8.3 `@vitejs/plugin-react 4 → 6` in `dashboard/package.json`. Gates on React 19 landing first (8.1). Rerun the Dashboard gates.

## 9. Dashboard — remaining majors

- [ ] 9.1 `eslint 9 → 10` + `@eslint/js 9 → 10` in lockstep. The flat-config file already exists in `dashboard/eslint.config.*`; bump, resolve any removed rules, rerun `pnpm lint`.
- [ ] 9.2 `typescript 5 → 6` in `dashboard/package.json`. Same playbook as 7.1 — check `tsconfig*.json`, rerun `pnpm build`.
- [ ] 9.3 `vite 7 → 8` in `dashboard/package.json`. Same playbook as 7.2 — audit `vite.config.*`, rerun `pnpm build`.
- [ ] 9.4 `jsdom 27 → 29` (two majors) in `dashboard/package.json`. Used by vitest's `environment: 'jsdom'`; check `dashboard/vitest.config.*` for any jsdom-specific options. Rerun `pnpm test:run`.
- [ ] 9.5 `tailwind-merge 2 → 3` in `dashboard/package.json`. Surface is small (`twMerge` + `cn`) — grep for call sites. Rerun `pnpm build`.
- [ ] 9.6 `@types/node 24 → 25` in `dashboard/package.json`. Gate on the Node pin in CI (same playbook as 6.3).

## 10. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 10.1 Update or create documentation covering the implementation (CHANGELOG entry per migration under `### Changed`; if a migration ships a user-visible API change, a migration-guide note under `docs/migration/`).
- [ ] 10.2 Write tests covering the new behavior (each API-breaking migration must land with a test that exercises the post-bump call site; pure-version bumps rely on the existing suite).
- [ ] 10.3 Run tests and confirm they pass (the per-ecosystem gates listed in each section are the pass criteria).
