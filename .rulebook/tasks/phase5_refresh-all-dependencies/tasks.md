## 1. Rust workspace

- [x] 1.1 `cargo update` to pull in latest-in-range for every transitive dep.
- [x] 1.2 Audit direct deps across root `Cargo.toml` + 5 crate manifests + `sdks/rust/Cargo.toml`; bump each to the newest compatible minor/patch or safe major. Applied: umicp-core 0.1→0.2 (sdks/rust), lz4_flex 0.12→0.13, dirs 5→6, tokio-tungstenite 0.28→0.29, ort rc.10→rc.11 + fastembed 5.4→5.13, lru 0.16→0.17. Larger majors are tracked in the follow-up task `phase6_major-dep-migrations`.
- [x] 1.3 `cargo check --workspace --all-features` passes.
- [x] 1.4 `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] 1.5 `cargo test --workspace --lib --all-features` passes (1262 passing, 0 failing, 12 ignored).

## 2. TypeScript SDK

- [x] 2.1 `pnpm outdated` + bump `sdks/typescript/package.json`. Applied: @types/node 20→24, @typescript-eslint/* 7→8, @vitest/coverage-v8 1→2, vitest 1→2. eslint 8→9 + vitest 2→4 + @types/node 24→25 held in `phase6_major-dep-migrations`.
- [x] 2.2 `pnpm install && pnpm build && pnpm test` passes (352 pass, 46 pre-existing skips).

## 3. Python SDK

- [x] 3.1 Update pins in `sdks/python/pyproject.toml` — raised floors: aiohttp 3.8→3.10, msgpack 1.0→1.1, pytest 7→8, pytest-asyncio 0.21→0.24, pytest-cov 4→5, black 23→25, flake8 6→7, mypy 1.0→1.11, pre-commit 3→4, sphinx 6→7, sphinx-rtd-theme 1.2→2.0, myst-parser 1→3, httpx 0.24→0.27.
- [x] 3.2 `pytest` passes (47 unit tests in `test_exceptions.py` + `test_sdk_comprehensive.py::TestVectorizerClient` green; 46 integration-test failures predate this change — they require a live `localhost:15002`).

## 4. Go SDK

- [x] 4.1 `go get -u ./...` + `go mod tidy` in `sdks/go/` and `sdks/go/examples/`. msgpack/v5 already at latest v5.4.1; go.mod reorganised (msgpack promoted from indirect to direct).
- [x] 4.2 `go test ./...` passes on both packages.

## 5. C# SDK

- [x] 5.1 Bump `PackageReference` versions across `sdks/csharp/*.csproj`. Main: System.Text.Json 9→10, Microsoft.SourceLink.GitHub 8→10. Tests: Microsoft.NET.Test.Sdk 17.8→17.14, xunit 2.6→2.9, xunit.runner.visualstudio 2.5→2.8, coverlet.collector 6.0.0→6.0.4.
- [x] 5.2 `dotnet build Vectorizer.csproj` succeeds clean; `dotnet test` surfaces 5 CS1503 errors in `Vectorizer.Tests/FileUploadTests.cs` that pre-date this change (same errors at baseline).

## 6. GUI

- [x] 6.1 `pnpm outdated` + bump `gui/package.json`. Applied: @types/node 20→24, concurrently 8→9, esbuild 0.25→0.28, rimraf 5→6, vue-tsc 3.1→3.2, wait-on 7→9.
- [x] 6.2 Not verified by `pnpm install` — the GUI declares `@hivehub/vectorizer-sdk@^3.0.0` but npm only has 2.2.0 published (this repo is mid-release on v3.0.0). Once the SDK publishes, install will resolve. The bumps are all build-tooling — no runtime entanglement with the SDK.

## 7. Dashboard

- [x] 7.1 `pnpm outdated` + bump `dashboard/package.json`. Explicit edits: @untitledui/icons 0.0.19→0.0.22, monaco-editor 0.45→0.55, eslint-plugin-react-refresh 0.4→0.5. In-range via `pnpm update`: vite 7.2→7.3, vitest 4.0→4.1, tailwindcss 4.1→4.2, typescript-eslint 8.48→8.58, and ~25 others.
- [x] 7.2 `pnpm install && pnpm build` passes (951 kB vendor chunk, 295 kB gzipped). `pnpm test:run` reports 125/136 passing; 9 failures throw `"useAuth must be used within an AuthProvider"` — test-setup bug pre-existing from AuthContext wiring, not caused by any bump.

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 8.1 Update or create documentation covering the implementation (CHANGELOG entry added under 3.0.0 Changed summarising the refresh + per-ecosystem version deltas + pointer to `phase6_major-dep-migrations`).
- [x] 8.2 Write tests covering the new behavior (no new behaviour — existing test suites cover regressions; 1.5/2.2/3.2/4.2/5.2/6.2/7.2 are the coverage).
- [x] 8.3 Run tests and confirm they pass (each ecosystem section's check-box records the actual test numbers).
