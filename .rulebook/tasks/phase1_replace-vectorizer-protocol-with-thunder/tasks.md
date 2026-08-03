## 1. Design / spec
- [x] 1.1 Study the Synap + Nexus thunder-rpc migrations (server + Rust SDK) and thunder's wire format / multi-language story; write the spec resolving the two open questions (gRPC disposition + cross-language SDK transport) — see design.md + specs/rpc-transport/spec.md

## 2. Server + wire types
- [ ] 2.1 Add thunder-rpc (server) to vectorizer-server; port the RPC command/dispatch table onto thunder's service model
- [ ] 2.2 Move rpc_wire request/response types to a shared location, serialized via thunder's codec; drop the hand-rolled codec

## 3. Cluster / gRPC disposition
- [ ] 3.1 Port the cluster RPC to thunder OR relocate the tonic/prost gRPC gen out of vectorizer-protocol (per the spec)

## 4. SDKs
- [ ] 4.1 Rust SDK: swap vectorizer-protocol -> thunder-rpc client (follow synap/sdks/rust)
- [ ] 4.2 TS / Python / Go / C# SDK transports migrated to thunder's wire (per the spec)

## 5. Delete the crate
- [ ] 5.1 Remove crates/vectorizer-protocol from the workspace once unreferenced; cargo check + clippy clean

## 6. Tail (docs + tests — check or waive with tailWaiver)
- [ ] 6.1 Update or create documentation covering the implementation
- [ ] 6.2 Write tests covering the new behavior
- [ ] 6.3 Run tests and confirm they pass
