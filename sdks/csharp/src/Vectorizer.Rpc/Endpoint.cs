using System;
using System.Globalization;

namespace Vectorizer.Rpc;

/// <summary>Transport selected by <see cref="EndpointParser.Parse(string)"/>.</summary>
public enum EndpointKind
{
    Rpc,
    Rest,
}

/// <summary>
/// Discriminated parse result. Either <see cref="Host"/> + <see cref="Port"/>
/// (when <see cref="Kind"/> = <see cref="EndpointKind.Rpc"/>) or
/// <see cref="Url"/> (when <see cref="EndpointKind.Rest"/>) is populated.
/// </summary>
public sealed class Endpoint
{
    public EndpointKind Kind { get; }
    public string Host { get; }
    public int Port { get; }
    public string Url { get; }

    private Endpoint(EndpointKind kind, string host, int port, string url)
    {
        Kind = kind;
        Host = host;
        Port = port;
        Url = url;
    }

    /// <summary>Constructs an RPC endpoint programmatically (bypasses URL parsing).</summary>
    public static Endpoint Rpc(string host, int port)
    {
        ArgumentException.ThrowIfNullOrEmpty(host);
        if (port < 0 || port > 65535)
        {
            throw new ArgumentOutOfRangeException(nameof(port), port, "port must be in 0..65535");
        }
        return new Endpoint(EndpointKind.Rpc, host, port, string.Empty);
    }

    /// <summary>Constructs a REST endpoint programmatically (bypasses URL parsing).</summary>
    public static Endpoint Rest(string url)
    {
        ArgumentException.ThrowIfNullOrEmpty(url);
        return new Endpoint(EndpointKind.Rest, string.Empty, 0, url);
    }

    /// <summary>Host:port string suitable for <see cref="System.Net.Sockets.TcpClient"/>.</summary>
    public string Authority => Kind == EndpointKind.Rpc
        ? (Host.StartsWith('[') ? $"{Host}:{Port}" : $"{Host}:{Port}")
        : throw new InvalidOperationException("Authority is only valid for RPC endpoints");

    public override string ToString() => Kind switch
    {
        EndpointKind.Rpc => $"vectorizer://{Authority}",
        EndpointKind.Rest => Url,
        _ => base.ToString()!,
    };
}

/// <summary>
/// Canonical URL parser shared by <c>new VectorizerClient(string url)</c>
/// and <c>services.AddVectorizerClient(url)</c>. Mirrors the Rust/Go/
/// Python/TypeScript parsers byte-for-byte so polyglot deployments share
/// a single connection-string contract.
/// </summary>
public static class EndpointParser
{
    /// <summary>Default RPC port. Matches the server's <c>RpcConfig::default_port()</c>.</summary>
    public const int DefaultRpcPort = 15503;

    /// <summary>Default REST port. Matches the server's <c>ServerConfig::default()</c>.</summary>
    public const int DefaultHttpPort = 15002;

    /// <summary>
    /// Parses a connection string into a typed <see cref="Endpoint"/>.
    /// Throws <see cref="ArgumentException"/> for any invalid form.
    /// </summary>
    /// <param name="url">Connection string; see spec for accepted forms.</param>
    /// <returns>A typed <see cref="Endpoint"/>.</returns>
    public static Endpoint Parse(string? url)
    {
        if (url is null)
        {
            throw new ArgumentException("endpoint URL must not be null", nameof(url));
        }
        var trimmed = url.Trim();
        if (trimmed.Length == 0)
        {
            throw new ArgumentException("endpoint URL must not be empty", nameof(url));
        }

        var schemeIdx = trimmed.IndexOf("://", StringComparison.Ordinal);
        if (schemeIdx >= 0)
        {
            var scheme = trimmed[..schemeIdx].ToLowerInvariant();
            var rest = trimmed[(schemeIdx + 3)..];
            return scheme switch
            {
                "vectorizer" => ParseRpcAuthority(rest),
                "http" or "https" => ParseRest(scheme, rest, trimmed),
                _ => throw new ArgumentException(
                    $"unsupported URL scheme '{trimmed[..schemeIdx]}'; expected 'vectorizer', 'http', or 'https'",
                    nameof(url)),
            };
        }

        // No scheme — treat as bare host[:port] for RPC.
        return ParseRpcAuthority(trimmed);
    }

    private static Endpoint ParseRpcAuthority(string authority)
    {
        if (authority.Length == 0)
        {
            throw new ArgumentException("invalid authority: missing host", nameof(authority));
        }
        if (authority.Contains('@'))
        {
            throw new ArgumentException(
                "URL carries credentials in the userinfo section; "
                + "pass credentials to the HELLO handshake instead of embedding them in the URL",
                nameof(authority));
        }

        var hostPort = authority;
        foreach (var sep in new[] { '/', '?', '#' })
        {
            var idx = hostPort.IndexOf(sep);
            if (idx >= 0) hostPort = hostPort[..idx];
        }
        if (hostPort.Length == 0)
        {
            throw new ArgumentException(
                $"invalid authority '{authority}': missing host", nameof(authority));
        }

        // IPv6 literal — bracket-aware split.
        if (hostPort.StartsWith('['))
        {
            var close = hostPort.IndexOf(']');
            if (close < 0)
            {
                throw new ArgumentException(
                    $"invalid authority '{authority}': unterminated IPv6 literal '['",
                    nameof(authority));
            }
            var host = hostPort[..(close + 1)];
            var after = hostPort[(close + 1)..];
            if (after.Length == 0)
            {
                return Endpoint.Rpc(host, DefaultRpcPort);
            }
            if (!after.StartsWith(':'))
            {
                throw new ArgumentException(
                    $"invalid authority '{authority}': expected ':<port>' after IPv6 literal, got '{after}'",
                    nameof(authority));
            }
            return Endpoint.Rpc(host, ParsePort(after[1..], authority));
        }

        // Hostname or IPv4. Split on the last colon.
        var colon = hostPort.LastIndexOf(':');
        if (colon >= 0)
        {
            var host = hostPort[..colon];
            var portStr = hostPort[(colon + 1)..];
            if (host.Length == 0)
            {
                throw new ArgumentException(
                    $"invalid authority '{authority}': missing host before ':<port>'",
                    nameof(authority));
            }
            return Endpoint.Rpc(host, ParsePort(portStr, authority));
        }

        return Endpoint.Rpc(hostPort, DefaultRpcPort);
    }

    private static Endpoint ParseRest(string scheme, string rest, string raw)
    {
        if (rest.Length == 0)
        {
            throw new ArgumentException(
                $"invalid authority in URL '{raw}': missing host", nameof(raw));
        }
        if (rest.Contains('@'))
        {
            throw new ArgumentException(
                "URL carries credentials in the userinfo section; "
                + "pass credentials to the HELLO handshake instead of embedding them in the URL",
                nameof(raw));
        }
        return Endpoint.Rest(scheme + "://" + rest);
    }

    private static int ParsePort(string portStr, string authority)
    {
        if (!int.TryParse(portStr, NumberStyles.Integer, CultureInfo.InvariantCulture, out var port))
        {
            throw new ArgumentException(
                $"invalid authority '{authority}': invalid port '{portStr}'", nameof(authority));
        }
        if (port < 0 || port > 65535)
        {
            throw new ArgumentException(
                $"invalid authority '{authority}': port {port} is out of range 0..65535",
                nameof(authority));
        }
        return port;
    }
}
