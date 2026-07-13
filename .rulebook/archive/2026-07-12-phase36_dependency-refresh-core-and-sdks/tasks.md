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

- [x] 4.1 TypeScript: `pnpm update` within ranges — @types/node 25.9.5, vitest/coverage 4.1.10, eslint 10.7.0, typescript-eslint 8.63.0, js-yaml 4.3.0 (override `<5` respected); overrides block byte-identical; **515 passed / 12 env-gated** + tsc 0 errors + lint 0 errors; vite hard-pinned at 8.0.16 by the security override (pnpm rewrites the spec — documented)
- [x] 4.2 Python: floors raised in pyproject (aiohttp 3.10.11, msgpack 1.1.1, pytest 8.3.5, mypy 1.14.1, flake8 7.1.2, httpx 0.28.1, sphinx-rtd-theme 3.1.0); **94 passed / 5 env-gated**; 1 failing test + 2 collection errors are pre-existing live-server/bit-rot issues (import `vectorizer_sdk` inexistente; connection-refused sem servidor) — registrados no escopo do phase39
- [x] 4.3 Go: `go get -u` + `go mod tidy` = zero changes (msgpack v5.4.1 e tagparser v2.0.0 já são as últimas da major line); `go vet` limpo; `go test -short` ok (2 módulos)
- [x] 4.4 C#: System.Text.Json 10.0.9, SourceLink 10.0.300, MessagePack 2.5.302; Microsoft.Extensions.* já na última patch da linha 8.x (DI core não tem 8.0.2 — verificado no índice NuGet); 7 csproj build limpos; RPC tests **66/66**; 21 falhas em Vectorizer.Tests são pré-existentes (detecção de connection-refused por substring em inglês quebra em Windows pt-BR) — item 3.4 do phase39

## 5. Dependabot config hygiene

- [x] 5.1 `.github/dependabot.yml` estendido: npm (sdks/typescript), pip (sdks/python), gomod (sdks/go), nuget (sdks/csharp), weekly monday 09:00; sdks/rust coberto pela entry cargo raiz (workspace member, lock compartilhado — entry separada duplicaria PRs)
- [x] 5.2 Config validada estruturalmente por teste (tests/dependabot_coverage.rs, tail 6.2); a aba Insights → Dependabot só reflete após push do branch

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Update or create documentation covering the implementation — CHANGELOG [3.5.0] Build section (majors, cargo update, dependabot SDK coverage)
- [x] 6.2 Write tests covering the new behavior — tests/dependabot_coverage.rs pins the SDK-ecosystem invariant
- [x] 6.3 Run tests and confirm they pass — dependabot_coverage 1/1; full gate nextest 1822 passed / clippy 0
