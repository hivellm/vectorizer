using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Retrieve complete file content from a collection
    /// </summary>
    public async Task<Dictionary<string, object>> GetFileContentAsync(
        GetFileContentRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/file/content", request, cancellationToken);
    }

    /// <summary>
    /// List all indexed files in a collection
    /// </summary>
    public async Task<Dictionary<string, object>> ListFilesInCollectionAsync(
        ListFilesInCollectionRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/file/list", request, cancellationToken);
    }

    /// <summary>
    /// Get extractive or structural summary of an indexed file
    /// </summary>
    public async Task<Dictionary<string, object>> GetFileSummaryAsync(
        GetFileSummaryRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/file/summary", request, cancellationToken);
    }

    /// <summary>
    /// Retrieve chunks in original file order for progressive reading
    /// </summary>
    public async Task<Dictionary<string, object>> GetFileChunksOrderedAsync(
        GetFileChunksOrderedRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/file/chunks", request, cancellationToken);
    }

    /// <summary>
    /// Generate hierarchical project structure overview
    /// </summary>
    public async Task<Dictionary<string, object>> GetProjectOutlineAsync(
        GetProjectOutlineRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/file/outline", request, cancellationToken);
    }

    /// <summary>
    /// Find semantically related files using vector similarity
    /// </summary>
    public async Task<Dictionary<string, object>> GetRelatedFilesAsync(
        GetRelatedFilesRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/file/related", request, cancellationToken);
    }

    /// <summary>
    /// Semantic search filtered by file type
    /// </summary>
    public async Task<Dictionary<string, object>> SearchByFileTypeAsync(
        SearchByFileTypeRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/file/search_by_type", request, cancellationToken);
    }
}

