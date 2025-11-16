using Vectorizer.Models;

namespace Vectorizer;

/// <summary>
/// Intelligent search request
/// </summary>
public class IntelligentSearchRequest
{
    public string Query { get; set; } = string.Empty;
    public List<string> Collections { get; set; } = new();
    public int MaxResults { get; set; }
    public bool MMREnabled { get; set; }
    public bool DomainExpansion { get; set; }
    public bool TechnicalFocus { get; set; }
    public double MMRLambda { get; set; } = 0.7;
}

/// <summary>
/// Intelligent search result
/// </summary>
public class IntelligentSearchResult : SearchResult
{
    public string? Collection { get; set; }
}

public partial class VectorizerClient
{
    /// <summary>
    /// Performs an intelligent search
    /// </summary>
    public async Task<List<IntelligentSearchResult>> IntelligentSearchAsync(
        IntelligentSearchRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<List<IntelligentSearchResult>>(
            "POST", "/intelligent_search", request, cancellationToken);
    }

    /// <summary>
    /// Performs a semantic search
    /// </summary>
    public async Task<SemanticSearchResponse> SemanticSearchAsync(
        SemanticSearchRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<SemanticSearchResponse>(
            "POST", "/semantic_search", request, cancellationToken);
    }

    /// <summary>
    /// Performs a contextual search
    /// </summary>
    public async Task<ContextualSearchResponse> ContextualSearchAsync(
        ContextualSearchRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ContextualSearchResponse>(
            "POST", "/contextual_search", request, cancellationToken);
    }

    /// <summary>
    /// Performs a multi-collection search
    /// </summary>
    public async Task<MultiCollectionSearchResponse> MultiCollectionSearchAsync(
        MultiCollectionSearchRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<MultiCollectionSearchResponse>(
            "POST", "/multi_collection_search", request, cancellationToken);
    }
}

