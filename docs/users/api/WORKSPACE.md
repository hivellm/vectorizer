---
title: Workspace Management API
module: api
id: workspace-api
order: 9
description: Workspace management for multi-project indexing
tags: [api, workspace, projects, indexing]
---

# Workspace Management API

The Workspace Management API enables managing multiple projects and collections through workspace configuration.

## Overview

Workspace management enables:

- Adding and removing workspace directories
- Listing configured workspaces
- Managing workspace configuration
- Multi-project indexing
- File watcher integration

## Workspace Endpoints

### List Workspaces

List all configured workspace directories.

**Endpoint:** `GET /workspace/list`

**Response:**

```json
{
  "workspaces": [
    {
      "path": "/path/to/project1",
      "collection_name": "project1_docs",
      "status": "active"
    },
    {
      "path": "/path/to/project2",
      "collection_name": "project2_code",
      "status": "active"
    }
  ]
}
```

**Example:**

```bash
curl http://localhost:15002/workspace/list
```

**Python SDK:**

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

workspaces = await client.list_workspaces()

for workspace in workspaces["workspaces"]:
    print(f"Path: {workspace['path']}")
    print(f"Collection: {workspace['collection_name']}")
```

### Add Workspace

Add a new workspace directory for indexing.

**Endpoint:** `POST /workspace/add`

**Request Body:**

```json
{
  "path": "/path/to/project",
  "collection_name": "project_docs"
}
```

**Parameters:**

| Parameter         | Type   | Required | Description                         |
| ----------------- | ------ | -------- | ----------------------------------- |
| `path`            | string | Yes      | Directory path to index             |
| `collection_name` | string | Yes      | Collection name for indexed content |

**Response:**

```json
{
  "success": true,
  "message": "Workspace added successfully"
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/workspace/add \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/path/to/project",
    "collection_name": "project_docs"
  }'
```

**Python SDK:**

```python
result = await client.add_workspace(
    path="/path/to/project",
    collection_name="project_docs"
)

if result["success"]:
    print("Workspace added successfully")
```

### Remove Workspace

Remove a workspace directory from indexing.

**Endpoint:** `POST /workspace/remove`

**Request Body:**

```json
{
  "path": "/path/to/project"
}
```

**Parameters:**

| Parameter | Type   | Required | Description              |
| --------- | ------ | -------- | ------------------------ |
| `path`    | string | Yes      | Directory path to remove |

**Response:**

```json
{
  "success": true,
  "message": "Workspace removed successfully"
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/workspace/remove \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/path/to/project"
  }'
```

**Python SDK:**

```python
result = await client.remove_workspace(path="/path/to/project")

if result["success"]:
    print("Workspace removed successfully")
```

### Get Workspace Configuration

Get workspace configuration settings.

**Endpoint:** `GET /workspace/config`

**Response:**

```json
{
  "file_watcher": {
    "enabled": true,
    "watch_paths": ["/path/to/project"],
    "auto_discovery": true,
    "enable_auto_update": true,
    "hot_reload": true
  },
  "projects": [
    {
      "name": "my-project",
      "path": "/path/to/project",
      "collections": [
        {
          "name": "docs",
          "include_patterns": ["docs/**/*.md"],
          "exclude_patterns": ["**/node_modules/**"]
        }
      ]
    }
  ]
}
```

**Example:**

```bash
curl http://localhost:15002/workspace/config
```

### Update Workspace Configuration

Update workspace configuration settings.

**Endpoint:** `POST /workspace/config`

**Request Body:**

```json
{
  "file_watcher": {
    "enabled": true,
    "watch_paths": ["/path/to/project"],
    "auto_discovery": true,
    "enable_auto_update": true,
    "hot_reload": true
  },
  "projects": [
    {
      "name": "my-project",
      "path": "/path/to/project",
      "collections": [
        {
          "name": "docs",
          "include_patterns": ["docs/**/*.md"],
          "exclude_patterns": ["**/node_modules/**"]
        }
      ]
    }
  ]
}
```

**Response:**

```json
{
  "success": true,
  "message": "Workspace configuration updated successfully"
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/workspace/config \
  -H "Content-Type: application/json" \
  -d @workspace-config.json
```

## Workspace Configuration File

Workspaces are configured via `workspace.yml`:

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

global_settings:
  file_watcher:
    watch_paths:
      - "/path/to/project"
    auto_discovery: true
    enable_auto_update: true
    hot_reload: true
```

## Use Cases

### Multi-Project Indexing

Index multiple projects into separate collections:

```python
# Add multiple projects
projects = [
    {"path": "/path/to/project1", "collection": "project1_docs"},
    {"path": "/path/to/project2", "collection": "project2_code"},
    {"path": "/path/to/project3", "collection": "project3_docs"}
]

for project in projects:
    await client.add_workspace(
        path=project["path"],
        collection_name=project["collection"]
    )
```

### Dynamic Workspace Management

Manage workspaces dynamically:

```python
# List current workspaces
workspaces = await client.list_workspaces()

# Add new workspace
await client.add_workspace(
    path="/new/project",
    collection_name="new_project"
)

# Remove workspace
await client.remove_workspace(path="/old/project")
```

### Workspace Configuration

Manage workspace configuration:

```python
# Get current configuration
config = await client.get_workspace_config()

# Update configuration
config["file_watcher"]["enabled"] = True
config["file_watcher"]["watch_paths"] = ["/path/to/project"]

await client.update_workspace_config(config)
```

## Related Topics

- [File Operations API](./FILE_OPERATIONS.md) - File-level operations
- [Collections Guide](../collections/COLLECTIONS.md) - Collection management
- [Configuration Guide](../configuration/CONFIGURATION.md) - Server configuration
