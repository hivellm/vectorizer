using System;
using System.Collections.Generic;
using System.Globalization;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace Vectorizer.Rpc;

/// <summary>
/// REST fallback implementation of <see cref="IVectorizerClient"/>. Wraps
/// the existing HTTP surface so callers that must point at a legacy
/// <c>http(s)://</c> URL still get the same typed API as the RPC client.
///
/// <para>This is intentionally a thin, self-contained HTTP client so the
/// Vectorizer.Rpc package has no runtime dependency on the legacy
/// flat-layout Vectorizer assembly.</para>
/// </summary>
public sealed class HttpVectorizerClient : IVectorizerClient
{
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web);

    private readonly HttpClient _http;
    private readonly bool _ownsHttp;
    private readonly Uri _baseUri;
    private readonly string? _apiKey;
    private int _disposed;

    public HttpVectorizerClient(
        Endpoint endpoint,
        string? apiKey = null,
        HttpClient? httpClient = null,
        TimeSpan? timeout = null)
    {
        ArgumentNullException.ThrowIfNull(endpoint);
        if (endpoint.Kind != EndpointKind.Rest)
        {
            throw new ArgumentException(
                $"HttpVectorizerClient requires a REST endpoint; got {endpoint.Kind}", nameof(endpoint));
        }
        _baseUri = new Uri(endpoint.Url.TrimEnd('/') + "/", UriKind.Absolute);
        _apiKey = apiKey;

        if (httpClient is null)
        {
            _http = new HttpClient { Timeout = timeout ?? TimeSpan.FromSeconds(30) };
            _ownsHttp = true;
        }
        else
        {
            _http = httpClient;
            _ownsHttp = false;
        }

        _http.DefaultRequestHeaders.Accept.Clear();
        _http.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
        if (!string.IsNullOrEmpty(_apiKey))
        {
            // JWT shape (three non-empty `.`-separated segments) →
            // `Authorization: Bearer`; raw API keys from
            // `POST /auth/keys` → `X-API-Key`. The server's auth
            // middleware treats every Bearer string as a JWT and
            // never falls back to the API-key validator, so the
            // routing has to happen client-side.
            if (LooksLikeJwt(_apiKey))
            {
                _http.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", _apiKey);
            }
            else
            {
                _http.DefaultRequestHeaders.Add("X-API-Key", _apiKey);
            }
        }
    }

    private static bool LooksLikeJwt(string token)
    {
        var parts = token.Split('.');
        if (parts.Length != 3)
        {
            return false;
        }
        foreach (var p in parts)
        {
            if (string.IsNullOrEmpty(p))
            {
                return false;
            }
        }
        return true;
    }

    public EndpointKind Transport => EndpointKind.Rest;

    public async Task<string> PingAsync(CancellationToken ct = default)
    {
        using var resp = await _http.GetAsync(new Uri(_baseUri, "health"), ct).ConfigureAwait(false);
        resp.EnsureSuccessStatusCode();
        return "PONG";
    }

    public async Task<IReadOnlyList<string>> ListCollectionsAsync(CancellationToken ct = default)
    {
        using var resp = await _http.GetAsync(new Uri(_baseUri, "collections"), ct).ConfigureAwait(false);
        resp.EnsureSuccessStatusCode();
        await using var body = await resp.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
        using var doc = await JsonDocument.ParseAsync(body, cancellationToken: ct).ConfigureAwait(false);
        var names = new List<string>();
        var root = doc.RootElement;
        if (root.ValueKind == JsonValueKind.Array)
        {
            foreach (var item in root.EnumerateArray())
            {
                if (item.ValueKind == JsonValueKind.String)
                {
                    names.Add(item.GetString()!);
                }
                else if (item.ValueKind == JsonValueKind.Object && item.TryGetProperty("name", out var n))
                {
                    names.Add(n.GetString() ?? string.Empty);
                }
            }
        }
        else if (root.TryGetProperty("collections", out var coll) && coll.ValueKind == JsonValueKind.Array)
        {
            foreach (var item in coll.EnumerateArray())
            {
                if (item.ValueKind == JsonValueKind.String)
                {
                    names.Add(item.GetString()!);
                }
                else if (item.ValueKind == JsonValueKind.Object && item.TryGetProperty("name", out var n))
                {
                    names.Add(n.GetString() ?? string.Empty);
                }
            }
        }
        return names;
    }

    public async Task<CollectionInfo> GetCollectionInfoAsync(string name, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(name);
        using var resp = await _http.GetAsync(new Uri(_baseUri, $"collections/{Uri.EscapeDataString(name)}"), ct)
            .ConfigureAwait(false);
        resp.EnsureSuccessStatusCode();
        await using var body = await resp.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
        using var doc = await JsonDocument.ParseAsync(body, cancellationToken: ct).ConfigureAwait(false);
        var root = doc.RootElement;
        return new CollectionInfo
        {
            Name = root.TryGetProperty("name", out var nm) ? nm.GetString() ?? name : name,
            VectorCount = GetLong(root, "vector_count", "vectorCount"),
            DocumentCount = GetLong(root, "document_count", "documentCount"),
            Dimension = GetLong(root, "dimension"),
            Metric = root.TryGetProperty("metric", out var m) ? m.GetString() ?? string.Empty : string.Empty,
            CreatedAt = root.TryGetProperty("created_at", out var ca)
                ? ca.GetString() ?? string.Empty
                : root.TryGetProperty("createdAt", out var ca2) ? ca2.GetString() ?? string.Empty : string.Empty,
            UpdatedAt = root.TryGetProperty("updated_at", out var ua)
                ? ua.GetString() ?? string.Empty
                : root.TryGetProperty("updatedAt", out var ua2) ? ua2.GetString() ?? string.Empty : string.Empty,
        };
    }

    public async Task<VectorizerValue> GetVectorAsync(string collection, string vectorId, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(vectorId);
        using var resp = await _http.GetAsync(
            new Uri(_baseUri,
                $"collections/{Uri.EscapeDataString(collection)}/vectors/{Uri.EscapeDataString(vectorId)}"),
            ct).ConfigureAwait(false);
        resp.EnsureSuccessStatusCode();
        var json = await resp.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        return VectorizerValue.OfStr(json);
    }

    public async Task<IReadOnlyList<SearchHit>> SearchBasicAsync(
        string collection, string query, int limit, CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(collection);
        ArgumentNullException.ThrowIfNull(query);

        using var req = new HttpRequestMessage(
            HttpMethod.Post,
            new Uri(_baseUri, $"collections/{Uri.EscapeDataString(collection)}/search"))
        {
            Content = JsonContent(new { query, limit }),
        };
        using var resp = await _http.SendAsync(req, ct).ConfigureAwait(false);
        resp.EnsureSuccessStatusCode();
        return await ParseHitsAsync(resp, ct).ConfigureAwait(false);
    }

    public async Task<IReadOnlyList<SearchHit>> SearchIntelligentAsync(
        string query,
        IReadOnlyList<string>? collections = null,
        int? maxResults = null,
        bool? domainExpansion = null,
        double? threshold = null,
        CancellationToken ct = default)
    {
        ArgumentNullException.ThrowIfNull(query);
        using var req = new HttpRequestMessage(HttpMethod.Post, new Uri(_baseUri, "intelligent_search"))
        {
            Content = JsonContent(new
            {
                query,
                collections,
                max_results = maxResults,
                domain_expansion = domainExpansion,
                threshold,
            }),
        };
        using var resp = await _http.SendAsync(req, ct).ConfigureAwait(false);
        resp.EnsureSuccessStatusCode();
        return await ParseHitsAsync(resp, ct).ConfigureAwait(false);
    }

    private static HttpContent JsonContent(object payload)
        => new StringContent(JsonSerializer.Serialize(payload, JsonOptions), Encoding.UTF8, "application/json");

    private static async Task<IReadOnlyList<SearchHit>> ParseHitsAsync(HttpResponseMessage resp, CancellationToken ct)
    {
        await using var body = await resp.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
        using var doc = await JsonDocument.ParseAsync(body, cancellationToken: ct).ConfigureAwait(false);
        var root = doc.RootElement;
        var arr = root.ValueKind == JsonValueKind.Array
            ? root
            : (root.TryGetProperty("results", out var r) ? r
                : root.TryGetProperty("hits", out var h) ? h
                : default);

        if (arr.ValueKind != JsonValueKind.Array) return System.Array.Empty<SearchHit>();

        var hits = new List<SearchHit>();
        foreach (var entry in arr.EnumerateArray())
        {
            var id = entry.TryGetProperty("id", out var idProp) ? idProp.GetString() ?? string.Empty : string.Empty;
            var score = entry.TryGetProperty("score", out var sc) && sc.ValueKind == JsonValueKind.Number
                ? sc.GetDouble()
                : 0;
            string? payload = null;
            if (entry.TryGetProperty("payload", out var p))
            {
                payload = p.ValueKind == JsonValueKind.String ? p.GetString() : p.GetRawText();
            }
            hits.Add(new SearchHit { Id = id, Score = score, Payload = payload });
        }
        return hits;
    }

    private static long GetLong(JsonElement root, params string[] names)
    {
        foreach (var n in names)
        {
            if (root.TryGetProperty(n, out var prop))
            {
                if (prop.ValueKind == JsonValueKind.Number) return prop.GetInt64();
                if (prop.ValueKind == JsonValueKind.String
                    && long.TryParse(prop.GetString(), NumberStyles.Integer, CultureInfo.InvariantCulture, out var v))
                {
                    return v;
                }
            }
        }
        return 0;
    }

    public ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0) return ValueTask.CompletedTask;
        if (_ownsHttp) _http.Dispose();
        return ValueTask.CompletedTask;
    }

    public void Dispose()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0) return;
        if (_ownsHttp) _http.Dispose();
    }
}
