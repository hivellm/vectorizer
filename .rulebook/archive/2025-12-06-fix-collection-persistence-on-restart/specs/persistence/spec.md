# Fix Collection Persistence on Restart Specification (Vectorizer)

## MODIFIED Requirements

### Requirement: API-Created Collection Persistence
The system SHALL persist collections created via API immediately to disk.

#### Scenario: REST API collection creation persistence
Given a user creates a collection via POST /collections
When the collection is created successfully
Then the system SHALL save the collection to disk immediately
And the system SHALL include the collection in vectorizer.vecdb
And the system SHALL make the collection available after server restart

#### Scenario: GraphQL collection creation persistence
Given a user creates a collection via GraphQL createCollection mutation
When the collection is created successfully
Then the system SHALL save the collection to disk immediately
And the system SHALL include the collection in vectorizer.vecdb
And the system SHALL make the collection available after server restart

#### Scenario: Collection with vectors persistence
Given a user creates a collection via API and inserts vectors
When the collection and vectors are created
Then the system SHALL save both collection and vectors to disk
And the system SHALL load both collection and vectors after server restart
And the system SHALL make all vectors searchable after restart

### Requirement: Collection Loading on Startup
The system SHALL load all persisted collections from vectorizer.vecdb on server startup.

#### Scenario: Load API-created collections
Given collections were created via API and saved to vectorizer.vecdb
When the server starts
Then the system SHALL load all collections from vectorizer.vecdb
And the system SHALL make API-created collections available
And the system SHALL preserve collection configuration and metadata

#### Scenario: Load collections with vectors
Given collections with vectors were saved to vectorizer.vecdb
When the server starts
Then the system SHALL load all collections
And the system SHALL load all vectors for each collection
And the system SHALL make vectors searchable immediately after load

### Requirement: Immediate Persistence
The system SHALL save collections to disk immediately upon creation, not waiting for auto-save interval.

#### Scenario: Immediate save on creation
Given a collection is created via API
When the creation request completes
Then the system SHALL have saved the collection to disk
And the system SHALL have written the collection to vectorizer.vecdb (or marked for immediate compaction)
And the system SHALL NOT require waiting for 5-minute auto-save interval

#### Scenario: Persistence before response
Given a collection creation request is received
When the collection is created in memory
Then the system SHALL save to disk before returning success response
And the system SHALL handle save errors gracefully (log warning, don't fail request if save fails)

### Requirement: Auto-Save Integration
The system SHALL ensure API-created collections trigger auto-save mechanisms.

#### Scenario: Changes detected flag
Given a collection is created via API
When mark_collection_for_save() is called
Then the system SHALL set changes_detected flag in AutoSaveManager
And the system SHALL include the collection in next compaction cycle

#### Scenario: Compaction includes API collections
Given collections were created via API
When auto-save compaction runs
Then the system SHALL include API-created collections in compaction
And the system SHALL write all collections to vectorizer.vecdb

## ADDED Requirements

### Requirement: Collection Persistence Verification
The system SHALL provide a way to verify collections are persisted correctly.

#### Scenario: Verify collection persistence
Given a collection is created via API
When persistence is verified
Then the system SHALL confirm collection exists in vectorizer.vecdb
And the system SHALL confirm collection metadata is saved
And the system SHALL confirm collection can be loaded after restart

