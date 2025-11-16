namespace Vectorizer.Models;

/// <summary>
/// Get file content request
/// </summary>
public class GetFileContentRequest
{
    public string Collection { get; set; } = string.Empty;
    public string FilePath { get; set; } = string.Empty;
    public int? MaxSizeKb { get; set; }
}

/// <summary>
/// List files in collection request
/// </summary>
public class ListFilesInCollectionRequest
{
    public string Collection { get; set; } = string.Empty;
    public List<string>? FilterByType { get; set; }
    public int? MinChunks { get; set; }
    public int? MaxResults { get; set; }
    public string? SortBy { get; set; } // name, size, chunks, recent
}

/// <summary>
/// Get file summary request
/// </summary>
public class GetFileSummaryRequest
{
    public string Collection { get; set; } = string.Empty;
    public string FilePath { get; set; } = string.Empty;
    public string? SummaryType { get; set; } // extractive, structural, both
    public int? MaxSentences { get; set; }
}

/// <summary>
/// Get file chunks ordered request
/// </summary>
public class GetFileChunksOrderedRequest
{
    public string Collection { get; set; } = string.Empty;
    public string FilePath { get; set; } = string.Empty;
    public int? StartChunk { get; set; }
    public int? Limit { get; set; }
    public bool? IncludeContext { get; set; }
}

/// <summary>
/// Get project outline request
/// </summary>
public class GetProjectOutlineRequest
{
    public string Collection { get; set; } = string.Empty;
    public int? MaxDepth { get; set; }
    public bool? IncludeSummaries { get; set; }
    public bool? HighlightKeyFiles { get; set; }
}

/// <summary>
/// Get related files request
/// </summary>
public class GetRelatedFilesRequest
{
    public string Collection { get; set; } = string.Empty;
    public string FilePath { get; set; } = string.Empty;
    public int? Limit { get; set; }
    public float? SimilarityThreshold { get; set; }
    public bool? IncludeReason { get; set; }
}

/// <summary>
/// Search by file type request
/// </summary>
public class SearchByFileTypeRequest
{
    public string Collection { get; set; } = string.Empty;
    public string Query { get; set; } = string.Empty;
    public List<string> FileTypes { get; set; } = new();
    public int? Limit { get; set; }
    public bool? ReturnFullFiles { get; set; }
}

