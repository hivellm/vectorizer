## 1. TypeScript SDK

- [x] 1.1 `RuntimeMetrics` / `RouteStats` / `WalSnapshot` interfaces added in `sdks/typescript/src/models/admin.ts`; `VectorCountSample` in `models/collection.ts`. Wildcard re-exports already pick them up
- [x] 1.2 `Stats` grows optional `default_quantization: string` + `compression_ratio: number`. snake_case kept everywhere — TS SDK already mirrors the server's wire shape verbatim
- [x] 1.3 `Collection` grows optional `vector_count_history: VectorCountSample[]`. The legacy `CollectionInfo` interface stays unchanged (it predates the read-path sampling and is used for write/list paths that don't carry the history)
- [x] 1.4 `AdminClient.getRuntimeMetrics()` shipped in `sdks/typescript/src/client/admin.ts` calling `GET /metrics/runtime`
- [x] 1.5 7 new tests in `tests/runtime-metrics.test.ts` cover full + partial `RuntimeMetrics` payloads, the new `Stats` quantization fields, and the `vector_count_history` round-trip; `pnpm build` clean, `vitest` 7/7 (suite total 515/527, 12 pre-existing skips)
- [x] 1.6 `sdks/typescript/CHANGELOG.md` `[Unreleased]` block documents the additions

## 2. Python SDK

- [x] 2.1 New dataclasses `RuntimeMetrics`, `RouteStats`, `WalSnapshot`, `VectorCountSample` added in `sdks/python/models.py`. Each ships a `from_dict` classmethod for tolerant decoding
- [x] 2.2 `Stats` grows `default_quantization: str` and `compression_ratio: float` (defaulting to `("none", 1.0)`); `CollectionInfo` grows `vector_count_history: List[Any]` and hydrates dict entries from `**data` kwargs in `__post_init__`
- [x] 2.3 `AdminClient.get_runtime_metrics()` shipped in `sdks/python/vectorizer/admin.py` calling `GET /metrics/runtime`; new types re-exported from `vectorizer/__init__.py`
- [x] 2.4 8 new unit tests in `sdks/python/tests/test_runtime_metrics.py` cover the full + partial + empty `RuntimeMetrics` payloads, the new `Stats` quantization fields, and the `vector_count_history` hydration; `pytest` 8/8
- [x] 2.5 `sdks/python/CHANGELOG.md` `[Unreleased]` block documents the additions

## 3. Go SDK

- [x] 3.1 New types in `sdks/go/models.go`: `RuntimeMetrics`, `RouteStats`, `WalSnapshot`, `VectorCountSample`. Every JSON field uses `omitempty` so older servers / partial payloads decode cleanly
- [x] 3.2 `Stats` grows `DefaultQuantization`/`CompressionRatio`; `CollectionInfo` grows `VectorCountHistory`. omitempty preserves the existing wire shape
- [x] 3.3 `Client.GetRuntimeMetrics()` shipped in `sdks/go/admin.go` calling `GET /metrics/runtime`
- [x] 3.4 7 new Go tests in `sdks/go/runtime_metrics_test.go` cover the `/metrics/runtime` route + decode, full + partial payloads, the new `Stats` quantization fields, and the `vector_count_history` JSON round-trip; `go test ./...` green
- [x] 3.5 `sdks/go/CHANGELOG.md` `[Unreleased]` block documents the additions

## 4. C# SDK

- [x] 4.1 New types in `sdks/csharp/Models/AdminModels.cs`: `RuntimeMetrics`, `RouteStats`, `WalSnapshot`, `VectorCountSample`. Each has `JsonPropertyName` for the snake_case wire fields and sensible default initialisers (`new()` for collections, `0` for numbers) so older / partial payloads decode cleanly
- [x] 4.2 `Stats` grows `DefaultQuantization` (default `"none"`) + `CompressionRatio` (default `1.0f`); `CollectionInfo` grows `VectorCountHistory: List<VectorCountSample>` (default `new()`)
- [x] 4.3 `VectorizerClient.GetRuntimeMetricsAsync()` shipped in `sdks/csharp/Admin.cs` calling `GET /metrics/runtime`
- [x] 4.4 7 new xUnit tests in `Vectorizer.Tests/RuntimeMetricsTests.cs` cover the route + decode, full + partial payloads, the new `Stats` quantization fields, and `vector_count_history` JSON round-trip; `dotnet test` 194/194 pass (4 pre-existing skips)
- [x] 4.5 `sdks/csharp/CHANGELOG.md` `[Unreleased]` block documents the additions

## 5. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 5.1 Update or create documentation covering the implementation — each SDK gets an `[Unreleased]` CHANGELOG block and inline doc comments on the new types / method (`RuntimeMetrics`, `RouteStats`, `WalSnapshot`, `VectorCountSample`, `getRuntimeMetrics()`)
- [x] 5.2 Write tests covering the new behavior — TS 7 (`tests/runtime-metrics.test.ts`), Python 8 (`tests/test_runtime_metrics.py`), Go 7 (`runtime_metrics_test.go`), C# 7 (`Vectorizer.Tests/RuntimeMetricsTests.cs`); each batch covers the route + full snapshot decode, partial payloads, the new `Stats` quantization fields, and `vector_count_history` round-trip
- [x] 5.3 Run tests and confirm they pass — TS 515/527 (12 pre-existing skips), Python 8/8, Go green via `go test ./...`, C# 194/194 (4 pre-existing skips). All four SDK builds (`pnpm build`, no Python build step, `go build ./...`, `dotnet build`) clean
