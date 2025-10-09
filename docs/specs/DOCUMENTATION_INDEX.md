# 📚 Vectorizer Documentation Index

This index organizes all Vectorizer documentation to facilitate navigation and updates.

## 🎯 **Main Documentation**

### **Overview**
- [README.md](../README.md) - Main project documentation
- [ROADMAP.md](ROADMAP.md) - Development roadmap and current status
- [TECHNICAL_DOCUMENTATION_INDEX.md](TECHNICAL_DOCUMENTATION_INDEX.md) - Detailed technical index

### **Configuration and Installation**
- [DOCKER.md](DOCKER.md) - Installation and usage with Docker
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guide
- [API.md](API.md) - REST API documentation

## 🏗️ **Architecture and Implementation**

### **Core Architecture**
- [REST_ARCHITECTURE.md](REST_ARCHITECTURE.md) - REST architecture and unified server
- [phase1/ARCHITECTURE.md](phase1/ARCHITECTURE.md) - Base system architecture
- [phase1/TECHNICAL_IMPLEMENTATION.md](phase1/TECHNICAL_IMPLEMENTATION.md) - Detailed technical implementation

### **Advanced Systems**
- [FILE_WATCHER_TECHNICAL_SPEC.md](../technical/FILE_WATCHER_TECHNICAL_SPEC.md) - File monitoring system (✅ COMPLETE)
- [CACHE_AND_INCREMENTAL_INDEXING.md](CACHE_AND_INCREMENTAL_INDEXING.md) - Cache and incremental indexing
- [WATCHER_SPECIFICATION.md](WATCHER_SPECIFICATION.md) - Monitoring system specification
- [../WORKSPACE_SIMPLIFICATION.md](../WORKSPACE_SIMPLIFICATION.md) - **NEW v0.26.0**: Simplified workspace configuration
- [METAL_GPU_IMPLEMENTATION.md](METAL_GPU_IMPLEMENTATION.md) - **NEW v0.24.0**: Metal GPU acceleration implementation

## 🧠 **Embeddings and Search**

### **Embedding System**
- [EMBEDDING_ADVANCED_FEATURES.md](EMBEDDING_ADVANCED_FEATURES.md) - Advanced embedding features
- [EMBEDDING_PERSISTENCE.md](EMBEDDING_PERSISTENCE.md) - Embedding persistence
- [CHUNK_OPTIMIZATION_GUIDE.md](CHUNK_OPTIMIZATION_GUIDE.md) - Chunk optimization

### **Performance**
- [PERFORMANCE_GUIDE.md](PERFORMANCE_GUIDE.md) - Performance and optimization guide
- [TESTING_COVERAGE.md](TESTING_COVERAGE.md) - Test coverage

## 🔧 **Integration and Protocols**

### **MCP (Model Context Protocol)**
- [MCP_INTEGRATION.md](MCP_INTEGRATION.md) - MCP integration for IDEs
- [MCP_TOOLS.md](MCP_TOOLS.md) - Available MCP tools
- [MCP_ENHANCEMENTS_AND_SUMMARIZATION.md](MCP_ENHANCEMENTS_AND_SUMMARIZATION.md) - **MCP Enhancements and Summarization**

### **Batch Operations**
- [phase6/BATCH_OPERATIONS_SPECIFICATION.md](phase6/BATCH_OPERATIONS_SPECIFICATION.md) - Batch operations specification
- [phase6/BATCH_OPERATIONS_EXAMPLES.md](phase6/BATCH_OPERATIONS_EXAMPLES.md) - Batch operations examples

## 📝 **Summarization and Intelligence**

### **Summarization System** ⭐ **NEW**
- **Configuration**: `config.yml` - `summarization` section
- **Implementation**: `src/summarization/` - Complete summarization module
- **Available Methods**:
  - `extractive` - Extractive summarization with MMR algorithm
  - `keyword` - Keyword-based summarization
  - `sentence` - Sentence-based summarization
  - `abstractive` - Abstractive summarization (planned)

### **Summarization Features**
- **Automatic Summarization**: During document indexing
- **Dynamic Collections**: `{collection_name}_summaries` and `{collection_name}_chunk_summaries`
- **Rich Metadata**: References to original files and derived content flags
- **MMR Algorithm**: Maximal Marginal Relevance for diversity and relevance

### **Summarization Configuration**
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
      max_length: 200
```

## 🚀 **Roadmap and Phases**

### **Implemented Phases**
- [phase1/](phase1/) - Phase 1: Foundation ✅
- [phase2/](phase2/) - Phase 2: Advanced Embeddings ✅
- [phase3/](phase3/) - Phase 3: Production APIs ✅
- [phase4/](phase4/) - Phase 4: REST, MCP & SDKs ✅
- [phase5/](phase5/) - Phase 5: Advanced Features ✅
- [phase6/](phase6/) - Phase 6: Batch Operations ✅

### **Future Features**
- [ADVANCED_FEATURES_ROADMAP.md](ADVANCED_FEATURES_ROADMAP.md) - Advanced features roadmap
- [CHAT_HISTORY_AND_MULTI_MODEL_DISCUSSIONS.md](CHAT_HISTORY_AND_MULTI_MODEL_DISCUSSIONS.md) - Chat history and multi-model discussions
- [future/](future/) - Future features documentation

## 📊 **Reviews and Quality**

### **Review Reports**
- [reviews/](reviews/) - Review analyses by different models
- [phase2/](phase2/) - Phase 2 reviews
- [phase3/](phase3/) - Phase 3 reviews
- [phase4/](phase4/) - Phase 4 reviews
- [phase5/](phase5/) - Phase 5 reviews

### **CI/CD and Testing**
- [CI_CD_MCP_COVERAGE.md](CI_CD_MCP_COVERAGE.md) - CI/CD and MCP coverage

## 🔄 **Recent Updates**

### **v0.18.0 - Automatic Summarization** ⭐
- ✅ Summarization system implemented
- ✅ Dynamic collections for summaries
- ✅ MMR algorithm for extractive summarization
- ✅ Integration with REST and MCP
- ✅ Rich metadata for derived content

### **Next Required Updates**
1. **Summarization Documentation**: Create specific guides
2. **Usage Examples**: Add practical examples
3. **Advanced Configuration**: Document detailed parameters
4. **Troubleshooting**: Problem resolution guide

## 📋 **Update Checklist**

### **Files that Need Summarization Updates**
- [ ] `README.md` - Add summarization section
- [ ] `CHANGELOG.md` - Document v0.18.0
- [ ] `config.example.yml` - Include summarization configuration
- [ ] `docs/MCP_TOOLS.md` - Add summarization tools
- [ ] `docs/API.md` - Document summarization endpoints
- [ ] `client-sdks/` - Update SDKs with summarization
- [ ] `docs/REST_ARCHITECTURE.md` - Document summarization endpoints

### **New Files Needed**
- [ ] `docs/SUMMARIZATION_GUIDE.md` - Complete summarization guide
- [ ] `docs/SUMMARIZATION_EXAMPLES.md` - Practical examples
- [ ] `docs/SUMMARIZATION_CONFIGURATION.md` - Detailed configuration
- [ ] `docs/SUMMARIZATION_TROUBLESHOOTING.md` - Problem resolution

## 🎯 **How to Use This Index**

1. **For Developers**: Start with [TECHNICAL_DOCUMENTATION_INDEX.md](TECHNICAL_DOCUMENTATION_INDEX.md)
2. **For Users**: Check [README.md](../README.md) and [API.md](API.md)
3. **For Configuration**: See [phase1/CONFIGURATION.md](phase1/CONFIGURATION.md)
4. **For Summarization**: Consult the "Summarization and Intelligence" section above
5. **For Contributing**: Read [CONTRIBUTING.md](CONTRIBUTING.md)

---

**Last Updated**: September 2025  
**Version**: v0.18.0  
**Status**: ✅ Summarization System Implemented
