using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Node replication role and state returned by GET /replication/status.
/// </summary>
public class ReplicationStatus
{
    [JsonPropertyName("role")]
    public string Role { get; set; } = string.Empty;

    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; }

    [JsonPropertyName("stats")]
    public ReplicationStats? Stats { get; set; }

    [JsonPropertyName("replicas")]
    public IReadOnlyList<ReplicaInfo>? Replicas { get; set; }
}

/// <summary>
/// Request body for POST /replication/configure.
/// </summary>
public class ReplicationConfig
{
    [JsonPropertyName("role")]
    public string Role { get; set; } = string.Empty;

    [JsonPropertyName("bind_address")]
    public string? BindAddress { get; set; }

    [JsonPropertyName("master_address")]
    public string? MasterAddress { get; set; }

    [JsonPropertyName("heartbeat_interval")]
    public long? HeartbeatInterval { get; set; }

    [JsonPropertyName("log_size")]
    public int? LogSize { get; set; }
}

/// <summary>
/// Raw replication statistics returned by GET /replication/stats.
/// </summary>
public class ReplicationStats
{
    [JsonPropertyName("role")]
    public string? Role { get; set; }

    [JsonPropertyName("bytes_sent")]
    public long? BytesSent { get; set; }

    [JsonPropertyName("bytes_received")]
    public long? BytesReceived { get; set; }

    [JsonPropertyName("last_sync")]
    public string? LastSync { get; set; }

    [JsonPropertyName("operations_pending")]
    public int? OperationsPending { get; set; }

    [JsonPropertyName("snapshot_size")]
    public int? SnapshotSize { get; set; }

    [JsonPropertyName("connected_replicas")]
    public int? ConnectedReplicas { get; set; }

    [JsonPropertyName("master_offset")]
    public long MasterOffset { get; set; }

    [JsonPropertyName("replica_offset")]
    public long ReplicaOffset { get; set; }

    [JsonPropertyName("lag_operations")]
    public long LagOperations { get; set; }

    [JsonPropertyName("total_replicated")]
    public long TotalReplicated { get; set; }
}

/// <summary>
/// Information about a replica node returned by GET /replication/replicas.
/// </summary>
public class ReplicaInfo
{
    [JsonPropertyName("replica_id")]
    public string ReplicaId { get; set; } = string.Empty;

    [JsonPropertyName("host")]
    public string Host { get; set; } = string.Empty;

    [JsonPropertyName("port")]
    public int Port { get; set; }

    [JsonPropertyName("status")]
    public string Status { get; set; } = string.Empty;

    [JsonPropertyName("last_heartbeat")]
    public string LastHeartbeat { get; set; } = string.Empty;

    [JsonPropertyName("operations_synced")]
    public long OperationsSynced { get; set; }

    [JsonPropertyName("offset")]
    public long? Offset { get; set; }

    [JsonPropertyName("lag")]
    public long? Lag { get; set; }
}
