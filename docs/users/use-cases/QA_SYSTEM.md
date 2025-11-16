---
title: Q&A System with RAG
module: use-cases
id: qa-system
order: 3
description: Build a Q&A system using RAG with Vectorizer
tags: [use-cases, qa, rag, retrieval-augmented-generation, examples]
---

# Q&A System with RAG

Build a Question-Answering system using Retrieval-Augmented Generation (RAG) with Vectorizer.

## Overview

This use case demonstrates how to build a RAG-based Q&A system that:
- Stores knowledge base documents
- Retrieves relevant context for questions
- Provides accurate answers using LLMs
- Handles multiple knowledge domains

## Architecture

```
Knowledge Base → Chunking → Embedding → Vectorizer → Retrieval → LLM → Answer
```

## Implementation

### Step 1: Create Knowledge Base Collection

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

# Create collection for knowledge base chunks
await client.create_collection(
    "knowledge_base",
    dimension=384,  # or 768 for higher quality
    metric="cosine",
    quantization={"enabled": True, "type": "scalar", "bits": 8},
    hnsw_config={"m": 16, "ef_search": 64}
)
```

### Step 2: Chunk and Index Documents

```python
def chunk_text(text, chunk_size=500, overlap=50):
    """Split text into overlapping chunks."""
    words = text.split()
    chunks = []
    
    for i in range(0, len(words), chunk_size - overlap):
        chunk = " ".join(words[i:i + chunk_size])
        chunks.append(chunk)
    
    return chunks

async def index_knowledge_base(documents):
    """Index knowledge base documents."""
    all_chunks = []
    all_metadatas = []
    
    for doc_id, doc in documents.items():
        chunks = chunk_text(doc["content"], chunk_size=500)
        
        for i, chunk in enumerate(chunks):
            all_chunks.append(chunk)
            all_metadatas.append({
                "doc_id": doc_id,
                "chunk_id": f"{doc_id}_chunk_{i}",
                "title": doc["title"],
                "source": doc.get("source", "unknown"),
                "chunk_index": i
            })
    
    # Batch insert for efficiency
    await client.batch_insert_text("knowledge_base", all_chunks, all_metadatas)
    print(f"Indexed {len(all_chunks)} chunks from {len(documents)} documents")

# Example usage
documents = {
    "doc1": {
        "title": "Vectorizer Documentation",
        "content": "Vectorizer is a high-performance vector database...",
        "source": "docs"
    },
    "doc2": {
        "title": "Python Guide",
        "content": "Python is a programming language...",
        "source": "tutorials"
    }
}

await index_knowledge_base(documents)
```

### Step 3: Retrieve Relevant Context

```python
async def retrieve_context(question, top_k=5, min_score=0.7):
    """Retrieve relevant context for a question."""
    results = await client.search(
        "knowledge_base",
        question,
        limit=top_k,
        similarity_threshold=min_score,
        with_payload=True
    )
    
    # Combine chunks from same document
    context_by_doc = {}
    for r in results:
        doc_id = r["payload"]["doc_id"]
        if doc_id not in context_by_doc:
            context_by_doc[doc_id] = {
                "title": r["payload"]["title"],
                "chunks": []
            }
        context_by_doc[doc_id]["chunks"].append({
            "text": r.get("text", ""),
            "score": r["score"],
            "chunk_id": r["payload"]["chunk_id"]
        })
    
    return context_by_doc

# Retrieve context for a question
context = await retrieve_context("What is Vectorizer?", top_k=5)
```

### Step 4: Generate Answer with LLM

```python
async def answer_question(question, model="gpt-4"):
    """Answer a question using RAG."""
    # Retrieve relevant context
    context = await retrieve_context(question, top_k=5)
    
    # Build context string
    context_text = ""
    for doc_id, doc_info in context.items():
        context_text += f"\n\n## {doc_info['title']}\n"
        for chunk in doc_info["chunks"]:
            context_text += f"{chunk['text']}\n"
    
    # Build prompt
    prompt = f"""Answer the following question using only the provided context.
If the answer is not in the context, say "I don't have enough information."

Context:
{context_text}

Question: {question}

Answer:"""
    
    # Call LLM (example with OpenAI API)
    # import openai
    # response = await openai.ChatCompletion.acreate(
    #     model=model,
    #     messages=[{"role": "user", "content": prompt}]
    # )
    # return response.choices[0].message.content
    
    return f"Answer based on: {context_text[:200]}..."

# Example usage
answer = await answer_question("What is Vectorizer?")
print(answer)
```

### Step 5: Advanced RAG with Intelligent Search

```python
async def intelligent_rag_answer(question):
    """Use intelligent search for better context retrieval."""
    # Use intelligent search for better query understanding
    results = await client.intelligent_search(
        "knowledge_base",
        question,
        max_results=10,
        mmr_enabled=True,  # Diversify results
        mmr_lambda=0.7,
        domain_expansion=True,  # Expand query with domain knowledge
        technical_focus=True
    )
    
    # Build context from results
    context = "\n\n".join([
        f"Source: {r['payload']['title']}\n{r.get('text', '')}"
        for r in results
    ])
    
    # Generate answer
    prompt = f"""Answer the question using the provided context.

Context:
{context}

Question: {question}

Answer:"""
    
    # Call LLM...
    return "Answer based on intelligent search results..."
```

## Real-World Example

```python
import asyncio
from vectorizer_sdk import VectorizerClient

async def main():
    client = VectorizerClient("http://localhost:15002")
    
    # Create knowledge base
    await client.create_collection("kb", dimension=384)
    
    # Index documents
    docs = {
        "vec_docs": {
            "title": "Vectorizer Guide",
            "content": "Vectorizer is a high-performance vector database written in Rust..."
        }
    }
    await index_knowledge_base(docs)
    
    # Answer questions
    answer = await answer_question("What is Vectorizer?")
    print(f"Answer: {answer}")

asyncio.run(main())
```

## Best Practices

1. **Chunk size**: 200-500 tokens for optimal retrieval
2. **Overlap**: 10-20% overlap between chunks preserves context
3. **Top-K selection**: 3-10 chunks usually sufficient
4. **Score threshold**: 0.7+ for high-quality context
5. **Metadata**: Store source, title, and chunk index for traceability

## Related Topics

- [Intelligent Search](../search/ADVANCED.md) - Advanced search for RAG
- [Collections Guide](../collections/COLLECTIONS.md) - Collection setup
- [Performance Guide](../performance/PERFORMANCE.md) - Optimization

