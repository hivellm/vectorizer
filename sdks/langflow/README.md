# vectorizer-langflow

Langflow components for integrating [Vectorizer](https://github.com/hivellm/vectorizer) vector database into your LangChain and Langflow workflows.

## Installation

```bash
pip install vectorizer-langflow
```

## Components

### VectorizerVectorStore

LangChain-compatible vector store for Vectorizer.

```python
from vectorizer_langflow import VectorizerVectorStore
from langchain.embeddings import OpenAIEmbeddings

vectorstore = VectorizerVectorStore(
    host="http://localhost:15002",
    collection_name="my-documents",
    embedding=OpenAIEmbeddings(),
)

# Add documents
vectorstore.add_texts(
    texts=["Document 1", "Document 2"],
    metadatas=[{"source": "doc1"}, {"source": "doc2"}]
)

# Similarity search
results = vectorstore.similarity_search("query text", k=5)
```

### VectorizerRetriever

Retriever component for RAG pipelines.

```python
from vectorizer_langflow import VectorizerRetriever

retriever = VectorizerRetriever(
    host="http://localhost:15002",
    collection_name="knowledge-base",
    search_kwargs={"k": 3, "score_threshold": 0.7}
)

# Use in RAG chain
from langchain.chains import RetrievalQA
from langchain.llms import OpenAI

qa_chain = RetrievalQA.from_chain_type(
    llm=OpenAI(),
    retriever=retriever
)

answer = qa_chain.run("What is Vectorizer?")
```

### VectorizerLoader

Load existing vectors from Vectorizer collection.

```python
from vectorizer_langflow import VectorizerLoader

loader = VectorizerLoader(
    host="http://localhost:15002",
    collection_name="existing-collection"
)

documents = loader.load()
```

## Langflow Integration

### Adding Custom Components

1. Set the `LANGFLOW_COMPONENTS_PATH` environment variable:
```bash
export LANGFLOW_COMPONENTS_PATH=/path/to/vectorizer_langflow
```

2. Components will appear in Langflow's sidebar under "Vectorizer" category

### Example Flow: RAG Pipeline

```
Document Loader → Text Splitter → VectorizerVectorStore
                                         ↓
User Query → VectorizerRetriever → LLM → Response
```

## Configuration

### Environment Variables

- `VECTORIZER_HOST`: Default host URL (default: `http://localhost:15002`)
- `VECTORIZER_API_KEY`: API key for authentication (optional)

### Component Parameters

**VectorizerVectorStore:**
- `host`: Vectorizer instance URL
- `collection_name`: Collection to use
- `embedding`: LangChain embedding model
- `dimension`: Vector dimension (auto-detected from embedding)
- `metric`: Distance metric (`cosine`, `euclidean`, `dot`)

**VectorizerRetriever:**
- `host`: Vectorizer instance URL
- `collection_name`: Collection to search
- `embedding`: LangChain embedding model
- `search_kwargs`: Search parameters
  - `k`: Number of results
  - `score_threshold`: Minimum similarity score
  - `filter`: Metadata filter

**VectorizerLoader:**
- `host`: Vectorizer instance URL
- `collection_name`: Collection to load from
- `limit`: Maximum documents to load
- `offset`: Pagination offset

## Examples

### Example 1: Document Ingestion

```python
from langchain.document_loaders import TextLoader
from langchain.text_splitter import RecursiveCharacterTextSplitter
from langchain.embeddings import OpenAIEmbeddings
from vectorizer_langflow import VectorizerVectorStore

# Load documents
loader = TextLoader("documents.txt")
documents = loader.load()

# Split into chunks
text_splitter = RecursiveCharacterTextSplitter(chunk_size=1000)
chunks = text_splitter.split_documents(documents)

# Create vector store and add documents
vectorstore = VectorizerVectorStore(
    host="http://localhost:15002",
    collection_name="docs",
    embedding=OpenAIEmbeddings()
)

vectorstore.add_documents(chunks)
```

### Example 2: Hybrid Search

```python
from vectorizer_langflow import VectorizerVectorStore

vectorstore = VectorizerVectorStore(
    host="http://localhost:15002",
    collection_name="products",
)

# Hybrid search combines vector and keyword matching
results = vectorstore.similarity_search(
    "laptop computer",
    k=10,
    search_type="hybrid"
)
```

### Example 3: Filtered Search

```python
results = vectorstore.similarity_search(
    "machine learning",
    k=5,
    filter={"category": "AI", "year": 2023}
)
```

## API Reference

See the [Vectorizer documentation](https://github.com/hivellm/vectorizer) for complete API reference.

## Requirements

- Python 3.9+
- LangChain >= 0.1.0
- Vectorizer >= 1.6.0

## License

Apache-2.0

## Links

- [Vectorizer GitHub](https://github.com/hivellm/vectorizer)
- [Langflow Documentation](https://docs.langflow.org/)
- [Report Issues](https://github.com/hivellm/vectorizer/issues)
