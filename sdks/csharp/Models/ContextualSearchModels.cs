namespace Vectorizer.Models;

/// <summary>
/// Contextual search request
/// </summary>
public class ContextualSearchRequest
{
    public string Query { get; set; } = string.Empty;
    public string Collection { get; set; } = string.Empty;
    public Dictionary<string, object>? ContextFilters { get; set; }
    public int MaxResults { get; set; } = 10;
    public bool ContextReranking { get; set; } = true;
    public float ContextWeight { get; set; } = 0.5f;
}

/// <summary>
/// Contextual search response
/// </summary>
public class ContextualSearchResponse
{
    public List<IntelligentSearchResult> Results { get; set; } = new();
    public int TotalResults { get; set; }
    public long DurationMs { get; set; }
    public string Collection { get; set; } = string.Empty;
    public Dictionary<string, object>? ContextFilters { get; set; }
    public Dictionary<string, object>? Metadata { get; set; }
}

/// <summary>
/// Multi-collection search request
/// </summary>
public class MultiCollectionSearchRequest
{
    public string Query { get; set; } = string.Empty;
    public List<string> Collections { get; set; } = new();
    public int? MaxPerCollection { get; set; }
    public int? MaxTotalResults { get; set; }
    public bool CrossCollectionReranking { get; set; } = true;
}

/// <summary>
/// Multi-collection search response
/// </summary>
public class MultiCollectionSearchResponse
{
    public List<IntelligentSearchResult> Results { get; set; } = new();
    public int TotalResults { get; set; }
    public long DurationMs { get; set; }
    public List<string> CollectionsSearched { get; set; } = new();
    public Dictionary<string, int>? ResultsPerCollection { get; set; }
    public Dictionary<string, object>? Metadata { get; set; }
}

/// <summary>
/// Intelligent search result (used by contextual and multi-collection search)
/// </summary>
public class IntelligentSearchResult
{
    public string Id { get; set; } = string.Empty;
    public float Score { get; set; }
    public string Content { get; set; } = string.Empty;
    public Dictionary<string, object>? Metadata { get; set; }
    public string? Collection { get; set; }
    public string? QueryUsed { get; set; }
}

