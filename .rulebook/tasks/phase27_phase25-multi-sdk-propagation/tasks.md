## 1. TypeScript SDK

- [x] 1.1 `RuntimeMetrics` / `RouteStats` / `WalSnapshot` interfaces added in `sdks/typescript/src/models/admin.ts`; `VectorCountSample` in `models/collection.ts`. Wildcard re-exports already pick them up
- [x] 1.2 `Stats` grows optional `default_quantization: string` + `compression_ratio: number`. snake_case kept everywhere — TS SDK already mirrors the server's wire shape verbatim
- [x] 1.3 `Collection` grows optional `vector_count_history: VectorCountSample[]`. The legacy `CollectionInfo` interface stays unchanged (it predates the read-path sampling and is used for write/list paths that don't carry the history)
- [x] 1.4 `AdminClient.getRuntimeMetrics()` shipped in `sdks/typescript/src/client/admin.ts` calling `GET /metrics/runtime`
- [x] 1.5 7 new tests in `tests/runtime-metrics.test.ts` cover full + partial `RuntimeMetrics` payloads, the new `Stats` quantization fields, and the `vector_count_history` round-trip; `pnpm build` clean, `vitest` 7/7 (suite total 515/527, 12 pre-existing skips)
- [x] 1.6 `sdks/typescript/CHANGELOG.md` `[Unreleased]` block documents the additions

## 2. Python SDK

- [ ] 2.1 Mirror the Rust models with dataclasses (or pydantic if used) under `sdks/python/vectorizer/models/`
- [ ] 2.2 Extend `Stats` + `CollectionInfo` with the new fields (snake_case is native here)
- [ ] 2.3 Add `get_runtime_metrics()` client method
- [ ] 2.4 Pytest coverage for full + partial payloads
- [ ] 2.5 `[Unreleased]` CHANGELOG entry

## 3. Go SDK

- [ ] 3.1 Mirror the Rust models with `omitempty` JSON tags under `sdks/go/`
- [ ] 3.2 Extend `Stats` + `CollectionInfo` structs
- [ ] 3.3 Add `GetRuntimeMetrics(ctx)` client method
- [ ] 3.4 Go tests covering full + partial payloads
- [ ] 3.5 `[Unreleased]` CHANGELOG entry

## 4. C# SDK

- [ ] 4.1 Mirror the Rust models with `JsonPropertyName` / nullable defaults under `sdks/dotnet/`
- [ ] 4.2 Extend `Stats` + `CollectionInfo`
- [ ] 4.3 Add `GetRuntimeMetricsAsync()` client method
- [ ] 4.4 xUnit / NUnit coverage for full + partial payloads
- [ ] 4.5 `[Unreleased]` CHANGELOG entry

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 5.1 Update or create documentation covering the implementation
- [ ] 5.2 Write tests covering the new behavior
- [ ] 5.3 Run tests and confirm they pass
