# 📝 Vectorizer Summarization Guide

## Overview

Vectorizer includes an intelligent automatic summarization system that processes documents during indexing to create concise summaries. This system helps AI models understand project context more efficiently by providing summarized content alongside original documents.

## 🧠 Summarization Methods

### **Extractive Summarization** (Default)
- **Algorithm**: MMR (Maximal Marginal Relevance)
- **Purpose**: Selects the most relevant sentences while maintaining diversity
- **Configuration**:
  ```yaml
  extractive:
    enabled: true
    max_sentences: 5
    lambda: 0.7  # Balance between relevance and diversity
    min_sentence_length: 10
    use_tfidf: true
  ```

### **Keyword Summarization**
- **Purpose**: Extracts key terms for quick content overview
- **Configuration**:
  ```yaml
  keyword:
    enabled: true
    max_keywords: 10
    min_keyword_length: 3
    use_stopwords: true
    language: "en"
  ```

### **Sentence Summarization**
- **Purpose**: Selects important sentences based on position and content
- **Configuration**:
  ```yaml
  sentence:
    enabled: true
    max_sentences: 3
    min_sentence_length: 15
    use_position_weight: true
  ```

### **Abstractive Summarization** (Planned)
- **Purpose**: Generate new summary text using external LLMs
- **Status**: Planned for future implementation
- **Configuration**:
  ```yaml
  abstractive:
    enabled: false
    max_length: 200
    model: "gpt-3.5-turbo"
    api_key: ""
    temperature: 0.3
  ```

## 📋 Dynamic Collections

### **File-Level Summaries**
- **Collection Pattern**: `{collection_name}_summaries`
- **Content**: Complete document summaries
- **Use Case**: High-level understanding of entire documents

### **Chunk-Level Summaries**
- **Collection Pattern**: `{collection_name}_chunk_summaries`
- **Content**: Individual chunk summaries
- **Use Case**: Detailed understanding of specific sections

## 🔧 Configuration

### **Basic Configuration**
```yaml
summarization:
  enabled: true
  default_method: "extractive"
  
  methods:
    extractive:
      enabled: true
      max_sentences: 5
      lambda: 0.7
    keyword:
      enabled: true
      max_keywords: 10
    sentence:
      enabled: true
      max_sentences: 3
    abstractive:
      enabled: false
```

### **Advanced Configuration**
```yaml
summarization:
  enabled: true
  default_method: "extractive"
  
  # Method-specific settings
  methods:
    extractive:
      enabled: true
      max_sentences: 5
      lambda: 0.7
      min_sentence_length: 10
      use_tfidf: true
      
    keyword:
      enabled: true
      max_keywords: 10
      min_keyword_length: 3
      use_stopwords: true
      language: "en"
      
    sentence:
      enabled: true
      max_sentences: 3
      min_sentence_length: 15
      use_position_weight: true
  
  # Language support
  languages:
    en:
      stopwords: true
      stemming: true
    pt:
      stopwords: true
      stemming: true
  
  # Metadata configuration
  metadata:
    include_original_id: true
    include_file_path: true
    include_timestamp: true
    include_method: true
    include_compression_ratio: true
  
  # Collection naming patterns
  collection_patterns:
    file_summaries: "{collection_name}_summaries"
    chunk_summaries: "{collection_name}_chunk_summaries"
  
  # Performance settings
  performance:
    parallel_processing: true
    max_concurrent_tasks: 4
    batch_size: 10
    timeout_seconds: 30
```

## 🚀 Usage

### **Automatic Summarization**
Summarization happens automatically during document indexing:

1. **Document Loading**: Documents are loaded and chunked
2. **Summarization**: Summaries are generated using the configured method
3. **Collection Creation**: Summary collections are created automatically
4. **Vector Storage**: Summaries are embedded and stored with rich metadata

### **Manual Summarization** (GRPC)
```rust
// Summarize specific text
let result = grpc_client.summarize_text(
    "Your text content here",
    SummarizationParams {
        method: SummarizationMethod::Extractive,
        max_sentences: Some(5),
        language: Some("en".to_string()),
        metadata: Some(HashMap::new()),
    }
).await?;
```

### **Context Summarization** (GRPC)
```rust
// Summarize context for AI models
let result = grpc_client.summarize_context(
    ContextSummarizationParams {
        query: "What are the main proposals?".to_string(),
        collection: "gov-proposals".to_string(),
        method: SummarizationMethod::Extractive,
        max_sentences: Some(3),
        language: Some("en".to_string()),
    }
).await?;
```

## 📊 Metadata Structure

### **Summary Vector Metadata**
```json
{
  "original_file_path": "/path/to/original/file.md",
  "original_collection": "gov-proposals",
  "summarization_method": "extractive",
  "compression_ratio": 0.3,
  "timestamp": "2025-09-28T10:30:00Z",
  "is_derived_content": true,
  "original_vector_count": 15,
  "summary_vector_count": 1
}
```

## 🔍 Searching Summaries

### **MCP Search**
```bash
# Search in summary collections
mcp search_vectors --collection "gov-proposals_summaries" --query "main proposals" --limit 5
```

### **REST API Search**
```bash
curl -X POST "http://127.0.0.1:15001/api/v1/search" \
  -H "Content-Type: application/json" \
  -d '{
    "collection": "gov-proposals_summaries",
    "query": "main proposals",
    "limit": 5
  }'
```

## 🎯 Best Practices

### **Method Selection**
- **Extractive**: Best for technical documents and specifications
- **Keyword**: Best for quick overviews and topic identification
- **Sentence**: Best for narrative content and reports

### **Configuration Tuning**
- **max_sentences**: Adjust based on document complexity (3-7 sentences)
- **lambda**: Higher values favor relevance, lower values favor diversity
- **min_sentence_length**: Filter out very short sentences (10-15 characters)

### **Performance Optimization**
- **parallel_processing**: Enable for large document sets
- **batch_size**: Adjust based on available memory (5-20 documents)
- **timeout_seconds**: Set appropriate timeout for your content (30-60 seconds)

## 🐛 Troubleshooting

### **Common Issues**

#### **Summaries Not Created**
- Check if `summarization.enabled: true` in configuration
- Verify the summarization method is enabled
- Check logs for summarization errors

#### **Poor Summary Quality**
- Adjust `max_sentences` parameter
- Tune `lambda` parameter for MMR algorithm
- Consider using different summarization method

#### **Performance Issues**
- Enable `parallel_processing`
- Reduce `batch_size` if memory constrained
- Increase `timeout_seconds` for complex documents

### **Debugging**
```bash
# Check summarization status
vzr workspace status

# View summary collections
mcp list_collections

# Search summaries
mcp search_vectors --collection "your-collection_summaries" --query "test"
```

## 📈 Performance Metrics

### **Typical Performance**
- **Processing Speed**: 50-200 documents/minute
- **Summary Quality**: 0.7-0.9 relevance score
- **Compression Ratio**: 20-40% of original content
- **Memory Usage**: 10-50MB per 1000 documents

### **Quality Metrics**
- **Relevance**: How well summaries capture main content
- **Diversity**: How well summaries avoid redundancy
- **Coverage**: How well summaries cover all important topics
- **Readability**: How clear and coherent summaries are

## 🔮 Future Enhancements

### **Planned Features**
- **Abstractive Summarization**: LLM-based summary generation
- **Multi-language Support**: Enhanced language processing
- **Custom Models**: Integration with custom summarization models
- **Quality Metrics**: Automatic summary quality assessment
- **Interactive Summarization**: User-guided summary generation

### **Advanced Features**
- **Hierarchical Summarization**: Multi-level summary generation
- **Topic-based Summarization**: Summaries organized by topics
- **Temporal Summarization**: Time-based summary organization
- **Comparative Summarization**: Cross-document summary comparison
