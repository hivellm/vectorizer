# Proposal: Add n8n and Langflow Integration

## Why

Vectorizer needs to integrate with popular automation and workflow orchestration platforms to enable users to build complex AI pipelines without coding. n8n is an open-source workflow automation tool with 40k+ GitHub stars, widely used for connecting services and automating tasks. Langflow is a visual framework for building LLM applications with 40k+ GitHub stars, specifically designed for LangChain-based workflows.

By providing official integration nodes/components for both platforms, Vectorizer will:
1. **Lower adoption barrier**: Users can integrate vector search into existing workflows visually
2. **Increase market reach**: Access the combined user base of n8n (self-hosted and cloud) and Langflow
3. **Enable RAG pipelines**: Visual construction of Retrieval-Augmented Generation workflows
4. **Support enterprise adoption**: Many enterprises use n8n for process automation

## What Changes

### n8n Integration
- Custom n8n node package `@vectorizer/n8n-nodes-vectorizer`
- Operations: Create Collection, Delete Collection, Insert Vectors, Search, Hybrid Search
- Support for batch operations and streaming
- Credentials configuration for Vectorizer connection
- Documentation and example workflows

### Langflow Integration
- Custom Langflow component package `vectorizer-langflow`
- Components: VectorizerVectorStore, VectorizerRetriever, VectorizerLoader
- Integration with LangChain ecosystem
- Support for document loaders and text splitters
- RAG-ready templates

### REST API Enhancements
- Ensure all endpoints are compatible with webhook-based automation
- Add batch operation endpoints if missing
- Improve error responses for automation debugging

## Impact

- **Affected specs**: REST API, SDK patterns
- **Affected code**: New packages in `sdks/n8n/` and `sdks/langflow/`
- **Breaking change**: NO
- **User benefit**: Visual workflow integration, no-code AI pipeline building, enterprise automation support
