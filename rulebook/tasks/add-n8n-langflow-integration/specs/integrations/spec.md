# Integration Platforms Specification

This specification defines the integration requirements for n8n and Langflow platforms with Vectorizer.

---

## ADDED Requirements

### Requirement: n8n Node Package
The system SHALL provide an official n8n node package (`@vectorizer/n8n-nodes-vectorizer`) that enables workflow automation with Vectorizer operations.

#### Scenario: Collection Management via n8n
Given a user has configured Vectorizer credentials in n8n
When they use the Vectorizer node with "Create Collection" operation
Then a new collection MUST be created in Vectorizer with the specified configuration

#### Scenario: Vector Search via n8n
Given a user has a workflow with search query input
When they execute the Vectorizer node with "Search" operation
Then the system MUST return search results matching the query vector

#### Scenario: Batch Operations via n8n
Given a user has multiple vectors to insert
When they use the "Batch Insert" operation
Then all vectors MUST be inserted in a single API call for efficiency

---

### Requirement: n8n Credentials Type
The system SHALL provide a VectorizerApi credentials type for secure connection configuration.

#### Scenario: Credential Configuration
Given a user accesses n8n credentials settings
When they add new Vectorizer credentials
Then the system MUST require host URL and optional API key fields

#### Scenario: Credential Validation
Given a user saves Vectorizer credentials
When the credentials are tested
Then the system MUST validate connectivity before saving

---

### Requirement: Langflow Vector Store Component
The system SHALL provide a VectorizerVectorStore component compatible with LangChain's vector store interface.

#### Scenario: Document Ingestion via Langflow
Given a user connects a Document Loader to VectorizerVectorStore
When documents are processed
Then the system MUST embed and store documents in Vectorizer collection

#### Scenario: Similarity Search via Langflow
Given a user connects VectorizerVectorStore to a Retriever
When a query is processed
Then the system MUST return similar documents based on vector similarity

---

### Requirement: Langflow Retriever Component
The system SHALL provide a VectorizerRetriever component for RAG pipeline integration.

#### Scenario: RAG Pipeline Integration
Given a user builds a RAG pipeline in Langflow
When they use VectorizerRetriever as the retrieval component
Then the system MUST provide documents to the LLM context

#### Scenario: Configurable Search Parameters
Given a user configures the retriever component
When they set top_k, threshold, and filter parameters
Then the search MUST respect these configuration options

---

### Requirement: Langflow Loader Component
The system SHALL provide a VectorizerLoader component for loading existing vectors.

#### Scenario: Load Existing Collection
Given a user needs to access an existing Vectorizer collection
When they use VectorizerLoader component
Then the system MUST load documents from the specified collection

---

### Requirement: Error Handling Standards
The system SHALL provide consistent error handling across all integration components.

#### Scenario: Connection Error Handling
Given a Vectorizer server is unavailable
When an operation is attempted
Then the system MUST return a clear error message with retry guidance

#### Scenario: Validation Error Handling
Given a user provides invalid parameters
When the operation is executed
Then the system MUST return descriptive validation errors

---

### Requirement: Documentation and Examples
The system SHALL provide comprehensive documentation for both integrations.

#### Scenario: n8n Workflow Examples
Given a user reads the n8n integration documentation
When they look for examples
Then the documentation MUST include at least 3 complete workflow examples

#### Scenario: Langflow Flow Templates
Given a user reads the Langflow integration documentation
When they look for templates
Then the documentation MUST include RAG pipeline templates

---

## Technical Notes

### n8n Package Structure
```
sdks/n8n/
├── package.json
├── tsconfig.json
├── credentials/
│   └── VectorizerApi.credentials.ts
├── nodes/
│   └── Vectorizer/
│       ├── Vectorizer.node.ts
│       ├── VectorizerDescription.ts
│       └── operations/
│           ├── collection.ts
│           ├── vector.ts
│           └── search.ts
└── README.md
```

### Langflow Package Structure
```
sdks/langflow/
├── pyproject.toml
├── vectorizer_langflow/
│   ├── __init__.py
│   ├── vectorstore.py
│   ├── retriever.py
│   ├── loader.py
│   └── utils.py
└── README.md
```

### Supported Operations

| Operation | n8n Node | Langflow Component |
|-----------|----------|-------------------|
| Create Collection | ✅ | N/A |
| Delete Collection | ✅ | N/A |
| List Collections | ✅ | N/A |
| Insert Vectors | ✅ | VectorizerVectorStore |
| Search | ✅ | VectorizerRetriever |
| Hybrid Search | ✅ | VectorizerRetriever |
| Batch Insert | ✅ | VectorizerVectorStore |
| Load Documents | N/A | VectorizerLoader |

