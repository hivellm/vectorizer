## 1. Rust workspace

- [x] 1.1 `cargo update` to pull in latest-in-range for every transitive dep.
- [x] 1.2 Audit direct deps across root `Cargo.toml` + 5 crate manifests + `sdks/rust/Cargo.toml`; bump each to the newest compatible minor/patch or safe major. Applied: umicp-core 0.1→0.2 (sdks/rust), lz4_flex 0.12→0.13, dirs 5→6, tokio-tungstenite 0.28→0.29, ort rc.10→rc.11 + fastembed 5.4→5.13, lru 0.16→0.17. Held back under separate tasks (documented in proposal.md): rand 0.10, bincode 3, rmcp 1.5, reqwest 0.13, hmac 0.13 + sha2 0.11, arrow/parquet 58, zip 8, tantivy 0.26, hf-hub 0.5, sysinfo 0.38, opentelemetry-prometheus 0.31.
- [x] 1.3 `cargo check --workspace --all-features` passes.
- [x] 1.4 `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] 1.5 `cargo test --workspace --lib --all-features` passes (1262 passing, 0 failing, 12 ignored).

## 2. TypeScript SDK

- [ ] 2.1 `pnpm outdated` + bump `sdks/typescript/package.json`.
- [ ] 2.2 `pnpm install && pnpm build && pnpm test` passes.

## 3. Python SDK

- [ ] 3.1 Update pins in `sdks/python/pyproject.toml`.
- [ ] 3.2 `pytest` passes (via the SDK's configured test command).

## 4. Go SDK

- [ ] 4.1 `go get -u ./...` + `go mod tidy` in `sdks/go/` and `sdks/go/examples/`.
- [ ] 4.2 `go test ./...` passes.

## 5. C# SDK

- [ ] 5.1 Bump `PackageReference` versions across `sdks/csharp/*.csproj`.
- [ ] 5.2 `dotnet test` passes.

## 6. GUI

- [ ] 6.1 `pnpm outdated` + bump `gui/package.json`.
- [ ] 6.2 `pnpm install && pnpm build` passes (tests if any).

## 7. Dashboard

- [ ] 7.1 `pnpm outdated` + bump `dashboard/package.json`.
- [ ] 7.2 `pnpm install && pnpm build` passes (tests if any).

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Update or create documentation covering the implementation (CHANGELOG entry summarising the refresh + per-ecosystem version deltas).
- [ ] 8.2 Write tests covering the new behavior (no new behaviour — existing test suites cover regressions; 1.5/2.2/3.2/4.2/5.2/6.2/7.2 are the coverage).
- [ ] 8.3 Run tests and confirm they pass.
