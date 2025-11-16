---
title: Monitoring
module: monitoring
id: monitoring-index
order: 0
description: Monitoring and observability for Vectorizer
tags: [monitoring, metrics, observability]
---

# Monitoring

Monitor Vectorizer performance, health, and metrics.

## Guides

### [Monitoring Guide](./MONITORING.md)
Complete monitoring and observability guide:
- Health checks
- Prometheus metrics
- Grafana dashboards
- Logging and log aggregation
- Alerting configuration
- Performance monitoring

## Quick Start

```bash
# Health check
curl http://localhost:15002/health

# Prometheus metrics
curl http://localhost:15002/metrics

# Check service status
sudo systemctl status vectorizer  # Linux
Get-Service Vectorizer  # Windows
```

## Related Topics

- [Performance Guide](../performance/PERFORMANCE.md) - Performance optimization
- [Troubleshooting Guide](../troubleshooting/TROUBLESHOOTING.md) - Debugging

