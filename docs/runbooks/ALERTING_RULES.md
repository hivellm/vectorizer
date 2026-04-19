# Vectorizer Alerting Rules

Prometheus alerting rules for monitoring Vectorizer deployments.

## Installation

Add these rules to your Prometheus configuration:

```yaml
# prometheus.yml
rule_files:
  - /etc/prometheus/rules/vectorizer-alerts.yml
```

## Alert Rules

### vectorizer-alerts.yml

```yaml
groups:
  - name: vectorizer-availability
    interval: 30s
    rules:
      # Instance Health
      - alert: VectorizerDown
        expr: up{job="vectorizer"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Vectorizer instance {{ $labels.instance }} is down"
          description: "Vectorizer instance has been unreachable for more than 1 minute."
          runbook_url: "https://docs.example.com/runbooks/vectorizer-down"

      - alert: VectorizerHighRestartRate
        expr: increase(process_start_time_seconds{job="vectorizer"}[1h]) > 3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer instance {{ $labels.instance }} restarting frequently"
          description: "Instance has restarted {{ $value }} times in the last hour."

  - name: vectorizer-performance
    interval: 30s
    rules:
      # Search Latency
      - alert: VectorizerSearchLatencyHigh
        expr: histogram_quantile(0.99, sum(rate(vectorizer_search_latency_seconds_bucket[5m])) by (le, instance)) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High search latency on {{ $labels.instance }}"
          description: "P99 search latency is {{ $value | humanizeDuration }} (threshold: 100ms)."

      - alert: VectorizerSearchLatencyCritical
        expr: histogram_quantile(0.99, sum(rate(vectorizer_search_latency_seconds_bucket[5m])) by (le, instance)) > 0.5
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical search latency on {{ $labels.instance }}"
          description: "P99 search latency is {{ $value | humanizeDuration }} (threshold: 500ms)."

      # Insert Latency
      - alert: VectorizerInsertLatencyHigh
        expr: histogram_quantile(0.99, sum(rate(vectorizer_insert_latency_seconds_bucket[5m])) by (le, instance)) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High insert latency on {{ $labels.instance }}"
          description: "P99 insert latency is {{ $value | humanizeDuration }}."

      # Request Queue
      - alert: VectorizerRequestQueueGrowing
        expr: rate(vectorizer_request_queue_size[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Request queue growing on {{ $labels.instance }}"
          description: "Request queue is growing at {{ $value }}/s."

  - name: vectorizer-resources
    interval: 30s
    rules:
      # Memory
      - alert: VectorizerMemoryHigh
        expr: vectorizer_memory_usage_bytes / vectorizer_memory_limit_bytes > 0.85
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage on {{ $labels.instance }}"
          description: "Memory usage is at {{ $value | humanizePercentage }}."

      - alert: VectorizerMemoryCritical
        expr: vectorizer_memory_usage_bytes / vectorizer_memory_limit_bytes > 0.95
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical memory usage on {{ $labels.instance }}"
          description: "Memory usage is at {{ $value | humanizePercentage }}. OOM risk."

      # CPU
      - alert: VectorizerCPUHigh
        expr: rate(process_cpu_seconds_total{job="vectorizer"}[5m]) > 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU usage is at {{ $value | humanizePercentage }}."

      # Disk
      - alert: VectorizerDiskSpaceLow
        expr: vectorizer_disk_free_bytes / vectorizer_disk_total_bytes < 0.15
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Low disk space on {{ $labels.instance }}"
          description: "Only {{ $value | humanizePercentage }} disk space remaining."

      - alert: VectorizerDiskSpaceCritical
        expr: vectorizer_disk_free_bytes / vectorizer_disk_total_bytes < 0.05
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical disk space on {{ $labels.instance }}"
          description: "Only {{ $value | humanizePercentage }} disk space remaining."

  - name: vectorizer-replication
    interval: 30s
    rules:
      # Replication Lag
      - alert: VectorizerReplicationLag
        expr: vectorizer_replication_lag_seconds > 60
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Replication lag on {{ $labels.instance }}"
          description: "Replica is {{ $value | humanizeDuration }} behind master."

      - alert: VectorizerReplicationLagCritical
        expr: vectorizer_replication_lag_seconds > 300
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical replication lag on {{ $labels.instance }}"
          description: "Replica is {{ $value | humanizeDuration }} behind master."

      # Replica Health
      - alert: VectorizerReplicaUnhealthy
        expr: vectorizer_replica_healthy == 0
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Unhealthy replica {{ $labels.instance }}"
          description: "Replica has been unhealthy for more than 2 minutes."

      # Master Failover
      - alert: VectorizerFailoverOccurred
        expr: increase(vectorizer_failover_total[5m]) > 0
        labels:
          severity: critical
        annotations:
          summary: "Failover occurred in Vectorizer cluster"
          description: "A master failover event was detected."

  - name: vectorizer-hub
    interval: 30s
    rules:
      # HiveHub Connection
      - alert: VectorizerHubConnectionFailed
        expr: vectorizer_hub_connection_healthy == 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "HiveHub connection failed on {{ $labels.instance }}"
          description: "Unable to connect to HiveHub API."

      # Authentication Failures
      - alert: VectorizerAuthFailuresHigh
        expr: rate(vectorizer_auth_failures_total[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High authentication failure rate on {{ $labels.instance }}"
          description: "{{ $value }} auth failures per second."

      # Quota Exceeded
      - alert: VectorizerQuotaExceeded
        expr: increase(vectorizer_quota_exceeded_total[1h]) > 100
        labels:
          severity: warning
        annotations:
          summary: "High quota exceeded rate"
          description: "{{ $value }} quota exceeded events in the last hour."

      # Rate Limiting
      - alert: VectorizerRateLimitingActive
        expr: rate(vectorizer_rate_limited_requests_total[5m]) > 50
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Rate limiting active on {{ $labels.instance }}"
          description: "{{ $value }} requests/s being rate limited."

  - name: vectorizer-backup
    interval: 60s
    rules:
      # Backup Age
      - alert: VectorizerBackupStale
        expr: time() - vectorizer_last_backup_timestamp_seconds > 86400
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer backup is stale"
          description: "Last backup was {{ $value | humanizeDuration }} ago."

      - alert: VectorizerBackupFailed
        expr: increase(vectorizer_backup_failures_total[1h]) > 0
        labels:
          severity: warning
        annotations:
          summary: "Vectorizer backup failed"
          description: "Backup operation failed {{ $value }} times in the last hour."

  - name: vectorizer-data-integrity
    interval: 60s
    rules:
      # Index Errors
      - alert: VectorizerIndexErrors
        expr: increase(vectorizer_index_errors_total[1h]) > 0
        labels:
          severity: warning
        annotations:
          summary: "Index errors detected on {{ $labels.instance }}"
          description: "{{ $value }} index errors in the last hour."

      # Data Corruption
      - alert: VectorizerChecksumErrors
        expr: increase(vectorizer_checksum_errors_total[1h]) > 0
        labels:
          severity: critical
        annotations:
          summary: "Data corruption detected on {{ $labels.instance }}"
          description: "{{ $value }} checksum errors detected. Investigate immediately."

  - name: vectorizer-tenant
    interval: 30s
    rules:
      # Per-Tenant Usage
      - alert: VectorizerTenantUsageHigh
        expr: vectorizer_tenant_storage_bytes / vectorizer_tenant_quota_bytes > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Tenant {{ $labels.tenant_id }} approaching storage quota"
          description: "Tenant is using {{ $value | humanizePercentage }} of storage quota."

      # Tenant Error Rate
      - alert: VectorizerTenantErrorsHigh
        expr: rate(vectorizer_tenant_errors_total[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate for tenant {{ $labels.tenant_id }}"
          description: "{{ $value }} errors/s for this tenant."
```

## Grafana Dashboards

### Overview Dashboard

Import the JSON dashboard from `docs/grafana/vectorizer-dashboard.json`.

Key panels:
- Request rate and latency
- Memory and CPU usage
- Active connections
- Collection statistics
- Error rates
- Replication status

### HiveHub Multi-Tenant Dashboard

Import from `docs/grafana/vectorizer-hub-dashboard.json`.

Key panels:
- Requests per tenant
- Storage usage per tenant
- Quota utilization
- Authentication metrics
- Rate limiting status

## PagerDuty Integration

```yaml
# alertmanager.yml
receivers:
  - name: vectorizer-critical
    pagerduty_configs:
      - service_key: YOUR_PAGERDUTY_SERVICE_KEY
        severity: critical
        description: '{{ .CommonAnnotations.summary }}'
        details:
          firing: '{{ template "pagerduty.default.instances" .Alerts.Firing }}'

  - name: vectorizer-warning
    slack_configs:
      - api_url: YOUR_SLACK_WEBHOOK_URL
        channel: '#vectorizer-alerts'
        title: '{{ .CommonAnnotations.summary }}'
        text: '{{ .CommonAnnotations.description }}'

route:
  receiver: vectorizer-warning
  group_by: [alertname, instance]
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  routes:
    - match:
        severity: critical
      receiver: vectorizer-critical
      continue: true
```

## OpsGenie Integration

```yaml
receivers:
  - name: vectorizer-opsgenie
    opsgenie_configs:
      - api_key: YOUR_OPSGENIE_API_KEY
        message: '{{ .CommonAnnotations.summary }}'
        description: '{{ .CommonAnnotations.description }}'
        priority: '{{ if eq .CommonLabels.severity "critical" }}P1{{ else if eq .CommonLabels.severity "warning" }}P3{{ else }}P5{{ end }}'
        tags: 'vectorizer,{{ .CommonLabels.alertname }}'
```

## Alert Silencing

During maintenance windows:

```bash
# Silence all Vectorizer alerts for 2 hours
amtool silence add alertname=~"Vectorizer.*" --duration=2h --comment="Planned maintenance"

# Silence specific instance
amtool silence add instance="vectorizer-1:15002" --duration=1h --comment="Node replacement"
```

## Testing Alerts

```bash
# Send test alert
curl -H "Content-Type: application/json" -d '[{
  "labels": {
    "alertname": "VectorizerTestAlert",
    "severity": "warning",
    "instance": "test:15002"
  },
  "annotations": {
    "summary": "Test alert from Vectorizer",
    "description": "This is a test alert to verify alerting pipeline."
  }
}]' http://alertmanager:9093/api/v1/alerts
```
