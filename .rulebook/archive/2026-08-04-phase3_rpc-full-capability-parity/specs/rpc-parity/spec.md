# RPC capability parity

## ADDED Requirements

### Requirement: RPC is the primary data-plane surface

The VectorizerRPC command surface SHALL expose every data-plane
capability declared in the capability registry. An RPC-only client MUST
NOT need an HTTP fallback to reach any registry capability.

#### Scenario: Every registry capability is reachable over RPC
Given the capability registry declares a data-plane capability
When a client connects over VectorizerRPC and authenticates
Then a command exists that performs that capability
And the command name appears in the HELLO capabilities list

#### Scenario: Boot refuses a registry entry with no RPC command
Given a registry entry marked as reachable over RPC
When the server boots and runs the inventory invariant assertion
Then the assertion fails if no dispatch command covers that entry

### Requirement: Graph is reachable end to end over RPC

The server SHALL let a client create a graph-enabled collection and drive
every graph command without leaving RPC.

#### Scenario: Enabling a graph over RPC
Given a collection created over RPC with vectors inserted
When the client sends `graph.enable` with the collection name
Then the graph is enabled and the node count is returned
And subsequent `graph.list_nodes` succeeds for that collection

#### Scenario: Graph status over RPC
Given a collection whose graph is enabled
When the client sends `graph.status` with the collection name
Then the reply reports whether the graph is enabled plus node and edge counts

#### Scenario: Graph config survives collection creation
Given a `collections.create` call whose config sets `graph.enabled` true
When the collection is created
Then the stored collection config carries the graph settings
And graph commands work against it without a separate enable call

### Requirement: Collection creation honours the full config

`collections.create` SHALL apply every field of `CollectionConfig` it is
given and MUST NOT silently substitute a default for a field the caller
supplied.

#### Scenario: Quantization from the create config is applied
Given a `collections.create` config requesting scalar quantization
When the collection is created
Then `collections.get_info` reports scalar quantization

#### Scenario: Unknown config field is rejected, not ignored
Given a `collections.create` config carrying an unrecognised key
When the collection is created
Then the server answers with an error naming the unrecognised key

### Requirement: User management over RPC

The server SHALL expose user creation, listing, deletion and password
change over RPC, gated on the admin role.

#### Scenario: Admin creates a user over RPC
Given an authenticated admin session
When the client sends `auth.users_create` with a username and password
Then the user is created and can authenticate on a new connection

#### Scenario: Non-admin cannot list users
Given an authenticated non-admin session
When the client sends `auth.users_list`
Then the server answers with a `NOPERM` error

### Requirement: Cluster inspection over RPC

The server SHALL expose cluster node listing, single-node lookup, node
removal, leader lookup and role lookup over RPC.

#### Scenario: Cluster role on a single node
Given a server running without cluster mode
When the client sends `cluster.role`
Then the reply names the standalone role rather than failing

### Requirement: Capability advertisement stays truthful

`rpc_capability_names()` SHALL list exactly the commands that perform
work, and MUST NOT list a command that only returns a
transport-unavailable error.

#### Scenario: A newly added command is advertised
Given a command added to the dispatch table
When a client completes HELLO
Then the command name appears in the capabilities array

## MODIFIED Requirements

### Requirement: RPC command catalog documentation

`docs/specs/VECTORIZER_RPC.md` SHALL catalog every routed command with
its argument shape and return shape, and MUST NOT claim command names
match registry ids when the naming differs.

#### Scenario: Catalog covers the dispatch table
Given the dispatch table routes N commands
When the spec's command catalog is read
Then every routed command appears in the catalog
