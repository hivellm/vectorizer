---
title: Troubleshooting Guide
module: troubleshooting
id: troubleshooting-guide
order: 1
description: Common issues and solutions for Vectorizer
tags: [troubleshooting, problems, issues, solutions]
---

# Troubleshooting Guide

Common issues and their solutions.

## Service Issues

### Service Won't Start

**Symptoms**: Service fails to start or immediately stops.

**Solutions**:
1. Check logs: `sudo journalctl -u vectorizer -n 50` (Linux) or `Get-EventLog -LogName Application -Source Vectorizer -Newest 50` (Windows)
2. Verify port 15002 is not in use: `netstat -tuln | grep 15002` (Linux) or `netstat -ano | findstr 15002` (Windows)
3. Check binary permissions: `ls -l /usr/local/bin/vectorizer` (Linux)
4. Verify user exists: `id vectorizer` (Linux)

### Service Keeps Restarting

**Symptoms**: Service starts but immediately restarts in a loop.

**Solutions**:
1. Check logs for crash reasons
2. Verify configuration is valid
3. Check system resources (memory, disk space)
4. Review recent changes to configuration

## Connection Issues

### Cannot Connect to API

**Symptoms**: `curl http://localhost:15002/health` fails.

**Solutions**:
1. Verify service is running: `sudo systemctl status vectorizer` (Linux) or `Get-Service Vectorizer` (Windows)
2. Check firewall settings
3. Verify host binding: Should be `0.0.0.0` or `127.0.0.1`
4. Check port availability

### Connection Refused

**Symptoms**: Connection refused errors.

**Solutions**:
1. Service may not be running
2. Port may be blocked by firewall
3. Service may be binding to wrong interface

## Performance Issues

### Slow Searches

**Symptoms**: Search operations take too long.

**Solutions**:
1. Check collection size and dimension
2. Verify HNSW index is built
3. Consider enabling quantization
4. Check system resources (CPU, memory)

### High Memory Usage

**Symptoms**: Vectorizer uses too much memory.

**Solutions**:
1. Enable quantization to reduce memory
2. Reduce collection size
3. Check for memory leaks in logs
4. Consider using compression

## Data Issues

### Collections Not Found

**Symptoms**: "Collection not found" errors.

**Solutions**:
1. Verify collection exists: `curl http://localhost:15002/collections`
2. Check collection name spelling
3. Verify data directory permissions

### Data Loss

**Symptoms**: Collections or vectors disappear.

**Solutions**:
1. Check data directory: `/var/lib/vectorizer` (Linux) or `%ProgramData%\Vectorizer` (Windows)
2. Verify disk space
3. Check logs for errors
4. Review backup procedures

## Installation Issues

### Binary Not Found

**Symptoms**: `vectorizer: command not found`.

**Solutions**:
1. Verify installation: `which vectorizer` (Linux) or `Get-Command vectorizer` (Windows)
2. Check PATH environment variable
3. Reinstall if necessary

### Permission Denied

**Symptoms**: Permission errors during installation or operation.

**Solutions**:
1. Use `sudo` for Linux operations requiring root
2. Run PowerShell as Administrator on Windows
3. Check file permissions

## Getting Help

If you're still experiencing issues:

1. Check the [main documentation](../../README.md)
2. Review [API Reference](../../specs/API_REFERENCE.md)
3. Check logs for detailed error messages
4. Open an issue on GitHub with:
   - Error messages
   - Steps to reproduce
   - System information
   - Relevant logs

## Related Topics

- [Service Management](../service-management/SERVICE_MANAGEMENT.md) - Service troubleshooting
- [Configuration](../configuration/CONFIGURATION.md) - Configuration issues
- [Installation Guide](../installation/INSTALLATION.md) - Installation problems

