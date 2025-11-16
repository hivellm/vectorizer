namespace Vectorizer.Models;

/// <summary>
/// Sparse vector representation
/// </summary>
public class SparseVector
{
    public int[] Indices { get; set; } = Array.Empty<int>();
    public float[] Values { get; set; } = Array.Empty<float>();
}

/// <summary>
/// Hybrid search request
/// </summary>
public class HybridSearchRequest
{
    public string Collection { get; set; } = string.Empty;
    public string Query { get; set; } = string.Empty;
    public SparseVector? QuerySparse { get; set; }
    public float Alpha { get; set; } = 0.7f;
    public string Algorithm { get; set; } = "rrf"; // rrf, weighted, alpha
    public int DenseK { get; set; } = 20;
    public int SparseK { get; set; } = 20;
    public int FinalK { get; set; } = 10;
}

/// <summary>
/// Hybrid search result
/// </summary>
public class HybridSearchResult
{
    public string Id { get; set; } = string.Empty;
    public float Score { get; set; }
    public float[]? Vector { get; set; }
    public Dictionary<string, object>? Payload { get; set; }
}

/// <summary>
/// Hybrid search response
/// </summary>
public class HybridSearchResponse
{
    public List<HybridSearchResult> Results { get; set; } = new();
    public string Query { get; set; } = string.Empty;
    public SparseVector? QuerySparse { get; set; }
    public float Alpha { get; set; }
    public string Algorithm { get; set; } = string.Empty;
    public long? DurationMs { get; set; }
}

