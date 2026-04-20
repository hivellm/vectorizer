using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Vectorizer.Rpc;

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

/// <summary>
/// Extension methods adding typed wrappers for every entry in the v1
/// RPC command catalog. Keeps <see cref="RpcClient"/> generic while
/// still offering ergonomic access to the common shapes.
/// </summary>
public static class RpcCommands
{
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
        var hits = new List<SearchHit>(arr.Count);
        foreach (var entry in arr)
        {
            hits.Add(new SearchHit
            {
                Id = RequireStr(entry, "id"),
                Score = RequireFloat(entry, "score"),
                Payload = entry.TryMapGet("payload", out var p) && p.TryAsStr(out var ps) ? ps : null,
            });
        }
        return hits;
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
        var hits = new List<SearchHit>(arr.Count);
        foreach (var entry in arr)
        {
            hits.Add(new SearchHit
            {
                Id = RequireStr(entry, "id"),
                Score = RequireFloat(entry, "score"),
                Payload = entry.TryMapGet("payload", out var p) && p.TryAsStr(out var ps) ? ps : null,
            });
        }
        return hits;
    }

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
}
