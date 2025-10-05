# Vectorizer Integrations

## Integrations Overview

Vectorizer provides native integrations with popular AI frameworks and development tools, enabling seamless use in existing ML pipelines and assisted development workflows.

## LangChain Integration

### Overview

LangChain is a framework for developing applications with language models. The integration with Vectorizer allows using the vector database as a native VectorStore, enabling RAG (Retrieval-Augmented Generation) and other semantic search applications.

### Integration Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   LangChain     │    │  Vectorizer      │    │   Embedding     │
│   Application   │───▶│  LangChain       │───▶│   Function      │
│                 │    │  VectorStore     │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   Vectorizer    │
                       │   Core Engine   │
                       │   (Rust)        │
                       └─────────────────┘
```

### Python LangChain Integration

#### Installation

```bash
pip install langchain-vectorizer
# or
pip install vectorizer[langchain]
```

#### VectorStore Class

```python
from langchain.vectorstores import VectorizerStore
from langchain.embeddings import OpenAIEmbeddings
from vectorizer import VectorizerDB

# Initialize Vectorizer
db = VectorizerDB(persist_path="data/vectorizer.bin")

# Create embedding function
embeddings = OpenAIEmbeddings(
    model="text-embedding-ada-002",
    openai_api_key="your-api-key"
)

# Ou usar Sentence Transformers
from langchain.embeddings import HuggingFaceEmbeddings
embeddings = HuggingFaceEmbeddings(
    model_name="sentence-transformers/all-MiniLM-L6-v2"
)

# Criar VectorStore
vectorstore = VectorizerStore(
    db=db,
    embedding_function=embeddings.embed_query,
    collection_name="documents"
)
```

#### Basic Usage

```python
# Add documents
from langchain.document_loaders import TextLoader
from langchain.text_splitter import RecursiveCharacterTextSplitter

# Load and split document
loader = TextLoader("document.txt")
documents = loader.load()

text_splitter = RecursiveCharacterTextSplitter(
    chunk_size=1000,
    chunk_overlap=200
)
docs = text_splitter.split_documents(documents)

# Add to vectorstore
vectorstore.add_documents(docs)
print(f"Added {len(docs)} chunks")

# Semantic search
query = "What is machine learning?"
results = vectorstore.similarity_search(query, k=5)

for doc in results:
    print(f"Score: {doc.metadata.get('score', 'N/A')}")
    print(f"Content: {doc.page_content[:200]}...")
    print("---")
```

#### RetrievalQA Chain

```python
from langchain.chains import RetrievalQA
from langchain.llms import OpenAI

# Configurar LLM
llm = OpenAI(
    model_name="gpt-3.5-turbo",
    temperature=0,
    openai_api_key="your-api-key"
)

# Criar chain RAG
qa_chain = RetrievalQA.from_chain_type(
    llm=llm,
    chain_type="stuff",
    retriever=vectorstore.as_retriever(search_kwargs={"k": 3}),
    return_source_documents=True
)

# Fazer pergunta
query = "How does vector search work?"
result = qa_chain({"query": query})

print("Answer:", result["result"])
print("\nSources:")
for doc in result["source_documents"]:
    print(f"- {doc.page_content[:100]}...")
```

#### ConversationalRetrievalChain

```python
from langchain.chains import ConversationalRetrievalChain
from langchain.memory import ConversationBufferMemory

# Configurar memória
memory = ConversationBufferMemory(
    memory_key="chat_history",
    return_messages=True
)

# Criar chain conversacional
conversational_chain = ConversationalRetrievalChain.from_llm(
    llm=llm,
    retriever=vectorstore.as_retriever(search_kwargs={"k": 4}),
    memory=memory,
    verbose=True
)

# Conversa com contexto
chat_history = []
while True:
    query = input("You: ")
    if query.lower() in ['quit', 'exit']:
        break

    result = conversational_chain({
        "question": query,
        "chat_history": chat_history
    })

    print(f"AI: {result['answer']}")
    chat_history.append((query, result['answer']))
```

#### Customização Avançada

```python
# VectorStore com configuração customizada
vectorstore = VectorizerStore(
    db=db,
    embedding_function=custom_embedding_function,
    collection_name="tech_docs",
    # Configurações específicas do Vectorizer
    vectorizer_config={
        "hnsw_m": 32,
        "hnsw_ef_construction": 400,
        "metric": "cosine"
    }
)

# Busca com filtros
results = vectorstore.similarity_search_with_score(
    query="neural networks",
    k=10,
    filter={
        "metadata.category": "machine_learning",
        "metadata.year": {"$gte": 2020}
    }
)
```

### TypeScript LangChain.js Integration

#### Instalação

```bash
npm install @langchain/vectorizer langchain
```

#### Uso Básico

```typescript
import { VectorizerStore } from '@langchain/vectorizer';
import { OpenAIEmbeddings } from '@langchain/openai';
import { VectorizerDB } from '@hivellm/vectorizer';

// Inicializar
const db = new VectorizerDB({ persistPath: 'data/vectorizer.bin' });

const embeddings = new OpenAIEmbeddings({
  modelName: 'text-embedding-ada-002',
  openAIApiKey: process.env.OPENAI_API_KEY,
});

const vectorstore = new VectorizerStore({
  db,
  embeddingFunction: embeddings.embedQuery.bind(embeddings),
  collectionName: 'documents',
});

// Adicionar documentos
import { TextLoader } from 'langchain/document_loaders/fs/text';
import { RecursiveCharacterTextSplitter } from 'langchain/text_splitter';

const loader = new TextLoader('document.txt');
const docs = await loader.load();

const textSplitter = new RecursiveCharacterTextSplitter({
  chunkSize: 1000,
  chunkOverlap: 200,
});

const splitDocs = await textSplitter.splitDocuments(docs);
await vectorstore.addDocuments(splitDocs);

// Busca
const results = await vectorstore.similaritySearch('machine learning', 5);
```

#### RetrievalQA Chain

```typescript
import { RetrievalQAChain } from 'langchain/chains';
import { ChatOpenAI } from '@langchain/openai';

const llm = new ChatOpenAI({
  modelName: 'gpt-3.5-turbo',
  temperature: 0,
  openAIApiKey: process.env.OPENAI_API_KEY,
});

const qaChain = RetrievalQAChain.fromLLM(
  llm,
  vectorstore.asRetriever({ k: 3 }),
  {
    returnSourceDocuments: true,
  }
);

const result = await qaChain.call({
  query: 'How does vector search work?',
});

console.log('Answer:', result.text);
console.log('Sources:', result.sourceDocuments);
```

## Aider Integration

### Visão Geral

Aider é uma ferramenta de programação assistida por IA que ajuda desenvolvedores a escrever código através de prompts em linguagem natural. A integração com Vectorizer permite busca semântica no codebase para fornecer contexto relevante durante a geração de código.

### Arquitetura

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Developer     │    │  Aider +         │    │   Vectorizer     │
│   Prompt        │───▶│  Vectorizer      │───▶│   Code Search    │
│                 │    │  Plugin          │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                       │
                                                       ▼
                                               ┌─────────────────┐
                                               │   Codebase       │
                                               │   Embeddings     │
                                               │                 │
                                               └─────────────────┘
```

### Python Aider Plugin

#### Instalação

```bash
pip install aider-vectorizer
# ou
pip install vectorizer[aider]
```

#### Configuração

```python
# aider.conf.yml
plugins:
  - vectorizer

vectorizer:
  persist_path: "./.aider/vectorizer.bin"
  collection: "codebase"
  embedding_model: "sentence-transformers/code-search-net"
  chunk_size: 512
  similarity_threshold: 0.7
  max_results: 5
```

#### Funcionalidades

##### Busca Automática de Contexto

```python
# Quando o desenvolvedor escreve um prompt como:
# "Add error handling to the database connection function"

# O plugin automaticamente:
# 1. Busca funções relacionadas a "database connection"
# 2. Encontra padrões de error handling existentes
# 3. Fornece contexto relevante para Aider
```

##### Hooks de Desenvolvimento

```python
from aider_vectorizer import VectorizerPlugin

plugin = VectorizerPlugin(
    persist_path="./.aider/vectorizer.bin",
    embedding_model="code-search-net",
    chunk_size=512
)

# Indexar codebase
plugin.index_codebase("./src")

# Durante desenvolvimento
context = plugin.find_relevant_context(
    task="implement user authentication",
    current_file="auth.py",
    max_results=3
)

# context contém:
# - Funções similares de auth
# - Padrões de segurança existentes
# - Imports relevantes
```

##### Integração com Git

```python
# Indexar apenas arquivos modificados
plugin.index_changed_files(git_diff_output)

# Buscar contexto baseado em commit messages
context = plugin.search_by_commit_pattern(
    pattern="auth|login|security",
    since="1 week ago"
)
```

### TypeScript Aider Plugin

#### Instalação

```bash
npm install @hivellm/aider-vectorizer
```

#### Configuração

```typescript
// .aider/config.json
{
  "plugins": ["@hivellm/aider-vectorizer"],
  "vectorizer": {
    "persistPath": "./.aider/vectorizer.bin",
    "collection": "codebase",
    "embeddingModel": "microsoft/codebert-base",
    "chunkSize": 512,
    "similarityThreshold": 0.7,
    "maxResults": 5
  }
}
```

#### Exemplo de Uso

```typescript
import { VectorizerAiderPlugin } from '@hivellm/aider-vectorizer';

const plugin = new VectorizerAiderPlugin({
  persistPath: './.aider/vectorizer.bin',
  collection: 'codebase'
});

// Indexar projeto TypeScript
await plugin.indexProject('./src', {
  include: ['*.ts', '*.tsx'],
  exclude: ['node_modules/**', '*.test.ts']
});

// Buscar contexto para tarefa
const context = await plugin.findContext({
  task: 'implement React authentication hook',
  currentFile: 'hooks/useAuth.ts',
  language: 'typescript',
  framework: 'react'
});

// Retorna contexto relevante:
// - Hooks de auth existentes
// - Padrões de state management
// - Tipos TypeScript relacionados
```

## Integrações Avançadas

### LangChain + RAG Pipeline Completo

```python
from langchain.vectorstores import VectorizerStore
from langchain.llms import OpenAI
from langchain.chains import RetrievalQA
from langchain.agents import initialize_agent, Tool
from vectorizer import VectorizerDB

# Configurar componentes
db = VectorizerDB(persist_path="data/rag.bin")
embeddings = OpenAIEmbeddings()
vectorstore = VectorizerStore(db, embeddings.embed_query)

# Criar ferramentas para o agente
search_tool = Tool(
    name="DocumentSearch",
    description="Search through technical documentation",
    func=lambda q: vectorstore.similarity_search(q, k=3)
)

qa_tool = Tool(
    name="QuestionAnswering",
    description="Answer questions using retrieved documents",
    func=lambda q: RetrievalQA.from_chain_type(
        llm=OpenAI(),
        retriever=vectorstore.as_retriever()
    )({"query": q})["result"]
)

# Inicializar agente
agent = initialize_agent(
    [search_tool, qa_tool],
    OpenAI(temperature=0),
    agent="zero-shot-react-description",
    verbose=True
)

# Usar o agente
result = agent.run("How do I implement vector search in my application?")

# Exemplo completo: RAG com embedding automático
def create_rag_system():
    # Configurar componentes
    db = VectorizerDB(persist_path="data/rag.bin")

    # Embedding function integrada
    embedder = SentenceTransformerEmbedder(
        model_name="sentence-transformers/all-MiniLM-L6-v2"
    )

    vectorstore = VectorizerStore(
        db=db,
        embedding_function=lambda texts: embedder.embed_texts(texts),
        collection_name="knowledge_base"
    )

    # Carregar e processar documentos
    loader = PyPDFLoader("documentation.pdf")
    documents = loader.load()

    text_splitter = RecursiveCharacterTextSplitter(
        chunk_size=1000,
        chunk_overlap=200
    )

    docs = text_splitter.split_documents(documents)

    # Inserir com embeddings automáticos
    vectorstore.add_documents(docs)

    # Configurar QA chain
    llm = OpenAI(temperature=0)
    qa_chain = RetrievalQA.from_chain_type(
        llm=llm,
        chain_type="stuff",
        retriever=vectorstore.as_retriever(search_kwargs={"k": 3})
    )

    return qa_chain

# Usar o sistema RAG
qa_system = create_rag_system()
answer = qa_system.run("What are the main features of vector search?")
```

### Aider + Code Generation Workflow

```python
from aider_vectorizer import DevelopmentWorkflow
from vectorizer import VectorizerDB

# Configurar workflow
db = VectorizerDB(persist_path="./.dev/vectorizer.bin")
workflow = DevelopmentWorkflow(db)

# Pipeline de desenvolvimento
async def develop_feature(feature_request: str):
    # 1. Analisar pedido
    analysis = await workflow.analyze_request(feature_request)

    # 2. Encontrar código relacionado
    relevant_code = await workflow.find_similar_implementations(
        analysis["components"]
    )

    # 3. Gerar contexto para Aider
    context = workflow.build_aider_context(
        feature_request,
        relevant_code,
        analysis["patterns"]
    )

    # 4. Executar Aider com contexto
    result = await workflow.run_aider_with_context(
        feature_request,
        context
    )

    # 5. Indexar novo código gerado
    await workflow.index_new_code(result["files"])

    return result

# Uso
result = await develop_feature(
    "Add OAuth2 authentication to the API"
)
```

### Integração com ML Frameworks

#### Com PyTorch

```python
import torch
from vectorizer import VectorizerDB
from transformers import AutoTokenizer, AutoModel

class TorchVectorizerIntegration:
    def __init__(self, db_path: str, model_name: str):
        self.db = VectorizerDB(db_path)
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoModel.from_pretrained(model_name)

    @torch.no_grad()
    def embed_text(self, texts: List[str]) -> List[List[float]]:
        inputs = self.tokenizer(texts, padding=True, truncation=True, return_tensors="pt")
        outputs = self.model(**inputs)
        embeddings = outputs.last_hidden_state.mean(dim=1)
        return embeddings.numpy().tolist()

    def semantic_search(self, query: str, k: int = 5) -> List[Dict]:
        # Gerar embedding da query
        query_embedding = self.embed_text([query])[0]

        # Buscar no Vectorizer
        return self.db.search("documents", query_embedding, k)

# Uso
integrator = TorchVectorizerIntegration("data/torch.bin", "bert-base-uncased")
results = integrator.semantic_search("machine learning algorithms")
```

#### Com TensorFlow

```python
import tensorflow as tf
from vectorizer import VectorizerDB
import tensorflow_hub as hub

class TensorFlowVectorizerIntegration:
    def __init__(self, db_path: str):
        self.db = VectorizerDB(db_path)
        self.embedder = hub.load("https://tfhub.dev/google/universal-sentence-encoder/4")

    def embed_text(self, texts: List[str]) -> List[List[float]]:
        embeddings = self.embedder(texts)
        return embeddings.numpy().tolist()

    def find_similar_documents(self, query: str, threshold: float = 0.5):
        query_embedding = self.embed_text([query])[0]

        # Buscar todos os documentos similares
        all_results = self.db.search("documents", query_embedding, k=100)

        # Filtrar por threshold
        return [r for r in all_results if r["score"] >= threshold]

# Uso
integrator = TensorFlowVectorizerIntegration("data/tf.bin")
similar_docs = integrator.find_similar_documents("deep learning", threshold=0.7)
```

## Estratégias de Deployment

### Docker Integration

#### Dockerfile para Aplicação Completa

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Build Vectorizer core
RUN cargo build --release --bin vectorizer-server

# Python stage
FROM python:3.11-slim

COPY --from=builder /app/target/release/vectorizer-server /usr/local/bin/
COPY requirements.txt .

RUN pip install -r requirements.txt

# Instalar integrações
RUN pip install vectorizer[langchain,aider]

EXPOSE 8080 8081

CMD ["vectorizer-server", "--host", "0.0.0.0"]
```

#### Docker Compose para Desenvolvimento

```yaml
version: '3.8'
services:
  vectorizer:
    build: .
    ports:
      - "8080:8080"  # REST API
      - "8081:8081"  # MCP API
    volumes:
      - ./data:/app/data
    environment:
      - VECTORIZER_PERSIST_PATH=/app/data/vectorizer.bin
      - VECTORIZER_COLLECTION=development

  aider:
    image: aider:latest
    volumes:
      - .:/workspace
      - ./data:/workspace/.aider
    environment:
      - AIDERS_VECTORIZER_URL=http://vectorizer:8080
    depends_on:
      - vectorizer
```

### Cloud Deployment

#### AWS Integration

```python
import boto3
from vectorizer import VectorizerDB

class AWSVectorizerIntegration:
    def __init__(self, db_path: str, s3_bucket: str):
        self.db = VectorizerDB(db_path)
        self.s3 = boto3.client('s3')
        self.bucket = s3_bucket

    def backup_to_s3(self, key: str):
        """Backup do índice para S3"""
        self.db.persist()
        self.s3.upload_file(self.db.persist_path, self.bucket, key)

    def restore_from_s3(self, key: str):
        """Restore do índice do S3"""
        self.s3.download_file(self.bucket, key, self.db.persist_path)
        self.db.load()

    def scale_with_lambda(self, query: str) -> List[Dict]:
        """Processamento serverless com Lambda"""
        lambda_client = boto3.client('lambda')

        response = lambda_client.invoke(
            FunctionName='vectorizer-search',
            Payload=json.dumps({
                'query': query,
                'k': 5
            })
        )

        return json.loads(response['Payload'].read())
```

## Monitoramento e Observabilidade

### Métricas de Integração

```python
from vectorizer import VectorizerDB
import time

class IntegrationMonitor:
    def __init__(self, db: VectorizerDB):
        self.db = db
        self.metrics = {}

    def measure_langchain_performance(self, operation: str, func, *args, **kwargs):
        start_time = time.time()
        try:
            result = func(*args, **kwargs)
            duration = time.time() - start_time
            self.metrics[f"langchain_{operation}"] = duration
            return result
        except Exception as e:
            self.metrics[f"langchain_{operation}_error"] = 1
            raise

    def measure_aider_performance(self, task: str, context_size: int):
        """Monitorar performance do Aider integration"""
        self.metrics["aider_context_size"] = context_size
        self.metrics["aider_task"] = task

    def get_metrics(self):
        return {
            **self.db.get_stats(),
            **self.metrics
        }
```

---

Estas integrações fornecem uma ponte seamless entre o Vectorizer e ecossistemas populares de IA, permitindo uso imediato em aplicações existentes e workflows de desenvolvimento modernos.
