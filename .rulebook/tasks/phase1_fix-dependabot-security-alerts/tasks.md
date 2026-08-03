## 1. Dashboard (dashboard/) — 11 alerts
- [ ] 1.1 Bump/override postcss 8.5.18, brace-expansion (1.1.16 + >=5.0.7), dompurify 3.4.12, js-yaml (stay patched 4.x)
- [ ] 1.2 react-router to the patched line (7.18.0 / 8.3.0) incl. the direct dep in package.json; pnpm install
- [ ] 1.3 pnpm build passes; confirm dashboard alerts clear

## 2. GUI (gui/) — 15 alerts
- [ ] 2.1 Bump/override axios 1.18.0, app-builder-lib 26.15.0, builder-util-runtime 9.7.0, fast-uri 3.1.4, shell-quote 1.9.0, tar 7.5.18, postcss 8.5.18, brace-expansion, js-yaml (patched 4.x)
- [ ] 2.2 pnpm install + build/type-check passes; confirm gui alerts clear

## 3. TypeScript SDK (sdks/typescript/) — 1 alert
- [ ] 3.1 postcss 8.5.18 override + pnpm install; tsc build + eslint pass

## 4. Verify
- [ ] 4.1 Re-check the Dependabot alerts list is empty (or only accepted exceptions remain)

## 5. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 5.1 Update or create documentation covering the implementation
- [ ] 5.2 Write tests covering the new behavior
- [ ] 5.3 Run tests and confirm they pass
