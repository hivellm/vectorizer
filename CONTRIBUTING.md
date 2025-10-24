# Contributing to Vectorizer

Thank you for your interest in contributing to Vectorizer! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to the Contributor Covenant [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to team@hivellm.org.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/vectorizer.git
   cd vectorizer
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/hivellm/vectorizer.git
   ```

## Development Setup

### Prerequisites

- **Rust**: Nightly toolchain 1.85+ with Edition 2024
- **Git**: For version control
- **WSL/Linux/macOS**: For development (Windows users should use WSL)

### Install Rust Nightly

```bash
rustup toolchain install nightly
rustup default nightly
rustup update nightly
```

### Build the Project

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Build with GPU support (if available)
cargo build --features gpu
```

### Run the Server

```bash
# Development mode
cargo run

# Production mode
cargo run --release
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Follow the [Rust style guide](https://doc.rust-lang.org/1.0.0/style/)
- Write clear, concise commit messages
- Add tests for new functionality
- Update documentation as needed

### 3. Commit Your Changes

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```bash
# Feature
git commit -m "feat: Add new vector search algorithm"

# Bug fix
git commit -m "fix: Resolve memory leak in batch processing"

# Documentation
git commit -m "docs: Update API documentation"

# Performance improvement
git commit -m "perf: Optimize embedding generation"

# Refactoring
git commit -m "refactor: Simplify storage interface"

# Tests
git commit -m "test: Add integration tests for replication"
```

## Testing

### Run All Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests only
cargo test --test '*'
```

### Coverage

```bash
# Generate coverage report
cargo llvm-cov --all --ignore-filename-regex 'examples'

# Generate HTML coverage report
cargo llvm-cov --html --all --ignore-filename-regex 'examples'
```

**Minimum coverage requirement: 95%**

## Code Quality

### Format Code

```bash
# Format all code
cargo +nightly fmt --all

# Check formatting
cargo +nightly fmt --all -- --check
```

### Lint Code

```bash
# Run clippy (must pass with no warnings)
cargo clippy --workspace -- -D warnings

# Run clippy on all targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Spell Check

```bash
# Install codespell
pip install 'codespell[toml]'

# Run spell check
codespell \
  --skip="*.lock,*.json,target,node_modules,.git" \
  --ignore-words-list="crate,ser,deser"
```

### Quality Checklist

Before committing, ensure:

- ✅ Code is formatted: `cargo +nightly fmt --all`
- ✅ No clippy warnings: `cargo clippy --workspace -- -D warnings`
- ✅ All tests pass: `cargo test`
- ✅ Coverage ≥ 95%: `cargo llvm-cov`
- ✅ No typos: `codespell`
- ✅ Documentation updated
- ✅ CHANGELOG.md updated (for significant changes)

## Documentation

### Code Documentation

- Add doc comments (`///`) to all public APIs
- Include examples in doc comments
- Document error conditions
- Run doc tests: `cargo test --doc`

Example:

```rust
/// Searches for vectors similar to the query.
///
/// # Arguments
///
/// * `query` - The search query text
/// * `limit` - Maximum number of results to return
///
/// # Examples
///
/// ```
/// use vectorizer::search;
///
/// let results = search("machine learning", 10)?;
/// ```
///
/// # Errors
///
/// Returns an error if the query is empty or invalid.
pub fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    // Implementation
}
```

### Project Documentation

Update relevant documentation in `/docs`:

- `/docs/specs/` - Feature specifications
- `/docs/ARCHITECTURE.md` - Architecture changes
- `/docs/ROADMAP.md` - Implementation progress
- `README.md` - User-facing changes
- `CHANGELOG.md` - Version history

## Submitting Changes

### Before Submitting

1. **Sync with upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run quality checks**:
   ```bash
   cargo +nightly fmt --all
   cargo clippy --workspace -- -D warnings
   cargo test
   ```

3. **Update documentation**

4. **Update CHANGELOG.md** (for significant changes)

### Create Pull Request

1. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. Open a Pull Request on GitHub

3. Fill out the PR template:
   - Description of changes
   - Related issues
   - Testing performed
   - Breaking changes (if any)

### PR Review Process

- All PRs require at least one approval
- CI/CD checks must pass
- Coverage must be ≥ 95%
- No merge conflicts with main branch

## Release Process

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0 → 2.0.0): Breaking changes
- **MINOR** (1.0.0 → 1.1.0): New features (backwards compatible)
- **PATCH** (1.0.0 → 1.0.1): Bug fixes (backwards compatible)

### Creating a Release

1. **Update version** in `Cargo.toml`

2. **Update CHANGELOG.md**:
   ```markdown
   ## [1.2.0] - 2024-01-15
   
   ### Added
   - New feature X
   
   ### Fixed
   - Bug in component Y
   
   ### Changed
   - Refactored module Z
   ```

3. **Run quality checks**:
   ```bash
   cargo +nightly fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --all-features
   cargo doc --no-deps
   ```

4. **Commit version bump**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: Release version 1.2.0"
   ```

5. **Create annotated tag**:
   ```bash
   git tag -a v1.2.0 -m "Release version 1.2.0
   
   Major changes:
   - Feature X
   - Bug fix Y
   
   All tests passing ✅
   Coverage: 95%+ ✅
   Linting: Clean ✅
   Build: Success ✅"
   ```

6. **Push changes** (manual, as per project rules):
   ```bash
   git push origin main
   git push origin v1.2.0
   ```

## Getting Help

- **Documentation**: Check `/docs` directory
- **Issues**: Search existing issues on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Email**: team@hivellm.org

## License

By contributing to Vectorizer, you agree that your contributions will be licensed under the project's license (MIT or Apache 2.0).

---

Thank you for contributing to Vectorizer! 🚀

