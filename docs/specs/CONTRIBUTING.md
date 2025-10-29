# Contributing to Vectorizer

## 🤝 Welcome Contributors!

Thank you for your interest in contributing to Vectorizer! This document outlines the development practices, testing guidelines, and contribution workflow for the project.

## 🏷️ Release Process

### Creating Releases

Vectorizer uses automated GitHub Actions for releases. The process is triggered by version tags:

#### **Automatic Release (Recommended)**
```bash
# 1. Update version in Cargo.toml
# 2. Commit changes
git add .
git commit -m "chore: bump version to 0.23.0"

# 3. Create and push version tag
git tag -a "v0.23.0" -m "Release v0.23.0"
git push origin main
git push origin "v0.23.0"

# GitHub Actions will automatically:
# - Build binaries for 6 platforms (Linux x86_64/ARM64, Windows x86_64, macOS x86_64/ARM64)
# - Create installation scripts
# - Generate GitHub release with downloads
# - Include all configuration files
```

#### **Release Contents**
Each release includes:
- **Binaries**: `vectorizer-cli`, `vectorizer-server`, `vectorizer-mcp-server`, `vzr`
- **Configuration**: `config.yml`, `vectorize-workspace.yml`
- **Documentation**: `README.md`, `LICENSE`
- **Installation Scripts**: `install.sh` (Linux/macOS), `install.bat` (Windows)

#### **Version Tagging**
- Use semantic versioning: `vX.Y.Z` (e.g., `v1.0.0`, `v0.22.1`)
- Tags must include patch version: `v1.0.0` ✅, `v1.0` ❌
- Prerelease tags supported: `v1.0.0-beta`

#### **Monitoring Releases**
- **GitHub Actions**: [Tag Release Workflow](https://github.com/hivellm/vectorizer/actions/workflows/tag-release.yml)
- **Release Page**: `https://github.com/hivellm/vectorizer/releases/tag/v{VERSION}`

## 📋 Development Guidelines

### Code Quality Standards

- **Rust Best Practices**: Follow the official Rust guidelines and idioms
- **Error Handling**: Use the custom `VectorizerError` type consistently
- **Documentation**: Document all public APIs with comprehensive examples
- **Testing**: Maintain high test coverage with both unit and integration tests

### Testing Strategy

#### Unit Tests (Algorithmic Validation)
Use controlled, deterministic vectors for testing core algorithms:
```rust
#[test]
fn test_distance_calculation() {
    // Use predictable vectors for algorithmic validation
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    // Test core distance/similarity calculations
}
```

#### Integration Tests (Real-World Validation)
**REQUIRED**: Use real embeddings for integration tests that validate end-to-end workflows:
```rust
#[test]
fn test_semantic_search_integration() {
    // 1. Set up real embedding provider
    let mut embedder = TfIdfEmbedding::new(64);
    embedder.build_vocabulary(&training_corpus);

    // 2. Generate embeddings from meaningful text
    let query_embedding = embedder.embed("machine learning algorithms").unwrap();

    // 3. Test complete workflow: embed → store → search → validate
    store.insert("collection", vectors).unwrap();
    let results = store.search("collection", &query_embedding, 5).unwrap();

    // 4. Validate semantic relevance
    assert!(results.iter().any(|r| r.score > 0.5));
}
```

#### Test Coverage Requirements
- **Unit Tests**: >90% coverage of algorithmic components
- **Integration Tests**: Complete embedding-to-search pipelines
- **Persistence Tests**: Verify data consistency after save/load cycles
- **Performance Tests**: Benchmarks with realistic embedding patterns

### Benchmarking

**CRITICAL**: Benchmarks require the `benchmarks` feature flag to isolate their dependencies from core builds.

#### Running Benchmarks
```bash
# Run all benchmarks
cargo bench --features benchmarks

# Run specific benchmark
cargo bench --features benchmarks --bench core_operations_bench

# Run GPU benchmarks (requires GPU feature)
cargo bench --features benchmarks,hive-gpu-cuda --bench cuda_bench
cargo bench --features benchmarks,hive-gpu-metal --bench metal_hnsw_search_bench

# Run embedding benchmarks
cargo bench --features benchmarks,fastembed --bench embeddings_bench
```

#### Building Benchmarks
```bash
# Build all benchmarks without running
cargo build --benches --features benchmarks

# Build release benchmarks
cargo build --release --benches --features benchmarks
```

#### Why Feature Flag?
- **Isolated Dependencies**: Benchmark tools (criterion, proptest) are heavy dependencies
- **Faster Core Builds**: Server and CLI builds don't need benchmark dependencies
- **Selective Compilation**: Only build benchmarks when needed for performance testing

#### Adding New Benchmarks
1. Create benchmark file in appropriate `benchmark/` subdirectory
2. Add `[[bench]]` entry to `Cargo.toml`
3. Include `required-features = ["benchmarks"]` in the entry
4. Add any additional features if needed (e.g., `["benchmarks", "hive-gpu-cuda"]`)

Example:
```toml
[[bench]]
name = "my_benchmark"
path = "benchmark/category/my_benchmark.rs"
harness = false
required-features = ["benchmarks"]
```

### Persistence Considerations

**CRITICAL**: Always preserve insertion order during persistence operations:

```rust
// ✅ CORRECT: Preserve original order
let vectors: Vec<PersistedVector> = collection
    .get_all_vectors()
    .into_iter()
    .map(PersistedVector::from)
    .collect();

// ❌ WRONG: Don't sort alphabetically
// vectors.sort_by(|a, b| a.id.cmp(&b.id));
```

**Why?** HNSW index performance and search accuracy depend on insertion order.

### Embedding System Guidelines

#### When to Use Manual Vectors
- Unit tests for core algorithms (distance calculations, HNSW operations)
- Performance benchmarks with controlled data patterns
- Edge case testing with specific vector configurations

#### When to Use Real Embeddings
- Integration tests validating complete workflows
- API endpoint testing
- User-facing feature validation
- Performance testing with realistic data patterns

#### Adding New Embedding Providers
1. Implement the `EmbeddingProvider` trait
2. Add comprehensive tests with real text data
3. Document vocabulary building requirements
4. Include performance characteristics

## 🔄 Contribution Workflow

### 1. Fork and Clone
```bash
git clone https://github.com/your-username/vectorizer.git
cd vectorizer
```

### 2. Set Up Development Environment
```bash
# Install Rust (nightly for edition 2024)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install nightly
rustup default nightly

# Run tests to ensure everything works
cargo test
```

### 3. Create a Feature Branch
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

### 4. Make Changes
- Follow the testing guidelines above
- Add tests for new functionality
- Update documentation as needed
- Run `cargo test` frequently

### 5. Commit Changes
```bash
# Stage your changes
git add .

# Commit with descriptive message
git commit -m "feat: add semantic search with real embeddings

- Implement TF-IDF embedding integration test
- Add persistence consistency validation
- Fix HNSW ordering issue in save/load cycles

Closes #123"
```

### 6. Push and Create Pull Request
```bash
git push origin feature/your-feature-name
```
Then create a PR on GitHub with:
- Clear description of changes
- Reference to any related issues
- Test results (CI should pass)

## 🧪 Testing Checklist

Before submitting a PR, ensure:

- [ ] All tests pass: `cargo test`
- [ ] New functionality has appropriate test coverage
- [ ] Integration tests use real embeddings where applicable
- [ ] Persistence tests verify search accuracy after save/load
- [ ] Documentation updated for any API changes
- [ ] Code follows Rust best practices
- [ ] No performance regressions introduced

## 🚨 Critical Checks

### Persistence Consistency
```bash
# Run persistence tests specifically
cargo test persistence

# Verify search accuracy is maintained
cargo test test_search_accuracy_after_persistence
```

### Embedding Integration
```bash
# Test real embedding workflows
cargo test test_vector_database_with_real_embeddings

# Test semantic search functionality
cargo test embedding_tests
```

## 📚 Documentation Updates

When making changes:

1. **Code Documentation**: Update rustdoc comments for any changed APIs
2. **README.md**: Update if adding new features or changing usage
3. **CHANGELOG.md**: Add entries for user-facing changes
4. **ROADMAP.md**: Update progress if completing roadmap items

## 🐛 Bug Reports and Issues

When reporting bugs:
- Include the full error message and stack trace
- Describe the steps to reproduce
- Include your Rust version (`rustc --version`)
- Mention if it's related to persistence or embeddings

## 💡 Feature Requests

For new features:
- Describe the use case and value proposition
- Consider how it fits into the existing architecture
- Think about testing requirements upfront
- Check if it aligns with the project roadmap

## 🎯 Code Review Guidelines

### For Reviewers
- Verify tests use appropriate embedding patterns
- Check persistence operations preserve ordering
- Ensure error handling is comprehensive
- Validate performance implications

### For Contributors
- Be open to feedback and suggestions
- Explain the reasoning behind implementation choices
- Provide context for complex changes
- Update tests and documentation as requested

## 📞 Getting Help

- **Issues**: Use GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub discussions for questions and ideas
- **Documentation**: Check the `/docs` folder for detailed guides

---

Thank you for contributing to Vectorizer! Your efforts help build a robust, high-performance vector database for the AI community. 🚀</contents>
</xai:function_call"> 
