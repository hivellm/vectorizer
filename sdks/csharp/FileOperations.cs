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

    /// <summary>
    /// Upload a file for automatic text extraction, chunking, and indexing
    /// </summary>
    /// <param name="fileStream">File stream to upload</param>
    /// <param name="filename">Name of the file</param>
    /// <param name="collectionName">Target collection name</param>
    /// <param name="chunkSize">Optional chunk size in characters</param>
    /// <param name="chunkOverlap">Optional chunk overlap in characters</param>
    /// <param name="metadata">Optional metadata to attach to all chunks</param>
    /// <param name="publicKey">Optional ECC public key for payload encryption (PEM, base64, or hex format)</param>
    /// <param name="cancellationToken">Cancellation token</param>
    /// <returns>File upload response</returns>
    public async Task<FileUploadResponse> UploadFileAsync(
        Stream fileStream,
        string filename,
        string collectionName,
        int? chunkSize = null,
        int? chunkOverlap = null,
        Dictionary<string, object>? metadata = null,
        string? publicKey = null,
        CancellationToken cancellationToken = default)
    {
        using var content = new MultipartFormDataContent();

        // Add file
        var streamContent = new StreamContent(fileStream);
        content.Add(streamContent, "file", filename);

        // Add collection name
        content.Add(new StringContent(collectionName), "collection_name");

        // Add optional parameters
        if (chunkSize.HasValue)
            content.Add(new StringContent(chunkSize.Value.ToString()), "chunk_size");

        if (chunkOverlap.HasValue)
            content.Add(new StringContent(chunkOverlap.Value.ToString()), "chunk_overlap");

        if (metadata != null)
        {
            var metadataJson = System.Text.Json.JsonSerializer.Serialize(metadata);
            content.Add(new StringContent(metadataJson), "metadata");
        }

        if (publicKey != null)
            content.Add(new StringContent(publicKey), "public_key");

        var response = await _httpClient.PostAsync("/files/upload", content, cancellationToken);
        response.EnsureSuccessStatusCode();

        var responseJson = await response.Content.ReadAsStringAsync(cancellationToken);
        return System.Text.Json.JsonSerializer.Deserialize<FileUploadResponse>(responseJson)
            ?? throw new InvalidOperationException("Failed to deserialize upload response");
    }

    /// <summary>
    /// Upload file content directly as a string
    /// </summary>
    /// <param name="content">File content as string</param>
    /// <param name="filename">Name of the file (used for extension detection)</param>
    /// <param name="collectionName">Target collection name</param>
    /// <param name="chunkSize">Optional chunk size in characters</param>
    /// <param name="chunkOverlap">Optional chunk overlap in characters</param>
    /// <param name="metadata">Optional metadata to attach to all chunks</param>
    /// <param name="publicKey">Optional ECC public key for payload encryption (PEM, base64, or hex format)</param>
    /// <param name="cancellationToken">Cancellation token</param>
    /// <returns>File upload response</returns>
    public async Task<FileUploadResponse> UploadFileContentAsync(
        string content,
        string filename,
        string collectionName,
        int? chunkSize = null,
        int? chunkOverlap = null,
        Dictionary<string, object>? metadata = null,
        string? publicKey = null,
        CancellationToken cancellationToken = default)
    {
        using var stream = new MemoryStream(System.Text.Encoding.UTF8.GetBytes(content));
        return await UploadFileAsync(
            stream, filename, collectionName,
            chunkSize, chunkOverlap, metadata, publicKey,
            cancellationToken);
    }

    /// <summary>
    /// Get file upload configuration from server
    /// </summary>
    /// <param name="cancellationToken">Cancellation token</param>
    /// <returns>File upload configuration</returns>
    public async Task<FileUploadConfig> GetUploadConfigAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<FileUploadConfig>(
            "GET", "/files/config", null, cancellationToken);
    }
}

