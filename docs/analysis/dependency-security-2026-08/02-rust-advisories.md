# 02 — Rust advisories: what is actually fixable

`cargo audit` reports 9 vulnerabilities. Sorted by what we can do about them
today, that is **2 free fixes, 5 blocked upstream, 1 with no patch anywhere,
and 1 that is probably not compiled at all**.

## Fixable now — a lockfile update reaches a patched version

| Advisory | Crate | Have | Patched | Reached via |
|---|---|---|---|---|
| RUSTSEC-2026-0258 | `h2` | 0.4.15 | ≥ 0.4.16 | `hyper`, `reqwest`, `tonic` |
| RUSTSEC-2026-0253 | `lru` | 0.18.1 | verify | **direct** dependency of `vectorizer`, `vectorizer-server` |

`cargo update -p h2` moves 0.4.15 → **0.4.19**. This is the one with real
runtime exposure: *unbounded empty DATA frames* is an HTTP/2 denial of service
against the server we ship.

`cargo update -p lru@0.18.1` moves to 0.18.3. RUSTSEC-2026-0253 is classified
*unsound* (use-after-free if `LruCache::pop()` panics) and the advisory listed
no patched range in this run — **confirm 0.18.3 actually clears it** rather
than assuming the bump is the fix.

## Blocked upstream — a parent pins the version

`cargo update -p <spec> --dry-run` reports `Locking 0 packages` for every one
of these. The ceiling is the parent's semver requirement, so the fix is a
parent bump, an upstream PR, or dropping the feature.

| Advisory | Crate | Have | Patched | Pinned by |
|---|---|---|---|---|
| RUSTSEC-2026-0194, -0195 | `quick-xml` | 0.36.2 | ≥ 0.41.0 | `docx-rs` 0.4.20 |
| RUSTSEC-2026-0194, -0195 | `quick-xml` | 0.37.5 | ≥ 0.41.0 | `transmutation` 0.3.3, `umya-spreadsheet` 2.3.3 |
| RUSTSEC-2026-0187 | `lopdf` | 0.34.0 | ≥ 0.42.0 | `pdf-extract` 0.8.2 |
| RUSTSEC-2026-0187 | `lopdf` | 0.35.0 | ≥ 0.42.0 | `transmutation` 0.3.3 |
| RUSTSEC-2026-0253 | `lru` | 0.16.4 | verify | `tantivy` 0.26.1 |

**These are the ones that matter most, and the ones we control least.** All
three advisories are denial of service on *parsed input*:

- `lopdf` — stack overflow on deeply nested PDF objects
- `quick-xml` — quadratic time on duplicate attribute names, and unbounded
  namespace allocation

They sit on the **file upload / transmutation path**, which takes documents
from callers. And `transmutation` is in the crate's **default feature set**
(`default = ["hive-gpu", "fastembed", "transmutation", "simd", ...]`), so a
stock build ships all of it.

Options, in the order they should be considered:

1. Bump `transmutation` (ours — `hivellm`) so its own `quick-xml` and `lopdf`
   move. That clears two of the four rows directly.
2. Upstream PRs to `docx-rs`, `umya-spreadsheet`, `pdf-extract`, or replace
   them.
3. Make `transmutation` non-default, so operators opt into the document
   parsers rather than shipping them unknowingly. This is a behaviour change
   and needs its own decision.

## No patch exists

| Advisory | Crate | Reached via |
|---|---|---|
| RUSTSEC-2023-0071 | `rsa` 0.9.10 | `jsonwebtoken` 10.4.0 |

The Marvin attack — key recovery through timing side channels. `patched: []`;
there is no fixed version to move to. This is on the **auth path**, so it
cannot simply be waved away, but it also cannot be fixed by bumping.

Real options are: confirm our JWT algorithms never exercise the RSA code path
(if we only sign HS256, the vulnerable code may never run — **verify, do not
assume**), or move off `jsonwebtoken`'s RSA support. Whatever is decided, it
belongs in `audit.toml` as an explicit ignore **with the reasoning written
down**, not as silence.

## Probably not compiled

| Advisory | Crate | Note |
|---|---|---|
| RUSTSEC-2026-0235 | `rkyv` 0.7.46 | Only `rust_decimal` lists it, and `cargo tree -i rkyv` prints nothing |

Present in `Cargo.lock` through an optional feature that nothing appears to
enable. Confirm, then ignore it in `audit.toml` with that reason recorded.

## Unmaintained, unsound and yanked

Not vulnerabilities, but they are how a dependency becomes one with no one
watching. Two are **ours directly** and deserve a decision rather than a
shrug:

- `rustls-pemfile` 2.2.0 (RUSTSEC-2025-0134) — direct dependency of
  `vectorizer` and `vectorizer-server`. Upstream folded its functionality into
  `rustls-pki-types`; this is a real migration, not a bump.
- `bincode` 2.0.1 (RUSTSEC-2025-0141) — direct dependency of all three crates,
  and it is on the **persistence path**. `bincode` 1.3.3 also arrives via
  `hnsw_rs`.

The rest are transitive: `fxhash` (via `selectors`), `paste` (already ignored
in `audit.toml`), `ttf-parser`. Yanked: `chacha20` 0.10.1, `spin` 0.9.8.

## Duplicate versions

`lopdf` ×2, `quick-xml` ×2, `lru` ×2, `bincode` ×2. Each duplicate is a second
copy to patch and a second copy to audit. Worth collapsing where the parents
allow, and worth *recording* where they do not.
