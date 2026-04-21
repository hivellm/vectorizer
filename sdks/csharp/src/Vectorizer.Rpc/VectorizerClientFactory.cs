using System;
using System.Net.Http;

namespace Vectorizer.Rpc;

/// <summary>
/// Transport preferred when building a <see cref="IVectorizerClient"/>.
/// Defaults to <see cref="Auto"/> — pick RPC when the URL supports it.
/// </summary>
public enum TransportKind
{
    /// <summary>Pick the transport dictated by the URL scheme (RPC for vectorizer://, REST for http(s)://).</summary>
    Auto,
    /// <summary>Force the RPC fast path (throws on REST URLs).</summary>
    Rpc,
    /// <summary>Force the legacy REST fallback (throws on vectorizer:// URLs).</summary>
    Http,
}

/// <summary>
/// Options shared by <see cref="VectorizerClientFactory"/> and the
/// DI extension. Mutable so DI consumers can reconfigure via
/// <c>IOptions&lt;VectorizerClientOptions&gt;</c>.
/// </summary>
public sealed class VectorizerClientOptions
{
    /// <summary>Connection URL (e.g. <c>vectorizer://host:15503</c>).</summary>
    public string Url { get; set; } = $"vectorizer://localhost:{EndpointParser.DefaultRpcPort}";

    /// <summary>Transport override; defaults to <see cref="TransportKind.Auto"/>.</summary>
    public TransportKind Transport { get; set; } = TransportKind.Auto;

    /// <summary>HELLO payload sent on RPC connections. Ignored for REST.</summary>
    public HelloPayload Hello { get; set; } = new() { ClientName = "vectorizer-csharp/3.0.0" };

    /// <summary>RPC-level TCP client options.</summary>
    public RpcClientOptions RpcOptions { get; set; } = new();

    /// <summary>REST API key (ignored for RPC — use <see cref="Hello"/>).</summary>
    public string? ApiKey { get; set; }

    /// <summary>Supply a custom REST HttpClient (defaults to a new one).</summary>
    public HttpClient? HttpClient { get; set; }

    /// <summary>REST request timeout.</summary>
    public TimeSpan? HttpTimeout { get; set; }
}

/// <summary>
/// Builds <see cref="IVectorizerClient"/> instances from a URL string.
/// The same parsing rules apply as in the DI extension — keep one
/// code path so both entry points stay consistent.
/// </summary>
public static class VectorizerClientFactory
{
    /// <summary>
    /// Parses <paramref name="url"/> and returns the matching transport.
    /// <list type="bullet">
    ///  <item><c>vectorizer://host[:port]</c> → <see cref="RpcVectorizerClient"/>.</item>
    ///  <item><c>host[:port]</c> (no scheme) → <see cref="RpcVectorizerClient"/>.</item>
    ///  <item><c>http(s)://host[:port]</c> → <see cref="HttpVectorizerClient"/>.</item>
    /// </list>
    /// </summary>
    public static IVectorizerClient Create(string url)
        => Create(new VectorizerClientOptions { Url = url });

    /// <summary>Overload that honours <see cref="VectorizerClientOptions"/>.</summary>
    public static IVectorizerClient Create(VectorizerClientOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        var endpoint = EndpointParser.Parse(options.Url);
        var kind = ResolveTransport(options.Transport, endpoint);

        return kind switch
        {
            EndpointKind.Rpc => new RpcVectorizerClient(
                endpoint, options.Hello, options.RpcOptions),
            EndpointKind.Rest => new HttpVectorizerClient(
                endpoint, options.ApiKey, options.HttpClient, options.HttpTimeout),
            _ => throw new InvalidOperationException($"unreachable: {kind}"),
        };
    }

    private static EndpointKind ResolveTransport(TransportKind requested, Endpoint endpoint)
    {
        switch (requested)
        {
            case TransportKind.Auto:
                return endpoint.Kind;
            case TransportKind.Rpc:
                if (endpoint.Kind != EndpointKind.Rpc)
                {
                    throw new ArgumentException(
                        $"Transport=Rpc requires a vectorizer:// URL; got REST '{endpoint.Url}'");
                }
                return EndpointKind.Rpc;
            case TransportKind.Http:
                if (endpoint.Kind != EndpointKind.Rest)
                {
                    throw new ArgumentException(
                        $"Transport=Http requires an http(s):// URL; got RPC endpoint");
                }
                return EndpointKind.Rest;
            default:
                throw new ArgumentOutOfRangeException(nameof(requested), requested, "unknown TransportKind");
        }
    }
}
