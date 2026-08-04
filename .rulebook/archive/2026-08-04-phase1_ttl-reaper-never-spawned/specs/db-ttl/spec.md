# TTL reaper

## ADDED Requirements

### Requirement: A recorded expiry must be acted on

The server SHALL run a TTL reaper that deletes vectors whose `__expires_at`
payload field is in the past. An expiry accepted by `vectors.set_expiry` or
`PATCH /collections/{name}/vectors/{id}/expiry` MUST NOT be recorded and then
ignored.

#### Scenario: An expired vector is removed
Given a vector whose `__expires_at` is in the past
When a sweep runs
Then the vector is deleted from its collection

#### Scenario: A live vector survives the sweep
Given a vector whose `__expires_at` is in the future, and one with no expiry
When a sweep runs
Then both vectors remain readable

#### Scenario: The reaper reports its work
Given a sweep that deleted at least one vector
When `/prometheus/metrics` is scraped
Then `ttl_reaper_scans_total` has advanced for that collection
And `ttl_vectors_expired_total` reports the deletion

### Requirement: The reaper covers collections created after it started

The sweep SHALL enumerate collections on each tick rather than binding to the
set present when it started. Collections are created at runtime over REST, RPC,
MCP and disk load, and there is no single choke point a per-collection spawn
could hook into.

#### Scenario: A collection created later is swept
Given a running reaper
When a collection is created afterwards and given an expired vector
Then a later sweep deletes that vector

#### Scenario: A collection removed between ticks is not an error
Given a collection that disappears between enumeration and sweep
When the sweep reaches it
Then the sweep logs and continues without panicking

### Requirement: The reaper stops with the server

The reaper SHALL stop when the server shuts down, and its handle MUST be
retained for the server's lifetime — `Drop` signals shutdown, so dropping the
handle silently stops the sweep.

#### Scenario: Shutdown stops the sweep
Given a running server
When it shuts down
Then the reaper's shutdown flag is set and the loop exits on its next wake-up

#### Scenario: A stopped reaper deletes nothing
Given a reaper whose shutdown flag was set before its first tick
When an expired vector is inserted
Then it is not deleted and no scan is reported
