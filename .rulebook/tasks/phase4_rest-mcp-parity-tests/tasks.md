## 1. Design

- [ ] 1.1 Inventory every MCP tool and every REST handler; document in `design.md`
- [ ] 1.2 Identify the delta (MCP-only, REST-only)

## 2. Implementation

- [ ] 2.1 Create `src/server/capabilities.rs` with a `Capability` struct and a static registry (Vec or phf map)
- [ ] 2.2 Migrate MCP tool construction to read from the registry
- [ ] 2.3 Migrate REST route construction to read from the registry (or add per-route registry assertions)
- [ ] 2.4 Fill in the delta: add missing REST or MCP counterparts for each divergent operation

## 3. Enforcement

- [ ] 3.1 Add an integration test `tests/api/parity.rs` that iterates the registry and verifies both transports produce equivalent responses for a set of operations
- [ ] 3.2 Add a compile-time or boot-time assertion that no handler exists outside the registry

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Document the capability registry pattern in `docs/architecture/capabilities.md`
- [ ] 4.2 Publish the generated OpenAPI spec under `docs/api/openapi.yaml`
- [ ] 4.3 Run `cargo test --all-features -- parity` and confirm all parity tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
