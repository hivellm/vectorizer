---
title: Backup and Restore API
module: api
id: backup-restore-api
order: 8
description: Backup and restore operations for collections
tags: [api, backup, restore, data-protection]
---

# Backup and Restore API

The Backup and Restore API provides programmatic backup and restore operations for collections.

## Overview

Backup and restore operations enable:
- Creating backups of collections
- Restoring collections from backups
- Listing available backups
- Managing backup storage

## Backup Endpoints

### List Backups

List all available backups.

**Endpoint:** `GET /api/backups`

**Response:**

```json
{
  "backups": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "daily_backup_2024-01-15",
      "date": "2024-01-15T10:30:00Z",
      "size": 10485760,
      "collections": ["docs", "code"]
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "weekly_backup_2024-01-14",
      "date": "2024-01-14T10:30:00Z",
      "size": 52428800,
      "collections": ["docs", "code", "archive"]
    }
  ]
}
```

**Example:**

```bash
curl http://localhost:15002/api/backups
```

**Python SDK:**

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

backups = await client.list_backups()

for backup in backups["backups"]:
    print(f"Backup: {backup['name']}")
    print(f"  Date: {backup['date']}")
    print(f"  Size: {backup['size']} bytes")
    print(f"  Collections: {backup['collections']}")
```

### Create Backup

Create a backup of specified collections.

**Endpoint:** `POST /api/backups/create`

**Request Body:**

```json
{
  "name": "daily_backup_2024-01-15",
  "collections": ["docs", "code"]
}
```

**Parameters:**

| Parameter     | Type          | Required | Description                          |
| ------------- | ------------- | -------- | ------------------------------------ |
| `name`        | string        | Yes      | Backup name                          |
| `collections` | array[string] | No       | Collections to backup (default: all) |

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "daily_backup_2024-01-15",
  "date": "2024-01-15T10:30:00Z",
  "size": 10485760,
  "collections": ["docs", "code"]
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/api/backups/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "daily_backup_2024-01-15",
    "collections": ["docs", "code"]
  }'
```

**Python SDK:**

```python
backup = await client.create_backup(
    name="daily_backup_2024-01-15",
    collections=["docs", "code"]
)

print(f"Backup created: {backup['id']}")
print(f"Size: {backup['size']} bytes")
```

### Restore Backup

Restore collections from a backup.

**Endpoint:** `POST /api/backups/restore`

**Request Body:**

```json
{
  "backup_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Parameters:**

| Parameter    | Type   | Required | Description              |
| ------------ | ------ | -------- | ------------------------ |
| `backup_id`  | string | Yes      | Backup ID to restore     |

**Response:**

```json
{
  "success": true,
  "message": "Backup restored successfully"
}
```

**Warning:** Restoring a backup will overwrite existing collections with the same name.

**Example:**

```bash
curl -X POST http://localhost:15002/api/backups/restore \
  -H "Content-Type: application/json" \
  -d '{
    "backup_id": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

**Python SDK:**

```python
result = await client.restore_backup(
    backup_id="550e8400-e29b-41d4-a716-446655440000"
)

if result["success"]:
    print("Backup restored successfully")
```

### Get Backup Directory

Get the backup storage directory path.

**Endpoint:** `GET /api/backups/directory`

**Response:**

```json
{
  "path": "./backups"
}
```

**Example:**

```bash
curl http://localhost:15002/api/backups/directory
```

## Backup Format

Backups are stored as JSON files with the following structure:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "daily_backup_2024-01-15",
  "date": "2024-01-15T10:30:00Z",
  "collections": ["docs", "code"],
  "size": 10485760,
  "data": {
    "docs": {
      "config": {
        "dimension": 384,
        "metric": "cosine"
      },
      "vectors": [
        {
          "id": "vec_001",
          "vector": [0.1, 0.2, 0.3, ...],
          "metadata": {
            "text": "Document content",
            "source": "file.md"
          }
        }
      ]
    }
  }
}
```

## Use Cases

### Automated Daily Backups

Create automated daily backups:

```python
import asyncio
from datetime import datetime
from vectorizer_sdk import VectorizerClient

async def create_daily_backup():
    client = VectorizerClient("http://localhost:15002")
    
    # Get all collections
    collections_response = await client.list_collections()
    collection_names = [c["name"] for c in collections_response["collections"]]
    
    # Create backup with timestamp
    backup_name = f"daily_backup_{datetime.now().strftime('%Y-%m-%d')}"
    
    backup = await client.create_backup(
        name=backup_name,
        collections=collection_names
    )
    
    print(f"Daily backup created: {backup['id']}")

# Run daily
asyncio.run(create_daily_backup())
```

### Selective Collection Backup

Backup specific collections:

```python
# Backup only important collections
important_collections = ["docs", "code", "production_data"]

backup = await client.create_backup(
    name="important_collections_backup",
    collections=important_collections
)
```

### Backup Before Updates

Create backup before major updates:

```python
# Create backup before update
backup = await client.create_backup(
    name="pre_update_backup",
    collections=["docs"]
)

# Perform updates
await client.batch_insert_text("docs", new_documents)

# If something goes wrong, restore
if update_failed:
    await client.restore_backup(backup_id=backup["id"])
```

### Backup Management

Manage backups programmatically:

```python
# List all backups
backups = await client.list_backups()

# Find backups older than 7 days
from datetime import datetime, timedelta
cutoff_date = datetime.now() - timedelta(days=7)

old_backups = [
    b for b in backups["backups"]
    if datetime.fromisoformat(b["date"].replace("Z", "+00:00")) < cutoff_date
]

# Restore most recent backup
if backups["backups"]:
    latest_backup = max(
        backups["backups"],
        key=lambda b: b["date"]
    )
    await client.restore_backup(backup_id=latest_backup["id"])
```

## Best Practices

1. **Regular backups**: Create backups on a regular schedule
2. **Test restores**: Periodically test restore operations
3. **Backup naming**: Use descriptive names with timestamps
4. **Selective backups**: Backup only necessary collections to save space
5. **Backup storage**: Ensure backup directory has sufficient space
6. **Backup retention**: Implement retention policy to manage old backups
7. **Backup verification**: Verify backup integrity after creation

## Backup Storage

Backups are stored in the `./backups` directory by default. Each backup is saved as a JSON file named `{backup_id}.backup`.

**Backup File Location:**

```
./backups/
  ├── 550e8400-e29b-41d4-a716-446655440000.backup
  ├── 660e8400-e29b-41d4-a716-446655440001.backup
  └── ...
```

**Custom Backup Directory:**

To use a custom backup directory, ensure the directory exists and has write permissions. The backup API will use the configured directory.

## Related Topics

- [Operations Guide](../operations/BACKUP.md) - Backup procedures and best practices
- [Collections Guide](../collections/COLLECTIONS.md) - Collection management
- [Configuration Guide](../configuration/CONFIGURATION.md) - Server configuration

