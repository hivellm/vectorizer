## 1. TypeScript SDK

- [ ] 1.1 Add `RuntimeMetrics` / `RouteStats` / `WalSnapshot` / `VectorCountSample` types in `sdks/typescript/src/models/`
- [ ] 1.2 Extend `Stats` with `defaultQuantization` + `compressionRatio` (camelCase) — wire-format passthrough preserves snake_case via decoders
- [ ] 1.3 Extend `Collection` / `CollectionInfo` with `vectorCountHistory`
- [ ] 1.4 Add `getRuntimeMetrics()` client method on the admin / observability surface
- [ ] 1.5 Unit tests for full + partial payloads
- [ ] 1.6 `[Unreleased]` CHANGELOG entry

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
