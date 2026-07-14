# Proposal: phase1_dashboard-gui-parity

## Why
The user asked to review the legacy Electron/Vue GUI (`gui/`) and reproduce
its functionality in the new React dashboard (`dashboard/`). A parity audit
(4 parallel researchers comparing all 8 GUI views against their dashboard
equivalents) found the dashboard is at or ahead of the GUI on most screens
(it adds QPS/p99/cache/CPU metrics, KPI cards, WS streaming, FileBrowser).
The real gaps are concentrated in a few pages: dead/mock action buttons on
Collections/Vectors, the structured config-form tabs on Configuration, and
small omissions on Logs/Connections/Graph/Overview. This task tracks closing
those gaps so the dashboard reaches (and keeps) functional parity.

## What Changes
Prioritized gaps (evidence from the GUI ↔ dashboard comparison):

**P1 — dead/mock actions (highest user impact; "buttons don't work")**
- Collections: wire Create / Delete / Reindex / Copy-ID (dead or
  TODO(actions) today).
- Vectors: wire Insert (modal), Delete vector, Copy vector-ID.

**P2 — search parity on collection vectors**
- Vector search modes (semantic / intelligent / discover) with a limit
  parameter and similarity-score display (GUI has these; dashboard only
  client-filters the listed vectors).

**P3 — Configuration structured forms**
- Reproduce the GUI's structured config editors (General, Embedding,
  Collections defaults, Performance, File Watcher, Logging, Storage, API,
  etc.) instead of only the raw YAML editor + read-only cards. Large;
  previously deferred (TODO(config-features-deferred)).

**P4 — small omissions**
- Logs: text search filter, max-lines control, export-as-JSON.
- Connections: "Test Connection" before save.
- Graph: right-click context menu, double-click -> load neighbors.
- Overview: "Add Directory" and "Create Backup" quick actions.

## Impact
- Affected specs: dashboard/console
- Affected code: dashboard/src/pages/{CollectionsPage,VectorsPage,
  ConfigurationPage,LogsPage,ConnectionsPage,GraphPage,OverviewPage}.tsx
  and supporting hooks; no backend change expected for P1/P2/P4 (endpoints
  exist).
- Breaking change: NO (frontend)
- User benefit: every visible action works and mirrors the proven GUI flows
