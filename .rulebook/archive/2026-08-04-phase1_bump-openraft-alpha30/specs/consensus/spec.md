# Consensus dependency pinning and HA validation

## MODIFIED Requirements

### Requirement: The consensus stack is pinned as a whole

Every crate in the openraft family the build resolves — `openraft`,
`openraft-memstore`, `openraft-macros`, `openraft-rt`, `openraft-rt-tokio` —
MUST resolve to the same alpha version. The manifests pin `openraft` and
`openraft-memstore` with `=`; the lockfile SHALL hold the sibling crates at
that same version, because upstream declares them with a caret and Cargo
otherwise floats them to a newer alpha upstream does not ship together.

#### Scenario: A bump keeps the family aligned
Given `openraft` is pinned to a specific alpha in the manifests
When the lockfile is refreshed
Then every `openraft*` entry in `Cargo.lock` names that same alpha

#### Scenario: cargo update cannot drift the consensus layer
Given the `=` pins are in place
When `cargo update` runs without an explicit `-p openraft`
Then the openraft family stays on the pinned alpha

### Requirement: Snapshot data belongs to the components that carry it

Since openraft 0.10.0-alpha.29 `SnapshotData` is no longer part of
`RaftTypeConfig`. The state machine, its snapshot builder and the v2 network
SHALL each declare `type SnapshotData`, and they MUST agree on one type.

#### Scenario: The three implementations agree
Given the cluster state machine, its snapshot builder and the network v2 impl
When the crate is compiled
Then all three resolve `SnapshotData` to the same type and the build succeeds

## ADDED Requirements

### Requirement: A consensus bump is validated on a live cluster

A change to the pinned consensus version SHALL be validated against a cluster
of at least three nodes communicating over real sockets. Single-node
bootstrap, or membership bootstrapped against addresses that never answer, is
NOT sufficient evidence that HA works.

#### Scenario: Three nodes elect a leader
Given three Raft nodes each serving their Raft RPC endpoints on loopback
When the first node bootstraps the cluster with all three members
Then exactly one node reports itself leader within the election window

#### Scenario: A write replicates over the wire
Given a three-node cluster with an elected leader
When a command is proposed on the leader
Then both followers' state machines observe the command

#### Scenario: The cluster survives losing its leader
Given a three-node cluster with an elected leader
When the leader is shut down and its listener stops answering
Then the surviving majority elects a different leader
And that leader accepts a new write
And the write made before the failover is still present
