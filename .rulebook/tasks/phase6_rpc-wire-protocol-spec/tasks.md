## 1. Research Synap reference

- [ ] 1.1 Read Synap's wire format in full: `../Synap/synap-server/src/protocol/synap_rpc/{codec.rs,types.rs,server.rs}` and `../Synap/synap-server/src/protocol/resp3/` — capture exact line refs for the spec
- [ ] 1.2 Read Synap's SDK implementations under `../Synap/sdks/` to understand client-side expectations

## 2. Draft specs

- [ ] 2.1 Draft `docs/specs/VECTORIZER_RPC.md` covering framing, envelope, error shape, command catalog skeleton, auth, streaming, versioning
- [ ] 2.2 Draft `/.rulebook/specs/RPC.md` with the project-rule-level contract (must-reuse-SynapRPC-wire, must-mirror-REST-capability-registry)
- [ ] 2.3 Build the command catalog table by enumerating every current REST route and MCP tool; map each to an `rpc.command.name`
- [ ] 2.4 Document the error-code taxonomy matching `VectorizerError` kinds (coordinate with `phase3_unify-error-enums`)
- [ ] 2.5 Decide streaming strategy (single frame vs chunked) for large search responses; record rationale
- [ ] 2.6 Decide wire-level versioning (HELLO vs magic byte); record rationale
- [ ] 2.7 Add an ASCII diagram showing a full handshake + request/response exchange

## 3. Review

- [ ] 3.1 Circulate the spec for internal review before `phase6_add-rpc-protocol-server` starts
- [ ] 3.2 Incorporate review feedback; freeze v1 of the spec

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 4.1 Publish both spec documents and link them from `README.md` under a "Protocols" section
- [ ] 4.2 The spec itself is the deliverable; add a placeholder integration-test module referencing the command catalog so it surfaces in coverage once the server task implements it
- [ ] 4.3 Run `cargo test --all-features` and confirm existing tests still pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
