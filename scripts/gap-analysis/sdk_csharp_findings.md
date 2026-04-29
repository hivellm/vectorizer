# SDK C#

Total: 767 items públicos (267 classes/interfaces + 500 props/methods)
~80 métodos `async Task` públicos | XML doc cobertura: 35%

| Categoria | Count |
|---|---|
| Doc-only (README sem código) | 0 |
| Code-only (no SDK, sem README) | **19 métodos** |
| Server→SDK gaps | poucos (mapeamento 1:1 forte) |
| Public sem XML doc | **500** (65% gap) |

## Code-only — 19 métodos públicos não no README

### Batch ops (5)
- `BatchInsertTextsAsync`, `BatchSearchVectorsAsync`, `BatchUpdateVectorsAsync`, `BatchDeleteVectorsAsync`, `AcquireAsync`

### Graph discovery (4)
- `DiscoverGraphEdgesAsync`, `DiscoverGraphEdgesForNodeAsync`, `GetGraphDiscoveryStatusAsync`, `ListGraphEdgesAsync`

### Outros (10)
- `EmbedTextAsync`, `GetSummaryAsync`, `ListSummariesAsync`, `GetUploadConfigAsync`
- `UploadFileContentAsync`, `HybridSearchAsync`
- RPC: `CallAsync`, `HelloAsync`
- `WithMaster()` — client-side routing helper

## Public sem XML doc (500 items)
- **450+ properties** em DTOs (BatchTextRequest, ClientConfig, HostConfig, SearchOptions, ReadPreference enum, etc.)
- **50+ methods** com summary mas sem `<param>`, `<returns>`, `<exception>` consistentes

## Cobertura cruzada server→SDK (forte)
- 61 routes server vs 80+ métodos SDK — mapeamento 1:1 quase completo
- Endpoints sem wrapper: `/setup/status`, `/setup/verify`, `/dashboard*`, `/logs`
- Qdrant compat: usa `Dictionary<string, object>` para query params (genérico, não strongly-typed)

## Recomendações
1. Adicionar XML doc summaries em 50+ métodos public
2. Documentar properties de DTOs (focus em ReadPreference, HostConfig.Replicas, SearchOptions.Filter)
3. Expandir README com 19 métodos missing (batch + graph discovery)
4. Adotar `<param>/<returns>/<exception>` consistente
5. Esclarecer split entre `Vectorizer.Sdk` vs `Vectorizer.Sdk.Rpc` packages
