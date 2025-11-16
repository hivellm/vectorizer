using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Performs hybrid search combining dense and sparse vectors
    /// </summary>
    public async Task<HybridSearchResponse> HybridSearchAsync(
        HybridSearchRequest request,
        CancellationToken cancellationToken = default)
    {
        var payload = new Dictionary<string, object>
        {
            ["query"] = request.Query,
            ["alpha"] = request.Alpha,
            ["algorithm"] = request.Algorithm,
            ["dense_k"] = request.DenseK,
            ["sparse_k"] = request.SparseK,
            ["final_k"] = request.FinalK
        };

        if (request.QuerySparse != null)
        {
            payload["query_sparse"] = new Dictionary<string, object>
            {
                ["indices"] = request.QuerySparse.Indices,
                ["values"] = request.QuerySparse.Values
            };
        }

        return await RequestAsync<HybridSearchResponse>(
            "POST",
            $"/collections/{Uri.EscapeDataString(request.Collection)}/hybrid_search",
            payload,
            cancellationToken);
    }
}

