using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;

namespace Vectorizer.Rpc;

/// <summary>
/// Options for opening an <see cref="RpcClient"/>.
/// </summary>
public sealed class RpcClientOptions
{
    /// <summary>TCP connect timeout. Defaults to 10 seconds.</summary>
    public TimeSpan ConnectTimeout { get; set; } = TimeSpan.FromSeconds(10);

    /// <summary>Per-call response timeout. Defaults to 30 seconds.</summary>
    public TimeSpan CallTimeout { get; set; } = TimeSpan.FromSeconds(30);

    /// <summary>Disable Nagle's algorithm. Defaults to true (RPC frames are complete requests).</summary>
    public bool NoDelay { get; set; } = true;
}

/// <summary>
/// Single TCP connection to a Vectorizer RPC server.
///
/// <para>Thread-safe: many callers can invoke <see cref="CallAsync(string, IReadOnlyList{VectorizerValue}, CancellationToken)"/>
/// concurrently. Writes serialise on an internal mutex; responses are
/// demultiplexed by <c>Request.Id</c> into per-call mailboxes.</para>
///
/// <para>Always issue <see cref="HelloAsync(HelloPayload, CancellationToken)"/> before
/// any data-plane command — otherwise the local gate raises
/// <see cref="RpcNotAuthenticatedException"/> and the server would also
/// reject the request.</para>
/// </summary>
public sealed class RpcClient : IAsyncDisposable, IDisposable
{
    private static readonly HashSet<string> AuthExempt = new(StringComparer.Ordinal)
    {
        "HELLO", "PING",
    };

    private readonly TcpClient _tcp;
    private readonly Stream _stream;
    private readonly RpcClientOptions _options;
    private readonly ConcurrentDictionary<uint, PendingCall> _pending = new();
    private readonly SemaphoreSlim _writeLock = new(1, 1);
    private readonly CancellationTokenSource _readerCts = new();
    private readonly TaskCompletionSource _readerDone = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private Task? _readerTask;
    private long _nextId;
    private long _authenticated; // 0/1 via Interlocked
    private int _disposed;

    private RpcClient(TcpClient tcp, Stream stream, RpcClientOptions options)
    {
        _tcp = tcp;
        _stream = stream;
        _options = options;
    }

    /// <summary>True once <see cref="HelloAsync(HelloPayload, CancellationToken)"/> has returned
    /// <c>Authenticated = true</c>.</summary>
    public bool IsAuthenticated => Interlocked.Read(ref _authenticated) == 1;

    /// <summary>Opens a TCP connection to <paramref name="host"/>:<paramref name="port"/>.</summary>
    public static async Task<RpcClient> ConnectAsync(
        string host, int port, RpcClientOptions? options = null, CancellationToken ct = default)
    {
        options ??= new RpcClientOptions();
        var tcp = new TcpClient();
        if (options.NoDelay) tcp.NoDelay = true;

        using var connectCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        connectCts.CancelAfter(options.ConnectTimeout);
        try
        {
            await tcp.ConnectAsync(host, port, connectCts.Token).ConfigureAwait(false);
        }
        catch (OperationCanceledException) when (!ct.IsCancellationRequested)
        {
            tcp.Dispose();
            throw new TimeoutException(
                $"RPC connect to {host}:{port} timed out after {options.ConnectTimeout}");
        }
        catch
        {
            tcp.Dispose();
            throw;
        }

        var stream = tcp.GetStream();
        var client = new RpcClient(tcp, stream, options);
        client._readerTask = Task.Run(() => client.ReaderLoopAsync(client._readerCts.Token));
        return client;
    }

    /// <summary>Parses <paramref name="url"/> and dials it. REST URLs are rejected.</summary>
    public static Task<RpcClient> ConnectAsync(
        string url, RpcClientOptions? options = null, CancellationToken ct = default)
    {
        var ep = EndpointParser.Parse(url);
        if (ep.Kind != EndpointKind.Rpc)
        {
            throw new ArgumentException(
                $"RpcClient cannot dial REST URL '{ep.Url}'; use the HTTP client instead "
                + "or pass a 'vectorizer://' URL",
                nameof(url));
        }
        return ConnectAsync(ep.Host, ep.Port, options, ct);
    }

    /// <summary>HELLO handshake. Must be the first call on a fresh connection.</summary>
    public async Task<HelloResponse> HelloAsync(HelloPayload payload, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(payload);
        if (payload.Version == 0) payload.Version = 1;

        var value = await RawCallAsync("HELLO", new[] { payload.ToValue() }, ct).ConfigureAwait(false);
        var response = HelloResponse.FromValue(value);
        if (response.Authenticated)
        {
            Interlocked.Exchange(ref _authenticated, 1);
        }
        return response;
    }

    /// <summary>Health-check (auth-exempt). Returns the server's PONG string.</summary>
    public async Task<string> PingAsync(CancellationToken ct = default)
    {
        var v = await RawCallAsync("PING", Array.Empty<VectorizerValue>(), ct).ConfigureAwait(false);
        if (!v.TryAsStr(out var s))
        {
            throw new RpcServerException("PING returned non-string payload");
        }
        return s;
    }

    /// <summary>
    /// Dispatches a generic command. Most callers should reach for a
    /// typed wrapper in <see cref="RpcCommands"/> instead.
    /// </summary>
    public Task<VectorizerValue> CallAsync(
        string command, IReadOnlyList<VectorizerValue> args, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(command);
        ArgumentNullException.ThrowIfNull(args);
        if (!AuthExempt.Contains(command) && !IsAuthenticated)
        {
            throw new RpcNotAuthenticatedException();
        }
        return RawCallAsync(command, args, ct);
    }

    private async Task<VectorizerValue> RawCallAsync(
        string command, IReadOnlyList<VectorizerValue> args, CancellationToken ct)
    {
        if (Volatile.Read(ref _disposed) != 0)
        {
            throw new RpcConnectionClosedException();
        }

        var id = AllocateId();
        var request = new RpcRequest(id, command, args);
        var frame = FrameCodec.EncodeFrame(request.ToWire());

        var pending = new PendingCall();
        _pending[id] = pending;

        try
        {
            await _writeLock.WaitAsync(ct).ConfigureAwait(false);
            try
            {
                await _stream.WriteAsync(frame, ct).ConfigureAwait(false);
                await _stream.FlushAsync(ct).ConfigureAwait(false);
            }
            finally
            {
                _writeLock.Release();
            }

            using var linked = CancellationTokenSource.CreateLinkedTokenSource(ct);
            linked.CancelAfter(_options.CallTimeout);
            try
            {
                var response = await pending.Task.WaitAsync(linked.Token).ConfigureAwait(false);
                if (response.Result.IsOk)
                {
                    return response.Result.Value;
                }
                throw new RpcServerException(response.Result.ErrorMessage ?? "unknown server error");
            }
            catch (OperationCanceledException) when (!ct.IsCancellationRequested)
            {
                throw new TimeoutException(
                    $"RPC call '{command}' timed out after {_options.CallTimeout}");
            }
        }
        catch (IOException ex)
        {
            throw new RpcConnectionClosedException($"send failed: {ex.Message}", ex);
        }
        catch (SocketException ex)
        {
            throw new RpcConnectionClosedException($"send failed: {ex.Message}", ex);
        }
        finally
        {
            _pending.TryRemove(id, out _);
        }
    }

    private uint AllocateId()
    {
        while (true)
        {
            var id = (uint)Interlocked.Increment(ref _nextId);
            if (id != 0) return id;
        }
    }

    private async Task ReaderLoopAsync(CancellationToken ct)
    {
        try
        {
            while (!ct.IsCancellationRequested)
            {
                var raw = await FrameCodec.ReadFrameAsync(_stream, ct).ConfigureAwait(false);
                RpcResponse response;
                try
                {
                    response = RpcResponse.FromWire(raw);
                }
                catch
                {
                    continue;
                }

                if (_pending.TryRemove(response.Id, out var pending))
                {
                    pending.TrySetResult(response);
                }
            }
        }
        catch (OperationCanceledException) when (ct.IsCancellationRequested)
        {
            // Normal shutdown.
        }
        catch (EndOfStreamException)
        {
            // Peer hung up between frames.
        }
        catch (Exception ex) when (ex is IOException or SocketException or FrameDecodeException)
        {
            FailAllPending(ex.Message);
        }
        finally
        {
            FailAllPending("connection closed");
            _readerDone.TrySetResult();
        }
    }

    private void FailAllPending(string message)
    {
        foreach (var kvp in _pending)
        {
            kvp.Value.TrySetException(new RpcConnectionClosedException(message));
        }
        _pending.Clear();
    }

    /// <summary>Closes the connection. In-flight calls raise <see cref="RpcConnectionClosedException"/>.</summary>
    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0) return;

        try { _readerCts.Cancel(); } catch { /* best-effort */ }
        try { _stream.Dispose(); } catch { /* best-effort */ }
        try { _tcp.Dispose(); } catch { /* best-effort */ }

        try
        {
            if (_readerTask is not null)
            {
                await _readerTask.WaitAsync(TimeSpan.FromSeconds(5)).ConfigureAwait(false);
            }
        }
        catch { /* best-effort */ }

        FailAllPending("connection closed");
        _readerCts.Dispose();
        _writeLock.Dispose();
    }

    /// <summary>Synchronous dispose for <c>using</c> blocks; prefer <see cref="DisposeAsync"/>.</summary>
    public void Dispose() => DisposeAsync().AsTask().GetAwaiter().GetResult();

    private sealed class PendingCall
    {
        private readonly TaskCompletionSource<RpcResponse> _tcs =
            new(TaskCreationOptions.RunContinuationsAsynchronously);

        public Task<RpcResponse> Task => _tcs.Task;

        public void TrySetResult(RpcResponse response) => _tcs.TrySetResult(response);

        public void TrySetException(Exception ex) => _tcs.TrySetException(ex);
    }
}
