using System.Text.Json.Serialization;
using Vectorizer.Models;

// SearchResponse model — mirrors Go models.go SearchResponse.
// Defined here because no equivalent exists in Models/Models.cs yet.
namespace Vectorizer.Models
{
    /// <summary>
    /// Response envelope returned by search endpoints
    /// (POST /collections/{n}/search/text, POST /collections/{n}/search/file).
    /// </summary>
    public class SearchResponse
    {
        [JsonPropertyName("results")]
        public IReadOnlyList<SearchResult> Results { get; set; } = Array.Empty<SearchResult>();

        [JsonPropertyName("query")]
        public string? Query { get; set; }

        [JsonPropertyName("limit")]
        public int Limit { get; set; }

        [JsonPropertyName("collection")]
        public string? Collection { get; set; }

        [JsonPropertyName("total_results")]
        public int TotalResults { get; set; }
    }
}

namespace Vectorizer
{

public partial class VectorizerClient
{
    /// <summary>
    /// Updates the metadata payload of a vector in-place without replacing its embedding.
    /// Calls POST /update with body {collection, id, metadata}.
    /// This is distinct from <see cref="UpdateVectorAsync"/> which replaces the full Vector
    /// struct (PUT /collections/{c}/vectors/{id}).
    /// </summary>
    /// <param name="collection">Target collection name.</param>
    /// <param name="id">ID of the vector to update.</param>
    /// <param name="metadata">Key-value pairs to set as the new metadata payload.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A minimal <see cref="Vector"/> with the server-confirmed ID.</returns>
    public async Task<Vector> UpdateVectorPayloadAsync(
        string collection,
        string id,
        IDictionary<string, object> metadata,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["collection"] = collection,
            ["id"] = id,
            ["metadata"] = metadata
        };
        // Server returns {message}; we synthesise a minimal Vector from the request parameters.
        await RequestAsync<object>("POST", "/update", body, cancellationToken);
        return new Vector { Id = id };
    }

    /// <summary>
    /// Inserts a single text document with an explicit caller-supplied ID.
    /// Calls POST /insert with body {collection, id, text, metadata?}.
    /// The server may reassign a different ID; the returned Vector carries the server-assigned value.
    /// </summary>
    /// <param name="collection">Target collection name.</param>
    /// <param name="id">Client-supplied ID hint.</param>
    /// <param name="text">Raw text to embed and store.</param>
    /// <param name="metadata">Optional metadata payload.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="Vector"/> whose <c>Id</c> is the server-assigned (or fallback client) ID.</returns>
    public async Task<Vector> InsertTextWithIdAsync(
        string collection,
        string id,
        string text,
        IDictionary<string, object>? metadata,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["collection"] = collection,
            ["id"] = id,
            ["text"] = text
        };
        if (metadata != null)
        {
            body["metadata"] = metadata;
        }
        var resp = await RequestAsync<InsertTextResponse>("POST", "/insert", body, cancellationToken);
        var assignedId = !string.IsNullOrEmpty(resp.Id) ? resp.Id : id;
        return new Vector { Id = assignedId };
    }

    /// <summary>
    /// Returns a paginated list of vectors in a collection.
    /// Calls GET /collections/{collection}/vectors?limit={limit}&amp;offset={offset}.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="limit">Maximum number of vectors to return.</param>
    /// <param name="offset">Zero-based start offset.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="VectorPage"/> containing the slice and pagination metadata.</returns>
    public async Task<VectorPage> ListVectorsAsync(
        string collection,
        int limit,
        int offset,
        CancellationToken cancellationToken = default)
    {
        var query = $"?limit={limit}&offset={offset}";
        var path = $"/collections/{Uri.EscapeDataString(collection)}/vectors{query}";
        return await RequestAsync<VectorPage>("GET", path, null, cancellationToken);
    }

    /// <summary>
    /// Batch-inserts free-form text documents via POST /batch_insert.
    /// Each item map may carry "text", "id", "metadata", etc.
    /// This is distinct from <see cref="BatchInsertTextsAsync(string, BatchInsertRequest, CancellationToken)"/>
    /// which accepts typed <see cref="BatchInsertRequest"/> items.
    /// </summary>
    /// <param name="collection">Target collection name.</param>
    /// <param name="items">Sequence of raw item maps to insert.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="BatchInsertReport"/> with inserted/failed counts.</returns>
    public async Task<BatchInsertReport> BatchInsertRawTextsAsync(
        string collection,
        IEnumerable<IDictionary<string, object>> items,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["collection"] = collection,
            ["texts"] = items.ToList()
        };
        return await RequestAsync<BatchInsertReport>("POST", "/batch_insert", body, cancellationToken);
    }

    /// <summary>
    /// Bulk-inserts pre-computed embeddings, bypassing server-side embedding.
    /// Calls POST /insert_vectors with body {collection, vectors}.
    /// </summary>
    /// <param name="collection">Target collection name.</param>
    /// <param name="vectors">Sequence of <see cref="Vector"/> objects carrying dense embeddings.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="BatchInsertReport"/> with inserted/failed counts.</returns>
    public async Task<BatchInsertReport> InsertVectorsAsync(
        string collection,
        IEnumerable<Vector> vectors,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["collection"] = collection,
            ["vectors"] = vectors.ToList()
        };
        return await RequestAsync<BatchInsertReport>("POST", "/insert_vectors", body, cancellationToken);
    }

    /// <summary>
    /// Runs multiple search queries in a single round-trip via POST /batch_search.
    /// Each query map may carry a "query" text string (embedded server-side) or a raw "vector".
    /// Server envelope: {results: [SearchResponse, ...]}.
    /// This is distinct from <see cref="BatchSearchVectorsAsync"/> which accepts typed
    /// <see cref="BatchSearchRequest"/> with string-only queries.
    /// </summary>
    /// <param name="collection">Target collection name.</param>
    /// <param name="queries">Sequence of query maps.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>One <see cref="SearchResponse"/> per input query.</returns>
    public async Task<IReadOnlyList<SearchResponse>> BatchSearchQueriesAsync(
        string collection,
        IEnumerable<IDictionary<string, object>> queries,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["collection"] = collection,
            ["queries"] = queries.ToList()
        };
        var envelope = await RequestAsync<BatchSearchEnvelope>("POST", "/batch_search", body, cancellationToken);
        return envelope.Results ?? Array.Empty<SearchResponse>();
    }

    /// <summary>
    /// Batch-updates vector payloads and/or dense vectors via POST /batch_update.
    /// Each update map must carry at minimum an "id" key plus the fields to patch.
    /// This is distinct from <see cref="BatchUpdateVectorsAsync(string, BatchUpdateRequest, CancellationToken)"/>
    /// which accepts typed <see cref="BatchUpdateRequest"/> items.
    /// </summary>
    /// <param name="collection">Target collection name.</param>
    /// <param name="updates">Sequence of update maps.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="BatchUpdateReport"/> with updated/failed counts.</returns>
    public async Task<BatchUpdateReport> BatchUpdateRawVectorsAsync(
        string collection,
        IEnumerable<IDictionary<string, object>> updates,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["collection"] = collection,
            ["updates"] = updates.ToList()
        };
        return await RequestAsync<BatchUpdateReport>("POST", "/batch_update", body, cancellationToken);
    }

    /// <summary>
    /// Searches a collection using a plain text query with a simple result limit.
    /// Calls POST /collections/{collection}/search/text with body {query, limit}.
    /// Unlike <see cref="SearchTextAsync"/> no filter or payload options are accepted.
    /// Returns the full <see cref="SearchResponse"/> envelope including aggregate metadata.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="query">Plain-text query string (embedded server-side).</param>
    /// <param name="limit">Maximum number of results to return.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="SearchResponse"/> with results and aggregate metadata.</returns>
    public async Task<SearchResponse> SearchByTextAsync(
        string collection,
        string query,
        int limit,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["query"] = query,
            ["limit"] = limit
        };
        return await RequestAsync<SearchResponse>(
            "POST",
            $"/collections/{Uri.EscapeDataString(collection)}/search/text",
            body,
            cancellationToken);
    }

    /// <summary>
    /// Searches a collection for vectors associated with a given file path.
    /// Calls POST /collections/{collection}/search/file with body {file_path, limit}.
    /// Returns an empty <see cref="SearchResponse"/> if the file has not been indexed.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="filePath">Absolute or relative path of the indexed file.</param>
    /// <param name="limit">Maximum number of results to return.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="SearchResponse"/> with any matching vectors.</returns>
    public async Task<SearchResponse> SearchByFileAsync(
        string collection,
        string filePath,
        int limit,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["file_path"] = filePath,
            ["limit"] = limit
        };
        return await RequestAsync<SearchResponse>(
            "POST",
            $"/collections/{Uri.EscapeDataString(collection)}/search/file",
            body,
            cancellationToken);
    }

    // Private envelope type for /batch_search which wraps results in {results: [...]}
    private sealed class BatchSearchEnvelope
    {
        [JsonPropertyName("results")]
        public IReadOnlyList<SearchResponse>? Results { get; set; }
    }
}

} // namespace Vectorizer
