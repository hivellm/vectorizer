using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Vectorizer.Rpc;

// ── Response DTOs ─────────────────────────────────────────────────────────────

/// <summary>
/// Metadata returned by the <c>collections.get_info</c> command.
/// </summary>
public sealed class CollectionInfo
{
    public string Name { get; init; } = string.Empty;
    public long VectorCount { get; init; }
    public long DocumentCount { get; init; }
    public long Dimension { get; init; }
    public string Metric { get; init; } = string.Empty;
    public string CreatedAt { get; init; } = string.Empty;
    public string UpdatedAt { get; init; } = string.Empty;
}

/// <summary>One result from <c>search.basic</c>.</summary>
public sealed class SearchHit
{
    public string Id { get; init; } = string.Empty;
    public double Score { get; init; }
    /// <summary>Optional JSON-encoded payload. The server stores payloads as <c>serde_json::Value</c>.</summary>
    public string? Payload { get; init; }
}

/// <summary>Response from <c>collections.create</c>.</summary>
public sealed class CreateCollectionResult
{
    public string Name { get; init; } = string.Empty;
    public long Dimension { get; init; }
    public string Metric { get; init; } = string.Empty;
    public bool Success { get; init; }
}

/// <summary>Response from <c>collections.cleanup_empty</c>.</summary>
public sealed class CleanupEmptyResult
{
    public long Removed { get; init; }
    public bool DryRun { get; init; }
}

/// <summary>Response from <c>vectors.insert</c>, <c>vectors.insert_text</c>, and <c>vectors.update</c>.</summary>
public sealed class VectorWriteResult
{
    public string Id { get; init; } = string.Empty;
    public bool Success { get; init; }
}

/// <summary>Per-item result inside batch operation responses.</summary>
public sealed class BatchItemResult
{
    public long Index { get; init; }
    public string? Id { get; init; }
    public string Status { get; init; } = string.Empty;
    public string? Error { get; init; }
}

/// <summary>Response from <c>vectors.batch_insert</c> and <c>vectors.batch_insert_texts</c>.</summary>
public sealed class BatchInsertResult
{
    public long Inserted { get; init; }
    public long Failed { get; init; }
    public IReadOnlyList<BatchItemResult> Results { get; init; } = Array.Empty<BatchItemResult>();
}

/// <summary>Response from <c>vectors.batch_update</c>.</summary>
public sealed class BatchUpdateResult
{
    public long Updated { get; init; }
    public long Failed { get; init; }
    public IReadOnlyList<BatchItemResult> Results { get; init; } = Array.Empty<BatchItemResult>();
}

/// <summary>Response from <c>vectors.batch_delete</c>.</summary>
public sealed class BatchDeleteResult
{
    public long Deleted { get; init; }
    public long Failed { get; init; }
    public IReadOnlyList<BatchItemResult> Results { get; init; } = Array.Empty<BatchItemResult>();
}

/// <summary>Per-query result from <c>vectors.batch_search</c>.</summary>
public sealed class BatchSearchResult
{
    public long Index { get; init; }
    public string Status { get; init; } = string.Empty;
    public IReadOnlyList<SearchHit> Results { get; init; } = Array.Empty<SearchHit>();
    public string? Error { get; init; }
}

/// <summary>Response from <c>vectors.move</c>.</summary>
public sealed class MoveVectorsResult
{
    public string Src { get; init; } = string.Empty;
    public string Dst { get; init; } = string.Empty;
    public long Moved { get; init; }
    public long Failed { get; init; }
}

/// <summary>Response from <c>vectors.copy</c>.</summary>
public sealed class CopyVectorsResult
{
    public string Src { get; init; } = string.Empty;
    public string Dst { get; init; } = string.Empty;
    public long Copied { get; init; }
    public long Failed { get; init; }
}

/// <summary>Response from <c>vectors.delete_by_filter</c>.</summary>
public sealed class DeleteByFilterResult
{
    public long Scanned { get; init; }
    public long Matched { get; init; }
    public long Deleted { get; init; }
}

/// <summary>Response from <c>vectors.bulk_update_metadata</c>.</summary>
public sealed class BulkUpdateMetadataResult
{
    public long Scanned { get; init; }
    public long Matched { get; init; }
    public long Updated { get; init; }
}

/// <summary>Response from <c>vectors.set_expiry</c>.</summary>
public sealed class SetExpiryResult
{
    public string Id { get; init; } = string.Empty;
    public long ExpiresAt { get; init; }
    public bool Success { get; init; }
}

/// <summary>Response from <c>vectors.embed</c>.</summary>
public sealed class EmbedResult
{
    public IReadOnlyList<double> Embedding { get; init; } = Array.Empty<double>();
    public string Model { get; init; } = string.Empty;
    public long Dimension { get; init; }
}

/// <summary>Paged vector list from <c>vectors.list</c>.</summary>
public sealed class VectorListResult
{
    public IReadOnlyList<VectorizerValue> Items { get; init; } = Array.Empty<VectorizerValue>();
    public long Total { get; init; }
    public long Page { get; init; }
    public long Limit { get; init; }
}

/// <summary>HNSW traversal trace inside <c>SearchExplainResult</c>.</summary>
public sealed class SearchTrace
{
    public long VisitedNodes { get; init; }
    public long EfSearch { get; init; }
    public double HnswSearchMs { get; init; }
    public double TotalMs { get; init; }
}

/// <summary>Response from <c>search.explain</c>.</summary>
public sealed class SearchExplainResult
{
    public IReadOnlyList<SearchHit> Hits { get; init; } = Array.Empty<SearchHit>();
    public string Collection { get; init; } = string.Empty;
    public long K { get; init; }
    public SearchTrace Trace { get; init; } = new();
}

/// <summary>Summary response from <c>discovery.discover</c>.</summary>
public sealed class DiscoverResult
{
    public string AnswerPrompt { get; init; } = string.Empty;
    public long Sections { get; init; }
    public long Bullets { get; init; }
    public long Chunks { get; init; }
}

/// <summary>One scored collection from <c>discovery.score_collections</c>.</summary>
public sealed class ScoredCollection
{
    public string Name { get; init; } = string.Empty;
    public double Score { get; init; }
    public long VectorCount { get; init; }
}

/// <summary>Response from <c>discovery.expand_queries</c>.</summary>
public sealed class ExpandQueriesResult
{
    public string OriginalQuery { get; init; } = string.Empty;
    public IReadOnlyList<string> ExpandedQueries { get; init; } = Array.Empty<string>();
    public long Count { get; init; }
}

/// <summary>One chunk from <c>discovery.broad_discovery</c> and <c>discovery.semantic_focus</c>.</summary>
public sealed class DiscoveryChunk
{
    public string Collection { get; init; } = string.Empty;
    public double Score { get; init; }
    public string ContentPreview { get; init; } = string.Empty;
}

/// <summary>One bullet from <c>discovery.compress_evidence</c>.</summary>
public sealed class CompressBullet
{
    public string Text { get; init; } = string.Empty;
    public string SourceId { get; init; } = string.Empty;
    public double Score { get; init; }
}

/// <summary>One section inside an answer plan.</summary>
public sealed class AnswerPlanSection
{
    public string Title { get; init; } = string.Empty;
    public long BulletsCount { get; init; }
}

/// <summary>Response from <c>discovery.build_answer_plan</c>.</summary>
public sealed class AnswerPlanResult
{
    public IReadOnlyList<AnswerPlanSection> Sections { get; init; } = Array.Empty<AnswerPlanSection>();
    public long TotalBullets { get; init; }
}

/// <summary>Response from <c>discovery.render_llm_prompt</c>.</summary>
public sealed class RenderPromptResult
{
    public string Prompt { get; init; } = string.Empty;
    public long Length { get; init; }
    public long EstimatedTokens { get; init; }
}

/// <summary>Response from <c>graph.discovery_status</c>.</summary>
public sealed class GraphDiscoveryStatus
{
    public long TotalNodes { get; init; }
    public long NodesWithEdges { get; init; }
    public long TotalEdges { get; init; }
    public double ProgressPercentage { get; init; }
}

/// <summary>Response from <c>graph.discover_edges</c>.</summary>
public sealed class DiscoverEdgesResult
{
    public bool Success { get; init; }
    public long TotalNodes { get; init; }
    public long NodesProcessed { get; init; }
    public long NodesWithEdges { get; init; }
    public long TotalEdgesCreated { get; init; }
}

/// <summary>Response from <c>graph.discover_edges_for_node</c>.</summary>
public sealed class DiscoverEdgesForNodeResult
{
    public bool Success { get; init; }
    public string NodeId { get; init; } = string.Empty;
    public long EdgesCreated { get; init; }
}

/// <summary>Response from <c>admin.stats</c>.</summary>
public sealed class AdminStats
{
    public long CollectionsCount { get; init; }
    public long TotalVectors { get; init; }
    public string Version { get; init; } = string.Empty;
}

/// <summary>Response from <c>admin.status</c>.</summary>
public sealed class AdminStatus
{
    public bool Ready { get; init; }
    public long CollectionsCount { get; init; }
    public string Version { get; init; } = string.Empty;
}

/// <summary>Response from <c>admin.slow_queries_config</c>.</summary>
public sealed class SlowQueryConfigResult
{
    public long ThresholdMs { get; init; }
    public long Capacity { get; init; }
    public string Status { get; init; } = string.Empty;
}

/// <summary>Response from <c>auth.me</c>.</summary>
public sealed class AuthMeResult
{
    public string Username { get; init; } = string.Empty;
    public bool Authenticated { get; init; }
}

/// <summary>Response from <c>auth.refresh_token</c>.</summary>
public sealed class RefreshTokenResult
{
    public string AccessToken { get; init; } = string.Empty;
    public string TokenType { get; init; } = string.Empty;
}

/// <summary>Response from <c>auth.validate_password</c>.</summary>
public sealed class ValidatePasswordResult
{
    public bool Valid { get; init; }
    public IReadOnlyList<string> Errors { get; init; } = Array.Empty<string>();
}

/// <summary>Response from <c>auth.api_keys_create</c> and <c>auth.api_keys_create_scoped</c>.</summary>
public sealed class ApiKeyCreated
{
    public string ApiKey { get; init; } = string.Empty;
    public string Id { get; init; } = string.Empty;
    public string Name { get; init; } = string.Empty;
}

/// <summary>Response from <c>auth.api_keys_rotate</c>.</summary>
public sealed class RotatedApiKey
{
    public string OldKeyId { get; init; } = string.Empty;
    public string NewKeyId { get; init; } = string.Empty;
    public string NewToken { get; init; } = string.Empty;
    public string? GraceUntil { get; init; }
}

/// <summary>Response from <c>replication.configure</c>.</summary>
public sealed class ReplicationConfigureResult
{
    public bool Success { get; init; }
    public string Role { get; init; } = string.Empty;
    public string Message { get; init; } = string.Empty;
}

/// <summary>Response from <c>cluster.rebalance_status</c>.</summary>
public sealed class RebalanceStatus
{
    /// <summary><c>"idle"</c> when no rebalance is active.</summary>
    public string? Status { get; init; }
    public string? Message { get; init; }
}

// ── Command wrappers ──────────────────────────────────────────────────────────

/// <summary>
/// Extension methods adding typed wrappers for every entry in the v1
/// RPC command catalog. Keeps <see cref="RpcClient"/> generic while
/// still offering ergonomic access to the common shapes.
/// </summary>
public static class RpcCommands
{
    // ══ Collections ══════════════════════════════════════════════════════════

    /// <summary>Returns every collection visible to the authenticated principal.</summary>
    public static async Task<IReadOnlyList<string>> ListCollectionsAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        var v = await client.CallAsync("collections.list", Array.Empty<VectorizerValue>(), ct)
            .ConfigureAwait(false);
        if (!v.TryAsArray(out var arr))
        {
            throw new RpcServerException("collections.list: expected Array response");
        }
        var names = new List<string>(arr.Count);
        foreach (var item in arr)
        {
            if (item.TryAsStr(out var s))
            {
                names.Add(s);
            }
            else if (item.TryAsMap(out _) && item.TryMapGet("name", out var n) && n.TryAsStr(out var ns))
            {
                names.Add(ns);
            }
        }
        return names;
    }

    /// <summary>Fetches metadata for one collection.</summary>
    public static async Task<CollectionInfo> GetCollectionInfoAsync(
        this RpcClient client, string name, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(name);

        var v = await client.CallAsync(
            "collections.get_info",
            new[] { VectorizerValue.OfStr(name) },
            ct).ConfigureAwait(false);

        return new CollectionInfo
        {
            Name = RequireStr(v, "name"),
            VectorCount = RequireInt(v, "vector_count"),
            DocumentCount = RequireInt(v, "document_count"),
            Dimension = RequireInt(v, "dimension"),
            Metric = RequireStr(v, "metric"),
            CreatedAt = RequireStr(v, "created_at"),
            UpdatedAt = RequireStr(v, "updated_at"),
        };
    }

    /// <summary>Creates a new collection. <paramref name="config"/> is an optional Map with
    /// <c>dimension</c> (Int) and <c>metric</c> (Str: cosine|euclidean|dot).</summary>
    public static async Task<CreateCollectionResult> CreateCollectionAsync(
        this RpcClient client, string name, VectorizerValue? config = null, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(name);

        var args = config is not null
            ? new[] { VectorizerValue.OfStr(name), config }
            : new[] { VectorizerValue.OfStr(name) };

        var v = await client.CallAsync("collections.create", args, ct).ConfigureAwait(false);
        return new CreateCollectionResult
        {
            Name = RequireStr(v, "name"),
            Dimension = RequireInt(v, "dimension"),
            Metric = RequireStr(v, "metric"),
            Success = RequireBool(v, "success"),
        };
    }

    /// <summary>Deletes a collection. Returns <c>true</c> on success.</summary>
    public static async Task<bool> DeleteCollectionAsync(
        this RpcClient client, string name, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(name);

        var v = await client.CallAsync(
            "collections.delete", new[] { VectorizerValue.OfStr(name) }, ct)
            .ConfigureAwait(false);
        return RequireBool(v, "success");
    }

    /// <summary>Returns collection names that contain zero vectors.</summary>
    public static async Task<IReadOnlyList<string>> ListEmptyCollectionsAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        var v = await client.CallAsync(
            "collections.list_empty", Array.Empty<VectorizerValue>(), ct)
            .ConfigureAwait(false);
        return DecodeStringArray(v, "collections.list_empty");
    }

    /// <summary>Removes empty collections. Pass <paramref name="dryRun"/>=<c>true</c> to preview.</summary>
    public static async Task<CleanupEmptyResult> CleanupEmptyCollectionsAsync(
        this RpcClient client, bool dryRun = false, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);

        var config = VectorizerValue.OfMap(new[]
        {
            new MapPair(VectorizerValue.OfStr("dry_run"), VectorizerValue.OfBool(dryRun)),
        });
        var v = await client.CallAsync(
            "collections.cleanup_empty", new[] { config }, ct)
            .ConfigureAwait(false);
        return new CleanupEmptyResult
        {
            Removed = RequireInt(v, "removed"),
            DryRun = RequireBool(v, "dry_run"),
        };
    }

    /// <summary>Flushes a collection's in-memory state to disk.</summary>
    public static async Task<bool> ForceSaveCollectionAsync(
        this RpcClient client, string name, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(name);

        var v = await client.CallAsync(
            "collections.force_save", new[] { VectorizerValue.OfStr(name) }, ct)
            .ConfigureAwait(false);
        return RequireBool(v, "success");
    }

    // ══ Vectors ═══════════════════════════════════════════════════════════════

    /// <summary>Returns the raw <see cref="VectorizerValue"/> for a single vector.</summary>
    public static Task<VectorizerValue> GetVectorAsync(
        this RpcClient client, string collection, string vectorId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(vectorId);
        return client.CallAsync(
            "vectors.get",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfStr(vectorId) },
            ct);
    }

    /// <summary>Inserts one pre-computed vector. <paramref name="payload"/> is an optional Map.</summary>
    public static async Task<VectorWriteResult> InsertVectorAsync(
        this RpcClient client,
        string collection,
        string? id,
        IReadOnlyList<float> data,
        VectorizerValue? payload = null,
        CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(data);

        var idVal = id is not null ? VectorizerValue.OfStr(id) : VectorizerValue.Null;
        var dataArr = new VectorizerValue[data.Count];
        for (var i = 0; i < data.Count; i++) dataArr[i] = VectorizerValue.OfFloat(data[i]);
        var dataVal = VectorizerValue.OfArray(dataArr);

        var args = new List<VectorizerValue> { VectorizerValue.OfStr(collection), idVal, dataVal };
        if (payload is not null) args.Add(payload);

        var v = await client.CallAsync("vectors.insert", args, ct).ConfigureAwait(false);
        return new VectorWriteResult { Id = RequireStr(v, "id"), Success = RequireBool(v, "success") };
    }

    /// <summary>Embeds <paramref name="text"/> server-side and inserts the result.</summary>
    public static async Task<VectorWriteResult> InsertTextVectorAsync(
        this RpcClient client,
        string collection,
        string? id,
        string text,
        VectorizerValue? payload = null,
        CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(text);

        var idVal = id is not null ? VectorizerValue.OfStr(id) : VectorizerValue.Null;
        var args = new List<VectorizerValue>
        {
            VectorizerValue.OfStr(collection),
            idVal,
            VectorizerValue.OfStr(text),
        };
        if (payload is not null) args.Add(payload);

        var v = await client.CallAsync("vectors.insert_text", args, ct).ConfigureAwait(false);
        return new VectorWriteResult { Id = RequireStr(v, "id"), Success = RequireBool(v, "success") };
    }

    /// <summary>Replaces a vector's data and/or payload.</summary>
    public static async Task<VectorWriteResult> UpdateVectorAsync(
        this RpcClient client,
        string collection,
        string id,
        IReadOnlyList<float> data,
        VectorizerValue? payload = null,
        CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(id);
        ArgumentNullException.ThrowIfNull(data);

        var dataArr = new VectorizerValue[data.Count];
        for (var i = 0; i < data.Count; i++) dataArr[i] = VectorizerValue.OfFloat(data[i]);

        var args = new List<VectorizerValue>
        {
            VectorizerValue.OfStr(collection),
            VectorizerValue.OfStr(id),
            VectorizerValue.OfArray(dataArr),
        };
        if (payload is not null) args.Add(payload);

        var v = await client.CallAsync("vectors.update", args, ct).ConfigureAwait(false);
        return new VectorWriteResult { Id = RequireStr(v, "id"), Success = RequireBool(v, "success") };
    }

    /// <summary>Deletes one vector by id.</summary>
    public static async Task<bool> DeleteVectorAsync(
        this RpcClient client, string collection, string id, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(id);

        var v = await client.CallAsync(
            "vectors.delete",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfStr(id) },
            ct).ConfigureAwait(false);
        return RequireBool(v, "success");
    }

    /// <summary>Pages through vectors in a collection. <paramref name="page"/> is zero-based.</summary>
    public static async Task<VectorListResult> ListVectorsAsync(
        this RpcClient client, string collection, long page, long limit, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);

        var v = await client.CallAsync(
            "vectors.list",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfInt(page), VectorizerValue.OfInt(limit) },
            ct).ConfigureAwait(false);

        var items = Array.Empty<VectorizerValue>();
        if (v.TryMapGet("items", out var itemsVal) && itemsVal.TryAsArray(out var arr))
        {
            items = new VectorizerValue[arr.Count];
            for (var i = 0; i < arr.Count; i++) items[i] = arr[i];
        }

        return new VectorListResult
        {
            Items = items,
            Total = OptInt(v, "total"),
            Page = OptInt(v, "page"),
            Limit = OptInt(v, "limit"),
        };
    }

    /// <summary>Embeds <paramref name="text"/> server-side and returns the raw embedding.</summary>
    public static async Task<EmbedResult> EmbedTextAsync(
        this RpcClient client, string text, string? model = null, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(text);

        var args = new List<VectorizerValue> { VectorizerValue.OfStr(text) };
        if (model is not null) args.Add(VectorizerValue.OfStr(model));

        var v = await client.CallAsync("vectors.embed", args, ct).ConfigureAwait(false);

        var embedding = Array.Empty<double>();
        if (v.TryMapGet("embedding", out var embVal) && embVal.TryAsArray(out var embArr))
        {
            embedding = new double[embArr.Count];
            for (var i = 0; i < embArr.Count; i++)
            {
                embArr[i].TryAsFloat(out var f);
                embedding[i] = f;
            }
        }

        return new EmbedResult
        {
            Embedding = embedding,
            Model = OptStr(v, "model", "bm25"),
            Dimension = OptInt(v, "dimension"),
        };
    }

    /// <summary>Inserts multiple pre-computed vectors in one round-trip. Each item in
    /// <paramref name="items"/> is a Map with at least <c>data</c> (Array&lt;Float&gt;).</summary>
    public static async Task<BatchInsertResult> BatchInsertVectorsAsync(
        this RpcClient client, string collection, IReadOnlyList<VectorizerValue> items, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(items);

        var v = await client.CallAsync(
            "vectors.batch_insert",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfArray(items) },
            ct).ConfigureAwait(false);
        return DecodeBatchInsert(v);
    }

    /// <summary>Embeds and inserts multiple text items. Each item is a Map with at least
    /// <c>text</c> (Str).</summary>
    public static async Task<BatchInsertResult> BatchInsertTextsAsync(
        this RpcClient client, string collection, IReadOnlyList<VectorizerValue> items, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(items);

        var v = await client.CallAsync(
            "vectors.batch_insert_texts",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfArray(items) },
            ct).ConfigureAwait(false);
        return DecodeBatchInsert(v);
    }

    /// <summary>Runs multiple searches in one round-trip. Each request is a Map with
    /// <c>collection</c>, <c>query</c>, and optional <c>limit</c>.</summary>
    public static async Task<IReadOnlyList<BatchSearchResult>> BatchSearchAsync(
        this RpcClient client, IReadOnlyList<VectorizerValue> requests, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(requests);

        var v = await client.CallAsync(
            "vectors.batch_search",
            new[] { VectorizerValue.OfArray(requests) },
            ct).ConfigureAwait(false);

        if (!v.TryAsArray(out var arr))
        {
            throw new RpcServerException("vectors.batch_search: expected Array response");
        }

        var results = new BatchSearchResult[arr.Count];
        for (var i = 0; i < arr.Count; i++)
        {
            var entry = arr[i];
            results[i] = new BatchSearchResult
            {
                Index = OptInt(entry, "index"),
                Status = OptStr(entry, "status", "unknown"),
                Results = DecodeSearchHits(entry.TryMapGet("results", out var rVal) &&
                    rVal.TryAsArray(out var rArr) ? rArr : Array.Empty<VectorizerValue>()),
                Error = OptStrNullable(entry, "error"),
            };
        }
        return results;
    }

    /// <summary>Updates multiple vectors' data and/or payload. Each item is a Map with
    /// <c>id</c> (Str) and optionally <c>data</c> and <c>payload</c>.</summary>
    public static async Task<BatchUpdateResult> BatchUpdateVectorsAsync(
        this RpcClient client, string collection, IReadOnlyList<VectorizerValue> updates, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(updates);

        var v = await client.CallAsync(
            "vectors.batch_update",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfArray(updates) },
            ct).ConfigureAwait(false);

        return new BatchUpdateResult
        {
            Updated = OptInt(v, "updated"),
            Failed = OptInt(v, "failed"),
            Results = DecodeBatchItems(v.TryMapGet("results", out var rVal) &&
                rVal.TryAsArray(out var rArr) ? rArr : Array.Empty<VectorizerValue>()),
        };
    }

    /// <summary>Deletes multiple vectors by id.</summary>
    public static async Task<BatchDeleteResult> BatchDeleteVectorsAsync(
        this RpcClient client, string collection, IReadOnlyList<string> ids, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(ids);

        var idsArr = new VectorizerValue[ids.Count];
        for (var i = 0; i < ids.Count; i++) idsArr[i] = VectorizerValue.OfStr(ids[i]);

        var v = await client.CallAsync(
            "vectors.batch_delete",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfArray(idsArr) },
            ct).ConfigureAwait(false);

        return new BatchDeleteResult
        {
            Deleted = OptInt(v, "deleted"),
            Failed = OptInt(v, "failed"),
            Results = DecodeBatchItems(v.TryMapGet("results", out var rVal) &&
                rVal.TryAsArray(out var rArr) ? rArr : Array.Empty<VectorizerValue>()),
        };
    }

    /// <summary>Moves vectors from <paramref name="src"/> to <paramref name="dst"/> collection.</summary>
    public static async Task<MoveVectorsResult> MoveVectorsAsync(
        this RpcClient client, string src, string dst, IReadOnlyList<string> ids, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(src);
        ArgumentNullException.ThrowIfNull(dst);
        ArgumentNullException.ThrowIfNull(ids);

        var idsArr = new VectorizerValue[ids.Count];
        for (var i = 0; i < ids.Count; i++) idsArr[i] = VectorizerValue.OfStr(ids[i]);

        var v = await client.CallAsync(
            "vectors.move",
            new[] { VectorizerValue.OfStr(src), VectorizerValue.OfStr(dst), VectorizerValue.OfArray(idsArr) },
            ct).ConfigureAwait(false);

        return new MoveVectorsResult
        {
            Src = RequireStr(v, "src"),
            Dst = RequireStr(v, "dst"),
            Moved = OptInt(v, "moved"),
            Failed = OptInt(v, "failed"),
        };
    }

    /// <summary>Copies vectors from <paramref name="src"/> to <paramref name="dst"/> without deleting.</summary>
    public static async Task<CopyVectorsResult> CopyVectorsAsync(
        this RpcClient client, string src, string dst, IReadOnlyList<string> ids, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(src);
        ArgumentNullException.ThrowIfNull(dst);
        ArgumentNullException.ThrowIfNull(ids);

        var idsArr = new VectorizerValue[ids.Count];
        for (var i = 0; i < ids.Count; i++) idsArr[i] = VectorizerValue.OfStr(ids[i]);

        var v = await client.CallAsync(
            "vectors.copy",
            new[] { VectorizerValue.OfStr(src), VectorizerValue.OfStr(dst), VectorizerValue.OfArray(idsArr) },
            ct).ConfigureAwait(false);

        return new CopyVectorsResult
        {
            Src = RequireStr(v, "src"),
            Dst = RequireStr(v, "dst"),
            Copied = OptInt(v, "copied"),
            Failed = OptInt(v, "failed"),
        };
    }

    /// <summary>Deletes all vectors matching a Qdrant-style filter predicate.</summary>
    public static async Task<DeleteByFilterResult> DeleteByFilterAsync(
        this RpcClient client, string collection, VectorizerValue filter, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(filter);

        var v = await client.CallAsync(
            "vectors.delete_by_filter",
            new[] { VectorizerValue.OfStr(collection), filter },
            ct).ConfigureAwait(false);

        return new DeleteByFilterResult
        {
            Scanned = OptInt(v, "scanned"),
            Matched = OptInt(v, "matched"),
            Deleted = OptInt(v, "deleted"),
        };
    }

    /// <summary>Applies a JSON-merge-patch to all vectors matching <paramref name="filter"/>.</summary>
    public static async Task<BulkUpdateMetadataResult> BulkUpdateMetadataAsync(
        this RpcClient client, string collection, VectorizerValue filter, VectorizerValue patch, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(filter);
        ArgumentNullException.ThrowIfNull(patch);

        var v = await client.CallAsync(
            "vectors.bulk_update_metadata",
            new[] { VectorizerValue.OfStr(collection), filter, patch },
            ct).ConfigureAwait(false);

        return new BulkUpdateMetadataResult
        {
            Scanned = OptInt(v, "scanned"),
            Matched = OptInt(v, "matched"),
            Updated = OptInt(v, "updated"),
        };
    }

    /// <summary>Attaches a TTL to one vector. <paramref name="expiresAt"/> may be a
    /// Unix ms timestamp or RFC3339 string.</summary>
    public static async Task<SetExpiryResult> SetVectorExpiryAsync(
        this RpcClient client, string collection, string id, string expiresAt, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(id);
        ArgumentNullException.ThrowIfNull(expiresAt);

        var v = await client.CallAsync(
            "vectors.set_expiry",
            new[]
            {
                VectorizerValue.OfStr(collection),
                VectorizerValue.OfStr(id),
                VectorizerValue.OfStr(expiresAt),
            },
            ct).ConfigureAwait(false);

        return new SetExpiryResult
        {
            Id = RequireStr(v, "id"),
            ExpiresAt = RequireInt(v, "expires_at"),
            Success = RequireBool(v, "success"),
        };
    }

    // ══ Search ════════════════════════════════════════════════════════════════

    /// <summary>Runs <c>search.basic</c> and returns ranked hits.</summary>
    public static async Task<IReadOnlyList<SearchHit>> SearchBasicAsync(
        this RpcClient client, string collection, string query, int limit, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(query);

        var args = new[]
        {
            VectorizerValue.OfStr(collection),
            VectorizerValue.OfStr(query),
            VectorizerValue.OfInt(limit),
        };
        var v = await client.CallAsync("search.basic", args, ct).ConfigureAwait(false);
        if (!v.TryAsArray(out var arr))
        {
            throw new RpcServerException("search.basic: expected Array response");
        }
        return DecodeSearchHits(arr);
    }

    /// <summary>Runs <c>search.intelligent</c> across one or more collections.</summary>
    public static async Task<IReadOnlyList<SearchHit>> SearchIntelligentAsync(
        this RpcClient client,
        string query,
        IReadOnlyList<string>? collections = null,
        int? maxResults = null,
        bool? domainExpansion = null,
        double? threshold = null,
        CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(query);

        var args = new List<VectorizerValue> { VectorizerValue.OfStr(query) };
        if (collections is not null)
        {
            var collArr = new VectorizerValue[collections.Count];
            for (var i = 0; i < collections.Count; i++) collArr[i] = VectorizerValue.OfStr(collections[i]);
            args.Add(VectorizerValue.OfArray(collArr));
        }
        if (maxResults.HasValue) args.Add(VectorizerValue.OfInt(maxResults.Value));
        if (domainExpansion.HasValue) args.Add(VectorizerValue.OfBool(domainExpansion.Value));
        if (threshold.HasValue) args.Add(VectorizerValue.OfFloat(threshold.Value));

        var v = await client.CallAsync("search.intelligent", args, ct).ConfigureAwait(false);
        if (!v.TryAsArray(out var arr))
        {
            throw new RpcServerException("search.intelligent: expected Array response");
        }
        return DecodeSearchHits(arr);
    }

    /// <summary>Searches one collection by text query.</summary>
    public static async Task<IReadOnlyList<SearchHit>> SearchByTextAsync(
        this RpcClient client, string collection, string query, int limit, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(query);

        var v = await client.CallAsync(
            "search.by_text",
            new[]
            {
                VectorizerValue.OfStr(collection),
                VectorizerValue.OfStr(query),
                VectorizerValue.OfInt(limit),
            },
            ct).ConfigureAwait(false);

        if (v.TryMapGet("results", out var rVal) && rVal.TryAsArray(out var rArr))
            return DecodeSearchHits(rArr);
        if (v.TryAsArray(out var arr))
            return DecodeSearchHits(arr);
        throw new RpcServerException("search.by_text: missing results array");
    }

    /// <summary>File-content-based search. <paramref name="request"/> is a Map describing the query.</summary>
    public static async Task<IReadOnlyList<SearchHit>> SearchByFileAsync(
        this RpcClient client, string collection, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync(
            "search.by_file",
            new[] { VectorizerValue.OfStr(collection), request },
            ct).ConfigureAwait(false);

        if (v.TryMapGet("results", out var rVal) && rVal.TryAsArray(out var rArr))
            return DecodeSearchHits(rArr);
        if (v.TryAsArray(out var arr))
            return DecodeSearchHits(arr);
        return Array.Empty<SearchHit>();
    }

    /// <summary>RRF / weighted-combination hybrid dense+sparse search.</summary>
    public static async Task<IReadOnlyList<SearchHit>> SearchHybridAsync(
        this RpcClient client, string collection, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync(
            "search.hybrid",
            new[] { VectorizerValue.OfStr(collection), request },
            ct).ConfigureAwait(false);

        if (v.TryMapGet("results", out var rVal) && rVal.TryAsArray(out var rArr))
            return DecodeSearchHits(rArr);
        if (v.TryAsArray(out var arr))
            return DecodeSearchHits(arr);
        throw new RpcServerException("search.hybrid: missing results array");
    }

    /// <summary>Semantic re-ranking search. Returns the raw response map.</summary>
    public static Task<VectorizerValue> SearchSemanticAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("search.semantic", new[] { request }, ct);
    }

    /// <summary>Context-filtered semantic search. Returns the raw response map.</summary>
    public static Task<VectorizerValue> SearchContextualAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("search.contextual", new[] { request }, ct);
    }

    /// <summary>Fan-out search across multiple collections. Returns the raw response map.</summary>
    public static Task<VectorizerValue> SearchMultiCollectionAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("search.multi_collection", new[] { request }, ct);
    }

    /// <summary>Runs a vector search and returns the HNSW traversal trace.</summary>
    public static async Task<SearchExplainResult> SearchExplainAsync(
        this RpcClient client, string collection, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync(
            "search.explain",
            new[] { VectorizerValue.OfStr(collection), request },
            ct).ConfigureAwait(false);

        var hitsArr = v.TryMapGet("hits", out var hv) && hv.TryAsArray(out var ha)
            ? ha : (IReadOnlyList<VectorizerValue>)Array.Empty<VectorizerValue>();

        SearchTrace trace;
        if (v.TryMapGet("trace", out var tv))
        {
            trace = new SearchTrace
            {
                VisitedNodes = OptInt(tv, "visited_nodes"),
                EfSearch = OptInt(tv, "ef_search"),
                HnswSearchMs = OptFloat(tv, "hnsw_search_ms"),
                TotalMs = OptFloat(tv, "total_ms"),
            };
        }
        else
        {
            trace = new SearchTrace();
        }

        return new SearchExplainResult
        {
            Hits = DecodeSearchHits(hitsArr),
            Collection = OptStr(v, "collection", string.Empty),
            K = OptInt(v, "k"),
            Trace = trace,
        };
    }

    // ══ Discovery ════════════════════════════════════════════════════════════

    /// <summary>Full discovery pipeline: embed → search → compress → build plan → render prompt.</summary>
    public static async Task<DiscoverResult> DiscoverAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.discover", new[] { request }, ct).ConfigureAwait(false);
        return new DiscoverResult
        {
            AnswerPrompt = RequireStr(v, "answer_prompt"),
            Sections = OptInt(v, "sections"),
            Bullets = OptInt(v, "bullets"),
            Chunks = OptInt(v, "chunks"),
        };
    }

    /// <summary>Filters a collection list by query relevance.</summary>
    public static async Task<IReadOnlyList<string>> FilterCollectionsAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.filter_collections", new[] { request }, ct)
            .ConfigureAwait(false);

        if (!v.TryMapGet("filtered_collections", out var fc) || !fc.TryAsArray(out var arr))
        {
            throw new RpcServerException("discovery.filter_collections: missing filtered_collections");
        }

        var names = new List<string>(arr.Count);
        foreach (var entry in arr)
        {
            if (entry.TryMapGet("name", out var nv) && nv.TryAsStr(out var name))
                names.Add(name);
        }
        return names;
    }

    /// <summary>Scores all collections for a query.</summary>
    public static async Task<IReadOnlyList<ScoredCollection>> ScoreCollectionsAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.score_collections", new[] { request }, ct)
            .ConfigureAwait(false);

        if (!v.TryMapGet("scored_collections", out var sc) || !sc.TryAsArray(out var arr))
        {
            throw new RpcServerException("discovery.score_collections: missing scored_collections");
        }

        var result = new ScoredCollection[arr.Count];
        for (var i = 0; i < arr.Count; i++)
        {
            result[i] = new ScoredCollection
            {
                Name = OptStr(arr[i], "name", string.Empty),
                Score = OptFloat(arr[i], "score"),
                VectorCount = OptInt(arr[i], "vector_count"),
            };
        }
        return result;
    }

    /// <summary>Generates query variants via baseline expansion.</summary>
    public static async Task<ExpandQueriesResult> ExpandQueriesAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.expand_queries", new[] { request }, ct)
            .ConfigureAwait(false);

        var expanded = Array.Empty<string>();
        if (v.TryMapGet("expanded_queries", out var eqv) && eqv.TryAsArray(out var eqArr))
        {
            expanded = new string[eqArr.Count];
            for (var i = 0; i < eqArr.Count; i++)
            {
                eqArr[i].TryAsStr(out var s);
                expanded[i] = s;
            }
        }

        return new ExpandQueriesResult
        {
            OriginalQuery = RequireStr(v, "original_query"),
            ExpandedQueries = expanded,
            Count = OptInt(v, "count"),
        };
    }

    /// <summary>Multi-query broad search across all collections.</summary>
    public static async Task<IReadOnlyList<DiscoveryChunk>> BroadDiscoveryAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.broad_discovery", new[] { request }, ct)
            .ConfigureAwait(false);

        if (!v.TryMapGet("chunks", out var cv) || !cv.TryAsArray(out var arr))
        {
            throw new RpcServerException("discovery.broad_discovery: missing chunks");
        }
        return DecodeDiscoveryChunks(arr);
    }

    /// <summary>Deep semantic search within one collection.</summary>
    public static async Task<IReadOnlyList<DiscoveryChunk>> SemanticFocusAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.semantic_focus", new[] { request }, ct)
            .ConfigureAwait(false);

        if (!v.TryMapGet("chunks", out var cv) || !cv.TryAsArray(out var arr))
        {
            throw new RpcServerException("discovery.semantic_focus: missing chunks");
        }
        return DecodeDiscoveryChunks(arr);
    }

    /// <summary>Promotes README chunks to the top of a chunk set. Returns raw response.</summary>
    public static Task<VectorizerValue> PromoteReadmeAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("discovery.promote_readme", new[] { request }, ct);
    }

    /// <summary>Compresses a chunk set into ranked bullets.</summary>
    public static async Task<IReadOnlyList<CompressBullet>> CompressEvidenceAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.compress_evidence", new[] { request }, ct)
            .ConfigureAwait(false);

        if (!v.TryMapGet("bullets", out var bv) || !bv.TryAsArray(out var arr))
        {
            throw new RpcServerException("discovery.compress_evidence: missing bullets");
        }

        var result = new CompressBullet[arr.Count];
        for (var i = 0; i < arr.Count; i++)
        {
            result[i] = new CompressBullet
            {
                Text = OptStr(arr[i], "text", string.Empty),
                SourceId = OptStr(arr[i], "source_id", string.Empty),
                Score = OptFloat(arr[i], "score"),
            };
        }
        return result;
    }

    /// <summary>Organises bullets into a structured answer plan.</summary>
    public static async Task<AnswerPlanResult> BuildAnswerPlanAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.build_answer_plan", new[] { request }, ct)
            .ConfigureAwait(false);

        var sections = Array.Empty<AnswerPlanSection>();
        if (v.TryMapGet("sections", out var sv) && sv.TryAsArray(out var sArr))
        {
            sections = new AnswerPlanSection[sArr.Count];
            for (var i = 0; i < sArr.Count; i++)
            {
                sections[i] = new AnswerPlanSection
                {
                    Title = OptStr(sArr[i], "title", string.Empty),
                    BulletsCount = OptInt(sArr[i], "bullets_count"),
                };
            }
        }

        return new AnswerPlanResult
        {
            Sections = sections,
            TotalBullets = OptInt(v, "total_bullets"),
        };
    }

    /// <summary>Renders an answer plan into an LLM prompt string.</summary>
    public static async Task<RenderPromptResult> RenderLlmPromptAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("discovery.render_llm_prompt", new[] { request }, ct)
            .ConfigureAwait(false);

        return new RenderPromptResult
        {
            Prompt = RequireStr(v, "prompt"),
            Length = OptInt(v, "length"),
            EstimatedTokens = OptInt(v, "estimated_tokens"),
        };
    }

    // ══ File ops ══════════════════════════════════════════════════════════════

    /// <summary>Retrieves raw file content stored in a collection. Returns raw response.</summary>
    public static Task<VectorizerValue> FileContentAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("file.content", new[] { request }, ct);
    }

    /// <summary>Lists files indexed in a collection. Returns raw response.</summary>
    public static Task<VectorizerValue> FileListAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("file.list", new[] { request }, ct);
    }

    /// <summary>Extractive or structural summary of one file. Returns raw response.</summary>
    public static Task<VectorizerValue> FileSummaryAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("file.summary", new[] { request }, ct);
    }

    /// <summary>Retrieves ordered chunks for one file. Returns raw response.</summary>
    public static Task<VectorizerValue> FileChunksAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("file.chunks", new[] { request }, ct);
    }

    /// <summary>Directory-tree outline of a collection's files. Returns raw response.</summary>
    public static Task<VectorizerValue> FileOutlineAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("file.outline", new[] { request }, ct);
    }

    /// <summary>Finds files semantically related to a given file. Returns raw response.</summary>
    public static Task<VectorizerValue> FileRelatedAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("file.related", new[] { request }, ct);
    }

    /// <summary>Searches within files of specific extension types. Returns raw response.</summary>
    public static Task<VectorizerValue> FileSearchByTypeAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("file.search_by_type", new[] { request }, ct);
    }

    // ══ Graph ══════════════════════════════════════════════════════════════

    /// <summary>Lists all graph nodes in a collection. Returns raw response.</summary>
    public static Task<VectorizerValue> GraphListNodesAsync(
        this RpcClient client, string collection, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        return client.CallAsync("graph.list_nodes", new[] { VectorizerValue.OfStr(collection) }, ct);
    }

    /// <summary>Fetches direct neighbors of a graph node. Returns raw response.</summary>
    public static Task<VectorizerValue> GraphNeighborsAsync(
        this RpcClient client, string collection, string nodeId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(nodeId);
        return client.CallAsync(
            "graph.neighbors",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfStr(nodeId) },
            ct);
    }

    /// <summary>Finds nodes reachable within <paramref name="maxHops"/> of a node. Returns raw response.</summary>
    public static Task<VectorizerValue> GraphFindRelatedAsync(
        this RpcClient client, string collection, string nodeId, long maxHops, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(nodeId);
        return client.CallAsync(
            "graph.find_related",
            new[]
            {
                VectorizerValue.OfStr(collection),
                VectorizerValue.OfStr(nodeId),
                VectorizerValue.OfInt(maxHops),
            },
            ct);
    }

    /// <summary>Returns the shortest path between two graph nodes. Returns raw response.</summary>
    public static Task<VectorizerValue> GraphFindPathAsync(
        this RpcClient client, string collection, string from, string to, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(from);
        ArgumentNullException.ThrowIfNull(to);
        return client.CallAsync(
            "graph.find_path",
            new[]
            {
                VectorizerValue.OfStr(collection),
                VectorizerValue.OfStr(from),
                VectorizerValue.OfStr(to),
            },
            ct);
    }

    /// <summary>Creates a directed edge between two nodes. Returns raw response.</summary>
    public static Task<VectorizerValue> GraphCreateEdgeAsync(
        this RpcClient client, string collection, VectorizerValue edge, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(edge);
        return client.CallAsync(
            "graph.create_edge",
            new[] { VectorizerValue.OfStr(collection), edge },
            ct);
    }

    /// <summary>Removes an edge by its id. Returns raw response.</summary>
    public static Task<VectorizerValue> GraphDeleteEdgeAsync(
        this RpcClient client, string collection, string edgeId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(edgeId);
        return client.CallAsync(
            "graph.delete_edge",
            new[] { VectorizerValue.OfStr(collection), VectorizerValue.OfStr(edgeId) },
            ct);
    }

    /// <summary>Lists all edges in a collection's graph. Returns raw response.</summary>
    public static Task<VectorizerValue> GraphListEdgesAsync(
        this RpcClient client, string collection, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        return client.CallAsync("graph.list_edges", new[] { VectorizerValue.OfStr(collection) }, ct);
    }

    /// <summary>Auto-discovers edges by vector similarity across the whole collection.</summary>
    public static async Task<DiscoverEdgesResult> GraphDiscoverEdgesAsync(
        this RpcClient client, string collection, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync(
            "graph.discover_edges",
            new[] { VectorizerValue.OfStr(collection), request },
            ct).ConfigureAwait(false);

        return new DiscoverEdgesResult
        {
            Success = OptBool(v, "success"),
            TotalNodes = OptInt(v, "total_nodes"),
            NodesProcessed = OptInt(v, "nodes_processed"),
            NodesWithEdges = OptInt(v, "nodes_with_edges"),
            TotalEdgesCreated = OptInt(v, "total_edges_created"),
        };
    }

    /// <summary>Auto-discovers edges for one node by vector similarity.</summary>
    public static async Task<DiscoverEdgesForNodeResult> GraphDiscoverEdgesForNodeAsync(
        this RpcClient client, string collection, string nodeId, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(nodeId);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync(
            "graph.discover_edges_for_node",
            new[]
            {
                VectorizerValue.OfStr(collection),
                VectorizerValue.OfStr(nodeId),
                request,
            },
            ct).ConfigureAwait(false);

        return new DiscoverEdgesForNodeResult
        {
            Success = OptBool(v, "success"),
            NodeId = OptStr(v, "node_id", nodeId),
            EdgesCreated = OptInt(v, "edges_created"),
        };
    }

    /// <summary>Returns the percentage of nodes that have edges in a collection.</summary>
    public static async Task<GraphDiscoveryStatus> GraphDiscoveryStatusAsync(
        this RpcClient client, string collection, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(collection);

        var v = await client.CallAsync(
            "graph.discovery_status",
            new[] { VectorizerValue.OfStr(collection) },
            ct).ConfigureAwait(false);

        return new GraphDiscoveryStatus
        {
            TotalNodes = OptInt(v, "total_nodes"),
            NodesWithEdges = OptInt(v, "nodes_with_edges"),
            TotalEdges = OptInt(v, "total_edges"),
            ProgressPercentage = OptFloat(v, "progress_percentage"),
        };
    }

    // ══ Admin ════════════════════════════════════════════════════════════════

    /// <summary>Returns aggregate vector and collection counts.</summary>
    public static async Task<AdminStats> AdminStatsAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        var v = await client.CallAsync("admin.stats", Array.Empty<VectorizerValue>(), ct)
            .ConfigureAwait(false);
        return new AdminStats
        {
            CollectionsCount = OptInt(v, "collections_count"),
            TotalVectors = OptInt(v, "total_vectors"),
            Version = OptStr(v, "version", string.Empty),
        };
    }

    /// <summary>Readiness probe and basic counts.</summary>
    public static async Task<AdminStatus> AdminStatusAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        var v = await client.CallAsync("admin.status", Array.Empty<VectorizerValue>(), ct)
            .ConfigureAwait(false);
        return new AdminStatus
        {
            Ready = OptBool(v, "ready"),
            CollectionsCount = OptInt(v, "collections_count"),
            Version = OptStr(v, "version", string.Empty),
        };
    }

    /// <summary>Returns recent server log lines. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminLogsAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.logs", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Returns current indexing progress. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminIndexingProgressAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.indexing_progress", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Gets the server configuration. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminConfigGetAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.config_get", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Updates server configuration. <paramref name="patch"/> is a Map of settings.</summary>
    public static Task<VectorizerValue> AdminConfigUpdateAsync(
        this RpcClient client, VectorizerValue patch, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(patch);
        return client.CallAsync("admin.config_update", new[] { patch }, ct);
    }

    /// <summary>Lists available backups. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminBackupsListAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.backups_list", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Creates a backup. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminBackupsCreateAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.backups_create", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Restores from a backup. <paramref name="request"/> identifies the backup.</summary>
    public static Task<VectorizerValue> AdminBackupsRestoreAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("admin.backups_restore", new[] { request }, ct);
    }

    /// <summary>Lists configured workspaces. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminWorkspacesListAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.workspaces_list", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Gets one workspace by name. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminWorkspaceGetAsync(
        this RpcClient client, string name, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(name);
        return client.CallAsync("admin.workspace_get", new[] { VectorizerValue.OfStr(name) }, ct);
    }

    /// <summary>Adds a workspace. <paramref name="request"/> is the workspace definition Map.</summary>
    public static Task<VectorizerValue> AdminWorkspaceAddAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("admin.workspace_add", new[] { request }, ct);
    }

    /// <summary>Removes a workspace by name.</summary>
    public static Task<VectorizerValue> AdminWorkspaceRemoveAsync(
        this RpcClient client, string name, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(name);
        return client.CallAsync("admin.workspace_remove", new[] { VectorizerValue.OfStr(name) }, ct);
    }

    /// <summary>Triggers a server restart. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminRestartAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.restart", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Returns recent slow query records. Returns raw response.</summary>
    public static Task<VectorizerValue> AdminSlowQueriesListAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("admin.slow_queries_list", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Gets or updates the slow-query ring configuration.</summary>
    public static async Task<SlowQueryConfigResult> AdminSlowQueriesConfigAsync(
        this RpcClient client, VectorizerValue? patch = null, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);

        var args = patch is not null
            ? new[] { patch }
            : Array.Empty<VectorizerValue>();

        var v = await client.CallAsync("admin.slow_queries_config", args, ct).ConfigureAwait(false);
        return new SlowQueryConfigResult
        {
            ThresholdMs = OptInt(v, "threshold_ms"),
            Capacity = OptInt(v, "capacity"),
            Status = OptStr(v, "status", string.Empty),
        };
    }

    // ══ Auth ══════════════════════════════════════════════════════════════════

    /// <summary>Returns the currently authenticated principal's details.</summary>
    public static async Task<AuthMeResult> AuthMeAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        var v = await client.CallAsync("auth.me", Array.Empty<VectorizerValue>(), ct)
            .ConfigureAwait(false);
        return new AuthMeResult
        {
            Username = OptStr(v, "username", string.Empty),
            Authenticated = OptBool(v, "authenticated"),
        };
    }

    /// <summary>Invalidates the current session token.</summary>
    public static Task<VectorizerValue> AuthLogoutAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("auth.logout", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Issues a fresh access token using the current credentials.</summary>
    public static async Task<RefreshTokenResult> AuthRefreshTokenAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        var v = await client.CallAsync("auth.refresh_token", Array.Empty<VectorizerValue>(), ct)
            .ConfigureAwait(false);
        return new RefreshTokenResult
        {
            AccessToken = RequireStr(v, "access_token"),
            TokenType = OptStr(v, "token_type", "bearer"),
        };
    }

    /// <summary>Validates a password against the server's policy without storing it.</summary>
    public static async Task<ValidatePasswordResult> AuthValidatePasswordAsync(
        this RpcClient client, string password, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(password);

        var v = await client.CallAsync(
            "auth.validate_password", new[] { VectorizerValue.OfStr(password) }, ct)
            .ConfigureAwait(false);

        var errors = Array.Empty<string>();
        if (v.TryMapGet("errors", out var ev) && ev.TryAsArray(out var eArr))
        {
            errors = new string[eArr.Count];
            for (var i = 0; i < eArr.Count; i++)
            {
                eArr[i].TryAsStr(out var s);
                errors[i] = s;
            }
        }

        return new ValidatePasswordResult
        {
            Valid = RequireBool(v, "valid"),
            Errors = errors,
        };
    }

    /// <summary>Creates a new API key with the name and optional scopes from <paramref name="request"/>.</summary>
    public static async Task<ApiKeyCreated> AuthApiKeysCreateAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("auth.api_keys_create", new[] { request }, ct)
            .ConfigureAwait(false);
        return new ApiKeyCreated
        {
            ApiKey = RequireStr(v, "api_key"),
            Id = RequireStr(v, "id"),
            Name = RequireStr(v, "name"),
        };
    }

    /// <summary>Lists all API keys for the authenticated principal. Returns raw response.</summary>
    public static Task<VectorizerValue> AuthApiKeysListAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("auth.api_keys_list", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Revokes an API key by id.</summary>
    public static Task<VectorizerValue> AuthApiKeysRevokeAsync(
        this RpcClient client, string keyId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(keyId);
        return client.CallAsync("auth.api_keys_revoke", new[] { VectorizerValue.OfStr(keyId) }, ct);
    }

    /// <summary>Rotates an API key, issuing a replacement with an optional grace period.</summary>
    public static async Task<RotatedApiKey> AuthApiKeysRotateAsync(
        this RpcClient client, string keyId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(keyId);

        var v = await client.CallAsync(
            "auth.api_keys_rotate", new[] { VectorizerValue.OfStr(keyId) }, ct)
            .ConfigureAwait(false);

        return new RotatedApiKey
        {
            OldKeyId = RequireStr(v, "old_key_id"),
            NewKeyId = RequireStr(v, "new_key_id"),
            NewToken = RequireStr(v, "new_token"),
            GraceUntil = OptStrNullable(v, "grace_until"),
        };
    }

    /// <summary>Creates a scoped API key (limited permissions). See <see cref="AuthApiKeysCreateAsync"/>.</summary>
    public static async Task<ApiKeyCreated> AuthApiKeysCreateScopedAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("auth.api_keys_create_scoped", new[] { request }, ct)
            .ConfigureAwait(false);
        return new ApiKeyCreated
        {
            ApiKey = RequireStr(v, "api_key"),
            Id = RequireStr(v, "id"),
            Name = RequireStr(v, "name"),
        };
    }

    /// <summary>Introspects a token and returns its claims. Returns raw response.</summary>
    public static Task<VectorizerValue> AuthIntrospectAsync(
        this RpcClient client, string token, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(token);
        return client.CallAsync("auth.introspect", new[] { VectorizerValue.OfStr(token) }, ct);
    }

    /// <summary>Returns recent authentication audit events. Returns raw response.</summary>
    public static Task<VectorizerValue> AuthAuditAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("auth.audit", Array.Empty<VectorizerValue>(), ct);
    }

    // ══ Replication ══════════════════════════════════════════════════════════

    /// <summary>Returns current replication status. Returns raw response.</summary>
    public static Task<VectorizerValue> ReplicationStatusAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("replication.status", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Configures the replication role. <paramref name="request"/> must include <c>role</c>
    /// (Str: master|replica|standalone) and optionally <c>bind_address</c>, <c>master_address</c>.</summary>
    public static async Task<ReplicationConfigureResult> ReplicationConfigureAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);

        var v = await client.CallAsync("replication.configure", new[] { request }, ct)
            .ConfigureAwait(false);

        return new ReplicationConfigureResult
        {
            Success = RequireBool(v, "success"),
            Role = RequireStr(v, "role"),
            Message = RequireStr(v, "message"),
        };
    }

    /// <summary>Returns replication throughput and lag statistics. Returns raw response.</summary>
    public static Task<VectorizerValue> ReplicationStatsAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("replication.stats", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Lists all known replicas (master-only). Returns raw response.</summary>
    public static Task<VectorizerValue> ReplicationReplicasListAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("replication.replicas_list", Array.Empty<VectorizerValue>(), ct);
    }

    // ══ Cluster ══════════════════════════════════════════════════════════════

    /// <summary>Triggers failover to the specified replica.</summary>
    public static Task<VectorizerValue> ClusterFailoverAsync(
        this RpcClient client, string replicaId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(replicaId);
        return client.CallAsync("cluster.failover", new[] { VectorizerValue.OfStr(replicaId) }, ct);
    }

    /// <summary>Forces a full resync of the specified replica.</summary>
    public static Task<VectorizerValue> ClusterReplicaResyncAsync(
        this RpcClient client, string replicaId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(replicaId);
        return client.CallAsync("cluster.replica_resync", new[] { VectorizerValue.OfStr(replicaId) }, ct);
    }

    /// <summary>Adds a peer to the cluster. <paramref name="request"/> must include
    /// <c>address</c> (Str) and optionally <c>role</c> (Str: member|observer).</summary>
    public static Task<VectorizerValue> ClusterPeerAddAsync(
        this RpcClient client, VectorizerValue request, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(request);
        return client.CallAsync("cluster.peer_add", new[] { request }, ct);
    }

    /// <summary>Triggers a shard-rebalance across the cluster. Returns raw response.</summary>
    public static Task<VectorizerValue> ClusterRebalanceAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        return client.CallAsync("cluster.rebalance", Array.Empty<VectorizerValue>(), ct);
    }

    /// <summary>Returns the status of the most recent rebalance job.</summary>
    public static async Task<RebalanceStatus> ClusterRebalanceStatusAsync(
        this RpcClient client, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(client);
        var v = await client.CallAsync("cluster.rebalance_status", Array.Empty<VectorizerValue>(), ct)
            .ConfigureAwait(false);
        return new RebalanceStatus
        {
            Status = OptStrNullable(v, "status"),
            Message = OptStrNullable(v, "message"),
        };
    }

    // ══ Private helpers ══════════════════════════════════════════════════════

    private static string RequireStr(VectorizerValue map, string key)
    {
        if (!map.TryMapGet(key, out var f))
        {
            throw new RpcServerException($"missing string field '{key}'");
        }
        if (!f.TryAsStr(out var s))
        {
            throw new RpcServerException($"field '{key}' is not a string");
        }
        return s;
    }

    private static long RequireInt(VectorizerValue map, string key)
    {
        if (!map.TryMapGet(key, out var f))
        {
            throw new RpcServerException($"missing integer field '{key}'");
        }
        if (!f.TryAsInt(out var i))
        {
            throw new RpcServerException($"field '{key}' is not an integer");
        }
        return i;
    }

    private static bool RequireBool(VectorizerValue map, string key)
    {
        if (!map.TryMapGet(key, out var f))
        {
            throw new RpcServerException($"missing bool field '{key}'");
        }
        if (!f.TryAsBool(out var b))
        {
            throw new RpcServerException($"field '{key}' is not a bool");
        }
        return b;
    }

    private static double RequireFloat(VectorizerValue map, string key)
    {
        if (!map.TryMapGet(key, out var f))
        {
            throw new RpcServerException($"missing numeric field '{key}'");
        }
        if (!f.TryAsFloat(out var d))
        {
            throw new RpcServerException($"field '{key}' is not numeric");
        }
        return d;
    }

    private static long OptInt(VectorizerValue map, string key, long fallback = 0)
    {
        if (map.TryMapGet(key, out var f) && f.TryAsInt(out var i)) return i;
        return fallback;
    }

    private static double OptFloat(VectorizerValue map, string key, double fallback = 0.0)
    {
        if (map.TryMapGet(key, out var f) && f.TryAsFloat(out var d)) return d;
        return fallback;
    }

    private static bool OptBool(VectorizerValue map, string key, bool fallback = false)
    {
        if (map.TryMapGet(key, out var f) && f.TryAsBool(out var b)) return b;
        return fallback;
    }

    private static string OptStr(VectorizerValue map, string key, string fallback)
    {
        if (map.TryMapGet(key, out var f) && f.TryAsStr(out var s)) return s;
        return fallback;
    }

    private static string? OptStrNullable(VectorizerValue map, string key)
    {
        if (map.TryMapGet(key, out var f) && f.TryAsStr(out var s)) return s;
        return null;
    }

    private static IReadOnlyList<string> DecodeStringArray(VectorizerValue v, string cmd)
    {
        if (!v.TryAsArray(out var arr))
        {
            throw new RpcServerException($"{cmd}: expected Array response");
        }
        var names = new List<string>(arr.Count);
        foreach (var item in arr)
        {
            if (item.TryAsStr(out var s)) names.Add(s);
        }
        return names;
    }

    private static IReadOnlyList<SearchHit> DecodeSearchHits(IReadOnlyList<VectorizerValue> arr)
    {
        var hits = new SearchHit[arr.Count];
        for (var i = 0; i < arr.Count; i++)
        {
            var entry = arr[i];
            hits[i] = new SearchHit
            {
                Id = OptStr(entry, "id", string.Empty),
                Score = OptFloat(entry, "score"),
                Payload = OptStrNullable(entry, "payload"),
            };
        }
        return hits;
    }

    private static IReadOnlyList<BatchItemResult> DecodeBatchItems(IReadOnlyList<VectorizerValue> arr)
    {
        var items = new BatchItemResult[arr.Count];
        for (var i = 0; i < arr.Count; i++)
        {
            var entry = arr[i];
            items[i] = new BatchItemResult
            {
                Index = OptInt(entry, "index"),
                Id = OptStrNullable(entry, "id"),
                Status = OptStr(entry, "status", "unknown"),
                Error = OptStrNullable(entry, "error"),
            };
        }
        return items;
    }

    private static BatchInsertResult DecodeBatchInsert(VectorizerValue v)
    {
        return new BatchInsertResult
        {
            Inserted = OptInt(v, "inserted"),
            Failed = OptInt(v, "failed"),
            Results = DecodeBatchItems(v.TryMapGet("results", out var rVal) &&
                rVal.TryAsArray(out var rArr) ? rArr : Array.Empty<VectorizerValue>()),
        };
    }

    private static IReadOnlyList<DiscoveryChunk> DecodeDiscoveryChunks(IReadOnlyList<VectorizerValue> arr)
    {
        var result = new DiscoveryChunk[arr.Count];
        for (var i = 0; i < arr.Count; i++)
        {
            result[i] = new DiscoveryChunk
            {
                Collection = OptStr(arr[i], "collection", string.Empty),
                Score = OptFloat(arr[i], "score"),
                ContentPreview = OptStr(arr[i], "content_preview", string.Empty),
            };
        }
        return result;
    }
}
