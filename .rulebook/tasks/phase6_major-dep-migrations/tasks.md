## 1. Rust major migrations

- [ ] 1.1 rand 0.9 → 0.10 (openraft-dependent; may need upstream PR).
- [ ] 1.2 bincode 2 → 3.
- [ ] 1.3 rmcp 0.10 → 1.5.
- [ ] 1.4 reqwest 0.12 → 0.13.
- [ ] 1.5 hmac 0.12 → 0.13 + sha2 0.10 → 0.11.
- [ ] 1.6 arrow 57 → 58 + parquet 57 → 58.
- [ ] 1.7 zip 6 → 8.
- [ ] 1.8 tantivy 0.25 → 0.26.
- [ ] 1.9 hf-hub 0.4 → 0.5.
- [ ] 1.10 sysinfo 0.37 → 0.38.
- [ ] 1.11 opentelemetry-prometheus 0.29 → 0.31 (track opentelemetry family).
- [ ] 1.12 ort 2.0.0-rc.11 → 2.0.0-rc.12 (waits on fastembed upstream).

## 2. TypeScript SDK migrations

- [ ] 2.1 eslint 8 → 9/10 with flat `eslint.config.js` migration.
- [ ] 2.2 vitest 2 → 4 with config-format migration.
- [ ] 2.3 @types/node 24 → 25 after bumping the CI Node pin.

## 3. GUI migrations

- [ ] 3.1 typescript 5 → 6.
- [ ] 3.2 vite 7 → 8.
- [ ] 3.3 vue-router 4 → 5.
- [ ] 3.4 uuid 13 → 14.
- [ ] 3.5 electron 39 → 41 (verify electron-builder + signing).

## 4. Dashboard migrations

- [ ] 4.1 react 18 → 19 (+ react-dom, @types/react, @types/react-dom).
- [ ] 4.2 react-router 6 → 7 (+ react-router-dom).
- [ ] 4.3 @vitejs/plugin-react 4 → 6.
- [ ] 4.4 eslint 9 → 10 (+ @eslint/js).
- [ ] 4.5 typescript 5 → 6.
- [ ] 4.6 vite 7 → 8.
- [ ] 4.7 jsdom 27 → 29.
- [ ] 4.8 tailwind-merge 2 → 3.
- [ ] 4.9 @types/node 24 → 25.

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Update or create documentation covering the implementation.
- [ ] 5.2 Write tests covering the new behavior.
- [ ] 5.3 Run tests and confirm they pass.
