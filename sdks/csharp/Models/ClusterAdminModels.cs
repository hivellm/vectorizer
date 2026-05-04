using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Result of POST /cluster/failover.
/// </summary>
public class FailoverReport
{
    [JsonPropertyName("promoted_replica_id")]
    public string PromotedReplicaId { get; set; } = string.Empty;

    [JsonPropertyName("master_offset_at_promotion")]
    public long MasterOffsetAtPromotion { get; set; }

    [JsonPropertyName("replica_offset_at_promotion")]
    public long ReplicaOffsetAtPromotion { get; set; }

    [JsonPropertyName("residual_lag_operations")]
    public long ResidualLagOperations { get; set; }
}

/// <summary>
/// Result of POST /cluster/replicas/{id}/resync.
/// </summary>
public class ResyncJob
{
    [JsonPropertyName("replica_id")]
    public string ReplicaId { get; set; } = string.Empty;

    [JsonPropertyName("snapshot_offset")]
    public long SnapshotOffset { get; set; }

    [JsonPropertyName("full_snapshot")]
    public bool FullSnapshot { get; set; }
}

/// <summary>
/// Request body for POST /cluster/peers.
/// </summary>
public class AddPeerRequest
{
    [JsonPropertyName("address")]
    public string Address { get; set; } = string.Empty;

    [JsonPropertyName("role")]
    public string? Role { get; set; }
}

/// <summary>
/// A cluster peer returned by GET /cluster/peers.
/// </summary>
public class PeerInfo
{
    [JsonPropertyName("node_id")]
    public string NodeId { get; set; } = string.Empty;

    [JsonPropertyName("address")]
    public string Address { get; set; } = string.Empty;

    [JsonPropertyName("role")]
    public string Role { get; set; } = string.Empty;
}

/// <summary>
/// Rebalance job descriptor returned by POST /cluster/rebalance.
/// </summary>
public class RebalanceJob
{
    [JsonPropertyName("job_id")]
    public string JobId { get; set; } = string.Empty;

    [JsonPropertyName("status")]
    public string Status { get; set; } = string.Empty;

    [JsonPropertyName("shards_to_move")]
    public int ShardsToMove { get; set; }

    [JsonPropertyName("shards_moved")]
    public int ShardsMoved { get; set; }

    [JsonPropertyName("last_checkpoint_node")]
    public string? LastCheckpointNode { get; set; }

    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;
}

/// <summary>
/// Query parameters for filtering the admin audit log.
/// </summary>
public class AuditQuery
{
    [JsonPropertyName("actor")]
    public string? Actor { get; set; }

    [JsonPropertyName("action")]
    public string? Action { get; set; }

    [JsonPropertyName("since")]
    public string? Since { get; set; }

    [JsonPropertyName("until")]
    public string? Until { get; set; }

    [JsonPropertyName("limit")]
    public int? Limit { get; set; }
}
