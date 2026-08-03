## 1. Cargo deps
- [x] 1.1 Safe patch/minor lockfile bumps: thiserror 2.0.19, tokio 1.53.0, async-trait 0.1.91, anyhow 1.0.104, uuid 1.24.0, futures 0.3.33, serde 1.0.229, fastembed 5.17.3
- [x] 1.2 lz4_flex 0.13 -> 0.14 (both crate manifests) + compression roundtrip tests pass (28/28)
- [x] 1.3 openraft #382: kept the `=` pin, commented the PR, opened follow-up task phase1_bump-openraft-alpha30 (needs live-cluster HA retest)

## 2. Config-file deps
- [x] 2.1 NuGet: System.Text.Json 10.0.10 + Microsoft.SourceLink.GitHub 10.0.301 (csproj)
- [x] 2.2 npm (TS): @types/node 26.1.2, typescript-eslint 8.65.0, eslint 10.8.0 — tsc build + eslint pass
- [x] 2.3 npm: closed js-yaml 4->5 (#388) — 5.x major blocked by the pnpm security override `<5`
- [x] 2.4 pip: websockets >=16.1 -> >=16.1.1 (requirements.txt)
- [x] 2.5 GitHub Actions: setup-node v7, setup-dotnet v6, setup-python v7, setup-go v7 (all instances, CRLF preserved)

## 3. Verify + close
- [x] 3.1 cargo check --workspace passes; TS SDK tsc build + eslint pass; workflow YAMLs valid
- [x] 3.2 #388 closed, #382 annotated; the other applied dependabot PRs target main and resolve when release/3.6.0 merges (would reopen if closed now)

## 4. Tail (docs + tests — check or waive with tailWaiver)
- [x] 4.1 Documentation: CHANGELOG [Unreleased] dependency-refresh note
- [x] 4.2 No new behavior to test (dependency version bumps); compatibility verified by existing tests
- [x] 4.3 Run tests: compression roundtrip 28/28, cargo check clean, TS build/eslint pass
