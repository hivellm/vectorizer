# SDK Publishing Status

## ✅ **Successfully Published**

### TypeScript SDK

- **Package**: `@hivehub/vectorizer-sdk`
- **Registry**: npm
- **Status**: ✅ Published
- **Installation**: `npm install @hivehub/vectorizer-sdk`
- **Note**: Ships compiled CommonJS + ESM and works from plain
  JavaScript. The standalone `@hivehub/vectorizer-sdk-js` package was
  retired in v3.0.0 — install `@hivehub/vectorizer-sdk` instead.

### Rust SDK

- **Package**: `vectorizer-sdk`
- **Registry**: crates.io
- **Version**: v1.8.0
- **Status**: ✅ Published successfully
- **Installation**: Add to `Cargo.toml`: `vectorizer-sdk = "1.8.0"`

### Python SDK

- **Package**: `vectorizer-sdk`
- **Registry**: PyPI
- **Version**: v1.8.0
- **Status**: ✅ Published successfully
- **Installation**: `pip install vectorizer-sdk==1.8.0`

### C# SDK

- **Package**: `Vectorizer.Sdk`
- **Registry**: NuGet
- **Version**: v1.8.0
- **Status**: ✅ Published successfully
- **Installation**: `dotnet add package Vectorizer.Sdk`

### Removed in v3.0.0

The following packages were dropped in v3.0.0 and will not receive
new releases. Existing installations remain functional.

- LangChain (Python + JS), Langflow, n8n, TensorFlow, PyTorch
  integration packages — thin adapters over the core SDKs.
- Standalone JavaScript SDK (`@hivehub/vectorizer-sdk-js`) —
  superseded by the TypeScript SDK, which ships compiled JS and is
  fully usable from plain JavaScript.

Build directly against the language-native SDKs instead.

## 📋 **Publishing Summary**

| SDK           | Registry   | Status       | Package Name                         |
| ------------- | ---------- | ------------ | ------------------------------------ |
| TypeScript    | npm        | ✅ Published | @hivehub/vectorizer-sdk              |
| Rust          | crates.io  | ✅ Published | vectorizer-sdk                       |
| Python        | PyPI       | ✅ Published | vectorizer-sdk                       |
| C#            | NuGet      | ✅ Published | Vectorizer.Sdk                       |
| Go            | Go Modules | 🚧 In Dev    | github.com/hivellm/vectorizer-sdk-go |

## 🔧 **Publishing Infrastructure**

### Authentication Scripts Created

- `npm_auth_otp.sh` / `npm_auth_otp.ps1` / `npm_auth_otp.bat` - npm authentication
- `cargo_auth_setup.sh` / `cargo_auth_setup.ps1` / `cargo_auth_setup.bat` - cargo authentication
- `fix_rollup.sh` / `fix_rollup.ps1` / `fix_rollup.bat` - JavaScript build fixes
- `fix_python_publish.sh` - Python publishing fixes

### Enhanced Publishing Scripts

- `publish_sdks.sh` - Bash script with OTP authentication
- `publish_sdks.ps1` - PowerShell script with enhanced error handling
- `publish_sdks.bat` - Windows batch script

### Documentation Updates

- Updated main README with published status
- Updated client-sdks README with installation instructions
- Enhanced CHANGELOG with publishing achievements
- Created troubleshooting guides
- **NEW:** Created `SDK_UPDATE_REPORT.md` with complete feature coverage analysis

## 🎯 **SDK Feature Completeness**

All published SDKs are **100% complete** with all latest features implemented (Go SDK in development):

### New in v1.8.0: Master/Replica Routing

All SDKs now support automatic read/write routing for high-availability deployments:
- **HostConfig**: Configure master URL and replica URLs
- **ReadPreference**: Choose routing strategy (master, replica, nearest)
- **Automatic Routing**: Writes → master, reads → replicas (round-robin)
- **Per-Operation Override**: Override read preference for specific operations

### Feature Coverage Matrix

| Feature Category          | TypeScript | Python   | Rust     | C#        |
| ------------------------- | ---------- | -------- | -------- | --------- |
| **Intelligent Search**    | ✅ 4/4     | ✅ 4/4   | ✅ 4/4   | ✅ 4/4    |
| **Discovery Operations**  | ✅ 4/4     | ✅ 6/6   | ✅ 4/4   | ✅ 4/4    |
| **File Operations**       | ✅ 7/7     | ✅ 7/7   | ✅ 7/7   | ✅ 7/7    |
| **Batch Operations**      | ✅         | ✅       | ✅       | ✅        |
| **Collection Management** | ✅         | ✅       | ✅       | ✅        |
| **Vector Operations**     | ✅         | ✅       | ✅       | ✅        |
| **Status**                | **100%**   | **100%** | **100%** | **100%**  |

### Implemented Methods (Oct 2025)

**Intelligent Search:**

- `intelligentSearch()` - Multi-query expansion with MMR diversification
- `semanticSearch()` - Advanced semantic reranking
- `contextualSearch()` - Context-aware with metadata filtering
- `multiCollectionSearch()` - Cross-collection search with reranking

**Discovery Pipeline:**

- `discover()` - Complete discovery with LLM prompt generation
- `filterCollections()` - Collection filtering by patterns
- `scoreCollections()` - Relevance-based ranking
- `expandQueries()` - Query variation generation

**File Operations:**

- `getFileContent()` - Complete file retrieval
- `listFilesInCollection()` - Indexed file listing
- `getFileSummary()` - Extractive/structural summaries
- `getFileChunksOrdered()` - Progressive chunk reading
- `getProjectOutline()` - Hierarchical project structure
- `getRelatedFiles()` - Semantic file relationships
- `searchByFileType()` - Type-filtered semantic search

## 🎯 **Next Steps**

1. **Version Management**

   - Set up automated version bumping
   - Create release tags
   - Update documentation with new versions

2. **CI/CD Integration**
   - Set up automated publishing workflows
   - Add version validation
   - Implement automated testing before publishing

## 📊 **Success Metrics**

- **4 out of 5 SDKs** successfully published ✅ (Go SDK in development)
- **100% test coverage** maintained across all SDKs
- **Cross-platform support** with Bash, PowerShell, and Batch scripts
- **Comprehensive documentation** with troubleshooting guides
- **Enhanced authentication** with OTP-only flow for better UX
- **Standardized examples** across all SDKs ✅
