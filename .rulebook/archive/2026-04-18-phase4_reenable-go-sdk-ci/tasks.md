## 1. Decision

- [x] 1.1 Inspect `sdks/go/` — found a real, maintained module (`github.com/hivellm/vectorizer-sdk-go`, Go 1.21). 16 source files including `client.go`, `collections.go`, `vectors.go`, `batch.go`, `graph.go`, plus test files `qdrant_test.go`, `graph_test.go`, `file_upload_test.go` and an `examples/` dir. `go.mod` is valid, `doc.go` is a proper package header with quickstart snippet.
- [x] 1.2 Chose **Option A (re-enable)**. Rationale: the SDK is actual working code with its own test suite and examples; deleting it would break any current Go consumer. The workflow had been renamed `.disabled` but the code underneath kept being maintained. Re-enabling is a rename + a local verification.

## 2. Option A — Re-enable

- [x] 2.1 `git mv .github/workflows/sdk-go-test.yml.disabled .github/workflows/sdk-go-test.yml`. Workflow triggers on push/PR scoped to `sdks/go/**` + itself.
- [x] 2.2 Ran the test suite locally:
  - `go vet ./...` — clean.
  - `go test -v -short -count=1 ./...` — 0.244s, PASS; integration tests that need a reachable server emit a readable "test not run — no server" log and return success.
- [x] 2.3 Elevating the job to a required check on `main` is a GitHub branch-protection settings action (not a code change). The commit message flags this so the user can click through Settings → Branches → Branch protection when ready.

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 3.1 CHANGELOG `[Unreleased] > CI / Testing` entry added naming the re-enable, the test matrix (Ubuntu + macOS, Go 1.21 + 1.22), and the local verification commands.
- [x] 3.2 Tests covering the new behavior — the Go SDK's own test suite IS the coverage. No Rust-side test needed; the workflow is the artifact.
- [x] 3.3 Local `go test -v -short -count=1 ./...` green; `go vet ./...` clean.

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation (CHANGELOG `CI / Testing` entry)
- [x] Write tests covering the new behavior (existing Go SDK test suite)
- [x] Run tests and confirm they pass (`go test -v -short` passing in 0.244s)
