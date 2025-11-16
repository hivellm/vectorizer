using Vectorizer.Models;

namespace Vectorizer;

/// <summary>
/// Batch insert request
/// </summary>
public class BatchInsertRequest
{
    public List<string> Texts { get; set; } = new();
    public List<Dictionary<string, object>>? Payload { get; set; }
}

/// <summary>
/// Batch insert response
/// </summary>
public class BatchInsertResponse
{
    public int Inserted { get; set; }
    public List<string> Ids { get; set; } = new();
}

/// <summary>
/// Batch search request
/// </summary>
public class BatchSearchRequest
{
    public List<string> Queries { get; set; } = new();
    public int Limit { get; set; }
    public Dictionary<string, object>? Filter { get; set; }
}

/// <summary>
/// Batch search response
/// </summary>
public class BatchSearchResponse
{
    public List<List<SearchResult>> Results { get; set; } = new();
}

public partial class VectorizerClient
{
    /// <summary>
    /// Performs batch insertion
    /// </summary>
    public async Task<BatchInsertResponse> BatchInsertAsync(
        string collectionName,
        BatchInsertRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<BatchInsertResponse>(
            "POST",
            $"/collections/{Uri.EscapeDataString(collectionName)}/batch/insert",
            request,
            cancellationToken);
    }

    /// <summary>
    /// Performs batch search
    /// </summary>
    public async Task<BatchSearchResponse> BatchSearchAsync(
        string collectionName,
        BatchSearchRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<BatchSearchResponse>(
            "POST",
            $"/collections/{Uri.EscapeDataString(collectionName)}/batch/search",
            request,
            cancellationToken);
    }
}

