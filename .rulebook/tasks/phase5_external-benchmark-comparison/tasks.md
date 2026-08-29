## 1. Implementation

Ordered so the thing most likely to be wrong — id mapping — is proven before
anything is measured.

- [x] 1.1 Scaffold `benchmarks/external/`: a runner that clones
      `qdrant/vector-db-benchmark` at a **pinned commit** into a gitignored
      workdir and overlays our engine client. Pinning is not optional — an
      unpinned upstream changes what the numbers mean between runs, silently.
      Registration in `engine/clients/client_factory.py` (three dicts +
      imports) is applied as a scripted edit rather than a vendored copy of
      their file, so upstream drift shows up as a failed patch instead of a
      stale fork.
      **Done when:** the runner produces a working tree where
      `python run.py --engines vectorizer --help` resolves the engine.
      `setup.py` clones at `91e39da0`, overlays 8 files and patches the three
      registries; `--check` reports pinned + registered, and the patched
      `client_factory.py` plus every client module compiles. Our files live in
      `overlay/` (committed); `.work/` is the materialised clone and is
      gitignored — a build directory, never edited by hand.
- [ ] 1.2 The `vectorizer` engine client — `config.py`, `configure.py`,
      `upload.py`, `search.py`, `parser.py`, `__init__.py` — against the SDK
      or plain REST.
      **The id contract is the whole ballgame.** The framework scores with
      `len(returned_ids ∩ query.expected_result[:top]) / top`, so the ids we
      return must be the dataset's integers. Vectorizer's `Vector.id` is a
      `String`, so upload writes `str(record.id)` and search returns
      `int(hit.id)`. Getting this wrong yields exactly 0.00% precision at full
      speed — the signature of the report this task retracts.
      **Done when:** a 1k-vector smoke run reports precision > 0.9 against a
      dataset with ground truth.
      **Written, not yet proven** — the smoke run needs the framework's deps
      installed and a live server, which is 1.4. Code is in
      `overlay/engine/clients/vectorizer/`; all modules compile. Leaving this
      unchecked on purpose: the point of the item is the measured precision,
      and claiming it from a clean compile is the mistake this task is about.
      Three traps closed with errors rather than silence while writing it:
      - `/insert_vectors` answers 200 with a per-row failure count, so a
        partial upload is a successful HTTP call. Fewer vectors than the
        dataset lowers recall on every later query, and the run would report
        that as search quality. Now raises.
      - The server clamps search `limit` to `MAX_SEARCH_LIMIT = 100`
        **silently**. A top-1000 dataset would be scored against 100 results
        and report ~10% recall — a truncation misread as a finding. The
        searcher refuses above the cap.
      - `parser.py` raises instead of returning an empty filter, so a
        filtered dataset cannot run unfiltered and be scored against filtered
        ground truth, which would look excellent.
      Transport is `http.client` with the socket held open per worker, not
      per-request `urllib`: pgvector compares via pooled `psycopg` with
      prepared statements and Qdrant via its own pooled client, so a
      connection per request would measure handshakes and blame the engine.
- [x] 1.3 Recall gate in the runner: refuse to emit a latency comparison when
      `mean_precisions` falls below a floor (default 0.9), naming the engine
      and the value. A latency number next to zero recall is not a slower or
      faster engine, it is a broken measurement, and the existing report is
      what happens without this.
      **Done when:** a deliberately broken client (ids not mapped) fails the
      gate instead of publishing a 5x win.
      `report.py` + `tests/test_report_gate.py`, 7 tests passing. The first
      reproduces the retracted report exactly — 0% recall at 6285 rps against
      Qdrant's 100% at 1183 — and asserts not merely a non-zero exit but that
      **no table is printed**: a caveat printed next to a number is how the
      original got quoted. Also refuses a result file that omits
      `mean_precisions`, or the gate would be bypassed by leaving the field
      out, and labels a single-engine run a baseline so it cannot be read as
      a comparison.
- [x] 1.4 Compose file bringing up Vectorizer, Qdrant, Weaviate and pgvector
      with comparable resource limits, plus the dataset fetch step. Unequal
      memory or thread caps between engines invalidates the comparison before
      it starts.
      **Done when:** all four answer a health probe from one `docker compose
      up`.
      Verified: `pgvector`, `qdrant`, `vectorizer`, `weaviate` all report
      `(healthy)` 36s after a single `up -d`. The dataset step works too —
      the framework's own downloader pulled `glove-100-angular.hdf5` (485 MB)
      into `.work/datasets/`.
      `setup.py --venv` was added here rather than left as a README
      instruction: the harness pins `python = ">=3.10,<3.13"`, and its
      dependency set is read from `poetry.lock`, not `pyproject.toml`, because
      the manifest asks for `qdrant-client` from a git *branch* — the same
      silent drift that pinning the upstream commit exists to prevent. The
      lock records the commit it was developed against
      (`286ae82b`).
- [ ] 1.5 First honest run on a shared dataset, results committed under
      `benchmarks/external/results/` with the engine versions, host specs and
      resource limits recorded alongside. Whatever it says.
      **Done when:** the four engines have comparable recall and the report
      states where Vectorizer loses as plainly as where it wins.
- [x] 1.6 Retract `docs/specs/benchmarks/qdrant_comparison_2025-11-24_*` (4
      files) with a header on each explaining that the result is void: it
      declares a 5.31x search win at 0.00% recall, and the harness that
      produced it was never a declared bench target. Point at the replacement.
      Done: a blockquote header on both `.md` files and a first-key
      `"RETRACTED"` object in both `.json` files — JSON carries no comments,
      and a machine reader that never sees the markdown must still meet the
      notice before a number. Kept rather than deleted because the figures
      were quoted while they stood, so an old link has to find the correction
      rather than a 404.
- [x] 1.7 Delete `benches/comparison/qdrant_comparison_benchmark.rs`. It is
      not in the 17 `[[bench]]` targets, so nothing compiles it, and it is the
      source of the void number. Removing it is what stops someone re-running
      it.
      Deleted, plus the two live references that claimed it was runnable:
      `benches/README.md` documented a `cargo run --bin
      qdrant_comparison_benchmark` that could not have worked (the `[[bin]]`
      has been commented out since the workspace split), and the Cargo.toml
      note listed it as pending re-registration. That note now records why it
      was removed instead — being unregistered was not a temporary state to
      fix, it is how the file drifted long enough to publish a void number.
      `docs/patches/v1.7.0-1.7.9.md` still mentions it and stays as-is: it is
      a historical record of what 1.7.x shipped.

## 2. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 2.1 Runbook in `docs/development/`: bringing the four engines up, which
      datasets, how to read the output, and what the numbers do **not** claim
      (single-host, specific dataset, specific parameters). Link it from
      `docs/specs/BENCHMARKING.md`, which today only covers the internal
      criterion suite.
- [ ] 2.2 Tests for the engine client's id round-trip — the one piece of
      logic that is ours and that silently produces a plausible-looking wrong
      answer. Assert an uploaded record comes back under the same integer id,
      and that the recall gate rejects a sub-threshold run.
- [ ] 2.3 Run them, plus the workspace gate if any Rust changed (1.7 removes
      a file; confirm nothing referenced it).
