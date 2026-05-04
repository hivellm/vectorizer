using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Aggregate server statistics returned by GET /stats.
///
/// Phase25 §5 added <see cref="DefaultQuantization"/> and
/// <see cref="CompressionRatio"/>. Older servers without phase25 §5
/// leave these at their default ("none", 1.0f).
/// </summary>
public class Stats
{
    [JsonPropertyName("collections")]
    public int Collections { get; set; }

    [JsonPropertyName("total_vectors")]
    public long TotalVectors { get; set; }

    [JsonPropertyName("uptime_seconds")]
    public long UptimeSeconds { get; set; }

    [JsonPropertyName("version")]
    public string Version { get; set; } = string.Empty;

    [JsonPropertyName("default_quantization")]
    public string DefaultQuantization { get; set; } = "none";

    [JsonPropertyName("compression_ratio")]
    public float CompressionRatio { get; set; } = 1.0f;
}

/// <summary>
/// One sample in <see cref="CollectionInfo.VectorCountHistory"/>
/// (phase25 §6). Sampled at most once per minute on the read path.
/// </summary>
public class VectorCountSample
{
    [JsonPropertyName("at")]
    public long At { get; set; }

    [JsonPropertyName("count")]
    public int Count { get; set; }
}

/// <summary>
/// Per-route latency / throughput inside <see cref="RuntimeMetrics"/>.
/// </summary>
public class RouteStats
{
    [JsonPropertyName("route")]
    public string Route { get; set; } = string.Empty;

    [JsonPropertyName("qps")]
    public double Qps { get; set; }

    [JsonPropertyName("p50_ms")]
    public double P50Ms { get; set; }

    [JsonPropertyName("p99_ms")]
    public double P99Ms { get; set; }
}

/// <summary>
/// WAL state surfaced inside <see cref="RuntimeMetrics"/> (phase25 §3).
/// All fields are zero on standalone servers without replication.
/// </summary>
public class WalSnapshot
{
    [JsonPropertyName("current_seq")]
    public ulong CurrentSeq { get; set; }

    [JsonPropertyName("size_bytes")]
    public ulong SizeBytes { get; set; }

    [JsonPropertyName("last_checkpoint_at")]
    public ulong LastCheckpointAt { get; set; }

    [JsonPropertyName("last_checkpoint_seq")]
    public ulong LastCheckpointSeq { get; set; }
}

/// <summary>
/// Runtime metrics snapshot returned by GET /metrics/runtime
/// (phase25). Older servers without phase25 §4 may return zero-valued
/// defaults instead of a populated payload.
/// </summary>
public class RuntimeMetrics
{
    [JsonPropertyName("cpu_percent")]
    public double CpuPercent { get; set; }

    [JsonPropertyName("memory_rss_bytes")]
    public ulong MemoryRssBytes { get; set; }

    [JsonPropertyName("memory_total_bytes")]
    public ulong MemoryTotalBytes { get; set; }

    [JsonPropertyName("memory_percent")]
    public double MemoryPercent { get; set; }

    [JsonPropertyName("active_connections")]
    public int ActiveConnections { get; set; }

    [JsonPropertyName("uptime_seconds")]
    public ulong UptimeSeconds { get; set; }

    [JsonPropertyName("qps_window_60s")]
    public double QpsWindow60s { get; set; }

    [JsonPropertyName("error_rate_5xx_60s")]
    public double ErrorRate5xx60s { get; set; }

    [JsonPropertyName("throughput_by_route")]
    public List<RouteStats> ThroughputByRoute { get; set; } = new();

    [JsonPropertyName("wal")]
    public WalSnapshot Wal { get; set; } = new();
}

/// <summary>
/// Server liveness state returned by GET /status.
/// </summary>
public class ServerStatus
{
    [JsonPropertyName("online")]
    public bool Online { get; set; }

    [JsonPropertyName("version")]
    public string Version { get; set; } = string.Empty;

    [JsonPropertyName("uptime_seconds")]
    public long UptimeSeconds { get; set; }

    [JsonPropertyName("collections_count")]
    public int CollectionsCount { get; set; }
}

/// <summary>
/// One log line returned by GET /logs.
/// </summary>
public class LogEntry
{
    [JsonPropertyName("timestamp")]
    public string Timestamp { get; set; } = string.Empty;

    [JsonPropertyName("level")]
    public string Level { get; set; } = string.Empty;

    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;

    [JsonPropertyName("source")]
    public string? Source { get; set; }
}

/// <summary>
/// Indexing progress for a single collection.
/// </summary>
public class CollectionProgress
{
    [JsonPropertyName("collection_name")]
    public string CollectionName { get; set; } = string.Empty;

    [JsonPropertyName("status")]
    public string Status { get; set; } = string.Empty;

    [JsonPropertyName("progress")]
    public double Progress { get; set; }

    [JsonPropertyName("vector_count")]
    public int VectorCount { get; set; }

    [JsonPropertyName("error_message")]
    public string? ErrorMessage { get; set; }

    [JsonPropertyName("last_updated")]
    public string LastUpdated { get; set; } = string.Empty;
}

/// <summary>
/// Per-collection indexing progress returned by GET /indexing/progress.
/// </summary>
public class IndexingProgress
{
    [JsonPropertyName("is_indexing")]
    public bool IsIndexing { get; set; }

    [JsonPropertyName("overall_status")]
    public string OverallStatus { get; set; } = string.Empty;

    [JsonPropertyName("collections")]
    public IReadOnlyList<CollectionProgress> Collections { get; set; } = Array.Empty<CollectionProgress>();
}

/// <summary>
/// Server configuration as a free-form map.
/// </summary>
public class ConfigSnapshot : Dictionary<string, object> { }

/// <summary>
/// Metadata for one server-side backup file.
/// </summary>
public class BackupInfo
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("date")]
    public string Date { get; set; } = string.Empty;

    [JsonPropertyName("size")]
    public long Size { get; set; }

    [JsonPropertyName("collections")]
    public IReadOnlyList<string> Collections { get; set; } = Array.Empty<string>();
}

/// <summary>
/// Result of DELETE /collections/cleanup.
/// </summary>
public class CleanupReport
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("removed")]
    public int Removed { get; set; }

    [JsonPropertyName("collections")]
    public IReadOnlyList<string> Collections { get; set; } = Array.Empty<string>();

    [JsonPropertyName("message")]
    public string? Message { get; set; }
}

/// <summary>
/// A workspace configuration entry as a free-form map.
/// </summary>
public class WorkspaceConfig : Dictionary<string, object> { }

/// <summary>
/// A workspace item returned from ListWorkspaces.
/// </summary>
public class WorkspaceItem
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("path")]
    public string Path { get; set; } = string.Empty;

    [JsonPropertyName("collections")]
    public IReadOnlyList<string> Collections { get; set; } = Array.Empty<string>();
}

/// <summary>
/// Request body for POST /workspace/add.
/// </summary>
public class AddWorkspaceRequest
{
    [JsonPropertyName("path")]
    public string Path { get; set; } = string.Empty;

    [JsonPropertyName("collection_name")]
    public string CollectionName { get; set; } = string.Empty;
}

/// <summary>
/// Request body for POST /backups/create.
/// </summary>
public class CreateBackupRequest
{
    [JsonPropertyName("name")]
    public string? Name { get; set; }

    [JsonPropertyName("collections")]
    public List<string>? Collections { get; set; }
}

/// <summary>
/// Request body for POST /backups/restore.
/// </summary>
public class RestoreBackupRequest
{
    [JsonPropertyName("backup_id")]
    public string BackupId { get; set; } = string.Empty;
}
