namespace Vectorizer.Models;

/// <summary>
/// Distance metric for vector similarity
/// </summary>
public enum DistanceMetric
{
    Cosine,
    Euclidean,
    DotProduct
}

/// <summary>
/// Collection configuration
/// </summary>
public class CollectionConfig
{
    public int Dimension { get; set; }
    public DistanceMetric Metric { get; set; }
}

/// <summary>
/// Request to create a collection
/// </summary>
public class CreateCollectionRequest
{
    public string Name { get; set; } = string.Empty;
    public CollectionConfig? Config { get; set; }
}

/// <summary>
/// Collection representation
/// </summary>
public class Collection
{
    public string Name { get; set; } = string.Empty;
    public CollectionConfig? Config { get; set; }
}

/// <summary>
/// Vector representation
/// </summary>
public class Vector
{
    public string Id { get; set; } = string.Empty;
    public float[] Data { get; set; } = Array.Empty<float>();
    public Dictionary<string, object>? Payload { get; set; }

    /// <summary>
    /// Optional ECC public key for payload encryption (PEM, base64, or hex format)
    /// </summary>
    public string? PublicKey { get; set; }
}

/// <summary>
/// Search options
/// </summary>
public class SearchOptions
{
    public int Limit { get; set; }
    public Dictionary<string, object>? Filter { get; set; }
    public List<string>? Payload { get; set; }
}

/// <summary>
/// Search result
/// </summary>
public class SearchResult
{
    public string Id { get; set; } = string.Empty;
    public double Score { get; set; }
    public Dictionary<string, object>? Payload { get; set; }
    public float[]? Vector { get; set; }
}

/// <summary>
/// Request to insert text
/// </summary>
public class InsertTextRequest
{
    public string Text { get; set; } = string.Empty;
    public Dictionary<string, object>? Payload { get; set; }
}

/// <summary>
/// Response from inserting text
/// </summary>
public class InsertTextResponse
{
    public string Id { get; set; } = string.Empty;
}

/// <summary>
/// Database statistics
/// </summary>
public class DatabaseStats
{
    public int Collections { get; set; }
    public int Vectors { get; set; }
}

/// <summary>
/// Collection information.
///
/// Phase25 §6 added <see cref="VectorCountHistory"/>: a per-collection
/// ring buffer of (unix_ts, count) samples surfaced by
/// GET /collections/{name}. Empty list on older servers or for
/// collections that have never been read.
/// </summary>
public class CollectionInfo
{
    public string Name { get; set; } = string.Empty;
    public int VectorCount { get; set; }
    public int Dimension { get; set; }
    public string Metric { get; set; } = string.Empty;

    [System.Text.Json.Serialization.JsonPropertyName("vector_count_history")]
    public List<VectorCountSample> VectorCountHistory { get; set; } = new();
}

/// <summary>
/// Error response from API
/// </summary>
public class ErrorResponse
{
    public string? ErrorType { get; set; }
    public string? Message { get; set; }
    public Dictionary<string, object>? Details { get; set; }
    public int? StatusCode { get; set; }
}

