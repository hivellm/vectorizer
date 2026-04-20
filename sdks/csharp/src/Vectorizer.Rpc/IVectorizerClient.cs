using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Vectorizer.Rpc;

/// <summary>
/// Transport-agnostic surface for the v1 Vectorizer command catalog.
/// Implemented by <see cref="RpcVectorizerClient"/> (binary MessagePack
/// fast path, default) and <see cref="HttpVectorizerClient"/> (legacy
/// REST fallback).
/// </summary>
public interface IVectorizerClient : IAsyncDisposable, IDisposable
{
    /// <summary>Selected transport. Informational — callers should not branch on this.</summary>
    EndpointKind Transport { get; }

    /// <summary>Pings the server. Returns the server's PONG string.</summary>
    Task<string> PingAsync(CancellationToken ct = default);

    /// <summary>Lists every collection the principal can see.</summary>
    Task<IReadOnlyList<string>> ListCollectionsAsync(CancellationToken ct = default);

    /// <summary>Returns metadata for one collection.</summary>
    Task<CollectionInfo> GetCollectionInfoAsync(string name, CancellationToken ct = default);

    /// <summary>Returns the raw <see cref="VectorizerValue"/> for a single vector.</summary>
    Task<VectorizerValue> GetVectorAsync(string collection, string vectorId, CancellationToken ct = default);

    /// <summary>Runs <c>search.basic</c>.</summary>
    Task<IReadOnlyList<SearchHit>> SearchBasicAsync(
        string collection, string query, int limit, CancellationToken ct = default);

    /// <summary>Runs <c>search.intelligent</c>.</summary>
    Task<IReadOnlyList<SearchHit>> SearchIntelligentAsync(
        string query,
        IReadOnlyList<string>? collections = null,
        int? maxResults = null,
        bool? domainExpansion = null,
        double? threshold = null,
        CancellationToken ct = default);
}
