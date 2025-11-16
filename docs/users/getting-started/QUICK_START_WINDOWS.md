---
title: Quick Start Guide for Windows
module: getting-started
id: quick-start-windows-guide
order: 2
description: Windows-specific quick start guide for Vectorizer
tags: [quick-start, windows, tutorial, getting-started]
---

# Quick Start Guide for Windows

Get started with Vectorizer on Windows!

## Prerequisites

- Windows 10/11 or Windows Server
- PowerShell 5.1+ or PowerShell Core
- Administrator privileges (for service installation)

## Installation

### Quick Install

```powershell
powershell -c "irm https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1 | iex"
```

This will:
- Install Vectorizer CLI
- Configure as Windows Service
- Start the service automatically

### Verify Installation

```powershell
# Check CLI version
vectorizer --version

# Check service status
Get-Service Vectorizer

# Test API endpoint
Invoke-WebRequest -Uri http://localhost:15002/health
```

## Quick Start

### Step 1: Create Collection

```powershell
$body = @{
    name = "my_documents"
    dimension = 384
    metric = "cosine"
} | ConvertTo-Json

Invoke-RestMethod -Uri http://localhost:15002/collections `
    -Method POST `
    -ContentType "application/json" `
    -Body $body
```

### Step 2: Insert Document

```powershell
$body = @{
    text = "Vectorizer is a high-performance vector database"
    metadata = @{
        source = "readme"
    }
} | ConvertTo-Json

Invoke-RestMethod -Uri http://localhost:15002/collections/my_documents/insert `
    -Method POST `
    -ContentType "application/json" `
    -Body $body
```

### Step 3: Search

```powershell
$body = @{
    query = "vector database"
    limit = 5
} | ConvertTo-Json

Invoke-RestMethod -Uri http://localhost:15002/collections/my_documents/search `
    -Method POST `
    -ContentType "application/json" `
    -Body $body
```

## Using PowerShell SDK

```powershell
# Install SDK (if available)
# Install-Module -Name VectorizerSDK

# Use SDK
$client = New-VectorizerClient -BaseUrl "http://localhost:15002"
New-Collection -Client $client -Name "my_docs" -Dimension 384
Add-Text -Client $client -Collection "my_docs" -Text "Hello, Vectorizer!"
Search-Collection -Client $client -Collection "my_docs" -Query "hello" -Limit 5
```

## Service Management

### Check Status

```powershell
Get-Service Vectorizer
```

### Start/Stop/Restart

```powershell
Start-Service Vectorizer
Stop-Service Vectorizer
Restart-Service Vectorizer
```

### View Logs

```powershell
Get-EventLog -LogName Application -Source Vectorizer -Newest 50
```

## Next Steps

- [Quick Start Guide](./QUICK_START.md) - General quick start
- [Collections Guide](../collections/COLLECTIONS.md) - Learn about collections
- [Service Management](../service-management/SERVICE_MANAGEMENT.md) - Manage the service
