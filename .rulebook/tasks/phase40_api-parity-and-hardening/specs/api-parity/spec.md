# API Parity & Hardening Spec

## ADDED Requirements

### Requirement: Registry matches the live router

Every REST route registered in the axum router MUST appear in the
capability registry with the correct HTTP method, or be listed in an
explicit exclusion set. The parity test MUST fail on any drift.

#### Scenario: Method mismatch detected

Given a capability declared GET whose router registration is POST
When the parity test runs
Then the test MUST fail naming the capability

#### Scenario: Unregistered route detected

Given a REST route present in the router but absent from both the
registry and the exclusion set
When the parity test runs
Then the test MUST fail naming the route

### Requirement: MCP mirrors REST operations

Every collection-level and search operation exposed via REST MUST
have an MCP tool equivalent, including `delete_collection`,
`embed_text`, `contextual_search`, `get_database_stats`, the
discovery pipeline, and batch operations.

#### Scenario: delete_collection via MCP

Given an existing collection
When the MCP `delete_collection` tool is invoked
Then the collection MUST be deleted and the response MUST match the
REST semantics

### Requirement: MCP error codes reflect error class

MCP handlers MUST map errors through `VectorizerError` →
`mcp_code()`. Not-found conditions MUST NOT surface as internal
errors.

#### Scenario: Collection not found over MCP

Given no collection named "ghost"
When an MCP search targets "ghost"
Then the JSON-RPC error code MUST be the mapped not-found code, not
−32603

### Requirement: RPC errors carry stable codes

RPC error frames MUST include the stable `VectorizerError::code()`
value alongside the message.

#### Scenario: RPC error code present

Given an RPC request against a missing collection
When the error response is returned
Then it MUST contain a machine-readable error code consistent with
REST `error_type`

### Requirement: Server-side limit clamp

Search endpoints on REST, MCP, and RPC MUST clamp the requested
`limit` to the documented maximum (100) regardless of client input.

#### Scenario: Oversized limit clamped

Given a search request with `limit: 1000000`
When any transport processes it
Then at most 100 results are returned and no allocation proportional
to the requested limit occurs

### Requirement: Consistent GraphQL tenant prefix

All GraphQL resolvers MUST derive tenant-scoped collection names from
a single shared helper.

#### Scenario: Upload targets the created collection

Given a tenant creates collection "docs" via GraphQL
When the same tenant uploads a file to "docs"
Then the vectors MUST land in the collection created, not a
differently-prefixed name

### Requirement: Uniform REST error shape

All REST error responses, including middleware-generated ones, MUST
use the standard `ErrorResponse` shape (`error_type`, `status_code`,
`details`).

#### Scenario: Unauthorized error shape

Given production auth is enabled and a request lacks credentials
When the auth middleware rejects it
Then the body MUST follow the standard `ErrorResponse` shape

### Requirement: Config loaded once, unknown keys surfaced

Server bootstrap MUST obtain configuration exactly once via the
layered loader, and unknown configuration keys MUST produce at least
a prominent warning.

#### Scenario: Typoed key surfaced

Given a config file containing a misspelled key
When the server boots
Then a warning naming the unknown key MUST be logged

#### Scenario: Mode overrides reach all subsystems

Given `VECTORIZER_MODE=production` with a production override for
auth settings
When the server boots
Then the auth subsystem MUST observe the overridden values

### Requirement: First boot succeeds with default config

The shipped default configuration MUST produce a running server on
first boot without manual secret provisioning.

#### Scenario: Fresh install boots

Given an unmodified default `config.yml` and no environment overrides
When the server starts
Then it MUST reach the listening state without a secret-related
boot failure
