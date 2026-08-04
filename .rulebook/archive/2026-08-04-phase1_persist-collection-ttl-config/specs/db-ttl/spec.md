# Durable collection-TTL configuration

## ADDED Requirements

### Requirement: The collection TTL rule survives a restart

A configured collection TTL SHALL be written to the collection's persisted
record and restored when the collection is loaded, so vectors inserted after a
restart expire on the same rule as vectors inserted before it.

#### Scenario: Restarting with a configured TTL
Given a collection with a TTL and vectors written under it
When the archive is written and a fresh store loads it
Then the TTL reads back on the loaded collection
And a vector inserted after the load is stamped with an expiry

#### Scenario: Restoring does not extend a vector's life
Given a vector stamped with an expiry before the archive was written
When the collection is loaded again
Then the vector keeps the expiry it was saved with, not one derived from the
load time

#### Scenario: Restoring a native snapshot
Given a snapshot of a collection that had a TTL
When the snapshot is restored
Then the restored collection carries the same TTL

#### Scenario: An archive written before the field existed
Given an archive whose collection record has no TTL field
When it is deserialised
Then the collection loads with no TTL, which is the behaviour it was saved with

### Requirement: A TTL change is scheduled for persistence

Setting or clearing a TTL SHALL mark the store as changed, so the rule reaches
disk on the next compaction rather than waiting for an unrelated write.

#### Scenario: Setting a TTL over REST or RPC
Given a caller sets a collection TTL
When the request completes
Then the store is marked changed, and the RPC command is classified as
mutating like any other write to persisted state

### Requirement: The rule follows the collection through its lifecycle

The TTL rule SHALL be tied to the collection it was configured on, not to a
name that outlives it.

#### Scenario: Deleting a collection
Given a collection with a TTL
When it is deleted and a new collection is created under the same name
Then the new collection has no TTL, and its inserts are not stamped

#### Scenario: Renaming a collection
Given a collection with a TTL
When it is renamed
Then the renamed collection keeps the TTL
And no rule is left behind under the old name

#### Scenario: Writing through an alias
Given a collection with a TTL and an alias pointing at it
When a vector is inserted addressing the alias
Then the vector is stamped, because the alias resolves to the target's rule
