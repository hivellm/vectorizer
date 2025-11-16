using Vectorizer.Models;

namespace Vectorizer;

/// <summary>
/// Batch text request
/// </summary>
public class BatchTextRequest
{
    public string Id { get; set; } = string.Empty;
    public string Text { get; set; } = string.Empty;
    public Dictionary<string, object>? Metadata { get; set; }
}

/// <summary>
/// Batch configuration
/// </summary>
public class BatchConfig
{
    public int? MaxBatchSize { get; set; }
    public int? ParallelWorkers { get; set; }
    public bool Atomic { get; set; } = false;
}

/// <summary>
/// Batch insert request
/// </summary>
public class BatchInsertRequest
{
    public List<BatchTextRequest> Texts { get; set; } = new();
    public BatchConfig? Config { get; set; }
}

/// <summary>
/// Batch insert response
/// </summary>
public class BatchInsertResponse
{
    public bool Success { get; set; }
    public string Collection { get; set; } = string.Empty;
    public string Operation { get; set; } = string.Empty;
    public int TotalOperations { get; set; }
    public int SuccessfulOperations { get; set; }
    public int FailedOperations { get; set; }
    public long DurationMs { get; set; }
    public List<string> Errors { get; set; } = new();
}

/// <summary>
/// Batch search query
/// </summary>
public class BatchSearchQuery
{
    public string Query { get; set; } = string.Empty;
    public int? Limit { get; set; }
    public float? ScoreThreshold { get; set; }
}

/// <summary>
/// Batch search request
/// </summary>
public class BatchSearchRequest
{
    public List<BatchSearchQuery> Queries { get; set; } = new();
    public BatchConfig? Config { get; set; }
}

/// <summary>
/// Batch search response
/// </summary>
public class BatchSearchResponse
{
    public bool Success { get; set; }
    public string Collection { get; set; } = string.Empty;
    public int TotalQueries { get; set; }
    public int SuccessfulQueries { get; set; }
    public int FailedQueries { get; set; }
    public long DurationMs { get; set; }
    public List<List<SearchResult>> Results { get; set; } = new();
    public List<string> Errors { get; set; } = new();
}

/// <summary>
/// Batch vector update
/// </summary>
public class BatchVectorUpdate
{
    public string Id { get; set; } = string.Empty;
    public float[]? Data { get; set; }
    public Dictionary<string, object>? Metadata { get; set; }
}

/// <summary>
/// Batch update request
/// </summary>
public class BatchUpdateRequest
{
    public List<BatchVectorUpdate> Updates { get; set; } = new();
    public BatchConfig? Config { get; set; }
}

/// <summary>
/// Batch delete request
/// </summary>
public class BatchDeleteRequest
{
    public List<string> VectorIds { get; set; } = new();
    public BatchConfig? Config { get; set; }
}

/// <summary>
/// Batch response (for update and delete)
/// </summary>
public class BatchResponse
{
    public bool Success { get; set; }
    public string Collection { get; set; } = string.Empty;
    public string Operation { get; set; } = string.Empty;
    public int TotalOperations { get; set; }
    public int SuccessfulOperations { get; set; }
    public int FailedOperations { get; set; }
    public long DurationMs { get; set; }
    public List<string> Errors { get; set; } = new();
}

public partial class VectorizerClient
{
    /// <summary>
    /// Performs batch insertion of texts
    /// </summary>
    public async Task<BatchInsertResponse> BatchInsertTextsAsync(
        string collectionName,
        BatchInsertRequest request,
        CancellationToken cancellationToken = default)
    {
        var payload = new Dictionary<string, object>
        {
            ["collection"] = collectionName,
            ["texts"] = request.Texts.Select(t => new Dictionary<string, object>
            {
                ["id"] = t.Id,
                ["text"] = t.Text,
                ["metadata"] = t.Metadata ?? new Dictionary<string, object>()
            }).ToList()
        };

        if (request.Config != null)
        {
            payload["config"] = new Dictionary<string, object>
            {
                ["max_batch_size"] = request.Config.MaxBatchSize ?? 100,
                ["parallel_workers"] = request.Config.ParallelWorkers ?? 4,
                ["atomic"] = request.Config.Atomic
            };
        }

        return await RequestAsync<BatchInsertResponse>(
            "POST",
            "/batch_insert",
            payload,
            cancellationToken);
    }

    /// <summary>
    /// Performs batch search
    /// </summary>
    public async Task<BatchSearchResponse> BatchSearchVectorsAsync(
        string collectionName,
        BatchSearchRequest request,
        CancellationToken cancellationToken = default)
    {
        var payload = new Dictionary<string, object>
        {
            ["collection"] = collectionName,
            ["queries"] = request.Queries.Select(q => new Dictionary<string, object>
            {
                ["query"] = q.Query,
                ["limit"] = q.Limit ?? 10,
                ["score_threshold"] = q.ScoreThreshold ?? 0.0
            }).ToList()
        };

        if (request.Config != null)
        {
            payload["config"] = new Dictionary<string, object>
            {
                ["parallel_workers"] = request.Config.ParallelWorkers ?? 2
            };
        }

        return await RequestAsync<BatchSearchResponse>(
            "POST",
            "/batch_search",
            payload,
            cancellationToken);
    }

    /// <summary>
    /// Performs batch update
    /// </summary>
    public async Task<BatchResponse> BatchUpdateVectorsAsync(
        string collectionName,
        BatchUpdateRequest request,
        CancellationToken cancellationToken = default)
    {
        var payload = new Dictionary<string, object>
        {
            ["collection"] = collectionName,
            ["updates"] = request.Updates.Select(u => new Dictionary<string, object>
            {
                ["id"] = u.Id,
                ["data"] = u.Data ?? Array.Empty<float>(),
                ["metadata"] = u.Metadata ?? new Dictionary<string, object>()
            }).ToList()
        };

        if (request.Config != null)
        {
            payload["config"] = new Dictionary<string, object>
            {
                ["atomic"] = request.Config.Atomic
            };
        }

        return await RequestAsync<BatchResponse>(
            "POST",
            "/batch_update",
            payload,
            cancellationToken);
    }

    /// <summary>
    /// Performs batch delete
    /// </summary>
    public async Task<BatchResponse> BatchDeleteVectorsAsync(
        string collectionName,
        BatchDeleteRequest request,
        CancellationToken cancellationToken = default)
    {
        var payload = new Dictionary<string, object>
        {
            ["collection"] = collectionName,
            ["vector_ids"] = request.VectorIds
        };

        if (request.Config != null)
        {
            payload["config"] = new Dictionary<string, object>
            {
                ["atomic"] = request.Config.Atomic
            };
        }

        return await RequestAsync<BatchResponse>(
            "POST",
            "/batch_delete",
            payload,
            cancellationToken);
    }
}

