# Proposal: phase6_sdk-framework-integrations-rpc

## Why

Beyond the six core SDKs, the repo ships integrations for popular ML/automation frameworks: `sdks/langchain/`, `sdks/langchain-js/`, `sdks/langflow/`, `sdks/n8n/`, `sdks/pytorch/`, `sdks/tensorflow/`.

These are thin wrappers around the core SDKs — so once `phase6_sdk-python-rpc` and `phase6_sdk-typescript-rpc` ship, each framework integration needs a **small** update to consume RPC by default and advertise the change to its ecosystem.

Grouping them in one task avoids 6 tiny rulebook tasks with nearly identical content.

## What Changes

Per integration:

1. **langchain (Python)**: update `VectorizerRetriever` / `VectorizerVectorStore` to use the updated Python SDK (RPC default); bump version; update README.
2. **langchain-js (TypeScript)**: same for JS/TS ecosystem; uses updated TS SDK.
3. **langflow**: update the Vectorizer node/component definition to reference RPC in examples.
4. **n8n**: update the Vectorizer credential + node to support RPC URL format (e.g., `vrpc://host:15503`); keep HTTP credential option for backwards compat.
5. **pytorch**: update dataset loader / indexer helpers to use the new Python SDK.
6. **tensorflow**: same.

Each integration's `README.md` quickstart uses RPC. Each publishes a new version to its respective registry (PyPI, npm, langchain hub, n8n community) with CHANGELOG notes.

## Impact

- Affected specs: SDK spec
- Affected code: `sdks/langchain/`, `sdks/langchain-js/`, `sdks/langflow/`, `sdks/n8n/`, `sdks/pytorch/`, `sdks/tensorflow/` (minor each), CHANGELOG
- Breaking change: NO in most cases (they keep functioning with HTTP if the underlying SDK does); n8n may be YES if credential shape changes — document clearly
- User benefit: ecosystem integrations get the fast path automatically; no per-framework friction.
