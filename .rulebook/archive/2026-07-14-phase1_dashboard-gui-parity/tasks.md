## 1. P1 — wire dead/mock actions
- [x] 1.1 Collections: Create collection (modal + POST /collections)
- [x] 1.2 Collections: Delete collection (confirm + DELETE /collections/{name})
- [x] 1.3 Collections: Reindex + Copy-ID actions
- [x] 1.4 Vectors: Insert (modal + POST /insert)
- [x] 1.5 Vectors: Delete vector + Copy vector-ID

## 2. P2 — vector search parity  (ALREADY COVERED by the dedicated SearchPage)
- [x] 2.1 Search modes (semantic/intelligent/discover) with limit param — SearchPage.tsx has intelligent/semantic/contextual/multi + collection + limit + threshold
- [x] 2.2 Similarity-score display on results — SearchPage.tsx:316-317 shows r.score.toFixed(3); no duplicate search needed on VectorsPage

## 3. P3 — Configuration structured forms
- [x] 3.1 Structured config editors (General: host/port/mcp_port; Logging: level + request/response/error toggles) replacing read-only cards; scoped to fields that exist in live /config

## 4. P4 — small omissions
- [x] 4.1 Logs: text search filter + max-lines control + export-as-JSON
- [x] 4.2 Connections: Test Connection before save
- [x] 4.3 Graph: context menu + double-click load neighbors
- [x] 4.4 Overview: Add Directory + Create Backup quick actions

## 5. Tail (mandatory — enforced by rulebook v5.3.0)
- [x] 5.1 Update or create documentation covering the implementation — CHANGELOG [Unreleased] Dashboard section
- [x] 5.2 Write tests covering the new behavior — updated/added page tests (Monitoring, Config, Overview nav, sidebar) + existing suite realigned
- [x] 5.3 Run tests and confirm they pass — full dashboard suite 228 passing (2 pre-existing ApiKeysPage failures unrelated to this work)
