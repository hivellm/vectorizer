using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Complete discovery pipeline with intelligent search and prompt generation
    /// </summary>
    public async Task<Dictionary<string, object>> DiscoverAsync(
        DiscoverRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/discover", request, cancellationToken);
    }

    /// <summary>
    /// Pre-filter collections by name patterns
    /// </summary>
    public async Task<Dictionary<string, object>> FilterCollectionsAsync(
        FilterCollectionsRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/discovery/filter_collections", request, cancellationToken);
    }

    /// <summary>
    /// Rank collections by relevance
    /// </summary>
    public async Task<Dictionary<string, object>> ScoreCollectionsAsync(
        ScoreCollectionsRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/discovery/score_collections", request, cancellationToken);
    }

    /// <summary>
    /// Generate query variations
    /// </summary>
    public async Task<Dictionary<string, object>> ExpandQueriesAsync(
        ExpandQueriesRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Dictionary<string, object>>(
            "POST", "/discovery/expand_queries", request, cancellationToken);
    }
}

