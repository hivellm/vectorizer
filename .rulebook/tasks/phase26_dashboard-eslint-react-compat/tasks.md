## 1. Diagnose & pick the upgrade path

- [ ] 1.1 Reproduce: run `pnpm lint` from `dashboard/` and capture the full `TypeError: contextOrFilename.getFilename is not a function` stack
- [ ] 1.2 Check `eslint-plugin-react` releases for an `eslint@10`-compatible version; if none, decide between (a) pinning eslint back to `^9.x` or (b) swapping to a fork / alternative plugin
- [ ] 1.3 Audit other lint plugins in `dashboard/package.json` (`@typescript-eslint/*`, `eslint-plugin-react-hooks`, `eslint-plugin-jsx-a11y`, etc.) for the same eslint-v10 context-shape regression

## 2. Apply the bump

- [ ] 2.1 Update `dashboard/package.json` with the chosen versions and any peer adjustments
- [ ] 2.2 Regenerate `dashboard/pnpm-lock.yaml` via `pnpm install`
- [ ] 2.3 If a rule was removed or renamed, update `dashboard/.eslintrc.*` accordingly

## 3. Verify

- [ ] 3.1 `pnpm lint` runs to completion across `src/` and `e2e/` with zero warnings under `--max-warnings 50`
- [ ] 3.2 `pnpm vitest --run` still green
- [ ] 3.3 `pnpm build` still green (no new type errors from updated plugin types)

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Update or create documentation covering the implementation
- [ ] 4.2 Write tests covering the new behavior
- [ ] 4.3 Run tests and confirm they pass
