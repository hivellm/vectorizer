# Architecture Decoupling Spec

## ADDED Requirements

### Requirement: No upward module dependencies

Foundation and core modules (`db`, `cache`, `config`, `persistence`)
MUST NOT import symbols from service modules (`monitoring`, `auth`,
`hub`, `cluster`). Cross-layer needs MUST go through traits defined
at or below the consumer's layer.

#### Scenario: Metrics via trait

Given `db/ttl_reaper.rs` needs to record a metric
When the code is compiled
Then it MUST reference only the `MetricsSink` trait, with no `use`
of `crate::monitoring`

#### Scenario: Back-reference count

Given the nine back-references cataloged in the 2026-07-11 analysis
When the decoupling work completes
Then a grep for the cataloged import paths MUST find zero remaining
occurrences

### Requirement: Config module owns no service types

The `config` module MUST NOT depend on `auth`, `hub`, or `cluster`
types. Service modules SHALL parse their own sections from generic
config data.

#### Scenario: Config compiles standalone

Given the `config` module and its imports
When dependencies are analyzed
Then `config` MUST have no edge to `auth`, `hub`, or `cluster`

### Requirement: Sharded collections depend on an abstraction

`db/distributed_sharded_collection.rs` MUST depend on a `ShardRouter`
trait rather than concrete cluster manager/client types.

#### Scenario: db compiles without cluster

Given the `ShardRouter` trait with a test stub implementation
When db-level sharding tests run
Then they MUST execute without constructing any real cluster type

### Requirement: No dead build stanzas

`crates/vectorizer/Cargo.toml` MUST NOT contain commented-out
`[[bin]]` stanzas or references to feature names that do not exist
in `[features]`.

#### Scenario: Feature references valid

Given every feature name mentioned in Cargo.toml (active or comment)
When checked against the `[features]` table
Then each MUST exist or the mention MUST be removed

### Requirement: No re-entrant DashMap deadlock path

Acquiring a mutable collection reference while holding a shared
reference to the same map MUST be impossible or fail fast in debug
builds — not deadlock.

#### Scenario: Re-entrant access guarded

Given a code path holding a collection `Ref`
When the same thread requests a mutable reference on the same shard
Then the operation MUST either be structurally prevented (via
`alter`/`entry` routing) or trigger a debug assertion, never an
untraceable deadlock

### Requirement: Documented lock-library convention

Each module under `db/` MUST use parking_lot locks for non-await
critical sections and tokio locks only where guards are held across
await points, with the convention stated in module documentation.

#### Scenario: Await-held guard uses tokio lock

Given a lock guard held across an `.await`
When the code is reviewed against the convention
Then that lock MUST be a `tokio::sync` primitive
