# Probe a published SDK against a live server before trusting its .d.ts

**Category**: integration
**Tags**: none

## Description

A green `type-check` against an SDK's `.d.ts` proves the client agrees with the SDK's *claims*, not with the server. Porting the GUI to `@hivehub/vectorizer-sdk` 3.6.0 type-checked clean while four different mismatches were live:

- `getBackupDirectory()` is typed `{directory: string}`; the server sends `{path}`. Reading the declared field silently fell back to a default.
- `listBackups()` requests `/backups/list`, which the server does not serve (`GET /backups` does). `restoreBackup()` sends `{filename}` where the server reads `backup_id`. The `*Typed` variants (`listBackupInfos`, `createBackupTyped`, `restoreBackupTyped`) are the ones matching the server.
- `SearchResult` declares `data: number[]` required; the server sends `vector`, and one search route omits the embedding entirely.
- The SDK *validates* responses: `validateSearchResponse` requires `total` (server sent `total_results`) and each hit needs a non-empty `data`. So a **successful** search threw inside the client.

Cheap way to find all of it in minutes: start the server on a spare port with a temp `VECTORIZER_DATA_DIR`, then run a node script that imports the SDK **from the client's own `node_modules`** (so resolution and version match what ships) and calls every method the client uses, printing the returned key set. Log in first — a fresh data dir writes root credentials to `<data_dir>/.root_credentials` and every route needs a bearer token.

Two operational notes: the probe file must live inside the client package or Node cannot resolve the SDK's own dependencies, and on Windows the running server holds `target/debug/vectorizer.exe`, so stop it before rebuilding or cargo fails with `os error 5`.

When the SDK is already published and the server is in-repo, mirroring the field names server-side (keeping both spellings) unblocks every consumer without waiting for an SDK release — and belongs in a test, since the response looks correct in `curl` and only a validating client rejects it.

## When to Use

Porting a client (GUI, dashboard, script) onto a published SDK, or changing call sites to match an SDK's types.

## When NOT to Use

Pure in-repo refactors where the client and server are compiled together — there the type checker is the contract.
