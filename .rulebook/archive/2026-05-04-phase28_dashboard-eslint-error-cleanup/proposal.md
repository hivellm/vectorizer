# Proposal: phase28_dashboard-eslint-error-cleanup

## Why

Phase26 fixed the `pnpm lint` crash (eslint@10 ×
eslint-plugin-react@7.37.5 incompat) by pinning `settings.react.version`
in `dashboard/eslint.config.js`. With the lint pipeline now running to
completion, **41 pre-existing errors** that the crash was hiding are
visible. They group as:

- **19 × `react/no-unescaped-entities`** — literal `"`/`'` inside JSX
  text, mostly user-facing strings.
- **13 × `preserve-caught-error`** (typescript-eslint v8) — thrown
  errors that drop the original cause (`throw new Error(msg)` instead
  of `throw new Error(msg, { cause: e })`).
- **2 × `no-constant-binary-expression`** in `cn.test.ts`.
- **2 × `react-hooks/immutability` "Cannot access variable before it is
  declared"** in `WorkspacePage.tsx` — `loadData` referenced inside a
  `useEffect` declared above its function definition.
- **2 × "Cannot access refs during render"** in a single page.
- **1 × `no-useless-assignment`**, **1 × `react-hooks/exhaustive-deps`
  array shape**, **1 × `react/display-name`** (a missing
  `displayName` on a component).

These are all real bugs that the lint never had a chance to flag while
the v10 crash was active. Fixing them is out of phase26's tight scope
("dependency-bump task"), so they live here.

## What Changes

Walk the rule list one bucket at a time, fixing the underlying code
instead of suppressing rules:

1. **Unescaped entities** — replace `"` / `'` in JSX text with
   `&quot;` / `&apos;` (or move the literal into a string expression).
2. **`preserve-caught-error`** — every `throw new Error(...)` inside a
   `catch (e) { ... }` block grows `{ cause: e }`.
3. **`no-constant-binary-expression`** — fix the test file's predicate
   so the `&&` actually has a meaningful left operand.
4. **`react-hooks/immutability`** — convert the affected `loadData`
   declarations to `useCallback` and reorder so they precede their
   `useEffect` consumers.
5. **`Cannot access refs during render`** — move the offending ref
   read into a `useEffect` or an event handler.
6. **`no-useless-assignment`**, **`react/display-name`**, dependency
   array shape — surgical per-occurrence fix.

After the cleanup, `pnpm lint` exits with **0 errors** under the
existing `--max-warnings 50` budget (24 warnings remain — those are
warnings by design and stay under the cap).

## Impact

- Affected code:
  - `dashboard/src/pages/*.tsx` — escape entities, add `cause` on
    rethrows, restructure `loadData` patterns.
  - `dashboard/src/utils/__tests__/cn.test.ts` — fix the constant
    binary expression in the predicate.
  - One-off fixes for the missing display name + ref-during-render
    site.
- Affected specs: none (purely lint hygiene).
- Breaking change: NO.
- User benefit: dashboard ships a green lint pipeline, pre-commit
  hooks become real signal again, and the `preserve-caught-error`
  bucket actually preserves the original errors so prod stack traces
  carry the underlying cause.
