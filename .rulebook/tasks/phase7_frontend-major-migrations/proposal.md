# Proposal: phase7_frontend-major-migrations

## Why

`phase6_major-dep-migrations` refreshed the Rust workspace, the
TypeScript SDK (eslint 9 flat config + vitest 3), and two safe
dashboard majors (tailwind-merge 3, jsdom 29). The remaining GUI +
Dashboard majors each require real application-code migration work
— breaking framework APIs (React 19 concurrent-features cleanup,
react-router 7 loader model, Vue Router 5), build-tool rewrites
(vite 8, typescript 6 tsconfig tightening), and runtime entanglement
(electron 39 → 41 + installer smoke-tests). Grouping them under
this follow-up keeps each migration as its own scoped diff so a
regression in one doesn't block the others from shipping.

## What Changes

Per-ecosystem migrations, each its own sub-task:

### GUI (`gui/package.json`, Electron + Vue 3)
- `typescript 5 → 6` — audit `tsconfig*.json` `moduleResolution`/`lib`/`target` entries for 6.x tightening.
- `vite 7 → 8` + `@vitejs/plugin-vue` matching major — audit `vite.config.ts` plugin options.
- `vue-router 4 → 5` — call-site sweep over every `useRouter()`/`useRoute()`/`<router-link>` prop (the prop surface changed).
- `uuid 13 → 14` — small surface, the default ESM export moved.
- `electron 39 → 41` (two majors) — installer smoke-tests on Windows + macOS; code-signing + auto-update verification.

### Dashboard (`dashboard/package.json`, React + vite)
- **React 19 family** (must co-move): `react 18 → 19` + `react-dom 18 → 19` + `@types/react 18 → 19` + `@types/react-dom 18 → 19`. Audit `forwardRef` patterns, `propTypes` surface, `useOptimistic`/`useFormStatus` availability in the existing components.
- **react-router 6 → 7 family** (must co-move): `react-router 6 → 7` + `react-router-dom 6 → 7`. The v7 API moves to the data/loader model — sweep `dashboard/src/router/**`.
- `@vitejs/plugin-react 4 → 6` — gates on React 19 landing first.
- `eslint 9 → 10` + `@eslint/js 9 → 10` — flat-config already exists; resolve any removed rules.
- `typescript 5 → 6` — same playbook as GUI.
- `vite 7 → 8` — same playbook as GUI.
- `@types/node 24 → 25` — gate on CI Node matrix bump first.

### TypeScript SDK (`sdks/typescript/package.json`)
- `vitest 3 → 4` — blocked on rewriting 19 test files' `vi.fn()` mock patterns (vitest 4 errors on mocks that don't use `function` or `class` in their implementation). Also pushes `eslint 9 → 10` + the typescript-eslint 9 line.
- `@types/node 24 → 25` — gate on CI Node matrix bump.

## Impact

- Affected specs: none (dep metadata + call-site adjustments only).
- Affected code: most `gui/src/**` (vue-router + uuid), most `dashboard/src/**` (react 19 + router 7), `sdks/typescript/tests/**` (vitest 4 mock rewrites).
- Breaking change: NO to downstream consumers; internal build-chain migrations only.
- User benefit: React 19 concurrent features, react-router 7 data-loader ergonomics, Electron 41 security fixes, access to the TypeScript 6 type-system tightening.
