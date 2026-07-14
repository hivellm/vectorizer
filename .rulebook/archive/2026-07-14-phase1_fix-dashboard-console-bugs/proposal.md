# Proposal: phase1_fix-dashboard-console-bugs

## Why
Bugs found during manual QA of the new console dashboard (served at the
Vite dev server, backed by the local REST API on :15002). Several actions
are dead or misleading: buttons with no handler, non-interactive lists that
look clickable, and a whole screen rendering hardcoded mock data as if it
were live. These undermine trust in the console while the new design is
being finished. This task collects the reported defects and fixes them.

## What Changes
1. **Create collection button is dead** — the "Create collection" button on
   the Collections page has no `onClick`; clicking does nothing. Wire it to
   open the create-collection flow and call the API.
2. **Top Collections rows don't navigate** — on the Overview page, the "Top
   Collections" table rows are static; clicking a row should navigate to
   that collection (Collections page with the collection selected).
3. **Replication (Cluster) screen shows mock data** — the Cluster page
   renders hardcoded `MOCK_REPLICAS`/`FALLBACK` replicas because no
   `/replication` backend endpoint exists yet. Remove the screen for now
   (route + navigation entry) rather than ship fake data.

## Impact
- Affected specs: dashboard/console
- Affected code:
  - `dashboard/src/pages/CollectionsPage.tsx` (Create collection button)
  - `dashboard/src/pages/OverviewPage.tsx` (Top Collections rows)
  - `dashboard/src/pages/ClusterPage.tsx` + `dashboard/src/router/AppRouter.tsx`
    + console nav (remove Cluster/Replication screen)
- Breaking change: NO (frontend only; removes a mock-only screen)
- User benefit: console actions work as they appear; no misleading fake data
