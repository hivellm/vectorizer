using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Aggregate server statistics returned by GET /stats.
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
