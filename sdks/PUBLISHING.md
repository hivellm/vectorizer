# SDK Publishing Guide

This guide explains how to publish all Hive Vectorizer client SDKs to their respective package registries using the automated publishing scripts.

## Prerequisites

Before publishing, ensure you have the following tools installed and configured:

### Required Tools

- **Node.js & npm** - For the TypeScript SDK
- **Python & pip** - For Python SDK
- **Rust & Cargo** - For Rust SDK
- **Git** - For version control

### Registry Credentials

#### npm (TypeScript SDK)

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

- **TypeScript SDK**: 352+ tests (✅ Published v3.0.0)
- **Python SDK**: 184+ tests (✅ Published v3.0.0)
- **Rust SDK**: 88+ tests (✅ Published v3.0.0)
- **C# SDK**: Tests included (✅ Published v3.0.0; `Vectorizer.Sdk.Rpc` adds 54 framing/transport tests)
- **Go SDK**: Tests included (✅ Released v3.0.0)

## Manual Publishing (Alternative)

If you prefer to publish manually or need more control:

### TypeScript SDK

```bash
cd typescript
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

#### TypeScript SDK

```json
// package.json
{
  "version": "3.0.0"
}
```

#### Python SDK

```python
# pyproject.toml
[project]
version = "3.0.0"
```

#### Rust SDK

```toml
# Cargo.toml
[package]
version = "3.0.0"
```

#### C# SDK

```xml
<!-- Vectorizer.csproj -->
<Version>3.0.0</Version>
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

## v3.0.0 Publish Runbook

This is the one-command-per-SDK sequence for cutting a v3 release.
Every command below is **token-gated** — the maintainer runs them
from a workstation that has the registry credentials below loaded:

| Credential | Scope | Load via |
|---|---|---|
| `NPM_TOKEN` | `@hivehub/vectorizer-sdk` | `npm login` (or `.npmrc` with `//registry.npmjs.org/:_authToken=$NPM_TOKEN`) |
| `TWINE_PASSWORD` | `vectorizer` on PyPI | `~/.pypirc` or `TWINE_PASSWORD=$PYPI_TOKEN` env |
| `CARGO_REGISTRY_TOKEN` | `vectorizer-sdk` on crates.io | `cargo login $CARGO_REGISTRY_TOKEN` |
| `NUGET_API_KEY` | `Vectorizer.Sdk.Rpc` on NuGet | `dotnet nuget push --api-key $NUGET_API_KEY` |
| `GITHUB_TOKEN` | `sdks/go/v3.0.0` tag push | `git push origin sdks/go/v3.0.0` |

**Pre-publish gate**: confirm every per-SDK follow-up task under
`phase8_fix-*-sdk-*` is archived (see
`.rulebook/archive/2026-04-21-*/`).

### 1. TypeScript — `@hivehub/vectorizer-sdk`

```bash
cd sdks/typescript
pnpm build
pnpm test             # vitest; must be green
pnpm pack             # verify tarball — pnpm prints the file list
pnpm publish --access public
```

After publish, re-run `cd gui && rm -rf node_modules pnpm-lock.yaml
&& pnpm install` to refresh the lockfile against the just-published
SDK. A successful lockfile refresh unblocks the GUI Electron bump
and the minimatch transitive alert.

### 2. Rust — `vectorizer-sdk` on crates.io

`vectorizer-sdk` depends on `vectorizer-protocol` (path-only in the
workspace); crates.io requires both to carry `version = ` in the
dep declaration AND for `vectorizer-protocol` to be published FIRST.

```bash
# Publish the protocol crate FIRST — the SDK's published manifest
# strips the `path = ...` and resolves `vectorizer-protocol` from
# crates.io on downstream builds.
cd crates/vectorizer-protocol
cargo publish --dry-run --allow-dirty   # verify
cargo publish

# Wait ~1 minute for the crates.io index to pick up the new version,
# then:
cd ../../sdks/rust
# One-time manifest tweak if not already landed: add
# `version = "3.0.0"` to the `vectorizer-protocol` dep in Cargo.toml.
cargo publish --dry-run --allow-dirty
cargo publish
```

### 3. Python — `vectorizer` on PyPI

```bash
cd sdks/python
python -m build
twine check dist/*
twine upload dist/vectorizer-3.0.0*
```

### 4. C# — `Vectorizer.Sdk.Rpc` on NuGet

```bash
cd sdks/csharp/src/Vectorizer.Rpc
dotnet pack -c Release
dotnet nuget push bin/Release/Vectorizer.Sdk.Rpc.3.0.0.nupkg \
  --source https://api.nuget.org/v3/index.json \
  --api-key "$NUGET_API_KEY"
```

### 5. Go — `github.com/hivellm/vectorizer-sdk-go`

Go modules are published via git tags on the module directory:

```bash
cd sdks/go
# Make sure the go.mod path matches `github.com/hivellm/vectorizer-sdk-go`
git tag sdks/go/v3.0.0
git push origin sdks/go/v3.0.0

# Verify from a clean checkout:
go install github.com/hivellm/vectorizer-sdk-go@v3.0.0
```

### 6. Post-publish smoke-install verification

The `sdk-publish-smoke` CI workflow
([.github/workflows/sdk-publish-smoke.yml](.github/workflows/sdk-publish-smoke.yml))
installs each just-published SDK from its canonical registry and
exercises a one-line import / construct so a broken tarball surfaces
before any downstream consumer hits it. It fires automatically on a
tag matching `v3.*.*` and can be triggered manually via
`workflow_dispatch` with a `version` input.

Each language job is independent — a failure in one SDK does not
mask another so operators see the full health picture in a single
run. None of the jobs carry publish credentials; they are pure
post-publish validators.

### 7. After all five publishes are green

Update `sdks/PUBLISHING_STATUS.md` with the published timestamp +
the tarball / crate / nupkg / module URL for each language, and
commit the change.

## Support

For publishing issues or questions:

- Check the troubleshooting section above
- Review SDK-specific documentation
- Open an issue in the project repository
- Contact the development team

---

**Note**: Always test the publishing process in a development environment before using in production.
