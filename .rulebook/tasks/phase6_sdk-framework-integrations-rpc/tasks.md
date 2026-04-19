## 1. Prerequisites

- [ ] 1.1 `phase6_sdk-python-rpc` published to PyPI
- [ ] 1.2 `phase6_sdk-typescript-rpc` published to npm

## 2. langchain (Python)

- [ ] 2.1 Bump `vectorizer` SDK dep in `sdks/langchain/pyproject.toml`
- [ ] 2.2 Ensure `VectorizerRetriever` / `VectorizerVectorStore` use the default RPC transport; expose `transport=` kwarg for overrides
- [ ] 2.3 Update `sdks/langchain/examples/` and README to RPC
- [ ] 2.4 Publish new version to PyPI + langchain hub

## 3. langchain-js (TypeScript)

- [ ] 3.1 Bump TS SDK dep in `sdks/langchain-js/package.json`
- [ ] 3.2 Update retriever/vector-store classes to use RPC by default
- [ ] 3.3 Update examples + README
- [ ] 3.4 Publish new version to npm

## 4. langflow

- [ ] 4.1 Update the Vectorizer component definition in `sdks/langflow/` to advertise RPC URL format
- [ ] 4.2 Update component docs/screenshots in the README

## 5. n8n

- [ ] 5.1 Extend `sdks/n8n/` credential type to accept the canonical RPC URL (`vectorizer://host:port`, NOT `vrpc://`) alongside `http(s)://`. URL parsing is done by the underlying SDK's `parseEndpoint` helper (see `phase6_sdk-typescript-rpc` items 3.3 + 3.4) — n8n adds no scheme-detection code of its own.
- [ ] 5.2 Update node action code to route requests through the chosen transport (the SDK does the routing automatically based on the URL scheme; n8n only forwards the credential URL).
- [ ] 5.3 Update README + publish a new version to n8n community

## 6. pytorch

- [ ] 6.1 Bump `vectorizer` Python SDK dep in `sdks/pytorch/pyproject.toml`
- [ ] 6.2 Update `VectorizerDataset` / indexer helpers to default to RPC
- [ ] 6.3 Update examples + README
- [ ] 6.4 Publish new version to PyPI

## 7. tensorflow

- [ ] 7.1 Bump Python SDK dep in `sdks/tensorflow/pyproject.toml`
- [ ] 7.2 Update `VectorizerTFDataset` helpers accordingly
- [ ] 7.3 Update examples + README
- [ ] 7.4 Publish new version to PyPI

## 8. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 8.1 Update each integration's README and CHANGELOG; link from the project-root README SDK matrix
- [ ] 8.2 Smoke tests: for each integration, one end-to-end workflow using the updated SDK against a running server
- [ ] 8.3 Run each integration's test command (pytest / npm test / etc) and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
