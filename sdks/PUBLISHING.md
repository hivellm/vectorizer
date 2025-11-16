# SDK Publishing Guide

This guide explains how to publish all Hive Vectorizer client SDKs to their respective package registries using the automated publishing scripts.

## Prerequisites

Before publishing, ensure you have the following tools installed and configured:

### Required Tools

- **Node.js & npm** - For TypeScript and JavaScript SDKs
- **Python & pip** - For Python SDK
- **Rust & Cargo** - For Rust SDK
- **Git** - For version control

### Registry Credentials

#### npm (TypeScript & JavaScript SDKs)

```bash
npm login
# Enter your npm username, password, and email
```

#### PyPI (Python SDK)

```bash
pip install twine
twine configure
# Or create ~/.pypirc file with your credentials
```

#### crates.io (Rust SDK)

```bash
cargo login <your-api-token>
# Get your API token from https://crates.io/me
```

## Publishing Scripts

We provide two publishing scripts for different platforms:

### Linux/macOS (Bash)

```bash
./publish_sdks.sh [OPTIONS] [SDK]
```

### Windows (PowerShell)

```powershell
.\publish_sdks.ps1 [OPTIONS] [SDK]
```

## Usage Examples

### Publish All SDKs

```bash
# Linux/macOS
./publish_sdks.sh

# Windows
.\publish_sdks.ps1
```

### Run Tests Only

```bash
# Linux/macOS
./publish_sdks.sh --test

# Windows
.\publish_sdks.ps1 -Test
```

### Publish Specific SDK

```bash
# Linux/macOS - TypeScript only
./publish_sdks.sh typescript

# Windows - Python only
.\publish_sdks.ps1 python
```

### Skip Tests and Force Publish

```bash
# Linux/macOS
./publish_sdks.sh --no-test --force all

# Windows
.\publish_sdks.ps1 -NoTest -Force all
```

## Script Options

### Bash Script Options

- `-h, --help` - Show help message
- `-t, --test` - Run tests only (don't publish)
- `-f, --force` - Skip confirmation prompts
- `--no-test` - Skip running tests before publishing

### PowerShell Script Options

- `-Help` - Show help message
- `-Test` - Run tests only (don't publish)
- `-Force` - Skip confirmation prompts
- `-NoTest` - Skip running tests before publishing

### SDK Selection

- `typescript` - Publish only TypeScript SDK to npm
- `javascript` - Publish only JavaScript SDK to npm
- `python` - Publish only Python SDK to PyPI
- `rust` - Publish only Rust SDK to crates.io
- `all` - Publish all SDKs (default)

## Publishing Process

The publishing scripts perform the following steps:

1. **Prerequisites Check** - Verify all required tools are installed
2. **Test Execution** - Run comprehensive test suites for all SDKs
3. **Build Process** - Build packages for distribution
4. **Registry Publishing** - Upload packages to respective registries
5. **Verification** - Confirm successful publication

### Test Coverage

Before publishing, the scripts run:

- **TypeScript SDK**: 150+ tests
- **JavaScript SDK**: 140+ tests
- **Python SDK**: 184+ tests (✅ Published v1.3.0)
- **Rust SDK**: 88+ tests (✅ Published v1.3.0)

## Manual Publishing (Alternative)

If you prefer to publish manually or need more control:

### TypeScript SDK

```bash
cd typescript
npm run build
npm publish
```

### JavaScript SDK

```bash
cd javascript
npm run build
npm publish
```

### Python SDK

```bash
cd python
python setup.py sdist bdist_wheel
twine upload dist/*
```

### Rust SDK

```bash
cd rust
cargo package --dry-run
cargo publish
```

## Version Management

### Updating Versions

Before publishing, update version numbers in:

#### TypeScript & JavaScript SDKs

```json
// package.json
{
  "version": "1.3.0"
}
```

#### Python SDK

```python
# pyproject.toml
[project]
version = "1.3.0"
```

#### Rust SDK

```toml
# Cargo.toml
[package]
version = "1.3.0"
```

### Version Guidelines

- Use semantic versioning (MAJOR.MINOR.PATCH)
- Increment PATCH for bug fixes
- Increment MINOR for new features
- Increment MAJOR for breaking changes

## Pre-Publication Checklist

Before running the publishing script:

- [ ] All tests are passing locally
- [ ] Version numbers are updated in all SDKs
- [ ] CHANGELOG.md files are updated
- [ ] Documentation is up to date
- [ ] Registry credentials are configured
- [ ] Git repository is clean and committed

## Post-Publication Tasks

After successful publishing:

1. **Verify Packages** - Check that packages appear in registries
2. **Update Documentation** - Update main README with new version numbers
3. **Create Release Notes** - Document new features and changes
4. **Announce Release** - Notify users of the new version
5. **Monitor Issues** - Watch for any post-release problems

## Troubleshooting

### Common Issues

#### npm Login Issues

```bash
# Clear npm cache and login again
npm cache clean --force
npm login
```

#### PyPI Upload Issues

```bash
# Check twine configuration
twine check dist/*
# Reconfigure if needed
twine configure
```

#### Cargo Publish Issues

```bash
# Verify package before publishing
cargo package --dry-run
# Check credentials
cargo login --help
```

#### Test Failures

- Fix failing tests before publishing
- Use `--no-test` flag only for emergency releases
- Always run tests locally first

### Getting Help

If you encounter issues:

1. Check the error messages carefully
2. Verify all prerequisites are installed
3. Ensure registry credentials are correct
4. Check network connectivity
5. Review the SDK-specific documentation

## Security Considerations

- Never commit registry credentials to version control
- Use environment variables for sensitive information
- Regularly rotate API tokens
- Use two-factor authentication where available
- Review package contents before publishing

## CI/CD Integration

For automated publishing in CI/CD pipelines:

```yaml
# GitHub Actions example
name: Publish SDKs
on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: "18"
          registry-url: "https://registry.npmjs.org"
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: "3.9"
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Publish SDKs
        run: ./publish_sdks.sh --force
        env:
          NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
          TWINE_USERNAME: ${{ secrets.PYPI_USERNAME }}
          TWINE_PASSWORD: ${{ secrets.PYPI_PASSWORD }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
```

## Support

For publishing issues or questions:

- Check the troubleshooting section above
- Review SDK-specific documentation
- Open an issue in the project repository
- Contact the development team

---

**Note**: Always test the publishing process in a development environment before using in production.
