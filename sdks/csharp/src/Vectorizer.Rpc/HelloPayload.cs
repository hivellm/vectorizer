using System.Collections.Generic;

namespace Vectorizer.Rpc;

/// <summary>
/// HELLO handshake payload sent as the first frame on every RPC
/// connection. At least one of <see cref="Token"/> / <see cref="ApiKey"/>
/// should be populated when the server has auth enabled.
/// </summary>
public sealed class HelloPayload
{
    /// <summary>Wire protocol version — defaults to 1.</summary>
    public long Version { get; set; } = 1;

    /// <summary>Bearer JWT (mutually exclusive with <see cref="ApiKey"/>).</summary>
    public string? Token { get; set; }

    /// <summary>API key (mutually exclusive with <see cref="Token"/>).</summary>
    public string? ApiKey { get; set; }

    /// <summary>Human-readable client identifier reported in server logs.</summary>
    public string? ClientName { get; set; }

    internal VectorizerValue ToValue()
    {
        var pairs = new List<MapPair>(4)
        {
            new(VectorizerValue.OfStr("version"), VectorizerValue.OfInt(Version)),
        };
        if (!string.IsNullOrEmpty(Token))
        {
            pairs.Add(new MapPair(VectorizerValue.OfStr("token"), VectorizerValue.OfStr(Token!)));
        }
        if (!string.IsNullOrEmpty(ApiKey))
        {
            pairs.Add(new MapPair(VectorizerValue.OfStr("api_key"), VectorizerValue.OfStr(ApiKey!)));
        }
        if (!string.IsNullOrEmpty(ClientName))
        {
            pairs.Add(new MapPair(VectorizerValue.OfStr("client_name"), VectorizerValue.OfStr(ClientName!)));
        }
        return VectorizerValue.OfMap(pairs);
    }
}

/// <summary>Decoded HELLO success payload from the server.</summary>
public sealed class HelloResponse
{
    public string ServerVersion { get; init; } = string.Empty;
    public long ProtocolVersion { get; init; }
    public bool Authenticated { get; init; }
    public bool Admin { get; init; }
    public IReadOnlyList<string> Capabilities { get; init; } = System.Array.Empty<string>();

    internal static HelloResponse FromValue(VectorizerValue value)
    {
        var serverVersion = value.TryMapGet("server_version", out var sv) && sv.TryAsStr(out var svStr)
            ? svStr : string.Empty;
        var protocolVersion = value.TryMapGet("protocol_version", out var pv) && pv.TryAsInt(out var pvL)
            ? pvL : 0L;
        var authenticated = value.TryMapGet("authenticated", out var au) && au.TryAsBool(out var auB) && auB;
        var admin = value.TryMapGet("admin", out var ad) && ad.TryAsBool(out var adB) && adB;

        var caps = new List<string>();
        if (value.TryMapGet("capabilities", out var capsVal) && capsVal.TryAsArray(out var arr))
        {
            foreach (var item in arr)
            {
                if (item.TryAsStr(out var s)) caps.Add(s);
            }
        }

        return new HelloResponse
        {
            ServerVersion = serverVersion,
            ProtocolVersion = protocolVersion,
            Authenticated = authenticated,
            Admin = admin,
            Capabilities = caps,
        };
    }
}
