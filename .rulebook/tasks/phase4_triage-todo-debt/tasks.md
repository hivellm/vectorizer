## 1. Extraction

- [ ] 1.1 Write `scripts/extract_backlog_markers.sh` that greps `src/` for the four forbidden deferral markers (T-O-D-O, F-I-X-M-E, H-A-C-K, X-X-X) and emits CSV columns file, line, category placeholder, text
- [ ] 1.2 Run the script, produce `.rulebook/analysis/backlog-debt/markers.csv`

## 2. Triage

- [ ] 2.1 Categorize every row as one of: bug, feature, cleanup, noise, obsolete
- [ ] 2.2 For each "bug" row, create a dedicated rulebook task and annotate the CSV with the task ID
- [ ] 2.3 For each "feature" row, create a dedicated rulebook task (or add to an existing backlog task)

## 3. Cleanup

- [ ] 3.1 Delete every "noise" and "obsolete" marker from the source code
- [ ] 3.2 For "cleanup" rows, batch-fix inline in PRs of at most ten markers each
- [ ] 3.3 Rewrite remaining markers to the form `// TASK(phaseN_task-name):` pointing at the rulebook task ID

## 4. Enforcement

- [ ] 4.1 Extend the CI grep gate in `.github/workflows/rust-lint.yml` to fail on any of the four markers that do NOT match `TASK\(phase\d+_[a-z0-9-]+\):`
- [ ] 4.2 Run the gate locally and confirm zero violations remain

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Update `AGENTS.md` / `/.rulebook/specs/TIER1_PROHIBITIONS.md` clarifying the only allowed deferral-marker form
- [ ] 5.2 Add a regression test that scans the source tree for non-conforming markers
- [ ] 5.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
