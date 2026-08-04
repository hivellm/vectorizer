# Collection-level TTL

## ADDED Requirements

### Requirement: A configured collection TTL reaches the vectors that arrive

When a collection carries a TTL of `N` seconds, the store SHALL stamp
`__expires_at = now + N * 1000` onto the payload of every vector inserted or
updated on that collection, so the existing reaper and read filter apply
without any further wiring.

#### Scenario: Inserting into a collection that has a TTL
Given a collection configured with a TTL of 600 seconds
When a client inserts a vector
Then the stored payload carries `__expires_at` about 600 seconds in the future
And the rest of the caller's payload is unchanged

#### Scenario: A vector with no payload still expires
Given a collection configured with a TTL
When a client inserts a vector that carries no payload at all
Then the store creates one so the expiry can be recorded

#### Scenario: Updating a vector does not strip the expiry
Given a vector in a TTL collection that was stamped at insert
When a client updates it with a payload that carries no expiry field
Then the store re-stamps the collection TTL rather than leaving it immortal

#### Scenario: A collection without a TTL is untouched
Given a collection with no TTL configured
When a client inserts a vector
Then no `__expires_at` is added

#### Scenario: The TTL is scoped to its own collection
Given collection A has a TTL and collection B does not
When a client inserts into B
Then no expiry is stamped

### Requirement: An explicit per-vector expiry outranks the collection rule

A vector that already carries `__expires_at` SHALL keep that value.

#### Scenario: Inserting a vector that carries its own expiry
Given a collection configured with a TTL of 600 seconds
When a client inserts a vector whose payload already sets `__expires_at`
Then the stored expiry is the value the client supplied

#### Scenario: Clearing a per-vector expiry under a collection TTL
Given a vector in a collection that has a TTL
When a client clears that vector's expiry
Then the collection TTL re-stamps it
And the response reports the expiry that is actually stored, not null

### Requirement: The TTL surface reports only what it does

The endpoint SHALL NOT report success for a configuration it cannot apply.

#### Scenario: A TTL of zero is rejected
Given a caller sets `ttl_secs` to 0
When the request is handled
Then it fails validation, because a zero TTL would expire every insert on
arrival and `null` is how a TTL is cleared

#### Scenario: A payload that cannot hold the field is rejected
Given a collection configured with a TTL
When a client inserts a vector whose payload root is not a JSON object
Then the insert fails with an error naming the cause
And nothing is stored

#### Scenario: The configured TTL can be read back
Given a caller sets a TTL
When the caller reads the TTL
Then the configured value is returned
And after clearing it, the read reports no TTL

### Requirement: The rule is applied before the write is logged

Stamping SHALL happen before the WAL record is written.

#### Scenario: Replaying the WAL after a restart
Given a vector inserted under a collection TTL
When the WAL is replayed later
Then the vector keeps the expiry computed at insert time rather than one
computed from the replay time

#### Scenario: A replica receives the expiry as data
Given a master with a collection TTL and a replica without the rule
When the vector replicates
Then the replica holds the same `__expires_at` and expires it on schedule

### Requirement: Both transports expose the TTL

The collection TTL SHALL be readable and writable over VectorizerRPC as well as
REST, since RPC is the default protocol.

#### Scenario: Setting the TTL over RPC
Given an RPC client
When it calls `collections.set_ttl` with a collection and a TTL
Then later inserts on that collection carry an expiry
And `collections.get_ttl` reports the configured value

#### Scenario: Setting a TTL on a collection that does not exist
Given an RPC client
When it calls `collections.set_ttl` for an unknown collection
Then the call fails instead of storing configuration for a missing collection
