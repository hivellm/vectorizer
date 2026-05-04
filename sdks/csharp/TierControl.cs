using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Vectorizer.Exceptions;
using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Deletes every vector in a collection that matches the given metadata filter.
    /// Calls POST /collections/{name}/vectors/delete_by_filter with body {"filter": filter}.
    /// An empty filter is rejected client-side to prevent accidental full-collection wipes.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="filter">Non-empty metadata filter. Keys/values must match vector payload fields.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="DeleteByFilterReport"/> with Scanned, Matched, Deleted, and Results.</returns>
    /// <exception cref="VectorizerException">Thrown with type "validation_error" when filter is null or empty.</exception>
    public async Task<DeleteByFilterReport> DeleteByFilterAsync(
        string collection,
        IDictionary<string, object> filter,
        CancellationToken cancellationToken = default)
    {
        if (filter == null || filter.Count == 0)
        {
            throw new VectorizerException(
                "validation_error",
                "filter must not be empty",
                0);
        }

        var body = new Dictionary<string, object> { ["filter"] = filter };
        var path = $"/collections/{Uri.EscapeDataString(collection)}/vectors/delete_by_filter";
        return await RequestAsync<DeleteByFilterReport>("POST", path, body, cancellationToken);
    }

    /// <summary>
    /// Deletes every vector in a collection that matches the typed Qdrant filter.
    /// Calls POST /collections/{name}/vectors/delete_by_filter with body {"filter": filter}.
    /// An empty filter is rejected client-side to prevent accidental full-collection wipes.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="filter">Non-empty typed Qdrant filter.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="DeleteByFilterReport"/> with Scanned, Matched, Deleted, and Results.</returns>
    /// <exception cref="ArgumentNullException">Thrown when <paramref name="filter"/> is null.</exception>
    /// <exception cref="ArgumentException">Thrown when <paramref name="filter"/> is empty (no conditions).</exception>
    public async Task<DeleteByFilterReport> DeleteByFilterAsync(
        string collection,
        QdrantFilter filter,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(filter);
        if (filter.IsEmpty())
            throw new ArgumentException(
                "filter must contain at least one must/should/must_not condition",
                nameof(filter));

        var body = new { filter };
        var path = $"/collections/{Uri.EscapeDataString(collection)}/vectors/delete_by_filter";
        return await RequestAsync<DeleteByFilterReport>("POST", path, body, cancellationToken);
    }

    /// <summary>
    /// Applies a JSON-merge-patch to every vector matching the given filter.
    /// Calls POST /collections/{name}/vectors/bulk_update_metadata with body
    /// {"filter": filter, "patch": patch}.
    /// An empty filter is rejected client-side to prevent accidental full-collection updates.
    /// Patch semantics follow RFC 7396: keys in patch overwrite existing payload values;
    /// null values remove keys.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="filter">Non-empty metadata filter.</param>
    /// <param name="patch">Key/value pairs to merge into matched vector payloads.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="BulkUpdateReport"/> with Scanned, Matched, Updated, and Results.</returns>
    /// <exception cref="VectorizerException">Thrown with type "validation_error" when filter is null or empty.</exception>
    public async Task<BulkUpdateReport> BulkUpdateMetadataAsync(
        string collection,
        IDictionary<string, object> filter,
        IDictionary<string, object> patch,
        CancellationToken cancellationToken = default)
    {
        if (filter == null || filter.Count == 0)
        {
            throw new VectorizerException(
                "validation_error",
                "filter must not be empty",
                0);
        }

        var body = new Dictionary<string, object>
        {
            ["filter"] = filter,
            ["patch"] = patch
        };
        var path = $"/collections/{Uri.EscapeDataString(collection)}/vectors/bulk_update_metadata";
        return await RequestAsync<BulkUpdateReport>("POST", path, body, cancellationToken);
    }

    /// <summary>
    /// Applies a JSON-merge-patch to every vector matching the typed Qdrant filter.
    /// Calls POST /collections/{name}/vectors/bulk_update_metadata with body
    /// {"filter": filter, "patch": patch}.
    /// An empty filter is rejected client-side to prevent accidental full-collection updates.
    /// Patch semantics follow RFC 7396: keys in patch overwrite existing payload values;
    /// null values remove keys.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="filter">Non-empty typed Qdrant filter.</param>
    /// <param name="patch">Key/value pairs to merge into matched vector payloads.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="BulkUpdateReport"/> with Scanned, Matched, Updated, and Results.</returns>
    /// <exception cref="ArgumentNullException">Thrown when <paramref name="filter"/> is null.</exception>
    /// <exception cref="ArgumentException">Thrown when <paramref name="filter"/> is empty (no conditions).</exception>
    public async Task<BulkUpdateReport> BulkUpdateMetadataAsync(
        string collection,
        QdrantFilter filter,
        IDictionary<string, object> patch,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(filter);
        if (filter.IsEmpty())
            throw new ArgumentException(
                "filter must contain at least one must/should/must_not condition",
                nameof(filter));

        var body = new { filter, patch };
        var path = $"/collections/{Uri.EscapeDataString(collection)}/vectors/bulk_update_metadata";
        return await RequestAsync<BulkUpdateReport>("POST", path, body, cancellationToken);
    }

    /// <summary>
    /// Copies vectors from src to dst without re-embedding. Unlike MoveVectors,
    /// the source vectors are not deleted.
    /// Calls POST /collections/{src}/vectors/copy with body
    /// {"destination": dst, "ids": ids}.
    /// Per-id status values: ok | missing_in_src | dst_insert_failed.
    /// </summary>
    /// <param name="src">Source collection name.</param>
    /// <param name="dst">Destination collection name.</param>
    /// <param name="ids">Vector IDs to copy.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="CopyReport"/> with Src, Dst, Requested, Copied, Failed, and Results.</returns>
    public async Task<CopyReport> CopyVectorsAsync(
        string src,
        string dst,
        IEnumerable<string> ids,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object>
        {
            ["destination"] = dst,
            ["ids"] = ids
        };
        var path = $"/collections/{Uri.EscapeDataString(src)}/vectors/copy";
        return await RequestAsync<CopyReport>("POST", path, body, cancellationToken);
    }

    /// <summary>
    /// Re-quantizes an existing collection in-place without re-embedding.
    /// Calls POST /collections/{name}/reencode with body {"target_encoding": targetEncoding}.
    /// Valid encoding values: "sq8", "binary", "fp32".
    /// </summary>
    /// <param name="name">Collection name.</param>
    /// <param name="targetEncoding">Target encoding format: "sq8", "binary", or "fp32".</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="ReencodeJob"/> with State == "completed" on success.</returns>
    public async Task<ReencodeJob> ReencodeCollectionAsync(
        string name,
        string targetEncoding,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object> { ["target_encoding"] = targetEncoding };
        var path = $"/collections/{Uri.EscapeDataString(name)}/reencode";
        return await RequestAsync<ReencodeJob>("POST", path, body, cancellationToken);
    }

    /// <summary>
    /// Sets or clears a per-collection TTL.
    /// Calls POST /collections/{name}/ttl with body {"ttl_secs": ttlSecs}.
    /// Pass null to clear the collection-level TTL. Existing vectors are not
    /// retroactively expired; only subsequent insertions that carry __expires_at
    /// in their payload are affected.
    /// For per-vector expiry use <see cref="SetVectorExpiryAsync"/>.
    /// </summary>
    /// <param name="name">Collection name.</param>
    /// <param name="ttlSecs">TTL in seconds, or null to clear.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    public async Task SetCollectionTtlAsync(
        string name,
        long? ttlSecs,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object?> { ["ttl_secs"] = ttlSecs };
        var path = $"/collections/{Uri.EscapeDataString(name)}/ttl";
        await RequestAsync<object>("POST", path, body, cancellationToken);
    }

    /// <summary>
    /// Sets or clears a per-vector expiry timestamp.
    /// Calls PATCH /collections/{name}/vectors/{id}/expiry with body {"expires_at": expiresAt}.
    /// Pass null to clear an existing expiry. The timestamp is stored as __expires_at
    /// inside the vector payload and is read by the per-collection TTL reaper.
    /// </summary>
    /// <param name="collection">Collection name.</param>
    /// <param name="id">Vector ID.</param>
    /// <param name="expiresAt">Unix timestamp (seconds) for expiry, or null to clear.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    public async Task SetVectorExpiryAsync(
        string collection,
        string id,
        long? expiresAt,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object?> { ["expires_at"] = expiresAt };
        var path = $"/collections/{Uri.EscapeDataString(collection)}/vectors/{Uri.EscapeDataString(id)}/expiry";
        await RequestAsync<object>("PATCH", path, body, cancellationToken);
    }
}
