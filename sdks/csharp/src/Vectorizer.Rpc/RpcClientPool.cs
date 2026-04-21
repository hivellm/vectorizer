using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Vectorizer.Rpc;

/// <summary>
/// Configuration for <see cref="RpcClientPool"/>. <see cref="Endpoint"/> is
/// required; the remaining options take safe defaults.
/// </summary>
public sealed class RpcClientPoolOptions
{
    /// <summary>Target endpoint. Every connection in the pool dials this.</summary>
    public Endpoint Endpoint { get; set; } = null!;

    /// <summary>HELLO payload sent on each fresh connection.</summary>
    public HelloPayload Hello { get; set; } = new();

    /// <summary>Low-level client options passed through on each dial.</summary>
    public RpcClientOptions ClientOptions { get; set; } = new();

    /// <summary>
    /// Max simultaneous live + checked-out connections. Defaults to 8.
    /// Callers block on <c>AcquireAsync</c> when the limit is reached.
    /// </summary>
    public int MaxConnections { get; set; } = 8;
}

/// <summary>
/// Bounded pool of <see cref="RpcClient"/> instances. Connections are
/// dialled lazily on first use; <c>MaxConnections</c> is enforced via a
/// semaphore so callers are never able to exceed the cap.
/// </summary>
public sealed class RpcClientPool : IAsyncDisposable
{
    private readonly RpcClientPoolOptions _options;
    private readonly SemaphoreSlim _permits;
    private readonly object _idleLock = new();
    private readonly Stack<RpcClient> _idle = new();
    private int _disposed;

    public RpcClientPool(RpcClientPoolOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        if (options.Endpoint is null)
        {
            throw new ArgumentException(
                $"{nameof(RpcClientPoolOptions)}.{nameof(RpcClientPoolOptions.Endpoint)} must be set",
                nameof(options));
        }
        if (options.Endpoint.Kind != EndpointKind.Rpc)
        {
            throw new ArgumentException("pool only supports RPC endpoints", nameof(options));
        }
        if (options.MaxConnections < 1) options.MaxConnections = 8;

        _options = options;
        _permits = new SemaphoreSlim(options.MaxConnections, options.MaxConnections);
    }

    /// <summary>
    /// Checks out a client from the pool, blocking until a slot frees
    /// when capacity is exhausted. The returned <see cref="PooledRpcClient"/>
    /// MUST be disposed to release the underlying slot.
    /// </summary>
    public async Task<PooledRpcClient> AcquireAsync(CancellationToken ct = default)
    {
        ThrowIfDisposed();

        await _permits.WaitAsync(ct).ConfigureAwait(false);

        RpcClient? client = null;
        try
        {
            lock (_idleLock)
            {
                if (_idle.Count > 0) client = _idle.Pop();
            }

            if (client is not null)
            {
                return new PooledRpcClient(this, client);
            }

            client = await RpcClient.ConnectAsync(
                _options.Endpoint.Host,
                _options.Endpoint.Port,
                _options.ClientOptions,
                ct).ConfigureAwait(false);

            try
            {
                await client.HelloAsync(_options.Hello, ct).ConfigureAwait(false);
            }
            catch
            {
                await client.DisposeAsync().ConfigureAwait(false);
                throw;
            }

            return new PooledRpcClient(this, client);
        }
        catch
        {
            _permits.Release();
            throw;
        }
    }

    /// <summary>Number of clients currently sitting idle. Diagnostic only.</summary>
    public int IdleCount
    {
        get
        {
            lock (_idleLock) return _idle.Count;
        }
    }

    internal void Return(RpcClient client, bool reusable)
    {
        if (reusable && Volatile.Read(ref _disposed) == 0)
        {
            lock (_idleLock) _idle.Push(client);
        }
        else
        {
            _ = client.DisposeAsync();
        }
        _permits.Release();
    }

    /// <summary>
    /// Closes every idle connection. In-flight <see cref="PooledRpcClient"/>
    /// instances continue to own their underlying client until disposed.
    /// </summary>
    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0) return;

        List<RpcClient> idle;
        lock (_idleLock)
        {
            idle = new List<RpcClient>(_idle);
            _idle.Clear();
        }
        foreach (var client in idle)
        {
            await client.DisposeAsync().ConfigureAwait(false);
        }
        _permits.Dispose();
    }

    private void ThrowIfDisposed()
    {
        if (Volatile.Read(ref _disposed) != 0)
        {
            throw new ObjectDisposedException(nameof(RpcClientPool));
        }
    }
}

/// <summary>
/// Handle returned by <see cref="RpcClientPool.AcquireAsync(CancellationToken)"/>.
/// Disposing returns the underlying client to the pool (or evicts it
/// when a transport error has happened — call <see cref="Invalidate"/>
/// to force eviction).
/// </summary>
public sealed class PooledRpcClient : IAsyncDisposable, IDisposable
{
    private readonly RpcClientPool _pool;
    private bool _invalid;
    private int _released;

    internal PooledRpcClient(RpcClientPool pool, RpcClient client)
    {
        _pool = pool;
        Client = client;
    }

    /// <summary>The wrapped RPC client. Undefined after dispose.</summary>
    public RpcClient Client { get; }

    /// <summary>
    /// Marks the underlying connection as unusable — subsequent dispose
    /// will close it rather than return it to the pool. Call this on
    /// any transport-level error.
    /// </summary>
    public void Invalidate() => _invalid = true;

    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _released, 1) != 0) return;
        _pool.Return(Client, reusable: !_invalid);
        await Task.CompletedTask;
    }

    public void Dispose() => DisposeAsync().AsTask().GetAwaiter().GetResult();
}
