using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Returns the current replication status and role of this node.
    /// Calls GET /replication/status.
    /// </summary>
    public async Task<ReplicationStatus> GetReplicationStatusAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ReplicationStatus>(
            "GET", "/replication/status", null, cancellationToken);
    }

    /// <summary>
    /// Configures this node's replication role and parameters.
    /// Calls POST /replication/configure.
    /// A server restart is required for the new configuration to take effect.
    /// </summary>
    public async Task ConfigureReplicationAsync(
        ReplicationConfig config,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "POST", "/replication/configure", config, cancellationToken);
    }

    /// <summary>
    /// Returns raw replication statistics for the active replication node.
    /// Calls GET /replication/stats.
    /// Returns an error when replication is not enabled on this node.
    /// </summary>
    public async Task<ReplicationStats> GetReplicationStatsAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ReplicationStats>(
            "GET", "/replication/stats", null, cancellationToken);
    }

    /// <summary>
    /// Returns the replica nodes connected to this master.
    /// Calls GET /replication/replicas.
    /// Only available on master nodes; returns an error otherwise.
    /// </summary>
    public async Task<IReadOnlyList<ReplicaInfo>> ListReplicasAsync(
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<ReplicasEnvelope>(
            "GET", "/replication/replicas", null, cancellationToken);
        return envelope.Replicas ?? Array.Empty<ReplicaInfo>();
    }

    // Private envelope type for the {replicas: [...]} response shape.
    private sealed class ReplicasEnvelope
    {
        [System.Text.Json.Serialization.JsonPropertyName("replicas")]
        public IReadOnlyList<ReplicaInfo>? Replicas { get; set; }
    }
}
