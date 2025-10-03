# Vectorizer API Documentation

Este diretório contém a documentação completa da API do Vectorizer, incluindo o schema OpenAPI 3.0.3.

## 📁 Arquivos

- **`openapi.yaml`** - Schema OpenAPI 3.0.3 completo da API do Vectorizer
- **`README.md`** - Este arquivo com instruções de uso

## 🚀 Como Usar

### 1. Visualizar a Documentação

#### Swagger UI Online
1. Acesse [Swagger Editor](https://editor.swagger.io/)
2. Copie o conteúdo do arquivo `openapi.yaml`
3. Cole no editor para visualizar a documentação interativa

#### Swagger UI Local
```bash
# Instalar Swagger UI
npm install -g swagger-ui-serve

# Servir a documentação localmente
swagger-ui-serve vectorizer/docs/api/openapi.yaml
```

#### Redoc
```bash
# Instalar Redoc CLI
npm install -g redoc-cli

# Gerar documentação HTML
redoc-cli build vectorizer/docs/api/openapi.yaml --output vectorizer/docs/api/index.html
```

### 2. Gerar SDKs

#### OpenAPI Generator
```bash
# Instalar OpenAPI Generator
npm install -g @openapitools/openapi-generator-cli

# Gerar SDK para TypeScript
openapi-generator-cli generate -i vectorizer/docs/api/openapi.yaml -g typescript-fetch -o ./sdks/typescript

# Gerar SDK para Python
openapi-generator-cli generate -i vectorizer/docs/api/openapi.yaml -g python -o ./sdks/python

# Gerar SDK para Rust
openapi-generator-cli generate -i vectorizer/docs/api/openapi.yaml -g rust -o ./sdks/rust
```

#### Swagger Codegen
```bash
# Instalar Swagger Codegen
npm install -g swagger-codegen

# Gerar SDK para JavaScript
swagger-codegen generate -i vectorizer/docs/api/openapi.yaml -l javascript -o ./sdks/javascript

# Gerar SDK para Java
swagger-codegen generate -i vectorizer/docs/api/openapi.yaml -l java -o ./sdks/java
```

### 3. Validar o Schema

```bash
# Instalar Swagger CLI
npm install -g swagger-cli

# Validar o schema
swagger-cli validate vectorizer/docs/api/openapi.yaml

# Bundle (resolver referências)
swagger-cli bundle vectorizer/docs/api/openapi.yaml -o vectorizer/docs/api/openapi-bundled.yaml
```

## 📋 Endpoints Principais

### 🏥 Sistema
- `GET /health` - Health check
- `GET /stats` - Estatísticas do sistema

### 📚 Collections
- `GET /collections` - Listar collections
- `POST /collections` - Criar collection
- `GET /collections/{name}` - Obter info da collection
- `DELETE /collections/{name}` - Deletar collection

### 🔍 Vectors
- `POST /collections/{name}/vectors` - Inserir textos
- `GET /collections/{name}/vectors` - Listar vectors
- `GET /collections/{name}/vectors/{id}` - Obter vector específico
- `DELETE /collections/{name}/vectors/{id}` - Deletar vector

### 🔎 Search
- `POST /collections/{name}/search` - Buscar vectors
- `POST /collections/{name}/search/text` - Buscar por texto

### 📦 Batch Operations
- `POST /collections/{name}/batch/insert` - Inserção em lote
- `POST /collections/{name}/batch/update` - Atualização em lote
- `POST /collections/{name}/batch/delete` - Deleção em lote
- `POST /collections/{name}/batch/search` - Busca em lote

### 🧠 Embedding
- `GET /embedding/providers` - Listar providers
- `POST /embedding/providers/set` - Definir provider

### 📊 Indexing
- `GET /indexing/progress` - Progresso da indexação

### 📝 Summarization
- `POST /summarize/text` - Resumir texto
- `GET /summaries` - Listar resumos
- `GET /summaries/{id}` - Obter resumo específico

## 🎯 Exemplos de Uso

### Criar Collection
```bash
curl -X POST "http://localhost:15001/api/v1/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-collection",
    "dimension": 512,
    "metric": "cosine"
  }'
```

### Inserir Textos
```bash
curl -X POST "http://localhost:15001/api/v1/collections/my-collection/vectors" \
  -H "Content-Type: application/json" \
  -d '{
    "texts": [
      {
        "id": "doc1",
        "text": "Este é um exemplo de texto para indexar",
        "metadata": {"source": "example"}
      }
    ]
  }'
```

### Buscar por Texto
```bash
curl -X POST "http://localhost:15001/api/v1/collections/my-collection/search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "exemplo de busca",
    "limit": 10,
    "score_threshold": 0.1
  }'
```

### Health Check
```bash
curl "http://localhost:15001/api/v1/health"
```

## 🔧 Configuração

### Servidor Local
- **URL Base**: `http://localhost:15001/api/v1`
- **Porta**: 15001
- **Versão**: 0.21.0

### Autenticação
Atualmente não há autenticação implementada. Para produção, considere implementar:
- API Keys
- JWT Tokens
- OAuth 2.0

## 📖 Especificações Técnicas

### Formatos Suportados
- **Input**: JSON
- **Output**: JSON
- **Encoding**: UTF-8

### Métricas de Distância
- `cosine` - Similaridade do cosseno
- `euclidean` - Distância euclidiana
- `dot_product` - Produto escalar

### Providers de Embedding
- `bm25` - BM25 (padrão)
- `tfidf` - TF-IDF
- `bert` - BERT
- `minilm` - MiniLM
- `bagofwords` - Bag of Words
- `charngram` - Character N-grams

### Métodos de Resumo
- `extractive` - Extrativo (padrão)
- `keyword` - Por palavras-chave
- `sentence` - Por sentenças
- `abstractive` - Abstrativo

## 🐛 Troubleshooting

### Erro 404 - Collection Not Found
```bash
# Verificar collections existentes
curl "http://localhost:15001/api/v1/collections"
```

### Erro 400 - Bad Request
- Verificar formato JSON
- Validar parâmetros obrigatórios
- Conferir tipos de dados

### Erro 500 - Internal Server Error
- Verificar logs do servidor
- Confirmar que o Vectorizer está rodando
- Verificar recursos do sistema

## 📝 Atualizações

Este schema é atualizado automaticamente quando:
- Novos endpoints são adicionados
- Estruturas de dados são modificadas
- Novos parâmetros são incluídos

Para contribuir com melhorias na documentação, consulte o [CONTRIBUTING.md](../../CONTRIBUTING.md).

## 🔗 Links Úteis

- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [Redoc](https://redoc.ly/)
- [OpenAPI Generator](https://openapi-generator.tech/)
- [Swagger Codegen](https://swagger.io/tools/swagger-codegen/)

## 📄 Licença

Este projeto está licenciado sob a [MIT License](../../LICENSE).
