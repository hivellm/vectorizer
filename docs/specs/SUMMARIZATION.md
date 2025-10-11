# Summarization System

**Version**: 1.0  
**Status**: ✅ Production Ready  
**Last Updated**: 2025-09-25

---

## Overview

Intelligent automatic summarization system that processes documents during indexing to create concise summaries for efficient AI model context usage.

---

## Summarization Methods

### Extractive Summarization (Default)

**Algorithm**: MMR (Maximal Marginal Relevance)  
**Purpose**: Select most relevant sentences while maintaining diversity

**Configuration**:
```yaml
extractive:
  enabled: true
  max_sentences: 5
  lambda: 0.7              # Relevance vs diversity
  min_sentence_length: 10
  use_tfidf: true
```

### Keyword Summarization

**Purpose**: Extract key terms for quick overview

**Configuration**:
```yaml
keyword:
  enabled: true
  max_keywords: 10
  min_keyword_length: 3
  use_stopwords: true
  language: "en"
```

### Sentence Summarization

**Purpose**: Select important sentences by position and content

**Configuration**:
```yaml
sentence:
  enabled: true
  max_sentences: 3
  min_sentence_length: 15
  use_position_weight: true
```

---

## Dynamic Collections

**File-Level Summaries**:
- Pattern: `{collection}_summaries`
- Content: Complete document summaries

**Chunk-Level Summaries**:
- Pattern: `{collection}_chunk_summaries`
- Content: Individual chunk summaries

---

## REST API

### Summarize Text
```http
POST /api/v1/summarize/text
{
  "text": "long document...",
  "method": "extractive",
  "max_length": 200
}
```

### Get Summary
```http
GET /api/v1/summaries/{summary_id}
```

### List Summaries
```http
GET /api/v1/summaries?method=extractive&language=en&page=1&limit=50
```

---

## MCP Tools

**summarize_text**: Summarize text content  
**summarize_context**: Context-aware summarization  
**get_summary**: Retrieve summary by ID  
**list_summaries**: List all summaries with filters

---

## Testing

**Coverage**:
- ✅ All summarization methods
- ✅ All interfaces (REST, MCP)
- ✅ Error handling
- ✅ Edge cases
- ✅ Multi-language support
- ✅ Pagination and filtering

---

**Status**: ✅ Production Ready  
**Maintained by**: HiveLLM Team

