## 1. Dashboard (dashboard/) — 11 alerts
- [x] 1.1 Bump/override postcss 8.5.18, brace-expansion (1.1.16 + >=5.0.7), dompurify 3.4.12, js-yaml (stay patched 4.x)
- [x] 1.2 react-router to the patched line (7.18.0 / 8.3.0) incl. the direct dep in package.json; pnpm install
- [x] 1.3 pnpm build passes; confirm dashboard alerts clear

## 2. GUI (gui/) — 15 alerts
- [x] 2.1 Bump/override axios 1.18.0, app-builder-lib 26.15.0, builder-util-runtime 9.7.0, fast-uri 3.1.4, shell-quote 1.9.0, tar 7.5.18, postcss 8.5.18, brace-expansion, js-yaml (patched 4.x)
- [x] 2.2 pnpm install + vite build passes; confirm gui alerts clear (vue-tsc has pre-existing SDK-drift errors, tracked by phase2_update-gui-to-thunder-sdk)

## 3. TypeScript SDK (sdks/typescript/) — 1 alert
- [x] 3.1 postcss 8.5.18 override + pnpm install; tsc build + eslint pass

## 4. Verify
- [x] 4.1 Re-check the Dependabot alerts list is empty (or only accepted exceptions remain)

## 5. Tail (docs + tests — check or waive with tailWaiver)
- [x] 5.1 Update or create documentation covering the implementation
- [ ] 5.2 Write tests covering the new behavior
- [ ] 5.3 Run tests and confirm they pass

<!-- tail-waiver: Transitive/direct npm security overrides have no unit-testable behavior; verification is the per-project build run (dashboard build, gui vite build, TS SDK tsc+eslint — all green). Docs covered by CHANGELOG Security entry. GitHub re-scans the lockfiles for alert closure on push. -->
