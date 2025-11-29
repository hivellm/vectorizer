using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Summarize text using various methods
    /// </summary>
    public async Task<SummarizeTextResponse> SummarizeTextAsync(
        SummarizeTextRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<SummarizeTextResponse>(
            "POST", "/summarize/text", request, cancellationToken);
    }

    /// <summary>
    /// Summarize context using various methods
    /// </summary>
    public async Task<SummarizeContextResponse> SummarizeContextAsync(
        SummarizeContextRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<SummarizeContextResponse>(
            "POST", "/summarize/context", request, cancellationToken);
    }

    /// <summary>
    /// Get summary by ID
    /// </summary>
    public async Task<SummarizeTextResponse> GetSummaryAsync(
        string summaryId,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<SummarizeTextResponse>(
            "GET", $"/summarize/{Uri.EscapeDataString(summaryId)}", null, cancellationToken);
    }

    /// <summary>
    /// List summaries
    /// </summary>
    public async Task<ListSummariesResponse> ListSummariesAsync(
        ListSummariesQuery? query = null,
        CancellationToken cancellationToken = default)
    {
        var queryString = "";
        if (query != null)
        {
            var paramsList = new List<string>();
            if (query.Limit.HasValue) paramsList.Add($"limit={query.Limit.Value}");
            if (query.Offset.HasValue) paramsList.Add($"offset={query.Offset.Value}");
            if (!string.IsNullOrEmpty(query.Method)) paramsList.Add($"method={Uri.EscapeDataString(query.Method)}");
            if (paramsList.Count > 0) queryString = "?" + string.Join("&", paramsList);
        }

        return await RequestAsync<ListSummariesResponse>(
            "GET", $"/summarize{queryString}", null, cancellationToken);
    }
}

