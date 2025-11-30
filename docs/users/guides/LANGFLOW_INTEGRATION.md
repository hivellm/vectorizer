---
title: Langflow Integration
module: guides
id: langflow-integration
order: 21
description: Visual LLM app building with Langflow
tags: [langflow, integration, langchain, llm, rag, visual]
---

# Langflow Integration

Vectorizer provides **LangChain-compatible components** for visual LLM workflow building with Langflow.

## Overview

The `vectorizer-langflow` package provides:

- **VectorizerVectorStore** - Full LangChain VectorStore implementation
- **VectorizerRetriever** - Retriever component for RAG pipelines
- **VectorizerLoader** - Loader for existing vectors

## Installation

```bash
pip install vectorizer-langflow
```

**Requirements:**
- Python 3.9+
- LangChain >= 0.1.0

## Components

### VectorizerVectorStore

A complete LangChain VectorStore implementation:

```python
from vectorizer_langflow import VectorizerVectorStore

# Initialize
vectorstore = VectorizerVectorStore(
    base_url="http://localhost:15002",
    collection_name="documents",
    api_key="your-api-key",  # Optional
    distance_metric="cosine"  # cosine, euclidean, dot
)
```

#### Methods

| Method | Description |
|--------|-------------|
| `add_texts()` | Add texts with automatic embedding |
| `add_documents()` | Add LangChain Document objects |
| `similarity_search()` | Search by query text |
| `similarity_search_with_score()` | Search with similarity scores |
| `similarity_search_by_vector()` | Search by vector |
| `from_texts()` | Create store from texts |
| `from_documents()` | Create store from documents |

### VectorizerRetriever

A LangChain Retriever for RAG pipelines:

```python
from vectorizer_langflow import VectorizerRetriever

retriever = VectorizerRetriever(
    base_url="http://localhost:15002",
    collection_name="documents",
    search_type="similarity",  # similarity, mmr, hybrid
    k=5
)
```

### VectorizerLoader

Load existing vectors as LangChain Documents:

```python
from vectorizer_langflow import VectorizerLoader

loader = VectorizerLoader(
    base_url="http://localhost:15002",
    collection_name="documents"
)

documents = loader.load()
```

## Langflow Usage

### Add Components

1. Open Langflow
2. Search for "Vectorizer" in components
3. Drag components to canvas

### VectorStore Component

Configure in Langflow:

| Parameter | Value |
|-----------|-------|
| Base URL | `http://localhost:15002` |
| Collection Name | Your collection |
| API Key | Optional |
| Distance Metric | cosine / euclidean / dot |

### Retriever Component

Configure in Langflow:

| Parameter | Value |
|-----------|-------|
| Base URL | `http://localhost:15002` |
| Collection Name | Your collection |
| Search Type | similarity / mmr / hybrid |
| K | Number of results |

## Example Workflows

### RAG Pipeline

Build a Retrieval-Augmented Generation pipeline:

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│    Input     │───▶│  Vectorizer  │───▶│    LLM       │───▶│   Output     │
│    Text      │    │  Retriever   │    │   (OpenAI)   │    │    Text      │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
```

**Python equivalent:**

```python
from langchain.chains import RetrievalQA
from langchain.chat_models import ChatOpenAI
from vectorizer_langflow import VectorizerRetriever

retriever = VectorizerRetriever(
    base_url="http://localhost:15002",
    collection_name="knowledge_base",
    k=5
)

llm = ChatOpenAI(model="gpt-4")

qa_chain = RetrievalQA.from_chain_type(
    llm=llm,
    retriever=retriever,
    return_source_documents=True
)

result = qa_chain.invoke("What is Vectorizer?")
```

### Document Ingestion

Index documents from files:

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│    File      │───▶│    Text      │───▶│  Vectorizer  │───▶│   Success    │
│   Loader     │    │   Splitter   │    │ VectorStore  │    │   Message    │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
```

**Python equivalent:**

```python
from langchain.document_loaders import PyPDFLoader
from langchain.text_splitter import RecursiveCharacterTextSplitter
from vectorizer_langflow import VectorizerVectorStore

# Load PDF
loader = PyPDFLoader("document.pdf")
documents = loader.load()

# Split into chunks
splitter = RecursiveCharacterTextSplitter(
    chunk_size=1000,
    chunk_overlap=200
)
chunks = splitter.split_documents(documents)

# Store in Vectorizer
vectorstore = VectorizerVectorStore.from_documents(
    documents=chunks,
    base_url="http://localhost:15002",
    collection_name="pdf_documents"
)
```

### Conversational RAG

Chat with memory and context:

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│    Chat      │───▶│  Vectorizer  │───▶│   Memory     │
│   Input      │    │  Retriever   │    │   Buffer     │
└──────────────┘    └──────────────┘    └──────────────┘
        │                                      │
        └──────────────────┬───────────────────┘
                           ▼
                   ┌──────────────┐    ┌──────────────┐
                   │     LLM      │───▶│    Chat      │
                   │   (GPT-4)    │    │   Output     │
                   └──────────────┘    └──────────────┘
```

## Search Types

### Similarity Search

Standard vector similarity search:

```python
results = vectorstore.similarity_search(
    query="What is machine learning?",
    k=5
)
```

### MMR Search

Maximal Marginal Relevance for diverse results:

```python
results = vectorstore.max_marginal_relevance_search(
    query="What is machine learning?",
    k=5,
    fetch_k=20,
    lambda_mult=0.5
)
```

### Hybrid Search

Combine vector and keyword search:

```python
retriever = VectorizerRetriever(
    base_url="http://localhost:15002",
    collection_name="documents",
    search_type="hybrid",
    search_kwargs={
        "alpha": 0.7,  # Vector weight
        "k": 5
    }
)
```

## Filtering

Apply metadata filters to search:

```python
results = vectorstore.similarity_search(
    query="machine learning",
    k=5,
    filter={"category": "technology", "year": {"$gte": 2020}}
)
```

## Embedding Configuration

### Default (BM25)

Uses Vectorizer's built-in BM25 embedding:

```python
vectorstore = VectorizerVectorStore(
    base_url="http://localhost:15002",
    collection_name="documents"
)
```

### Custom Embeddings

Use LangChain embeddings:

```python
from langchain.embeddings import OpenAIEmbeddings

embeddings = OpenAIEmbeddings()

vectorstore = VectorizerVectorStore(
    base_url="http://localhost:15002",
    collection_name="documents",
    embedding=embeddings
)
```

## Best Practices

### Collection Setup

Create collections with appropriate dimensions:

```python
# For OpenAI embeddings (1536 dimensions)
vectorstore = VectorizerVectorStore(
    base_url="http://localhost:15002",
    collection_name="openai_docs",
    dimension=1536,
    distance_metric="cosine"
)
```

### Chunking Strategy

Choose appropriate chunk sizes:

| Content Type | Chunk Size | Overlap |
|--------------|------------|---------|
| Documentation | 1000 | 200 |
| Code | 500 | 100 |
| Conversations | 300 | 50 |
| Legal | 2000 | 400 |

### Error Handling

```python
from vectorizer_langflow import VectorizerVectorStore

try:
    results = vectorstore.similarity_search("query")
except ConnectionError:
    # Handle connection issues
    pass
except ValueError as e:
    # Handle invalid parameters
    pass
```

## Troubleshooting

### Connection Refused

- Verify Vectorizer is running
- Check host and port
- Ensure firewall allows connection

### Empty Results

- Verify collection exists
- Check documents were indexed
- Adjust search parameters

### Dimension Mismatch

- Ensure collection dimension matches embedding size
- OpenAI: 1536, MiniLM: 384, etc.

## Related Topics

- [n8n Integration](./N8N_INTEGRATION.md) - No-code workflow automation
- [Python SDK](../sdks/PYTHON.md) - Direct Python API
- [Search Guide](../search/SEARCH.md) - Search operations

