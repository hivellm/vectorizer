# GraphQL API

Vectorizer provides a GraphQL API alongside its REST API for flexible data querying and manipulation.

## Endpoints

- **GraphQL Endpoint**: `POST /graphql`
- **GraphiQL Playground**: `GET /graphiql`

## Authentication

The GraphQL API uses the same JWT authentication as the REST API. Include the token in the `Authorization` header:

```
Authorization: Bearer <your-jwt-token>
```

## Schema Overview

### Types

#### Collection
```graphql
type Collection {
  name: String!
  tenantId: String
  createdAt: DateTime!
  updatedAt: DateTime!
  vectorCount: Int!
  documentCount: Int!
  config: CollectionConfig!
}
```

#### Vector
```graphql
type Vector {
  id: String!
  data: [Float!]!
  dimension: Int!
  payload: JSON
}
```

#### SearchResult
```graphql
type SearchResult {
  id: String!
  score: Float!
  vector: [Float!]
  payload: JSON
}
```

#### Graph Types
```graphql
type Node {
  id: String!
  nodeType: String!
  metadata: JSON!
  createdAt: DateTime!
}

type Edge {
  id: String!
  source: String!
  target: String!
  relationshipType: RelationshipType!
  weight: Float!
  metadata: JSON!
  createdAt: DateTime!
}

type RelatedNode {
  node: Node!
  hops: Int!
  weight: Float!
}

type GraphStats {
  nodeCount: Int!
  edgeCount: Int!
  enabled: Boolean!
}

enum RelationshipType {
  SIMILAR_TO
  REFERENCES
  CONTAINS
  DERIVED_FROM
}
```

#### File Upload Types
```graphql
input UploadFileInput {
  collectionName: String!
  filename: String!
  content: String!
  chunkSize: Int
  chunkOverlap: Int
  metadata: String
}

type FileUploadResult {
  success: Boolean!
  filename: String!
  collectionName: String!
  chunksCreated: Int!
  vectorsCreated: Int!
  fileSize: Int!
  language: String!
  processingTimeMs: Int!
}

type FileUploadConfig {
  maxFileSize: Int!
  maxFileSizeMb: Float!
  allowedExtensions: [String!]!
  rejectBinary: Boolean!
  defaultChunkSize: Int!
  defaultChunkOverlap: Int!
}
```

## Queries

### Collections

#### List all collections
```graphql
query {
  collections {
    name
    vectorCount
    config {
      dimension
      metric
      graphEnabled
    }
  }
}
```

#### Get a specific collection
```graphql
query {
  collection(name: "my-collection") {
    name
    vectorCount
    documentCount
    createdAt
  }
}
```

### Vectors

#### Get a vector by ID
```graphql
query {
  vector(collection: "my-collection", id: "vec-123") {
    id
    data
    payload
  }
}
```

#### List vectors with pagination
```graphql
query {
  vectors(input: {
    collection: "my-collection"
    limit: 10
    cursor: "0"
  }) {
    items {
      id
      dimension
      payload
    }
    totalCount
    hasNextPage
    nextCursor
  }
}
```

### Search

#### Semantic search
```graphql
query {
  search(input: {
    collection: "my-collection"
    vector: [0.1, 0.2, 0.3, ...]
    limit: 10
    scoreThreshold: 0.7
  }) {
    id
    score
    payload
  }
}
```

### Server Statistics

```graphql
query {
  stats {
    version
    collectionCount
    totalVectors
    uptimeSeconds
    memoryUsageBytes
  }
}
```

### Graph Queries

#### Get graph statistics
```graphql
query {
  graphStats(collection: "my-collection") {
    nodeCount
    edgeCount
    enabled
  }
}
```

#### List graph nodes
```graphql
query {
  graphNodes(collection: "my-collection", limit: 20) {
    items {
      id
      nodeType
      metadata
      createdAt
    }
    totalCount
    hasNextPage
  }
}
```

#### List graph edges
```graphql
query {
  graphEdges(collection: "my-collection", limit: 20) {
    items {
      id
      source
      target
      relationshipType
      weight
    }
    totalCount
  }
}
```

#### Get a specific node
```graphql
query {
  graphNode(collection: "my-collection", nodeId: "node-123") {
    id
    nodeType
    metadata
  }
}
```

#### Get neighbors of a node
```graphql
query {
  graphNeighbors(
    collection: "my-collection"
    nodeId: "node-123"
    relationshipType: SIMILAR_TO
  ) {
    node {
      id
      nodeType
    }
    hops
    weight
  }
}
```

#### Find related nodes (within N hops)
```graphql
query {
  graphRelated(
    collection: "my-collection"
    nodeId: "node-123"
    maxHops: 3
  ) {
    node {
      id
      nodeType
    }
    hops
    weight
  }
}
```

#### Find shortest path between nodes
```graphql
query {
  graphPath(
    collection: "my-collection"
    source: "node-123"
    target: "node-456"
  ) {
    id
    nodeType
  }
}
```

## Mutations

### Collection Mutations

#### Create a collection
```graphql
mutation {
  createCollection(input: {
    name: "my-collection"
    dimension: 384
    metric: COSINE
    hnswM: 16
    hnswEfConstruction: 200
    enableGraph: true
  }) {
    name
    config {
      dimension
      graphEnabled
    }
  }
}
```

#### Delete a collection
```graphql
mutation {
  deleteCollection(name: "my-collection") {
    success
    message
  }
}
```

### Vector Mutations

#### Upsert a single vector
```graphql
mutation {
  upsertVector(
    collection: "my-collection"
    input: {
      id: "vec-123"
      data: [0.1, 0.2, 0.3, ...]
      payload: { "title": "My Document" }
    }
  ) {
    id
    dimension
  }
}
```

#### Upsert multiple vectors
```graphql
mutation {
  upsertVectors(input: {
    collection: "my-collection"
    vectors: [
      { id: "vec-1", data: [0.1, 0.2, ...] },
      { id: "vec-2", data: [0.3, 0.4, ...] }
    ]
  }) {
    success
    affectedCount
  }
}
```

#### Delete a vector
```graphql
mutation {
  deleteVector(collection: "my-collection", id: "vec-123") {
    success
    message
  }
}
```

#### Update vector payload
```graphql
mutation {
  updatePayload(
    collection: "my-collection"
    id: "vec-123"
    payload: { "title": "Updated Title", "category": "docs" }
  ) {
    success
    message
  }
}
```

### Graph Mutations

#### Enable graph for a collection
```graphql
mutation {
  enableGraph(collection: "my-collection") {
    success
    message
  }
}
```

#### Add a node to the graph
```graphql
mutation {
  addGraphNode(
    collection: "my-collection"
    nodeId: "node-123"
    nodeType: "document"
    metadata: { "title": "My Document" }
  ) {
    id
    nodeType
    createdAt
  }
}
```

#### Remove a node from the graph
```graphql
mutation {
  removeGraphNode(collection: "my-collection", nodeId: "node-123") {
    success
    message
  }
}
```

#### Create an edge between nodes
```graphql
mutation {
  createGraphEdge(
    collection: "my-collection"
    input: {
      source: "node-123"
      target: "node-456"
      relationshipType: SIMILAR_TO
      weight: 0.85
    }
  ) {
    id
    source
    target
    weight
  }
}
```

#### Delete an edge
```graphql
mutation {
  deleteGraphEdge(
    collection: "my-collection"
    edgeId: "node-123:node-456:0"
  ) {
    success
    message
  }
}
```

### File Upload Mutations

#### Upload a file for indexing
```graphql
mutation {
  uploadFile(input: {
    collectionName: "my-collection"
    filename: "example.rs"
    content: "Zm4gbWFpbigpIHsKICAgIHByaW50bG4hKCJIZWxsbyB3b3JsZCIpOwp9"
    chunkSize: 1024
    chunkOverlap: 128
    metadata: "{\"author\": \"john\"}"
  }) {
    success
    filename
    collectionName
    chunksCreated
    vectorsCreated
    fileSize
    language
    processingTimeMs
  }
}
```

**Input Fields:**
- `collectionName` (required): Target collection name
- `filename` (required): Name of the file (used for language detection)
- `content` (required): Base64-encoded file content
- `chunkSize` (optional): Chunk size in characters (default: 2048)
- `chunkOverlap` (optional): Chunk overlap in characters (default: 256)
- `metadata` (optional): JSON string with additional metadata

**Response Fields:**
- `success`: Whether the upload was successful
- `filename`: Original filename
- `collectionName`: Target collection
- `chunksCreated`: Number of text chunks created
- `vectorsCreated`: Number of vectors stored
- `fileSize`: File size in bytes
- `language`: Detected programming language or file type
- `processingTimeMs`: Processing time in milliseconds

#### Get file upload configuration
```graphql
query {
  fileUploadConfig {
    maxFileSize
    maxFileSizeMb
    allowedExtensions
    rejectBinary
    defaultChunkSize
    defaultChunkOverlap
  }
}
```

## Error Handling

GraphQL errors are returned in the standard GraphQL error format:

```json
{
  "data": null,
  "errors": [
    {
      "message": "Collection not found: my-collection",
      "locations": [{ "line": 2, "column": 3 }],
      "path": ["collection"]
    }
  ]
}
```

## GraphiQL Playground

Access the interactive GraphQL playground at `/graphiql` to explore the schema and test queries. The playground includes:

- Schema documentation
- Query autocompletion
- Query history
- Variable editor
- Response viewer

## Performance Tips

1. **Request only needed fields**: GraphQL allows you to specify exactly which fields you need, reducing response size.

2. **Use pagination**: For large collections, use cursor-based pagination with `limit` and `cursor` parameters.

3. **Batch operations**: Use `upsertVectors` for bulk inserts instead of multiple `upsertVector` calls.

4. **Avoid N+1 queries**: When querying nested data, be mindful of the number of database calls.

## Example: Complete Workflow

```graphql
# 1. Create a collection with graph enabled
mutation {
  createCollection(input: {
    name: "documents"
    dimension: 384
    enableGraph: true
  }) {
    name
    config { graphEnabled }
  }
}

# 2. Insert vectors
mutation {
  upsertVectors(input: {
    collection: "documents"
    vectors: [
      { id: "doc-1", data: [...], payload: { "title": "Doc 1" } },
      { id: "doc-2", data: [...], payload: { "title": "Doc 2" } }
    ]
  }) {
    success
    affectedCount
  }
}

# 3. Search for similar documents
query {
  search(input: {
    collection: "documents"
    vector: [...]
    limit: 5
  }) {
    id
    score
    payload
  }
}

# 4. Explore graph relationships
query {
  graphRelated(
    collection: "documents"
    nodeId: "doc-1"
    maxHops: 2
  ) {
    node { id }
    hops
    weight
  }
}
```
