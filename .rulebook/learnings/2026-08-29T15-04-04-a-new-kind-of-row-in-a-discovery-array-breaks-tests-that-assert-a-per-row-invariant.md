# A new kind of row in a discovery array breaks tests that assert a per-row invariant
**Source**: manual
**Date**: 2026-08-29
**Related Task**: phase6_raw-vector-collections
**Tags**: discovery, testing, backward-compatibility
Adding the raw-vector sentinel to the `providers` inventory broke `stats_advertises_providers_block`, which asserted every row carries a numeric `dimension`. That assertion was exactly right when every row was a registered provider; the sentinel reports `dimension: null` because the caller's vectors set the width.

The fix is NOT to relax the assertion. Add a discriminator field to every row (`supports_text`), branch on it, and pin BOTH shapes: a registered provider must still report a fixed width, and a provider-less row must report `null` rather than a misleading number. The original invariant survives untouched — it just no longer speaks for every row.

Two traps found doing this:

1. The array had FOUR copies, not the three the task named: `GET /stats`, RPC `embedding.list_providers` AND RPC `stats.database` (these two share a helper), MCP `list_providers` AND MCP `get_database_stats` (which built its own copy). Grep for the literal field names, not for the endpoint names.

2. A non-empty assertion carrying a diagnostic ("register_all_providers did not run") is silently defeated by an unconditionally-appended row. Move the diagnostic to a count of rows that actually satisfy it.

For a test that points at a LIVE server of unknown version, treat a missing discriminator as the old meaning (`unwrap_or(true)`) so it passes against both old and new servers while still enforcing the new rule when the field is present.