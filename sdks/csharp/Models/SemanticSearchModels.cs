namespace Vectorizer.Models;

/// <summary>
/// Semantic search request
/// </summary>
public class SemanticSearchRequest
{
    public string Query { get; set; } = string.Empty;
    public string Collection { get; set; } = string.Empty;
    public int MaxResults { get; set; } = 10;
    public bool SemanticReranking { get; set; } = true;
    public bool CrossEncoderReranking { get; set; } = false;
    public float? SimilarityThreshold { get; set; }
}

/// <summary>
/// Semantic search response
/// </summary>
public class SemanticSearchResponse
{
    public List<IntelligentSearchResult> Results { get; set; } = new();
    public int TotalResults { get; set; }
    public long DurationMs { get; set; }
    public string Collection { get; set; } = string.Empty;
    public Dictionary<string, object>? Metadata { get; set; }
}

