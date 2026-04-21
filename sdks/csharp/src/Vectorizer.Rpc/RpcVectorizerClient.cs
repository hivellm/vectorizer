using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Vectorizer.Rpc;

/// <summary>
/// RPC implementation of <see cref="IVectorizerClient"/>. Owns a single
/// connection opened lazily on the first call, plus an eager HELLO
/// handshake on connect.
/// </summary>
public sealed class RpcVectorizerClient : IVectorizerClient
{
    private readonly Endpoint _endpoint;
    private readonly HelloPayload _hello;
    private readonly RpcClientOptions _clientOptions;
    private readonly SemaphoreSlim _initLock = new(1, 1);
    private RpcClient? _client;
    private int _disposed;

    public RpcVectorizerClient(Endpoint endpoint, HelloPayload? hello = null, RpcClientOptions? options = null)
    {
        ArgumentNullException.ThrowIfNull(endpoint);
        if (endpoint.Kind != EndpointKind.Rpc)
        {
            throw new ArgumentException(
                $"RpcVectorizerClient requires an RPC endpoint; got {endpoint.Kind}", nameof(endpoint));
        }
        _endpoint = endpoint;
        _hello = hello ?? new HelloPayload { ClientName = "vectorizer-csharp/3.0.0" };
        _clientOptions = options ?? new RpcClientOptions();
    }

    public EndpointKind Transport => EndpointKind.Rpc;

    private async Task<RpcClient> EnsureConnectedAsync(CancellationToken ct)
    {
        var existing = Volatile.Read(ref _client);
        if (existing is not null) return existing;

        await _initLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            existing = Volatile.Read(ref _client);
            if (existing is not null) return existing;

            ThrowIfDisposed();
            var client = await RpcClient.ConnectAsync(
                _endpoint.Host, _endpoint.Port, _clientOptions, ct).ConfigureAwait(false);
            try
            {
                await client.HelloAsync(_hello, ct).ConfigureAwait(false);
            }
            catch
            {
                await client.DisposeAsync().ConfigureAwait(false);
                throw;
            }

            Volatile.Write(ref _client, client);
            return client;
        }
        finally
        {
            _initLock.Release();
        }
    }

    public async Task<string> PingAsync(CancellationToken ct = default)
    {
        var client = await EnsureConnectedAsync(ct).ConfigureAwait(false);
        return await client.PingAsync(ct).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<string>> ListCollectionsAsync(CancellationToken ct = default)
    {
        var client = await EnsureConnectedAsync(ct).ConfigureAwait(false);
        return await client.ListCollectionsAsync(ct).ConfigureAwait(false);
    }

    public async Task<CollectionInfo> GetCollectionInfoAsync(string name, CancellationToken ct = default)
    {
        var client = await EnsureConnectedAsync(ct).ConfigureAwait(false);
        return await client.GetCollectionInfoAsync(name, ct).ConfigureAwait(false);
    }

    public async Task<VectorizerValue> GetVectorAsync(string collection, string vectorId, CancellationToken ct = default)
    {
        var client = await EnsureConnectedAsync(ct).ConfigureAwait(false);
        return await client.GetVectorAsync(collection, vectorId, ct).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<SearchHit>> SearchBasicAsync(
        string collection, string query, int limit, CancellationToken ct = default)
    {
        var client = await EnsureConnectedAsync(ct).ConfigureAwait(false);
        return await client.SearchBasicAsync(collection, query, limit, ct).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<SearchHit>> SearchIntelligentAsync(
        string query,
        IReadOnlyList<string>? collections = null,
        int? maxResults = null,
        bool? domainExpansion = null,
        double? threshold = null,
        CancellationToken ct = default)
    {
        var client = await EnsureConnectedAsync(ct).ConfigureAwait(false);
        return await client.SearchIntelligentAsync(
            query, collections, maxResults, domainExpansion, threshold, ct).ConfigureAwait(false);
    }

    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0) return;
        var client = Volatile.Read(ref _client);
        if (client is not null)
        {
            await client.DisposeAsync().ConfigureAwait(false);
        }
        _initLock.Dispose();
    }

    public void Dispose() => DisposeAsync().AsTask().GetAwaiter().GetResult();

    private void ThrowIfDisposed()
    {
        if (Volatile.Read(ref _disposed) != 0)
        {
            throw new ObjectDisposedException(nameof(RpcVectorizerClient));
        }
    }
}
