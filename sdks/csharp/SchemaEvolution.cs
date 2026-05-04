using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    private sealed class SnapshotsEnvelope
    {
        [System.Text.Json.Serialization.JsonPropertyName("snapshots")]
        public List<NativeSnapshotInfo>? Snapshots { get; set; }
    }

    private sealed class SlowQueryEntriesEnvelope
    {
        [System.Text.Json.Serialization.JsonPropertyName("entries")]
        public List<SlowQueryEntry>? Entries { get; set; }
    }

    /// <summary>
    /// Atomically renames a collection (POST /collections/{name}/rename).
    /// The server retains the old name as an in-memory alias for one minor
    /// version so existing clients can continue to work without reconfiguration.
    /// </summary>
    /// <param name="oldName">Current collection name.</param>
    /// <param name="newName">New collection name.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task RenameCollectionAsync(
        string oldName,
        string newName,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, string> { ["new_name"] = newName };
        await RequestAsync<object>(
            "POST",
            $"/collections/{Uri.EscapeDataString(oldName)}/rename",
            body,
            cancellationToken);
    }

    /// <summary>
    /// Rebuilds the HNSW index for a collection with new parameters
    /// (POST /collections/{name}/reindex).
    /// No re-embedding is required — existing stored vectors are reused.
    /// Returns a <see cref="ReindexJob"/> with State == "completed" on success.
    /// </summary>
    /// <param name="name">Collection name.</param>
    /// <param name="parameters">HNSW rebuild parameters.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task<ReindexJob> ReindexCollectionAsync(
        string name,
        ReindexParams parameters,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ReindexJob>(
            "POST",
            $"/collections/{Uri.EscapeDataString(name)}/reindex",
            parameters,
            cancellationToken);
    }

    /// <summary>
    /// Creates a native per-collection snapshot (POST /collections/{name}/snapshot).
    /// The server writes a gzip-compressed JSON snapshot and returns snapshot
    /// metadata including ID, collection name, creation time, and size.
    /// </summary>
    /// <param name="name">Collection name.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task<NativeSnapshotInfo> SnapshotCollectionNativeAsync(
        string name,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<NativeSnapshotInfo>(
            "POST",
            $"/collections/{Uri.EscapeDataString(name)}/snapshot",
            new Dictionary<string, object>(),
            cancellationToken);
    }

    /// <summary>
    /// Lists all native snapshots for a collection (GET /collections/{name}/snapshots).
    /// Returns snapshots newest-first as reported by the server.
    /// </summary>
    /// <param name="name">Collection name.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task<IReadOnlyList<NativeSnapshotInfo>> ListCollectionSnapshotsNativeAsync(
        string name,
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<SnapshotsEnvelope>(
            "GET",
            $"/collections/{Uri.EscapeDataString(name)}/snapshots",
            null,
            cancellationToken);
        return envelope.Snapshots ?? new List<NativeSnapshotInfo>();
    }

    /// <summary>
    /// Restores a collection from a native snapshot
    /// (POST /collections/{name}/snapshots/{id}/restore).
    /// Drops the current in-memory state and replaces it with the snapshot data.
    /// </summary>
    /// <param name="name">Collection name.</param>
    /// <param name="snapshotId">Snapshot identifier returned by <see cref="SnapshotCollectionNativeAsync"/>.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task RestoreCollectionSnapshotNativeAsync(
        string name,
        string snapshotId,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "POST",
            $"/collections/{Uri.EscapeDataString(name)}/snapshots/{Uri.EscapeDataString(snapshotId)}/restore",
            new Dictionary<string, object>(),
            cancellationToken);
    }

    /// <summary>
    /// Runs a vector search and returns the full HNSW execution trace
    /// (POST /collections/{name}/explain).
    /// The trace includes visited_nodes, ef_search, hnsw_search_ms,
    /// payload_filter_evals, quantization_score_ms, and total_ms.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="vector">Query vector.</param>
    /// <param name="k">Number of nearest neighbours to retrieve.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task<ExplainResponse> ExplainSearchAsync(
        string collection,
        IReadOnlyList<float> vector,
        int k,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["vector"] = vector,
            ["k"] = k
        };
        return await RequestAsync<ExplainResponse>(
            "POST",
            $"/collections/{Uri.EscapeDataString(collection)}/explain",
            body,
            cancellationToken);
    }

    /// <summary>
    /// Returns entries from the slow-query ring buffer (GET /slow_queries).
    /// Entries are returned in the order they were recorded (oldest first).
    /// Use <see cref="SetSlowQueryConfigAsync"/> to tune the threshold and capacity.
    /// </summary>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task<IReadOnlyList<SlowQueryEntry>> ListSlowQueriesAsync(
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<SlowQueryEntriesEnvelope>(
            "GET",
            "/slow_queries",
            null,
            cancellationToken);
        return envelope.Entries ?? new List<SlowQueryEntry>();
    }

    /// <summary>
    /// Reconfigures the slow-query ring buffer (POST /slow_queries/config).
    /// Existing entries are retained. If the new capacity is smaller than the
    /// current entry count the oldest entries are evicted by the server.
    /// Returns the updated configuration as echoed back by the server.
    /// </summary>
    /// <param name="config">New slow-query configuration.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task<SlowQueryConfig> SetSlowQueryConfigAsync(
        SlowQueryConfig config,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<SlowQueryConfig>(
            "POST",
            "/slow_queries/config",
            config,
            cancellationToken);
    }
}
