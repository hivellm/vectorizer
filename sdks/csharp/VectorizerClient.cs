using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using Vectorizer.Models;
using Vectorizer.Exceptions;
using System.Collections.Generic;
using System;

namespace Vectorizer;

/// <summary>
/// Main client for interacting with the Vectorizer service.
/// Supports master/replica topology for read/write routing.
/// </summary>
public partial class VectorizerClient : IDisposable
{
    private readonly HttpClient _httpClient;
    private readonly string _baseUrl;
    private readonly string? _apiKey;
    private readonly JsonSerializerOptions _jsonOptions;
    private bool _disposed;

    // Master/replica support
    private readonly HttpClient? _masterHttpClient;
    private readonly List<HttpClient> _replicaHttpClients;
    private readonly List<string> _replicaUrls;
    private readonly string? _masterUrl;
    private int _replicaIndex;
    private readonly ReadPreference _readPreference;
    private readonly bool _isReplicaMode;
    private readonly ClientConfig _config;

    /// <summary>
    /// Creates a new Vectorizer client
    /// </summary>
    public VectorizerClient(ClientConfig? config = null)
    {
        config ??= new ClientConfig();
        _config = config;

        _baseUrl = config.BaseUrl.TrimEnd('/');
        _apiKey = config.ApiKey;
        _readPreference = config.ReadPreference;
        _replicaHttpClients = new List<HttpClient>();
        _replicaUrls = new List<string>();

        _httpClient = config.HttpClient ?? new HttpClient
        {
            Timeout = TimeSpan.FromSeconds(config.TimeoutSeconds)
        };

        _httpClient.DefaultRequestHeaders.Accept.Add(
            new MediaTypeWithQualityHeaderValue("application/json"));

        if (!string.IsNullOrEmpty(_apiKey))
        {
            _httpClient.DefaultRequestHeaders.Authorization =
                new AuthenticationHeaderValue("Bearer", _apiKey);
        }

        _jsonOptions = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false
        };

        // Initialize replica mode if hosts are configured
        if (config.Hosts != null)
        {
            _isReplicaMode = true;
            _masterUrl = config.Hosts.Master.TrimEnd('/');
            _masterHttpClient = CreateHttpClient(config);

            foreach (var replicaUrl in config.Hosts.Replicas)
            {
                _replicaUrls.Add(replicaUrl.TrimEnd('/'));
                _replicaHttpClients.Add(CreateHttpClient(config));
            }
        }
    }

    private HttpClient CreateHttpClient(ClientConfig config)
    {
        var client = new HttpClient
        {
            Timeout = TimeSpan.FromSeconds(config.TimeoutSeconds)
        };
        client.DefaultRequestHeaders.Accept.Add(
            new MediaTypeWithQualityHeaderValue("application/json"));
        if (!string.IsNullOrEmpty(config.ApiKey))
        {
            client.DefaultRequestHeaders.Authorization =
                new AuthenticationHeaderValue("Bearer", config.ApiKey);
        }
        return client;
    }

    /// <summary>
    /// Gets the HTTP client and base URL for write operations (always master)
    /// </summary>
    private (HttpClient client, string baseUrl) GetWriteClient()
    {
        if (_isReplicaMode && _masterHttpClient != null && _masterUrl != null)
        {
            return (_masterHttpClient, _masterUrl);
        }
        return (_httpClient, _baseUrl);
    }

    /// <summary>
    /// Gets the HTTP client and base URL for read operations based on preference
    /// </summary>
    private (HttpClient client, string baseUrl) GetReadClient(ReadOptions? options = null)
    {
        if (!_isReplicaMode)
        {
            return (_httpClient, _baseUrl);
        }

        var preference = options?.ReadPreference ?? _readPreference;

        switch (preference)
        {
            case ReadPreference.Master:
                return (_masterHttpClient!, _masterUrl!);

            case ReadPreference.Replica:
            case ReadPreference.Nearest:
                if (_replicaHttpClients.Count == 0)
                {
                    return (_masterHttpClient!, _masterUrl!);
                }
                // Round-robin selection using Interlocked
                var idx = Interlocked.Increment(ref _replicaIndex) % _replicaHttpClients.Count;
                if (idx < 0) idx = 0;
                return (_replicaHttpClients[idx], _replicaUrls[idx]);

            default:
                return (_masterHttpClient!, _masterUrl!);
        }
    }

    /// <summary>
    /// Creates a new client that always routes reads to master.
    /// Useful for read-your-writes scenarios.
    /// </summary>
    public VectorizerClient WithMaster()
    {
        var masterConfig = new ClientConfig
        {
            BaseUrl = _config.BaseUrl,
            ApiKey = _config.ApiKey,
            TimeoutSeconds = _config.TimeoutSeconds,
            Hosts = _config.Hosts,
            ReadPreference = ReadPreference.Master
        };
        return new VectorizerClient(masterConfig);
    }

    /// <summary>
    /// Checks the server health
    /// </summary>
    public async Task HealthAsync(CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>("GET", "/health", null, cancellationToken);
    }

    /// <summary>
    /// Gets database statistics
    /// </summary>
    public async Task<DatabaseStats> GetStatsAsync(CancellationToken cancellationToken = default)
    {
        return await RequestAsync<DatabaseStats>("GET", "/stats", null, cancellationToken);
    }

    /// <summary>
    /// Lists all collections
    /// </summary>
    public async Task<List<string>> ListCollectionsAsync(CancellationToken cancellationToken = default)
    {
        var (client, baseUrl) = GetReadClient();
        var request = new HttpRequestMessage(HttpMethod.Get, $"{baseUrl}/collections");
        var response = await client.SendAsync(request, cancellationToken);
        var content = await response.Content.ReadAsStringAsync(cancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            throw new VectorizerException("REQUEST_FAILED", $"Failed to list collections: {content}", (int)response.StatusCode);
        }

        // Handle both array and {collections: [...]} response formats
        using var doc = JsonDocument.Parse(content);
        if (doc.RootElement.ValueKind == JsonValueKind.Array)
        {
            return JsonSerializer.Deserialize<List<string>>(content, _jsonOptions) ?? new List<string>();
        }
        else if (doc.RootElement.TryGetProperty("collections", out var collectionsElement))
        {
            var result = new List<string>();
            foreach (var item in collectionsElement.EnumerateArray())
            {
                if (item.TryGetProperty("name", out var nameElement))
                {
                    result.Add(nameElement.GetString() ?? "");
                }
            }
            return result;
        }
        return new List<string>();
    }

    /// <summary>
    /// Creates a new collection
    /// </summary>
    public async Task<Collection> CreateCollectionAsync(
        CreateCollectionRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Collection>("POST", "/collections", request, cancellationToken);
    }

    /// <summary>
    /// Gets collection information
    /// </summary>
    public async Task<CollectionInfo> GetCollectionInfoAsync(
        string collectionName,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<CollectionInfo>(
            "GET", $"/collections/{Uri.EscapeDataString(collectionName)}", null, cancellationToken);
    }

    /// <summary>
    /// Deletes a collection
    /// </summary>
    public async Task DeleteCollectionAsync(
        string collectionName,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "DELETE", $"/collections/{Uri.EscapeDataString(collectionName)}", null, cancellationToken);
    }

    /// <summary>
    /// Inserts text into a collection (with automatic embedding)
    /// </summary>
    public async Task<InsertTextResponse> InsertTextAsync(
        string collectionName,
        string text,
        Dictionary<string, object>? payload = null,
        CancellationToken cancellationToken = default)
    {
        var request = new InsertTextRequest
        {
            Text = text,
            Payload = payload
        };

        return await RequestAsync<InsertTextResponse>(
            "POST", $"/collections/{Uri.EscapeDataString(collectionName)}/vectors", request, cancellationToken);
    }

    /// <summary>
    /// Gets a vector by ID
    /// </summary>
    public async Task<Vector> GetVectorAsync(
        string collectionName,
        string vectorId,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Vector>(
            "GET",
            $"/collections/{Uri.EscapeDataString(collectionName)}/vectors/{Uri.EscapeDataString(vectorId)}",
            null,
            cancellationToken);
    }

    /// <summary>
    /// Updates a vector
    /// </summary>
    public async Task UpdateVectorAsync(
        string collectionName,
        string vectorId,
        Vector vector,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "PUT",
            $"/collections/{Uri.EscapeDataString(collectionName)}/vectors/{Uri.EscapeDataString(vectorId)}",
            vector,
            cancellationToken);
    }

    /// <summary>
    /// Deletes a vector
    /// </summary>
    public async Task DeleteVectorAsync(
        string collectionName,
        string vectorId,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "DELETE",
            $"/collections/{Uri.EscapeDataString(collectionName)}/vectors/{Uri.EscapeDataString(vectorId)}",
            null,
            cancellationToken);
    }

    /// <summary>
    /// Performs a vector search
    /// </summary>
    public async Task<List<SearchResult>> SearchAsync(
        string collectionName,
        float[] queryVector,
        SearchOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        var request = new Dictionary<string, object>
        {
            ["vector"] = queryVector
        };

        if (options != null)
        {
            if (options.Limit > 0)
                request["limit"] = options.Limit;
            if (options.Filter != null)
                request["filter"] = options.Filter;
            if (options.Payload != null && options.Payload.Count > 0)
                request["payload"] = options.Payload;
        }

        return await RequestAsync<List<SearchResult>>(
            "POST", $"/collections/{Uri.EscapeDataString(collectionName)}/search", request, cancellationToken);
    }

    /// <summary>
    /// Performs a text search (with automatic embedding)
    /// </summary>
    public async Task<List<SearchResult>> SearchTextAsync(
        string collectionName,
        string query,
        SearchOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        var request = new Dictionary<string, object>
        {
            ["query"] = query
        };

        if (options != null)
        {
            if (options.Limit > 0)
                request["limit"] = options.Limit;
            if (options.Filter != null)
                request["filter"] = options.Filter;
            if (options.Payload != null && options.Payload.Count > 0)
                request["payload"] = options.Payload;
        }

        return await RequestAsync<List<SearchResult>>(
            "POST",
            $"/collections/{Uri.EscapeDataString(collectionName)}/search/text",
            request,
            cancellationToken);
    }

    private async Task<T> RequestAsync<T>(
        string method,
        string path,
        object? body,
        CancellationToken cancellationToken)
    {
        var url = _baseUrl + path;
        HttpRequestMessage request;

        if (body != null)
        {
            var json = JsonSerializer.Serialize(body, _jsonOptions);
            request = new HttpRequestMessage(new HttpMethod(method), url)
            {
                Content = new StringContent(json, Encoding.UTF8, "application/json")
            };
        }
        else
        {
            request = new HttpRequestMessage(new HttpMethod(method), url);
        }

        var response = await _httpClient.SendAsync(request, cancellationToken);
        var content = await response.Content.ReadAsStringAsync(cancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            ErrorResponse? errorResponse = null;
            try
            {
                errorResponse = JsonSerializer.Deserialize<ErrorResponse>(content, _jsonOptions);
            }
            catch
            {
                // Ignore deserialization errors
            }

            if (errorResponse != null)
            {
                throw new VectorizerException(
                    errorResponse.ErrorType ?? "unknown_error",
                    errorResponse.Message ?? content,
                    (int)response.StatusCode,
                    errorResponse.Details);
            }

            throw new HttpRequestException(
                $"Request failed with status {response.StatusCode}: {content}");
        }

        if (typeof(T) == typeof(object) && string.IsNullOrEmpty(content))
        {
            return default(T)!;
        }

        try
        {
            return JsonSerializer.Deserialize<T>(content, _jsonOptions) ??
                   throw new InvalidOperationException("Response deserialization returned null");
        }
        catch (JsonException ex)
        {
            throw new InvalidOperationException($"Failed to deserialize response: {content}", ex);
        }
    }

    // ========== Graph Operations ==========

    /// <summary>
    /// Lists all nodes in a collection's graph
    /// </summary>
    public async Task<ListNodesResponse> ListGraphNodesAsync(
        string collection,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        return await RequestAsync<ListNodesResponse>(
            "GET", $"/graph/nodes/{Uri.EscapeDataString(collection)}", null, cancellationToken);
    }

    /// <summary>
    /// Gets neighbors of a specific node
    /// </summary>
    public async Task<GetNeighborsResponse> GetGraphNeighborsAsync(
        string collection,
        string nodeId,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (string.IsNullOrWhiteSpace(nodeId))
        {
            throw new ArgumentException("NodeId must be a non-empty string", nameof(nodeId));
        }

        return await RequestAsync<GetNeighborsResponse>(
            "GET", $"/graph/nodes/{Uri.EscapeDataString(collection)}/{Uri.EscapeDataString(nodeId)}/neighbors",
            null, cancellationToken);
    }

    /// <summary>
    /// Finds related nodes within N hops
    /// </summary>
    public async Task<FindRelatedResponse> FindRelatedNodesAsync(
        string collection,
        string nodeId,
        FindRelatedRequest request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (string.IsNullOrWhiteSpace(nodeId))
        {
            throw new ArgumentException("NodeId must be a non-empty string", nameof(nodeId));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        request.Validate();

        return await RequestAsync<FindRelatedResponse>(
            "POST", $"/graph/nodes/{Uri.EscapeDataString(collection)}/{Uri.EscapeDataString(nodeId)}/related",
            request, cancellationToken);
    }

    /// <summary>
    /// Finds shortest path between two nodes
    /// </summary>
    public async Task<FindPathResponse> FindGraphPathAsync(
        FindPathRequest request,
        CancellationToken cancellationToken = default)
    {
        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        request.Validate();

        return await RequestAsync<FindPathResponse>("POST", "/graph/path", request, cancellationToken);
    }

    /// <summary>
    /// Creates an explicit edge between two nodes
    /// </summary>
    public async Task<CreateEdgeResponse> CreateGraphEdgeAsync(
        CreateEdgeRequest request,
        CancellationToken cancellationToken = default)
    {
        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        request.Validate();

        return await RequestAsync<CreateEdgeResponse>("POST", "/graph/edges", request, cancellationToken);
    }

    /// <summary>
    /// Deletes an edge by ID
    /// </summary>
    public async Task DeleteGraphEdgeAsync(
        string edgeId,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(edgeId))
        {
            throw new ArgumentException("EdgeId must be a non-empty string", nameof(edgeId));
        }

        await RequestAsync<object>("DELETE", $"/graph/edges/{Uri.EscapeDataString(edgeId)}", null, cancellationToken);
    }

    /// <summary>
    /// Lists all edges in a collection
    /// </summary>
    public async Task<ListEdgesResponse> ListGraphEdgesAsync(
        string collection,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        return await RequestAsync<ListEdgesResponse>(
            "GET", $"/graph/collections/{Uri.EscapeDataString(collection)}/edges", null, cancellationToken);
    }

    /// <summary>
    /// Discovers SIMILAR_TO edges for entire collection
    /// </summary>
    public async Task<DiscoverEdgesResponse> DiscoverGraphEdgesAsync(
        string collection,
        DiscoverEdgesRequest request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        request.Validate();

        return await RequestAsync<DiscoverEdgesResponse>(
            "POST", $"/graph/discover/{Uri.EscapeDataString(collection)}", request, cancellationToken);
    }

    /// <summary>
    /// Discovers SIMILAR_TO edges for a specific node
    /// </summary>
    public async Task<DiscoverEdgesResponse> DiscoverGraphEdgesForNodeAsync(
        string collection,
        string nodeId,
        DiscoverEdgesRequest request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (string.IsNullOrWhiteSpace(nodeId))
        {
            throw new ArgumentException("NodeId must be a non-empty string", nameof(nodeId));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        request.Validate();

        return await RequestAsync<DiscoverEdgesResponse>(
            "POST", $"/graph/discover/{Uri.EscapeDataString(collection)}/{Uri.EscapeDataString(nodeId)}",
            request, cancellationToken);
    }

    /// <summary>
    /// Gets discovery status for a collection
    /// </summary>
    public async Task<DiscoveryStatusResponse> GetGraphDiscoveryStatusAsync(
        string collection,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        return await RequestAsync<DiscoveryStatusResponse>(
            "GET", $"/graph/discover/{Uri.EscapeDataString(collection)}/status", null, cancellationToken);
    }

    // ===== QDRANT ADVANCED FEATURES (1.14.x) =====

    /// <summary>
    /// Lists snapshots for a collection (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantListCollectionSnapshotsAsync(
        string collection,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "GET", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/snapshots", null, cancellationToken);
    }

    /// <summary>
    /// Creates snapshot for a collection (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantCreateCollectionSnapshotAsync(
        string collection,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/snapshots", null, cancellationToken);
    }

    /// <summary>
    /// Deletes snapshot (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantDeleteCollectionSnapshotAsync(
        string collection,
        string snapshotName,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (string.IsNullOrWhiteSpace(snapshotName))
        {
            throw new ArgumentException("SnapshotName must be a non-empty string", nameof(snapshotName));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "DELETE", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/snapshots/{Uri.EscapeDataString(snapshotName)}",
            null, cancellationToken);
    }

    /// <summary>
    /// Recovers collection from snapshot (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantRecoverCollectionSnapshotAsync(
        string collection,
        string location,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (string.IsNullOrWhiteSpace(location))
        {
            throw new ArgumentException("Location must be a non-empty string", nameof(location));
        }

        var request = new Dictionary<string, object> { { "location", location } };
        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/snapshots/recover", request, cancellationToken);
    }

    /// <summary>
    /// Lists all snapshots (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantListAllSnapshotsAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>("GET", "/qdrant/snapshots", null, cancellationToken);
    }

    /// <summary>
    /// Creates full snapshot (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantCreateFullSnapshotAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>("POST", "/qdrant/snapshots", null, cancellationToken);
    }

    /// <summary>
    /// Lists shard keys for a collection (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantListShardKeysAsync(
        string collection,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "GET", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/shards", null, cancellationToken);
    }

    /// <summary>
    /// Creates shard key (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantCreateShardKeyAsync(
        string collection,
        Dictionary<string, object> shardKey,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (shardKey == null)
        {
            throw new ArgumentNullException(nameof(shardKey));
        }

        var request = new Dictionary<string, object> { { "shard_key", shardKey } };
        return await RequestAsync<Dictionary<string, object>>(
            "PUT", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/shards", request, cancellationToken);
    }

    /// <summary>
    /// Deletes shard key (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantDeleteShardKeyAsync(
        string collection,
        Dictionary<string, object> shardKey,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (shardKey == null)
        {
            throw new ArgumentNullException(nameof(shardKey));
        }

        var request = new Dictionary<string, object> { { "shard_key", shardKey } };
        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/shards/delete", request, cancellationToken);
    }

    /// <summary>
    /// Gets cluster status (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantGetClusterStatusAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>("GET", "/qdrant/cluster", null, cancellationToken);
    }

    /// <summary>
    /// Recovers current peer (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantClusterRecoverAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>("POST", "/qdrant/cluster/recover", null, cancellationToken);
    }

    /// <summary>
    /// Removes peer from cluster (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantRemovePeerAsync(
        string peerId,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(peerId))
        {
            throw new ArgumentException("PeerId must be a non-empty string", nameof(peerId));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "DELETE", $"/qdrant/cluster/peer/{Uri.EscapeDataString(peerId)}", null, cancellationToken);
    }

    /// <summary>
    /// Lists metadata keys (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantListMetadataKeysAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>("GET", "/qdrant/cluster/metadata/keys", null, cancellationToken);
    }

    /// <summary>
    /// Gets metadata key (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantGetMetadataKeyAsync(
        string key,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(key))
        {
            throw new ArgumentException("Key must be a non-empty string", nameof(key));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "GET", $"/qdrant/cluster/metadata/keys/{Uri.EscapeDataString(key)}", null, cancellationToken);
    }

    /// <summary>
    /// Updates metadata key (Qdrant-compatible API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantUpdateMetadataKeyAsync(
        string key,
        Dictionary<string, object> value,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(key))
        {
            throw new ArgumentException("Key must be a non-empty string", nameof(key));
        }

        if (value == null)
        {
            throw new ArgumentNullException(nameof(value));
        }

        var request = new Dictionary<string, object> { { "value", value } };
        return await RequestAsync<Dictionary<string, object>>(
            "PUT", $"/qdrant/cluster/metadata/keys/{Uri.EscapeDataString(key)}", request, cancellationToken);
    }

    /// <summary>
    /// Queries points (Qdrant 1.7+ Query API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantQueryPointsAsync(
        string collection,
        Dictionary<string, object> request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/points/query", request, cancellationToken);
    }

    /// <summary>
    /// Batch queries points (Qdrant 1.7+ Query API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantBatchQueryPointsAsync(
        string collection,
        Dictionary<string, object> request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/points/query/batch", request, cancellationToken);
    }

    /// <summary>
    /// Queries points with groups (Qdrant 1.7+ Query API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantQueryPointsGroupsAsync(
        string collection,
        Dictionary<string, object> request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/points/query/groups", request, cancellationToken);
    }

    /// <summary>
    /// Searches points with groups (Qdrant Search Groups API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantSearchPointsGroupsAsync(
        string collection,
        Dictionary<string, object> request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/points/search/groups", request, cancellationToken);
    }

    /// <summary>
    /// Searches matrix pairs (Qdrant Search Matrix API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantSearchMatrixPairsAsync(
        string collection,
        Dictionary<string, object> request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/points/search/matrix/pairs", request, cancellationToken);
    }

    /// <summary>
    /// Searches matrix offsets (Qdrant Search Matrix API)
    /// </summary>
    public async Task<Dictionary<string, object>> QdrantSearchMatrixOffsetsAsync(
        string collection,
        Dictionary<string, object> request,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(collection));
        }

        if (request == null)
        {
            throw new ArgumentNullException(nameof(request));
        }

        return await RequestAsync<Dictionary<string, object>>(
            "POST", $"/qdrant/collections/{Uri.EscapeDataString(collection)}/points/search/matrix/offsets", request, cancellationToken);
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            _httpClient?.Dispose();
            _disposed = true;
        }
    }
}

