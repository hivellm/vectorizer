## 1. Prerequisites

- [ ] 1.1 Confirm `phase6_add-rpc-protocol-server` is merged and stable
- [ ] 1.2 Confirm at least the Rust + Python + TypeScript SDKs (`phase6_sdk-rust-rpc`, `phase6_sdk-python-rpc`, `phase6_sdk-typescript-rpc`) have RPC support

## 2. Config + docs reorder

- [ ] 2.1 Rewrite `config.example.yml` with `rpc:` as the first protocol block; add comments calling it "default / recommended"
- [ ] 2.2 Rewrite `README.md` "Getting Started" section to use RPC client example; move HTTP/gRPC/MCP to "Other transports"
- [ ] 2.3 Update `docs/deployment/` to flag RPC as default and REST as management plane

## 3. Deployment manifests

- [ ] 3.1 Update `docker-compose.yml` to expose RPC port 15503 as primary; keep REST 15002; update healthcheck to `nc -z host 15503`
- [ ] 3.2 Update `Dockerfile` EXPOSE directive to list 15503 first
- [ ] 3.3 Update `helm/vectorizer/values.yaml` and templates to set RPC as the default service port
- [ ] 3.4 Update `k8s/*.yaml` ClusterIP/Headless service specs accordingly

## 4. CLI + SDK defaults

- [ ] 4.1 Change default client transport in `src/bin/vectorizer-cli.rs` to RPC; add `--transport rest` flag for users who want old behavior
- [ ] 4.2 Update each SDK's `README.md` quickstart to show RPC first (coordinate with the six SDK tasks)

## 5. Migration guidance

- [ ] 5.1 Write `docs/migration/rpc-default.md` explaining what changed, why, and how to opt back into REST
- [ ] 5.2 Add CHANGELOG entry under "Changed: default transport is now RPC"

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Publish migration + deployment docs; link from README "Upgrading" section
- [ ] 6.2 Add an integration test running the CLI with no `--transport` flag and asserting it uses RPC; docker-compose smoke test asserting healthcheck succeeds on RPC
- [ ] 6.3 Run `cargo test --all-features` and the docker smoke test; confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
