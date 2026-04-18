## 1. Decision

- [ ] 1.1 Inspect `sdks/go/` state (coverage, staleness) and record findings in `design.md`
- [ ] 1.2 Choose Option A (re-enable) or Option B (deprecate) and record via `rulebook_decision_create`

## 2. Option A — Re-enable

- [ ] 2.1 Rename `.github/workflows/sdk-go-test.yml.disabled` back to `sdk-go-test.yml`
- [ ] 2.2 Run the workflow locally (via `act` or push a branch); fix any failures in `sdks/go/`
- [ ] 2.3 Add the job to required checks on `main` in branch protection

## 3. Option B — Deprecate

- [ ] 3.1 Delete `.github/workflows/sdk-go-test.yml.disabled`
- [ ] 3.2 Delete `sdks/go/`
- [ ] 3.3 Remove Go SDK references from README, docs, dockerhub-readme, CHANGELOG notes

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update `README.md` and `docs/sdks/` to reflect the current set of supported SDKs
- [ ] 4.2 Under Option A: add a smoke integration test that exercises a published server from Go; Under Option B: no test needed
- [ ] 4.3 Run the chosen path's test command and confirm green

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
