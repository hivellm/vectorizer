# Graph Database Specification

## Purpose

This specification defines the requirements for a lightweight internal graph database module within Vectorizer that enables automatic relationship discovery, simple graph traversal, and relationship tracking between documents and files without requiring external graph databases or complex query languages.

## ADDED Requirements

### Requirement: Graph Storage
The system SHALL maintain an internal graph structure that stores nodes representing documents/files and edges representing relationships between them, with support for relationship types, weights, and metadata.

#### Scenario: Create node for document
Given a vector is inserted into a collection
When the vector has file_path metadata
Then a graph node is automatically created with the file_path as node ID and document type

#### Scenario: Store relationship with metadata
Given two documents have a relationship
When a relationship edge is created between them
Then the edge stores relationship type, weight, creation timestamp, and source information

#### Scenario: Persist graph structure
Given a graph with nodes and edges exists
When graph persistence is enabled
Then the graph structure is saved to disk and can be recovered after restart

### Requirement: Automatic Relationship Discovery
The system SHALL automatically discover and create relationships between documents based on semantic similarity thresholds, shared metadata, and payload relationships when vectors are inserted or updated.

#### Scenario: Auto-create SIMILAR_TO relationship
Given a new vector is inserted into a collection
When the vector's similarity to existing vectors exceeds the configured threshold
Then SIMILAR_TO relationships are automatically created between the new vector and similar existing vectors with weights based on similarity scores

#### Scenario: Auto-create REFERENCES relationship
Given a vector payload contains a "references" field with file paths
When the vector is inserted
Then REFERENCES relationships are automatically created from the current document to each referenced file

#### Scenario: Auto-create CONTAINS relationship
Given a vector payload indicates it contains another document
When the vector is inserted
Then a CONTAINS relationship is created from the parent document to the contained document

#### Scenario: Update relationships on vector update
Given an existing vector with relationships is updated
When the updated vector's similarity scores change
Then existing SIMILAR_TO relationships are updated with new weights or removed if below threshold

### Requirement: Simple Graph Queries
The system SHALL provide simple graph traversal operations without requiring complex query languages, including finding neighbors, finding related nodes, and finding paths between nodes.

#### Scenario: Find neighbors of a node
Given a collection contains a graph with nodes and edges
When a query requests neighbors of a specific node
Then the system returns all nodes directly connected to the specified node with their relationship types and weights

#### Scenario: Find related nodes
Given a collection contains a graph with nodes and edges
When a query requests nodes related to a specific node within N hops
Then the system returns all nodes reachable within N hops, ordered by relationship strength

#### Scenario: Find shortest path
Given a collection contains a graph with nodes and edges
When a query requests the path between two nodes
Then the system returns the shortest path between the nodes if one exists, or indicates no path exists

#### Scenario: Filter relationships by type
Given a graph contains multiple relationship types
When a query requests neighbors with a specific relationship type
Then only neighbors connected via that relationship type are returned

### Requirement: Relationship Types
The system SHALL support multiple relationship types including SIMILAR_TO, REFERENCES, CONTAINS, and DERIVED_FROM, with configurable types per collection.

#### Scenario: Create SIMILAR_TO relationship
Given two documents have semantic similarity above threshold
When automatic relationship discovery runs
Then a SIMILAR_TO relationship is created with weight equal to the similarity score

#### Scenario: Create REFERENCES relationship
Given a document payload contains references to other files
When the vector is processed
Then REFERENCES relationships are created from the document to each referenced file

#### Scenario: Create CONTAINS relationship
Given a document payload indicates containment of another document
When the vector is processed
Then a CONTAINS relationship is created from parent to child document

#### Scenario: Create DERIVED_FROM relationship
Given a document payload indicates it was derived from another document
When the vector is processed
Then a DERIVED_FROM relationship is created from derived document to source document

### Requirement: Graph REST API
The system SHALL expose graph functionality through REST endpoints that allow querying nodes, edges, relationships, and performing graph traversals.

#### Scenario: List nodes in collection
Given a collection has documents with graph nodes
When a GET request is made to `/api/v1/graph/nodes/:collection`
Then the system returns a list of all nodes in the collection with their metadata

#### Scenario: Get node neighbors
Given a node has connected neighbors in the graph
When a GET request is made to `/api/v1/graph/nodes/:collection/:node_id/neighbors`
Then the system returns all neighbors with relationship information

#### Scenario: Find related nodes
Given a collection has a graph structure
When a POST request is made to `/api/v1/graph/nodes/:collection/:node_id/related` with max_hops parameter
Then the system returns all related nodes within the specified hop distance

#### Scenario: Find path between nodes
Given two nodes exist in the graph
When a POST request is made to `/api/v1/graph/path` with source and target node IDs
Then the system returns the shortest path between the nodes or indicates no path exists

#### Scenario: Create explicit relationship
Given two nodes exist in the graph
When a POST request is made to `/api/v1/graph/edges` with source, target, and relationship type
Then a new edge is created between the nodes with the specified type and metadata

#### Scenario: Delete relationship
Given an edge exists in the graph
When a DELETE request is made to `/api/v1/graph/edges/:edge_id`
Then the edge is removed from the graph

### Requirement: Graph MCP Tools
The system SHALL provide MCP tools for graph operations that enable AI assistants to query relationships, find related documents, and navigate the graph structure.

#### Scenario: Graph find related tool
Given a collection has graph relationships
When the `graph_find_related` MCP tool is called with a node ID and max_hops
Then the tool returns related nodes with their relationships and distances

#### Scenario: Graph find path tool
Given two nodes exist in the graph
When the `graph_find_path` MCP tool is called with source and target node IDs
Then the tool returns the path between nodes or indicates no path exists

#### Scenario: Graph get neighbors tool
Given a node has neighbors in the graph
When the `graph_get_neighbors` MCP tool is called with a node ID and optional relationship type filter
Then the tool returns all neighbors with relationship details

#### Scenario: Graph create relationship tool
Given two nodes exist in the graph
When the `graph_create_relationship` MCP tool is called with source, target, and relationship type
Then a new relationship edge is created and the edge ID is returned

### Requirement: Graph Configuration
The system SHALL allow configuring automatic relationship discovery rules per collection, including similarity thresholds, enabled relationship types, and relationship creation policies.

#### Scenario: Configure similarity threshold
Given a collection configuration
When the graph.auto_relationship.similarity_threshold is set to a value
Then SIMILAR_TO relationships are only created when similarity exceeds that threshold

#### Scenario: Enable/disable relationship types
Given a collection configuration
When specific relationship types are enabled or disabled in graph.auto_relationship.types
Then only enabled relationship types are automatically created

#### Scenario: Configure max relationships per node
Given a collection configuration
When graph.auto_relationship.max_per_node is set to a value
Then automatic relationship creation stops after creating that many relationships per node

### Requirement: Graph Integration with Vector Operations
The system SHALL automatically maintain graph relationships when vectors are inserted, updated, or deleted, ensuring graph consistency with vector store state.

#### Scenario: Create relationships on vector insert
Given a new vector is inserted into a collection
When the insertion completes successfully
Then graph nodes and relationships are automatically created if enabled for the collection

#### Scenario: Update relationships on vector update
Given an existing vector is updated
When the update changes similarity scores or metadata
Then graph relationships are automatically updated to reflect new similarity scores or relationships

#### Scenario: Remove relationships on vector delete
Given a vector is deleted from a collection
When the deletion completes
Then all graph edges connected to the vector's node are removed and the node is removed

#### Scenario: Batch relationship creation
Given multiple vectors are inserted in a batch operation
When batch insertion completes
Then relationships between all inserted vectors and existing vectors are created in batch for efficiency

### Requirement: Graph Persistence
The system SHALL persist graph data to disk in JSON format, allowing graph relationships to survive server restarts and be loaded efficiently.

#### Scenario: Save graph to disk
Given a collection has graph enabled with nodes and edges
When the collection is saved to disk
Then the graph structure is saved to a JSON file named `{collection_name}_graph.json` in the data directory
And the file contains version, collection_name, nodes array, and edges array

#### Scenario: Load graph from disk
Given a collection has a persisted graph file on disk
When the collection is loaded from disk
Then the graph structure is loaded from the JSON file
And all nodes and edges are restored with their metadata
And adjacency lists are rebuilt correctly

#### Scenario: Handle missing graph file
Given a collection is loaded from disk
When no graph file exists for the collection
Then an empty graph is created (normal for new collections)
And no error is raised

#### Scenario: Handle corrupted graph file
Given a collection has a corrupted graph file on disk
When the collection is loaded from disk
Then a warning is logged
And an empty graph is created instead of failing
And the collection load continues successfully

#### Scenario: Atomic graph save operation
Given a graph save operation is in progress
When the save completes
Then the graph is written to a temporary file first
And then atomically renamed to the final file
And the graph file is never in a corrupted state during save

#### Scenario: Graph persistence integration
Given a collection with graph enabled is saved
When save_collection_to_file is called
Then the graph is automatically saved alongside collection data
And graph file is saved to the same data directory as collection files

