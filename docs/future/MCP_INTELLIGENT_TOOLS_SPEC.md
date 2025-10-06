# MCP Intelligent Tools Specification

## Overview

This document provides detailed specifications for the 4 enhanced MCP tools that will transform Vectorizer into a Cursor-level intelligent search engine.

## 🛠️ **Tool Specifications**

### **1. intelligent_search**

**Purpose**: Primary intelligent search tool with multi-query generation and semantic reranking.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {"type": "string", "description": "User search query"},
    "collections": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Specific collections to search (empty = all)"
    },
    "max_results": {
      "type": "number",
      "default": 5,
      "description": "Maximum number of results"
    },
    "rerank": {
      "type": "boolean",
      "default": true,
      "description": "Enable semantic reranking"
    },
    "deduplicate": {
      "type": "boolean",
      "default": true,
      "description": "Enable deduplication"
    },
    "domain_hints": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Domain-specific hints"
    }
  },
  "required": ["query"]
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "content": {"type": "string"},
          "score": {"type": "number"},
          "collection": {"type": "string"},
          "metadata": {"type": "object"}
        }
      }
    },
    "query_expansion": {
      "type": "array",
      "items": {"type": "string"}
    },
    "search_metadata": {
      "type": "object",
      "properties": {
        "total_queries": {"type": "number"},
        "collections_searched": {"type": "number"},
        "total_results_found": {"type": "number"},
        "results_after_dedup": {"type": "number"}
      }
    }
  }
}
```

### **2. semantic_search**

**Purpose**: Pure semantic search using embedding similarity.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {"type": "string"},
    "collections": {
      "type": "array",
      "items": {"type": "string"}
    },
    "similarity_threshold": {
      "type": "number",
      "default": 0.7
    },
    "max_results": {
      "type": "number",
      "default": 10
    },
    "include_embeddings": {
      "type": "boolean",
      "default": false
    }
  },
  "required": ["query"]
}
```

### **3. contextual_search**

**Purpose**: Search with additional context to improve relevance.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {"type": "string"},
    "context": {"type": "string"},
    "collections": {
      "type": "array",
      "items": {"type": "string"}
    },
    "context_weight": {
      "type": "number",
      "default": 0.3
    },
    "max_results": {
      "type": "number",
      "default": 5
    }
  },
  "required": ["query", "context"]
}
```

### **4. multi_collection_search**

**Purpose**: Search across multiple collection groups with weighted results.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "query": {"type": "string"},
    "collection_groups": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "collections": {
            "type": "array",
            "items": {"type": "string"}
          },
          "weight": {
            "type": "number",
            "default": 1.0
          }
        },
        "required": ["name", "collections"]
      }
    },
    "cross_collection_rerank": {
      "type": "boolean",
      "default": true
    },
    "max_results_per_group": {
      "type": "number",
      "default": 3
    }
  },
  "required": ["query", "collection_groups"]
}
```

## 🎯 **Implementation Priority**

1. **intelligent_search** - Core functionality
2. **semantic_search** - Pure semantic search
3. **contextual_search** - Context-aware search
4. **multi_collection_search** - Advanced multi-group search

## 📊 **Performance Targets**

- **Search Latency**: <100ms per tool
- **Memory Overhead**: <50MB additional
- **Quality Score**: >95% relevance
- **Throughput**: >1000 searches/second

## 🔧 **Configuration**

```yaml
mcp_tools:
  intelligent_search:
    enabled: true
    max_queries: 8
    reranking_enabled: true
    deduplication_enabled: true
    
  semantic_search:
    enabled: true
    similarity_threshold: 0.7
    include_embeddings: false
    
  contextual_search:
    enabled: true
    context_weight: 0.3
    
  multi_collection_search:
    enabled: true
    cross_collection_rerank: true
```

---

**These tools will provide Cursor-level search intelligence while maintaining simple, powerful APIs for any client to leverage.**
