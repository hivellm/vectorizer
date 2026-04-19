# Proposal: phase6_make-rpc-default-transport

## Why

Once `phase6_add-rpc-protocol-server` ships and all SDKs have been updated (`phase6_sdk-*-rpc`), RPC becomes the recommended client ↔ server transport for every use case that is not operator-interactive. This task flips the default so:

- Example code in README, docs, blog posts uses RPC first.
- SDKs default to RPC unless the caller explicitly asks for REST/gRPC/MCP.
- Docker/Helm/k8s examples expose the RPC port first and mark REST as "management/dashboard".
- `config.example.yml` lists RPC before other transports and sets `rpc.enabled = true` explicitly.

This is a UX / marketing / defaults cutover — not a protocol change. REST, MCP, gRPC stay available and fully functional; we just stop treating them as the default example.

## What Changes

1. **Config order**: reorder `config.example.yml` so `rpc:` block comes first; annotate comments explaining which transport is for what.
2. **SDK defaults**: each SDK (`phase6_sdk-*-rpc`) updates its "quick start" example to use RPC; adds a "legacy REST mode" section for users who want HTTP.
3. **Docs**: rewrite `README.md` "Getting Started" to show the RPC client first. Move REST/MCP/gRPC to a "Other transports" subsection.
4. **Docker**: default `docker-compose.yml` and `Dockerfile` ports expose RPC (15503) alongside REST (15002). Health check uses RPC `PING`.
5. **Helm/k8s**: default service selects RPC as the primary port.
6. **CLI**: `vectorizer-cli` default client mode = RPC.
7. **Dashboard**: keep on REST (browsers don't speak RPC); document that REST port is primarily for dashboard/admin.

## Impact

- Affected specs: `/.rulebook/specs/RPC.md`, deployment spec
- Affected code: `README.md`, `config.example.yml`, `docker-compose.yml`, `Dockerfile`, `helm/vectorizer/values.yaml`, `k8s/*.yaml`, all SDK READMEs, `src/bin/vectorizer-cli.rs` (default client mode)
- Breaking change: NO for code (transports stay available); YES for docs/example reproducibility — users following old docs will still work, but new docs show RPC.
- User benefit: faster path out-of-the-box; new users discover the fastest transport first; aligns with Synap conventions across the HiveLLM ecosystem.
