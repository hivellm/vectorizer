# 🧠 Intelligent Search Tools Documentation

**Version:** 0.3.1  
**Status:** ✅ Production Ready  
**Last Updated:** 2025-01-06

## 🎯 Overview

Vectorizer v0.3.1 introduces advanced intelligent search capabilities that significantly improve upon traditional vector search methods. These tools provide 3-4x better coverage, automatic query generation, and sophisticated result ranking.

## 🚀 Available Tools

### 1. 🧠 **intelligent_search**

**Purpose**: Advanced semantic search with multi-query generation and domain expansion

**Key Features**:
- **Multi-Query Generation**: Automatically generates 4-8 related queries
- **Domain Expansion**: Expands queries with technical terms and synonyms
- **MMR Diversification**: Ensures diverse, high-quality results
- **Technical Focus**: Boosts scores for technical content
- **Collection Bonuses**: Prioritizes relevant collections

**Usage**:
```bash
curl -X POST "http://localhost:15002/intelligent_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "CMMV framework architecture",
    "collections": ["cmmv-core-docs"],
    "max_results": 10,
    "domain_expansion": true,
    "technical_focus": true,
    "mmr_enabled": true,
    "mmr_lambda": 0.7
  }'
```

**MCP Usage**:
```python
# Python MCP client
response = await client.post(
    "http://localhost:15002/mcp/message",
    json={
        "method": "tools/call",
        "params": {
            "name": "intelligent_search",
            "arguments": {
                "query": "CMMV framework",
                "collections": ["cmmv-core-docs"],
                "max_results": 5
            }
        }
    }
)
```

### 2. 🔬 **semantic_search**

**Purpose**: High-precision semantic search with rigorous filtering

**Key Features**:
- **Semantic Reranking**: Advanced relevance scoring
- **Similarity Thresholds**: Configurable quality filters (0.1-0.5)
- **Cross-Encoder Support**: Maximum precision matching
- **Quality Control**: Rigorous filters for high-quality results

**Usage**:
```bash
curl -X POST "http://localhost:15002/semantic_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "authentication system",
    "collection": "cmmv-core-docs",
    "similarity_threshold": 0.15,
    "semantic_reranking": true,
    "max_results": 10
  }'
```

**Recommended Thresholds**:
- **High Precision**: 0.15-0.2
- **Balanced**: 0.1-0.15
- **High Recall**: 0.05-0.1

### 3. 🌐 **multi_collection_search**

**Purpose**: Cross-collection search with intelligent reranking

**Key Features**:
- **Simultaneous Search**: Search across multiple collections
- **Cross-Collection Reranking**: Balanced results from different sources
- **Intelligent Deduplication**: Removes duplicate content across collections
- **Collection Balancing**: Ensures fair representation from each collection

**Usage**:
```bash
curl -X POST "http://localhost:15002/multi_collection_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "API documentation",
    "collections": ["cmmv-core-docs", "vectorizer-docs"],
    "max_per_collection": 5,
    "max_total_results": 15,
    "cross_collection_reranking": true
  }'
```

### 4. 🎯 **contextual_search**

**Purpose**: Context-aware search with metadata filtering

**Key Features**:
- **Metadata Filtering**: Filter by file type, chunk index, etc.
- **Context Reranking**: Reorder based on contextual relevance
- **Configurable Weights**: Balance between relevance and context
- **Flexible Filters**: Support for complex filtering criteria

**Usage**:
```bash
curl -X POST "http://localhost:15002/contextual_search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "database integration",
    "collection": "cmmv-core-docs",
    "context_filters": {
      "file_extension": ".md",
      "chunk_index": 0
    },
    "context_reranking": true,
    "context_weight": 0.3,
    "max_results": 10
  }'
```

## 📊 Performance Comparison

### Coverage Improvement

| Tool | Traditional Search | Intelligent Search | Improvement |
|------|-------------------|-------------------|-------------|
| **Results Found** | 4 results | 18 results (5 final) | 3-4x more |
| **Query Generation** | 1 query | 4-8 queries | Automatic |
| **Deduplication** | None | 18→9→5 results | Smart filtering |
| **Relevance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Superior |
| **Diversity** | ⭐⭐ | ⭐⭐⭐⭐⭐ | Much better |

### Quality Metrics

| Metric | intelligent_search | semantic_search | multi_collection | contextual_search |
|--------|-------------------|-----------------|------------------|-------------------|
| **Relevance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Coverage** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Diversity** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Intelligence** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Performance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

## 🔧 Configuration

### Recommended Settings

```json
{
  "intelligent_search": {
    "domain_expansion": true,
    "technical_focus": true,
    "mmr_enabled": true,
    "mmr_lambda": 0.7,
    "max_results": 10
  },
  "semantic_search": {
    "semantic_reranking": true,
    "similarity_threshold": 0.15,
    "max_results": 10
  },
  "multi_collection_search": {
    "cross_collection_reranking": true,
    "max_per_collection": 5,
    "max_total_results": 15
  },
  "contextual_search": {
    "context_reranking": true,
    "context_weight": 0.3,
    "max_results": 10
  }
}
```

### Use Case Recommendations

#### **For General Use**
- **🥇 Recommended**: `intelligent_search` - Best balance between quality and coverage
- **🥈 Alternative**: `search_vectors` - Simple and fast for basic searches

#### **For Specific Cases**
- **🎯 High Precision**: `semantic_search` with threshold 0.1-0.2
- **🌐 Multi-Collection**: `multi_collection_search` for broad search
- **🔍 Specific Filters**: `contextual_search` with metadata filters

## 🐛 Troubleshooting

### Common Issues

#### **"No default provider set" Error**
- **Cause**: Collection-specific embedding manager not initialized
- **Solution**: Automatically resolved in v0.3.1 with collection-specific managers

#### **Threshold Too Strict**
- **Issue**: `semantic_search` with threshold 0.5 returns 0 results
- **Solution**: Use threshold 0.1-0.2 for better results

#### **Performance Overhead**
- **Issue**: Intelligent tools are ~20% slower than traditional search
- **Solution**: Compensated by 3-4x better result quality

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# Set environment variable for debug logging
export RUST_LOG=debug

# Run vectorizer with debug output
cargo run --release
```

## 📈 Best Practices

### Query Optimization

1. **Use Specific Terms**: More specific queries yield better results
2. **Leverage Domain Expansion**: Enable for technical content
3. **Adjust Thresholds**: Fine-tune similarity thresholds for your use case
4. **Collection Selection**: Choose relevant collections for better results

### Performance Optimization

1. **Batch Operations**: Use multi-collection search for related queries
2. **Result Limits**: Set appropriate `max_results` to balance quality and speed
3. **Context Filters**: Use metadata filters to narrow down results
4. **Caching**: Results are automatically cached for repeated queries

### Integration Tips

1. **MCP Integration**: Use MCP tools for AI model integration
2. **REST API**: Use REST endpoints for web applications
3. **Error Handling**: Implement proper error handling for production use
4. **Monitoring**: Monitor performance metrics and adjust settings accordingly

## 🔗 Related Documentation

- **[API Documentation](../api/README.md)** - Complete API reference
- **[Quality Report](INTELLIGENT_SEARCH_QUALITY_REPORT.md)** - Detailed performance analysis
- **[MCP Integration](MCP_INTEGRATION.md)** - Model Context Protocol guide
- **[Performance Guide](PERFORMANCE_GUIDE.md)** - Optimization and tuning

## 📄 License

This project is licensed under the [MIT License](../../LICENSE).

---

**Intelligent Search Tools v0.3.1** - *Advanced semantic search for the modern AI era* 🧠
