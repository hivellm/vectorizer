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

/// <summary>
/// File upload request
/// </summary>
public class FileUploadRequest
{
    public string CollectionName { get; set; } = string.Empty;
    public int? ChunkSize { get; set; }
    public int? ChunkOverlap { get; set; }
    public Dictionary<string, object>? Metadata { get; set; }
}

/// <summary>
/// File upload response
/// </summary>
public class FileUploadResponse
{
    public bool Success { get; set; }
    public string Filename { get; set; } = string.Empty;
    public string CollectionName { get; set; } = string.Empty;
    public int ChunksCreated { get; set; }
    public int VectorsCreated { get; set; }
    public long FileSize { get; set; }
    public string Language { get; set; } = string.Empty;
    public long ProcessingTimeMs { get; set; }
}

/// <summary>
/// File upload configuration
/// </summary>
public class FileUploadConfig
{
    public long MaxFileSize { get; set; }
    public int MaxFileSizeMb { get; set; }
    public List<string> AllowedExtensions { get; set; } = new();
    public bool RejectBinary { get; set; }
    public int DefaultChunkSize { get; set; }
    public int DefaultChunkOverlap { get; set; }
}

