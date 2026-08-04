using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

// Aliased rather than imported: HiveLLM.Thunder declares its own `Value`,
// `ValueKind`, `Request` and `Response`, which this assembly also defines.
using Thunder = HiveLLM.Thunder;

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

    /// <summary>Retained for source compatibility. Thunder always disables
    /// Nagle: every RPC frame is a complete request, so latency matters more
    /// than packing several into one segment.</summary>
    public bool NoDelay { get; set; } = true;
}

/// <summary>
/// Single connection to a Vectorizer RPC server, backed by
/// <c>HiveLLM.Thunder</c> — the HiveLLM family's shared binary RPC client,
/// the same protocol <c>vectorizer-server</c> runs, so the two ends of the
/// wire cannot drift.
///
/// <para>Thread-safe: many callers can invoke <see cref="CallAsync(string, IReadOnlyList{VectorizerValue}, CancellationToken)"/>
/// concurrently. Their calls multiplex over the one connection and are
/// demultiplexed by frame id; the frame cap, per-call timeouts and lazy
/// re-dial come from Thunder.</para>
///
/// <para>Credentials travel in the connection handshake (<c>AUTH</c>), so
/// <see cref="HelloAsync(HelloPayload, CancellationToken)"/> re-dials when its
/// payload carries a token or an API key — that is what authenticates the
/// session every later command runs under.</para>
/// </summary>
public sealed class RpcClient : IAsyncDisposable, IDisposable
{
    /// <summary>Frame-body cap, matching the server's listener so neither end
    /// rejects a frame the other is willing to send.</summary>
    public const int MaxFrameBytes = 512 * 1024 * 1024;

    private readonly string _endpoint;
    private readonly SemaphoreSlim _redialLock = new(1, 1);
    private Thunder.ClientConfig _clientConfig;
    private Thunder.ThunderClient _client;
    private int _disposed;

    private RpcClient(string endpoint, Thunder.ThunderClient client, Thunder.ClientConfig clientConfig)
    {
        _endpoint = endpoint;
        _client = client;
        _clientConfig = clientConfig;
    }

    /// <summary>
    /// How Vectorizer uses the Thunder wire: the client half of the server's
    /// <c>vectorizer_config()</c>. The <c>vectorizer</c> scheme, an
    /// <c>AUTH</c>-command handshake, no HELLO negotiation (the <c>HELLO</c>
    /// <i>command</i> is Vectorizer's own), no server push, and RESP3-style
    /// error prefixes.
    /// </summary>
    /// <remarks>
    /// Declared here rather than imported from the server so the SDK depends
    /// only on published packages.
    /// </remarks>
    public static Thunder.Config ProtocolConfig() => Thunder.Config.Standard() with
    {
        Scheme = "vectorizer",
        DefaultPort = EndpointParser.DefaultRpcPort,
        Handshake = Thunder.Handshake.AuthCommand,
        HelloStyle = Thunder.HelloStyle.NotUsed,
        Push = Thunder.PushPolicy.Reserved,
        ErrorCodes = Thunder.ErrorConvention.Resp3Prefixes,
        MaxFrameBytes = MaxFrameBytes,
    };

    /// <summary>True once the connection's handshake authenticated. Always
    /// false against an open (single-user) server, which authenticates nobody
    /// because it gates nothing.</summary>
    public bool IsAuthenticated => Volatile.Read(ref _client).IsAuthenticated;

    /// <summary>Dials <paramref name="host"/>:<paramref name="port"/>. Does not
    /// authenticate — pass credentials to
    /// <see cref="HelloAsync(HelloPayload, CancellationToken)"/>.</summary>
    public static async Task<RpcClient> ConnectAsync(
        string host, int port, RpcClientOptions? options = null, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(host);
        options ??= new RpcClientOptions();
        var endpoint = $"vectorizer://{host}:{port}";
        var clientConfig = new Thunder.ClientConfig
        {
            ConnectTimeout = options.ConnectTimeout,
            CallTimeout = options.CallTimeout,
            ClientName = "vectorizer-csharp-sdk",
        };
        var client = await DialAsync(endpoint, clientConfig, ct).ConfigureAwait(false);
        return new RpcClient(endpoint, client, clientConfig);
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

    private static async Task<Thunder.ThunderClient> DialAsync(
        string endpoint, Thunder.ClientConfig clientConfig, CancellationToken ct)
    {
        try
        {
            return await Thunder.ThunderClient
                .ConnectAsync(endpoint, ProtocolConfig(), clientConfig, ct)
                .ConfigureAwait(false);
        }
        catch (Thunder.ThunderException ex)
        {
            throw ToRpcException(ex);
        }
    }

    /// <summary>
    /// HELLO handshake: returns the server's capability list and auth flags.
    /// </summary>
    /// <remarks>
    /// When <paramref name="payload"/> carries a token or an API key, the
    /// connection is re-dialed so those credentials travel in Thunder's
    /// <c>AUTH</c> handshake. A credential-free payload reuses the existing
    /// connection.
    /// </remarks>
    public async Task<HelloResponse> HelloAsync(HelloPayload payload, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(payload);
        if (payload.Version == 0) payload.Version = 1;

        var credentials = Credentials(payload);
        if (credentials is not null)
        {
            await RedialWithAsync(payload, credentials, ct).ConfigureAwait(false);
        }

        var value = await CallAsync("HELLO", new[] { payload.ToValue() }, ct).ConfigureAwait(false);
        return HelloResponse.FromValue(value);
    }

    /// <summary>The handshake credentials a HELLO payload carries, if any.</summary>
    private static Thunder.Credentials? Credentials(HelloPayload payload)
    {
        if (!string.IsNullOrEmpty(payload.Token))
        {
            return Thunder.Credentials.Token(payload.Token!);
        }
        if (!string.IsNullOrEmpty(payload.ApiKey))
        {
            return Thunder.Credentials.ApiKey(payload.ApiKey!);
        }
        return null;
    }

    /// <summary>Dial a fresh authenticated connection and swap it in, closing
    /// the previous one. Serialized so concurrent HELLOs cannot interleave.</summary>
    private async Task RedialWithAsync(
        HelloPayload payload, Thunder.Credentials credentials, CancellationToken ct)
    {
        await _redialLock.WaitAsync(ct).ConfigureAwait(false);
        try
        {
            var clientConfig = _clientConfig with
            {
                Credentials = credentials,
                ClientName = string.IsNullOrEmpty(payload.ClientName)
                    ? _clientConfig.ClientName
                    : payload.ClientName,
            };
            var fresh = await DialAsync(_endpoint, clientConfig, ct).ConfigureAwait(false);
            var previous = Interlocked.Exchange(ref _client, fresh);
            _clientConfig = clientConfig;
            if (previous is not null)
            {
                await previous.DisposeAsync().ConfigureAwait(false);
            }
        }
        finally
        {
            _redialLock.Release();
        }
    }

    /// <summary>Health-check (auth-exempt). Returns the server's PONG string.</summary>
    public async Task<string> PingAsync(CancellationToken ct = default)
    {
        var v = await CallAsync("PING", Array.Empty<VectorizerValue>(), ct).ConfigureAwait(false);
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
    /// <remarks>
    /// The server gates un-authenticated sessions, so a data-plane command on
    /// a session that never authenticated raises
    /// <see cref="RpcNotAuthenticatedException"/>.
    /// </remarks>
    public async Task<VectorizerValue> CallAsync(
        string command, IReadOnlyList<VectorizerValue> args, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(command);
        ArgumentNullException.ThrowIfNull(args);
        if (Volatile.Read(ref _disposed) != 0)
        {
            throw new RpcConnectionClosedException();
        }

        var wireArgs = new Thunder.Value[args.Count];
        for (var i = 0; i < wireArgs.Length; i++)
        {
            wireArgs[i] = args[i].ToThunder();
        }

        try
        {
            var result = await Volatile.Read(ref _client)
                .CallAsync(command, wireArgs, ct)
                .ConfigureAwait(false);
            return VectorizerValue.FromThunder(result);
        }
        catch (Thunder.ThunderException ex)
        {
            throw ToRpcException(ex);
        }
    }

    /// <summary>Maps a typed Thunder error onto this SDK's exception hierarchy.</summary>
    private static Exception ToRpcException(Thunder.ThunderException ex) => ex.ErrorClass switch
    {
        Thunder.ThunderErrorClass.Auth => new RpcNotAuthenticatedException(ex.Message),
        Thunder.ThunderErrorClass.Server => new RpcServerException(ex.Message),
        Thunder.ThunderErrorClass.Timeout => new TimeoutException(ex.Message, ex),
        _ => new RpcConnectionClosedException(ex.Message, ex),
    };

    /// <summary>Closes the connection. In-flight calls raise <see cref="RpcConnectionClosedException"/>.</summary>
    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0) return;
        await Volatile.Read(ref _client).DisposeAsync().ConfigureAwait(false);
        _redialLock.Dispose();
    }

    /// <summary>Synchronous dispose for <c>using</c> blocks; prefer <see cref="DisposeAsync"/>.</summary>
    public void Dispose() => DisposeAsync().AsTask().GetAwaiter().GetResult();
}
