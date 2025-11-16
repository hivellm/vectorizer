namespace Vectorizer.Models;

/// <summary>
/// Summarize text request
/// </summary>
public class SummarizeTextRequest
{
    public string Text { get; set; } = string.Empty;
    public string Method { get; set; } = "extractive"; // extractive, abstractive, keyword
    public float? CompressionRatio { get; set; }
    public string? Language { get; set; }
    public int? MaxLength { get; set; }
}

/// <summary>
/// Summarize context request
/// </summary>
public class SummarizeContextRequest
{
    public string Context { get; set; } = string.Empty;
    public string Method { get; set; } = "keyword"; // keyword, extractive, abstractive
    public int? MaxLength { get; set; }
    public string? Language { get; set; }
}

/// <summary>
/// Summarize text response
/// </summary>
public class SummarizeTextResponse
{
    public string SummaryId { get; set; } = string.Empty;
    public string Summary { get; set; } = string.Empty;
    public int OriginalLength { get; set; }
    public int SummaryLength { get; set; }
    public float CompressionRatio { get; set; }
    public string Method { get; set; } = string.Empty;
    public DateTime CreatedAt { get; set; }
}

/// <summary>
/// Summarize context response
/// </summary>
public class SummarizeContextResponse
{
    public string Summary { get; set; } = string.Empty;
    public int OriginalLength { get; set; }
    public int SummaryLength { get; set; }
    public string Method { get; set; } = string.Empty;
}

/// <summary>
/// Get summary request
/// </summary>
public class GetSummaryRequest
{
    public string SummaryId { get; set; } = string.Empty;
}

/// <summary>
/// List summaries query
/// </summary>
public class ListSummariesQuery
{
    public int? Limit { get; set; }
    public int? Offset { get; set; }
    public string? Method { get; set; }
}

/// <summary>
/// List summaries response
/// </summary>
public class ListSummariesResponse
{
    public List<SummarizeTextResponse> Summaries { get; set; } = new();
    public int TotalCount { get; set; }
}

