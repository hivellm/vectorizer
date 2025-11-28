# Code Guidelines

**Version**: 1.0  
**Status**: ✅ Active  
**Last Updated**: 2025-10-01

---

## Language Requirements

### MANDATORY: All Code in ENGLISH 🇺🇸

**Required in English**:
- ✅ File names: `detector.rs`, `vulkan_backend.rs`
- ✅ Function names: `detect_gpu()`, `initialize_backend()`
- ✅ Variable names: `gpu_context`, `backend_type`
- ✅ Comments: `// Initialize Metal backend`
- ✅ Log messages: `"GPU detected"`
- ✅ Error messages: `"Failed to initialize"`
- ✅ Struct/Enum names: `GpuBackendType`, `VulkanConfig`
- ✅ Documentation: Doc comments in English

**Portuguese Allowed**:
- ❌ User-facing CLI messages
- ❌ README files (README_PT.md)
- ❌ Project documentation (planning docs)
- ❌ Commit messages

---

## Rust Edition

### MANDATORY: Rust Edition 2024

```toml
[package]
edition = "2024"  # NON-NEGOTIABLE
```

**Why**: Required for latest language features, async patterns, and performance optimizations

**🚫 NEVER CHANGE THIS SETTING**

---

## Architecture Rules

### Unified Server Architecture

**MANDATORY: REST + MCP Functionality Parity**

1. Implement in core business logic first
2. Add REST endpoints
3. Add MCP tools

**🚫 NEVER implement features in only one interface!**

---

## Naming Conventions

```rust
// Functions and variables: snake_case
fn create_collection(name: &str) -> Result<()>
let vector_store = VectorStore::new();

// Structs, enums, traits: PascalCase
pub struct CollectionConfig {
    pub dimension: usize,
}

// Constants: SCREAMING_SNAKE_CASE
const DEFAULT_DIMENSION: usize = 512;
```

---

## Code Organization

**Module Structure**:
```
src/
├── db/           # Database layer
├── server/       # API layer
├── embedding/    # Embedding providers
├── discovery/    # Search intelligence
└── utils/        # Shared utilities
```

**File Organization**:
- One public struct per file (when large)
- Group related functionality
- Keep files under 1000 lines

---

## Error Handling

```rust
// Use Result types
pub fn operation() -> Result<T, VectorizerError>

// Use thiserror for custom errors
#[derive(Debug, thiserror::Error)]
pub enum VectorizerError {
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),
}

// Use anyhow for application errors
use anyhow::{Context, Result};

fn process() -> Result<()> {
    operation().context("Failed to process")?;
    Ok(())
}
```

---

## Performance

**Hot Path Optimization**:
- Avoid allocations in loops
- Use `&str` instead of `String` when possible
- Prefer iterators over collecting to `Vec`
- Use `Arc` for shared read-only data

**Async Best Practices**:
- Use `tokio::spawn` for CPU-bound work
- Avoid blocking in async functions
- Use `tokio::select!` for concurrency

---

## Testing

**Coverage Requirements**:
- Unit tests: >90%
- Integration tests for all APIs
- Benchmarks for performance-critical code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

---

## Documentation

**Doc Comments**:
```rust
/// Performs semantic search on a collection.
///
/// # Arguments
/// * `collection` - Collection name
/// * `query` - Search query text
/// * `limit` - Maximum results
///
/// # Returns
/// Vector of search results sorted by relevance
///
/// # Errors
/// Returns error if collection not found
pub fn search(
    collection: &str,
    query: &str,
    limit: usize
) -> Result<Vec<SearchResult>>
```

---

## Formatting

**Use rustfmt**:
```bash
cargo fmt
```

**Use clippy**:
```bash
cargo clippy -- -D warnings
```

---

**Version**: 1.0  
**Maintained by**: HiveLLM Team

