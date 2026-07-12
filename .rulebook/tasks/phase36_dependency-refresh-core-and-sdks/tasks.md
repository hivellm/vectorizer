## 1. Dependabot queue — low-risk bumps first (#336–#345)

- [x] 1.1 Retargeted + merged the low-risk PRs into `release/3.5.0`: uuid 1.23.4 (#336), tracing-opentelemetry 0.33.0 (#338), bcrypt 0.19.2 (#339), memmap2 0.9.11 (#340), transmutation 0.3.3 (#344), xxhash-rust 0.8.16 (#345)
- [x] 1.2 `cargo check --workspace` green after the low-risk wave

## 2. Dependabot queue — majors, one compat commit each

- [x] 2.1 `rmcp` 1.7.0 → 2.1.0: `Content` → `ContentBlock` rename across the 5 MCP handler modules (only breaking surface for us); vectorizer-server lib 214 tests + 30 MCP integration tests pass; clippy clean; #342 closed as superseded
- [x] 2.2 candle family 0.10.2 → 0.11.0 lockstep (commit 3bc27637, supersedes #341/#337) incl. vectorizer-core's own optional pin (two-error-types E0277 trap documented in Cargo.toml comments)
- [x] 2.3 `aes-gcm` 0.10.3 → 0.11.0 (commit cd9ad953, supersedes #343): Generate trait for key/nonce (aead 0.6 dropped OsRng re-export); p256 rand_core 0.6 OsRng for SecretKey::random
- [x] 2.4 Full gate after majors: `cargo clippy --workspace --all-targets` clean + `cargo nextest run --workspace --no-fail-fast` = **1822 passed, 13 excluded-by-ignore** (single gate run covers post-major + post-update state, §3.2)

## 3. Core workspace compatible-range refresh

- [x] 3.1 `cargo update` workspace-wide — ~120 compatible bumps (zerocopy 0.8.54, zeroize 1.9, winnow, yoke, wit-* removals, etc.); security pins survived: cmov 0.5.4, openraft =0.10.0-alpha.22
- [x] 3.2 Full gate re-run on the updated tree (same run as §2.4): clippy 0 warnings, nextest 1822/1822, no rollback pins needed

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
