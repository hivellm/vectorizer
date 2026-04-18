## 1. Design

- [ ] 1.1 Enumerate all leaf error enums in `src/` via `grep -rn '#\[derive.*Error\]' src/`; list in `design.md`
- [ ] 1.2 Define the `VectorizerError::ErrorKind` classification taxonomy (NotFound, Unauthorized, Forbidden, BadRequest, Conflict, Internal, Unavailable, etc.)
- [ ] 1.3 Document the HTTP ↔ gRPC ↔ MCP mapping table

## 2. Implementation

- [ ] 2.1 Add `#[from]` or explicit `impl From<LeafErr> for VectorizerError` for each of the 9 leaf enums
- [ ] 2.2 Replace `format!("{}: {}", ctx, e)` conversions in `src/server/mcp_handlers.rs:27-91` with `?` + `.map_err()` where context is truly new
- [ ] 2.3 Create `src/error/mapping.rs` with `impl From<VectorizerError> for (StatusCode, Body)`, `impl From<VectorizerError> for tonic::Status`, and MCP error-code mapping
- [ ] 2.4 Replace handler-side ad-hoc status picking with centralized mapping call

## 3. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 3.1 Update `docs/development/error-handling.md` (or create) with the taxonomy and mapping tables
- [ ] 3.2 Write unit tests for each `From` impl; integration tests proving a `NotFound` leaf becomes 404/NOT_FOUND/MCP-NotFound across all three layers
- [ ] 3.3 Run `cargo test --all-features` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
