# Qdrant Compatibility Documentation

Complete documentation for using Vectorizer with Qdrant-compatible APIs.

## Overview

Vectorizer provides **REST API compatibility** with Qdrant v1.14.x for easy migration. This documentation covers compatibility details, limitations, troubleshooting, and migration guides.

## Documentation Index

### Getting Started
- [API Compatibility Matrix](./API_COMPATIBILITY.md) - Complete endpoint, parameter, and response compatibility
- [Feature Parity](./FEATURE_PARITY.md) - Feature comparison and limitations
- [Migration Guide](../../specs/QDRANT_MIGRATION.md) - Step-by-step migration from Qdrant

### Usage Guides
- [Examples](./EXAMPLES.md) - Code examples and tutorials
- [Troubleshooting](./TROUBLESHOOTING.md) - Common issues and solutions
- [Troubleshooting Examples](./TROUBLESHOOTING_EXAMPLES.md) - Practical troubleshooting scripts
- [Testing](./TESTING.md) - Testing compatibility and validation

## Quick Links

- **Base URL**: `http://localhost:15002/qdrant`
- **API Reference**: See [API Compatibility Matrix](./API_COMPATIBILITY.md)
- **Migration**: See [Migration Guide](../../specs/QDRANT_MIGRATION.md)
- **Native API**: See [API Reference](../api/API_REFERENCE.md)

## Important Notes

⚠️ **Qdrant compatibility is REST API only** - gRPC is not supported.

⚠️ **Recommendation**: Migrate to native Vectorizer APIs for better performance and features.

✅ **All REST endpoints are fully functional** - See compatibility matrix for details.

## Support

- Documentation: [docs.vectorizer.io](https://docs.vectorizer.io)
- Issues: [GitHub Issues](https://github.com/hivellm/vectorizer/issues)
- Community: [Discord](https://discord.gg/vectorizer)

