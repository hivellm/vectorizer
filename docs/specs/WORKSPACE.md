# Workspace Management

**Version**: 1.0  
**Status**: ✅ Production Ready  
**Last Updated**: 2025-09-25

---

## Overview

Workspace management system for organizing and indexing multiple projects through `workspace.yml` configuration.

---

## Configuration

### Basic Workspace

```yaml
projects:
  - name: "my-project"
    path: "../my-project"
    description: "Project description"
    collections:
      - name: "docs"
        description: "Documentation"
        include_patterns:
          - "docs/**/*.md"
          - "*.md"
        exclude_patterns:
          - "**/node_modules/**"
          - "**/target/**"
```

### Advanced Features

**File Watcher**:
```yaml
global_settings:
  file_watcher:
    watch_paths:
      - "/path/to/project"
    auto_discovery: true
    enable_auto_update: true
    hot_reload: true
```

**Collection Settings**:
```yaml
collections:
  - name: "source-code"
    include_patterns:
      - "src/**/*.rs"
      - "src/**/*.ts"
    exclude_patterns:
      - "**/test/**"
      - "**/*.test.*"
    chunking:
      size: 2048
      overlap: 256
```

---

## Features

**Auto-Discovery**: Automatically detect projects and source files  
**Hot Reload**: Automatic reindexing on file changes  
**Multi-Project**: Support multiple projects in single workspace  
**Pattern Matching**: Flexible include/exclude patterns

---

## Management UI

**Dashboard Access**: `http://localhost:15002/dashboard`

**Features**:
- View all projects and collections
- Monitor indexing progress
- Manage workspace configuration
- Real-time file watcher status

---

**Status**: ✅ Production Ready  
**Maintained by**: HiveLLM Team

