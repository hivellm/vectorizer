# SIMD benchmarks + regression guard

`benches/simd/` is the Criterion harness phase 7g landed. It exists
for two reasons:

1. **Verifiable speedup claims.** The README and
   `docs/architecture/simd.md` quote 4–8× numbers for the SIMD
   backends; without a reproducible bench they're folklore.
2. **Regression protection.** A change that accidentally routes a
   hot path through scalar (forgetting an `is_x86_feature_detected!`
   gate, dropping FMA, etc.) doesn't fail any test — the binary
   still produces correct results, just at scalar speed. The
   regression-guard script catches the throughput drop in CI.

## Benches

One per primitive, one Criterion group per primitive, two rows per
dimension (dispatched backend + scalar oracle). The dispatched-row
label includes `selected_backend_name()` so the report shows which
ISA path the running CPU actually took.

| Bench | Primitive | Dimensions |
|---|---|---|
| `simd_dot_product` | `crate::simd::dot_product` | 64, 128, 256, 384, 512, 768, 1024, 1536, 3072 |
| `simd_euclidean` | `crate::simd::euclidean_distance` | same |
| `simd_cosine` | `crate::simd::cosine_similarity` | same |
| `simd_l2_norm` | `crate::simd::l2_norm` | same |
| `simd_manhattan` | `crate::simd::manhattan_distance` | same |
| `simd_quantize` | `quantize_f32_to_u8` + `dequantize_u8_to_f32` | same |

Run any one with:

```sh
cargo bench --bench simd_dot_product
```

Run the full sweep with:

```sh
for b in simd_dot_product simd_euclidean simd_cosine \
         simd_l2_norm simd_manhattan simd_quantize ; do
    cargo bench --bench "$b" -- --save-baseline ci
done
```

## Regression guard

`scripts/simd/check-regression.sh` compares Criterion's most recent
output against a committed baseline JSON under
`benches/simd/baselines/`. Default tolerance is 10%; override via
`VECTORIZER_SIMD_REGRESSION_TOLERANCE=0.05` for a stricter gate.

The script reads the median estimate from each
`target/criterion/<group>/<bench-id>/new/estimates.json`, looks up
the matching `<group>/<bench-id>` row in the baseline, computes the
relative delta, and prints a Markdown table. Exits non-zero (and
emits a `::error::` line GitHub Actions surfaces) when any row is
slower than tolerance.

It uses `jq` when available (CI), falls back to Python when not
(typical Windows / dev hosts).

## Committing a baseline

A baseline JSON is the file `scripts/simd/check-regression.sh`
diffs against. Commit one when the bench shape changes (new bench,
new dimension, new backend) or when a deliberate optimisation
shifts the numbers and the new floor should become the new
reference.

Workflow:

1. Run the bench cleanly on the target CPU:
   ```sh
   cargo bench --bench simd_dot_product
   ```
2. Convert Criterion's per-bench `estimates.json` files into the
   flat lookup table the regression script expects:
   ```sh
   python scripts/simd/build-baseline.py simd_dot_product \
       > benches/simd/baselines/simd_dot_product.json
   ```
3. Commit the new baseline JSON in the same change that
   introduced the perf shift, with a one-line note in the commit
   body about the source CPU (`Intel i7-13700K`, `Apple M2 Pro`,
   etc.) so reviewers can sanity-check the floor against expected
   silicon.

The `build-baseline.py` helper isn't shipped yet — phase 7g leaves
the conversion as a manual step until the matrix has produced its
first round of artefacts. The shape of the JSON the script reads
is one flat object: keys are `<criterion-group>/<bench-id>`,
values are the median estimate in nanoseconds.

## CI matrix

`.github/workflows/simd-matrix.yml` runs the same bench + regression
check across every ISA the project supports:

| Job | Runner | Backends |
|---|---|---|
| `x86-avx2` | `ubuntu-latest` (Cascade Lake) | scalar, sse2, avx2+fma |
| `x86-avx512` | `ubuntu-latest-xlarge` (Ice Lake) | + avx512, avx512vnni |
| `macos-arm` | `macos-14` (Apple M1) | scalar, neon |
| `linux-arm-neon` | `ubuntu-22.04-arm` (Graviton2) | scalar, neon |
| `linux-arm-sve` | `ubuntu-latest` + qemu-user-static | scalar, neon, sve, sve2 |
| `wasm-simd` | `ubuntu-latest` | scalar (compile-only until crate split) |

The `aggregate` job downloads each backend's `simd-report.md`
artefact and concatenates them into `simd-matrix.md` so a PR
reviewer sees one combined view.

Two jobs run with `continue-on-error: true`:

- `x86-avx512` because not every contributor has access to the
  paid `ubuntu-latest-xlarge` runner pool.
- `wasm-simd` because the crate's transitive deps (`mio`, `tokio`'s
  net feature) don't compile to wasm32 today; verifying the
  Wasm128Backend in CI needs a sub-crate split that's not in scope
  here (tracked in the SDK-split phase).

`linux-arm-sve` runs SVE under QEMU emulation — fine for
correctness, useless for benches. Native SVE benches need a
self-hosted Graviton3+ runner; that's a future infrastructure task
once the team commits to ARM-server support.

## Cross-references

- `benches/simd/util.rs` — shared helpers (`STANDARD_DIMS`, seeded
  vector generators, Criterion settings).
- `scripts/simd/check-regression.sh` — regression-guard logic.
- `.github/workflows/simd-matrix.yml` — CI matrix.
- `docs/architecture/simd.md` — the dispatch + backend architecture
  these benches measure.
