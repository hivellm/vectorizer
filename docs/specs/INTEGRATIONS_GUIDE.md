# Integrations Guide

**Version**: 1.0  
**Status**: ✅ Active  
**Last Updated**: 2025-10-01

---

## Overview

Vectorizer integrates with multiple frameworks, libraries, and tools for seamless adoption in existing systems.

---

## Framework Integrations

### LangChain

**Python**:
```python
from vectorizer_store import VectorizerStore

store = VectorizerStore(
    host="localhost",
    port=15002,
    collection="documents",
    api_key="your-key"
)

# Use with LangChain
from langchain.chains import RetrievalQA

qa = RetrievalQA.from_chain_type(
    llm=llm,
    retriever=store.as_retriever()
)
```

**TypeScript**:
```typescript
import { VectorizerStore } from '@hivellm/vectorizer-langchain';

const store = new VectorizerStore({
  host: 'localhost',
  port: 15002,
  collection: 'documents',
  apiKey: 'your-key'
});
```

### PyTorch

```python
from vectorizer.pytorch import PyTorchEmbedder

embedder = PyTorchEmbedder(
    model_name="sentence-transformers/all-MiniLM-L6-v2",
    device="cuda"  # or "cpu"
)

embeddings = embedder.embed_batch(texts)
```

### TensorFlow

```python
from vectorizer.tensorflow import TensorFlowEmbedder

embedder = TensorFlowEmbedder(
    model_name="universal-sentence-encoder"
)

embeddings = embedder.embed(texts)
```

---

## Rust Libraries

### Core Dependencies

**Vector Operations**:
- `ndarray`: N-dimensional arrays
- `nalgebra`: Linear algebra
- `rayon`: Data parallelism

**Serialization**:
- `serde`: Serialization framework
- `serde_json`: JSON support
- `bincode`: Binary serialization

**Async Runtime**:
- `tokio`: Async runtime
- `async-trait`: Async traits
- `futures`: Future utilities

**Web Frameworks**:
- `actix-web`: Web server
- `tonic`: gRPC server
- `tower`: Service abstraction

**Storage**:
- `rocksdb`: Persistent key-value
- `sled`: Embedded database
- `memmap2`: Memory-mapped files

---

## External Tools

### Docker

```bash
docker pull hivellm/vectorizer:latest
docker run -p 15002:15002 hivellm/vectorizer
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vectorizer
spec:
  replicas: 3
  selector:
    matchLabels:
      app: vectorizer
  template:
    spec:
      containers:
      - name: vectorizer
        image: hivellm/vectorizer:latest
        ports:
        - containerPort: 15002
```

---

## IDE Integrations

**VS Code**: MCP extension available  
**Cursor**: Built-in MCP support  
**IntelliJ**: Via REST API client

---

**Maintained by**: HiveLLM Team

