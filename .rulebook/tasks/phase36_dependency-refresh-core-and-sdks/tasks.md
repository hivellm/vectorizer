## 1. Dependabot queue — low-risk bumps first (#336–#345)

- [ ] 1.1 Retarget the 7 patch/minor PRs to `release/3.5.0`: uuid 1.23.4 (#336), tracing-opentelemetry 0.33.0 (#338), bcrypt 0.19.2 (#339), memmap2 0.9.11 (#340), transmutation 0.3.3 (#344), xxhash-rust 0.8.16 (#345) — merge cascade with rebase waits between waves
- [ ] 1.2 Confirm `cargo check --workspace` green after the low-risk wave

## 2. Dependabot queue — majors, one compat commit each

- [ ] 2.1 `rmcp` 1.7.0 → 2.1.0 (#342): merge, fix `crates/vectorizer-server/src/server/mcp/*` against the 2.x API (tool registration, CallToolResult, transport surface), verify MCP protocol version stays `2025-03-26`, run MCP-touching tests (`umicp::discovery`, capability inventory, mcp handler units)
- [ ] 2.2 `candle-core` + `candle-transformers` 0.10.2 → 0.11.0 (#341 + #337 together, plus `candle-nn` in the same commit — the family must move in lockstep): fix `crates/vectorizer/src/embedding/*` candle providers, run `cargo check --features candle-models`
- [ ] 2.3 `aes-gcm` 0.10.3 → 0.11.0 (#343): fix auth persistence (`crates/vectorizer/src/auth/persistence.rs`) + payload encryption (`src/security/payload_encryption.rs`) call sites against the 0.11 API (cipher init + nonce types moved), run the encryption test suites (`api/rest/encryption*.rs` unit-level, auth persistence round-trip)
- [ ] 2.4 Full gate after majors: `cargo nextest run --workspace --no-fail-fast` + `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## 3. Core workspace compatible-range refresh

- [ ] 3.1 `cargo update` (workspace-wide, semver-compatible) — captures the 120+ "behind latest" minors dependabot doesn't PR
- [ ] 3.2 Re-run the full gate (nextest + clippy + fmt); bisect any breakage with `cargo update -p <pkg> --precise` rollbacks — each rollback documented inline in Cargo.toml when a pin is required

## 4. SDK dependency refresh (one commit per SDK)

- [ ] 4.1 TypeScript: `pnpm update` within ranges in `sdks/typescript`; review `@msgpack/msgpack`, vitest, eslint for safe majors; keep phase-security overrides (js-yaml <5 pin, esbuild, vite) intact; `pnpm test` green
- [ ] 4.2 Python: refresh `sdks/python/pyproject.toml` dev/runtime pins (pytest, msgpack, httpx/requests line) to current stable; run the SDK's pytest suite
- [ ] 4.3 Go: `go get -u ./... && go mod tidy` in `sdks/go` (stay within module major); `go vet ./... && go test -short ./...` green
- [ ] 4.4 C#: refresh `Microsoft.Extensions.*` to latest 8.x patch across `sdks/csharp/**/*.csproj` (MessagePack already 2.5.301); `dotnet test` green

## 5. Dependabot config hygiene

- [ ] 5.1 Extend `.github/dependabot.yml` with update entries for `sdks/typescript` (npm), `sdks/python` (pip), `sdks/go` (gomod), `sdks/csharp` (nuget) at the same weekly cadence as the root cargo ecosystem
- [ ] 5.2 Verify the config parses (dependabot validates on push; check the Insights → Dependency graph → Dependabot tab shows the new ecosystems)

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
