# External benchmarks — Vectorizer vs Qdrant, Weaviate and pgvector

This runbook covers the **cross-engine** comparison in `benchmarks/external/`.
For the internal Criterion suite (per-function timing of our own code), see
[BENCHMARKING.md](../specs/BENCHMARKING.md).

The two are not interchangeable. Criterion answers "did this commit make our dot
product faster". This answers "how does Vectorizer compare to other vector
databases on the same workload", and it is far easier to get catastrophically
wrong — see [What this replaces](#what-this-replaces).

## Why we run someone else's harness

We do not write our own comparison harness. We run
[qdrant/vector-db-benchmark](https://github.com/qdrant/vector-db-benchmark) at a
pinned commit, with a Vectorizer engine client overlaid onto it.

A home-grown harness measures whatever its author believed each engine does. The
upstream one was written by a competitor whose own published numbers depend on
it being fair, it is what the rest of the field measures against, and —
decisively — it scores recall on every run, so a client that returns the wrong
results cannot look fast.

## Layout

| Path | What it is |
|---|---|
| `benchmarks/external/upstream.json` | The pinned upstream commit |
| `benchmarks/external/overlay/` | **Our** files — engine client + experiment config. Edit these. |
| `benchmarks/external/setup.py` | Clones upstream, applies the overlay, builds the venv |
| `benchmarks/external/docker-compose.yml` | The four engines under identical limits |
| `benchmarks/external/report.py` | Recall gate + comparison table |
| `benchmarks/external/.work/` | Materialised clone. Gitignored, a build directory. **Never edit by hand** — `setup.py` overwrites it. |
| `benchmarks/external/results/` | Committed runs |

## Running it

### 1. Materialise the workspace

```bash
cd benchmarks/external
python setup.py           # clone at the pinned commit + overlay + register the engine
python setup.py --venv    # .work/.venv with the dependencies poetry.lock pins
python setup.py --check   # pinned? registered? venv present?
```

`--venv` reads `poetry.lock`, not `pyproject.toml`. The manifest asks for
`qdrant-client` from a git *branch*, which resolves to whatever that branch
points at today; the lock records the commit the harness was developed against.
It also builds on Python 3.12, because upstream declares `>=3.10,<3.13`.

### 2. Bring the engines up

```bash
docker compose -f benchmarks/external/docker-compose.yml up -d
docker compose -f benchmarks/external/docker-compose.yml ps   # wait for four (healthy)
```

Vectorizer runs from the published `hivehub/vectorizer:3.7.1` — the first tag
carrying the raw-vector sentinel (`embedding_provider: "none"`) the benchmark
client needs to create its collection. No local build is required any more.

That tag is also the first built `FROM scratch`, so **there is no shell in the
image**: debug with `docker logs`, not `docker exec ... sh`.

Wait for `(healthy)`, not merely `Up`. Vectorizer's probe is `/ready`, not
`/health` — `/health` answers 200 while the collection catalog is still loading
(issue #391), so starting on it would benchmark a half-warm server. The binary
is its own probe (`--healthcheck`), since a scratch image has no `wget`.

### 3. Get a token and run

```bash
export VECTORIZER_API_KEY=$(curl -s -X POST http://localhost:15002/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"bench","password":"benchmark-only-not-a-secret"}' \
  | python -c "import sys,json;print(json.load(sys.stdin)['access_token'])")
export QDRANT_API_KEY=benchmark-only-not-a-secret

cd benchmarks/external/.work
./.venv/Scripts/python.exe run.py --engines vectorizer-default \
    --datasets glove-100-angular --host localhost
./.venv/Scripts/python.exe run.py --engines qdrant-default \
    --datasets glove-100-angular --host http://localhost
./.venv/Scripts/python.exe run.py --engines weaviate-default \
    --datasets glove-100-angular --host localhost
./.venv/Scripts/python.exe run.py --engines pgvector-default \
    --datasets glove-100-angular --host localhost
```

Qdrant takes `http://localhost` while the others take a bare `localhost`:
`qdrant-client` switches to TLS as soon as an API key is present, and against a
plain-HTTP server that fails with `SSL: WRONG_VERSION_NUMBER`. Vectorizer's
client builds `{scheme}://{host}:{port}` itself, so it must **not** be handed a
scheme.

### 4. Read the result

```bash
python benchmarks/external/report.py
python benchmarks/external/report.py --min-recall 0.95
```

`report.py` refuses to print a latency table **at all** when mean precision
falls below the floor (default 0.9). That is a precondition, not a column: a
caveat printed next to a number is how the retracted report got quoted.

## Smoke-testing the id contract

The one piece of logic here that is ours, and the one that fails silently:

```bash
./.venv/Scripts/python.exe run.py --engines vectorizer-default \
    --datasets random-100 --host localhost
```

`random-100` is 100 vectors bundled with the harness, with ground truth where
query *i*'s neighbour is vector *i*. Precision must come back **1.0**. Anything
else means the id round-trip is broken: the framework scores with
`len(returned_ids ∩ expected[:top]) / top`, and Vectorizer's vector ids are
strings while the dataset's are integers. `upload.py` writes `str(record.id)`,
`search.py` returns `int(hit["id"])`, and the two must stay a pair.

Run this before trusting any measurement.

## Fairness, and where it is enforced

Every one of these is load-bearing. Change them for all engines or not at all.

- **Resource limits.** `BENCH_CPUS` (4) and `BENCH_MEMORY` (8g) apply to all
  four through one YAML anchor, so they cannot drift apart a service at a time.
- **Auth on everywhere.** Vectorizer *cannot* bind `0.0.0.0` without
  authentication, so it always pays the per-request credential check. Qdrant is
  given an API key for the same reason — leaving it anonymous would hand it a
  head start in exactly the measurement being published.
- **Upload parameters.** `batch_size: 1024` for all four, matching upstream's
  own configs. Vectorizer's overlay was moved from 256 to match, rather than
  keeping a number that might flatter or penalise it.
- **Pinned versions, in pairs.** Qdrant is pinned to v1.17.1 rather than the
  newest release, because the pinned harness ships `qdrant-client` 1.16.3 and
  that client refuses a server more than one minor ahead of it. Bumping only the
  server is how a comparison quietly starts measuring a version mismatch.
- **Connection reuse.** Our client holds one `http.client` socket per worker.
  pgvector compares through pooled `psycopg` with prepared statements and Qdrant
  through its own pooled client; a connection per request would measure TCP
  handshakes and report them as engine latency.

## What the numbers do not claim

- **One host, one configuration.** A single machine, one dataset, one set of
  index parameters. Nothing here speaks to clustered deployments, other
  datasets, other dimensionalities, or per-engine tuning.
- **Default-ish settings.** The `*-default` experiments, not each engine's best
  achievable tuning. An engine that loses here may win when tuned.
- **Recall and latency together, or neither.** A latency figure is only reported
  alongside the recall it was achieved at. Quoting one without the other
  reproduces the exact failure this harness exists to prevent.
- **Not a marketing artifact.** Results land in `benchmarks/external/results/`
  with engine versions, host specs and resource limits recorded alongside —
  whatever they say.

## What this replaces

`docs/specs/benchmarks/qdrant_comparison_2025-11-24_*` declared a **5.31x search
win for Vectorizer at 0.00% Recall@10**, against Qdrant's 100.00%. Zero recall
means the returned ids never matched the expected ones, so the latency recorded
was the cost of returning wrong results quickly.

It was produced by `benches/comparison/qdrant_comparison_benchmark.rs`, which
was never registered as a `[[bench]]` target — nothing ever compiled or ran it,
and it drifted unnoticed until it published that number. Those four files are
retracted in place, and the harness is deleted.

The recall gate in `report.py` exists so the same failure cannot be published
again: not detected afterwards, *not printed in the first place*.
