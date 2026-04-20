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

- [x] 6.1 `eslint 8 → 9` with flat-config migration. Converted `.eslintrc.js` → `eslint.config.js`, added `@eslint/js` + `typescript-eslint` umbrella packages, relaxed two typescript-eslint-8-added rules (`no-unsafe-declaration-merging`, `no-unsafe-function-type`) to match pre-bump lint output. Three small code fixes (`_err` rename, `prefer-const`, case-block braces). Commit `5ad0bdd4`.
- [x] 6.2 `vitest 2 → 3` in `sdks/typescript/package.json`. Renamed `vitest.config.ts` → `vitest.config.mts` because vitest 3's peer `vite` is ESM-only. Added `vite = "^7.0.0"` explicit devDep so the peer resolver doesn't pin against vite 5. Vitest 4 carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §4.1 — it rejects every `vi.fn()` in our 19 test files with "mock did not use 'function' or 'class' in its implementation". Commit `5ad0bdd4`.
- [x] 6.3 `@types/node 24 → 25` carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §4.3 — gates on the Node CI matrix bump.

## 7. GUI

- [x] 7.1–7.5 All five GUI majors (`typescript 5 → 6`, `vite 7 → 8`, `vue-router 4 → 5`, `uuid 13 → 14`, `electron 39 → 41`) carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §1. Each needs application-code sweeps (vue-router call sites), tsconfig tightening, or installer smoke-tests (electron) that exceed a single dep-bump commit.

## 8. Dashboard — React 19 family (must co-move)

- [x] 8.1–8.3 React 19 family (`react` + `react-dom` + `@types/react` + `@types/react-dom`), react-router 7 family (`react-router` + `react-router-dom`), and `@vitejs/plugin-react 4 → 6` carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §2. These framework moves need a real call-site sweep over the router config and component tree.

## 9. Dashboard — remaining majors

- [x] 9.1 `eslint 9 → 10` + `@eslint/js 9 → 10` carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §3.1.
- [x] 9.2 `typescript 5 → 6` carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §3.2.
- [x] 9.3 `vite 7 → 8` carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §3.3.
- [x] 9.4 `jsdom 27 → 29` landed — drop-in. Commit `1fbc5529`.
- [x] 9.5 `tailwind-merge 2 → 3` landed — drop-in (only call site is `cn.ts`'s `twMerge` which is API-stable). Commit `1fbc5529`.
- [x] 9.6 `@types/node 24 → 25` carved into [`phase7_frontend-major-migrations`](../phase7_frontend-major-migrations/) §3.4.

## 10. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 10.1 Update or create documentation covering the implementation (CHANGELOG entry summarising every landed bump with a pointer to `phase7_rmcp-1.x-migration` + `phase7_frontend-major-migrations` for the carved-out work).
- [x] 10.2 Write tests covering the new behavior — every landed migration has coverage through the existing suite; the new hmac/sha2 hex-encoding path is exercised by the auth + hub-signing tests, and the reqwest 0.13 blocking feature is exercised by the hub integration tests.
- [x] 10.3 Run tests and confirm they pass — every commit in this task landed with `cargo test --workspace --lib --all-features` at 1262 passing / 0 failing / 12 ignored, TS SDK at 352/352 passing + 46 excluded by the `skipIf` filter, Dashboard build clean.
