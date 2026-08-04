# Spec: RPC transport migration to thunder-rpc

## ADDED Requirements

### Requirement: Server RPC runs on thunder-rpc
The `vectorizer-server` crate SHALL serve its binary RPC surface through
`thunder-rpc` (lib `thunder`, v0.2.2, `features = ["server"]`) instead of the
hand-rolled `vectorizer_protocol::rpc_wire` codec. The server SHALL implement
`thunder::server::Dispatch` and start its listener via
`thunder::server::spawn_listener`, with a `thunder::Config` whose scheme, port,
`Handshake::AuthCommand`, `PushPolicy`, `ErrorConvention::Resp3Prefixes`, and
`max_frame_bytes` match the client configuration the SDKs use.

#### Scenario: Existing command dispatched over thunder
Given a `vectorizer-server` built on the thunder transport
And a client issuing the `search.basic` command with its positional args
When the server receives the request frame
Then the same dispatch handler that served `search.basic` before the migration
  runs and returns its result as a `thunder::Value`
And the client receives a response correlated by the request id.

#### Scenario: Authentication maps onto the thunder handshake
Given the RPC listener configured with `Handshake::AuthCommand`
When a client presents an API key or user/password on connect
Then `Dispatch::authenticate` validates it through the existing auth stack
And an unauthenticated client is rejected before any command is dispatched.

### Requirement: Wire types serialize via thunder::Value
The RPC request/response value type SHALL be `thunder::Value` (or a type that
serializes to the identical MessagePack representation as the retired
`VectorizerValue`), preserving the frozen v1 wire (4-byte little-endian
length prefix + MessagePack body). The hand-rolled `rpc_wire::codec` and
`rpc_wire::types` MUST NOT be referenced once the migration completes.

#### Scenario: Value round-trips unchanged on the wire
Given a response carrying a string value
When it is encoded by the thunder transport and decoded by a thunder client
Then the decoded value equals the original
And its on-wire bytes match the frozen v1 MessagePack framing.

### Requirement: Every SDK uses its official thunder client library
Each SDK SHALL obtain its binary transport from the official per-language
thunder client rather than a bespoke implementation: Rust `thunder-rpc`,
Python `hivellm-thunder`, TypeScript `@hivehub/thunder`, Go
`github.com/hivellm/thunder-go`, C# `HiveLLM.Thunder`. The SDK command catalog
(the ~75 command names) SHALL remain unchanged; only the transport beneath it
changes. REST SHALL remain available as the documented fallback.

#### Scenario: Rust SDK connects through thunder::Client
Given the Rust SDK depending on `thunder-rpc` with `features = ["client"]`
When it dials a `vectorizer://host:port` endpoint
Then it connects via `thunder::Client::connect_with` using a config matching the
  server
And it issues commands and reads responses without importing `vectorizer-protocol`.

#### Scenario: A non-Rust SDK talks to the thunder server
Given the TypeScript SDK depending on `@hivehub/thunder`
When it sends a mapped command to a thunder-based `vectorizer-server`
Then the server dispatches and answers it over the shared wire
And no SDK hand-implements framing or MessagePack.

### Requirement: gRPC generation is relocated out of vectorizer-protocol
The tonic/prost gRPC generation (the `vectorizer`, `cluster`, and Qdrant proto
trees, their `build.rs`, and the generated modules) SHALL move to a dedicated
`crates/vectorizer-grpc` crate, carrying the `tonic`, `prost`, `tonic-prost`,
and `protoc-bin-vendored` dependencies. gRPC SHALL NOT be ported onto thunder;
the Qdrant-compatible surface MUST remain a gRPC service for external Qdrant
clients. Consumers in `crates/vectorizer` (`cluster/*`, `grpc_conversions.rs`)
SHALL import the generated types from `vectorizer-grpc`.

#### Scenario: Cluster gRPC behavior is unchanged after relocation
Given the cluster gRPC code repointed from `vectorizer_protocol::grpc_gen`
  to `vectorizer_grpc`
When a raft `RaftAppendEntries` call is made between two nodes
Then it uses the same generated `ClusterServiceClient` and messages as before
And the cluster behavior is identical to pre-migration.

### Requirement: vectorizer-protocol is deleted
Once no crate references `vectorizer-protocol`, it SHALL be removed from the
workspace `members`, and `cargo check` plus `cargo clippy -- -D warnings` SHALL
pass with the crate absent.

#### Scenario: Workspace builds without the crate
Given `crates/vectorizer-protocol` removed from the workspace
When `cargo check --workspace` runs
Then it completes with no unresolved `vectorizer_protocol` imports
And clippy reports no warnings.
