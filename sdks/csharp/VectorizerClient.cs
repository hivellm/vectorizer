using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using Vectorizer.Models;
using Vectorizer.Exceptions;
using System.Collections.Generic;
using System;

namespace Vectorizer;

/// <summary>
/// Main client for interacting with the Vectorizer service
/// </summary>
public partial class VectorizerClient : IDisposable
{
    private readonly HttpClient _httpClient;
    private readonly string _baseUrl;
    private readonly string? _apiKey;
    private readonly JsonSerializerOptions _jsonOptions;
    private bool _disposed;

    /// <summary>
    /// Creates a new Vectorizer client
    /// </summary>
    public VectorizerClient(ClientConfig? config = null)
    {
        config ??= new ClientConfig();

        _baseUrl = config.BaseUrl.TrimEnd('/');
        _apiKey = config.ApiKey;

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
        return await RequestAsync<List<string>>("GET", "/collections", null, cancellationToken);
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

    public void Dispose()
    {
        if (!_disposed)
        {
            _httpClient?.Dispose();
            _disposed = true;
        }
    }
}

