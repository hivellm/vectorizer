# 03 — npm advisories: six high, none in a shipped artifact

`pnpm audit` finds 6 high-severity advisories across the three JavaScript
projects. **Every one arrives through build or test tooling.** That does not
make them ignorable, but it does mean none of them reaches a user of the
published SDK or a served page — which is a materially different risk picture
from "6 high-severity vulnerabilities".

| Project | Advisory | Package | Path | Scope |
|---|---|---|---|---|
| `gui` | GHSA-2v37-7h3g-55p8 | `nanoid` 3.3.17 | `@vueuse/core` → `vue` → `@vue/compiler-sfc` → `postcss` | prod-classified, build-time in practice |
| `dashboard` | GHSA-2v37-7h3g-55p8 | `nanoid` 3.3.17 | `vite` / `@vitejs/plugin-react` → `postcss` | **dev** |
| `sdks/typescript` | GHSA-2v37-7h3g-55p8 | `nanoid` 3.3.16 | `vitest` → `vite` → `postcss` | **dev** |
| `sdks/typescript` | GHSA-5p4m-2wfm-xmqj | `js-yaml` 4.3.0 | **direct** | **dev** |
| `sdks/typescript` | GHSA-mh99-v99m-4gvg | `brace-expansion` 5.0.7 | `eslint` → `minimatch` | **dev** |
| `sdks/typescript` | GHSA-rgw5-rvv9-x895 | `brace-expansion` 5.0.7 | `eslint` → `minimatch` | **dev** |

## The one nuance worth catching

`nanoid` in `gui` is listed under `dependencies` rather than `devDependencies`,
because `@vueuse/core` is a production dependency and drags `vue` →
`@vue/compiler-sfc` → `postcss` behind it. The manifest classification says
production; the actual role is still compile-time CSS processing, and the
built bundle is very unlikely to contain it.

Do not resolve that by reading the manifest. **Check the built output** before
deciding how urgent this one is — the classification and the reality disagree,
and only one of them matters.

## Fixes

All four distinct advisories clear with routine bumps:

- `nanoid` → `postcss` ≥ 8.5.26 in all three projects. **PR #409 already does
  this for `gui`.**
- `js-yaml` → **4.3.1**. Note that open **PR #421 proposes 5.3.0**, a major
  version jump, for a vulnerability fixed in a patch release. See
  [04-open-pull-requests.md](04-open-pull-requests.md).
- `brace-expansion` → ≥ 5.0.9 via `eslint` / `minimatch`. **PR #413** bumps
  eslint 10.8.0 → 10.8.1; verify whether that carries the fix, rather than
  assuming it does.

## `sdks/javascript`

Dependabot's alert history references `sdks/javascript/pnpm-lock.yaml` and
`sdks/javascript/package-lock.json`, but `.github/dependabot.yml` has no entry
for that directory.

Checked: **the directory no longer exists.** Those are historical alerts
against a removed SDK, correctly absent from the config. No action — recorded
so the next person reading the alert history does not go looking for it.
