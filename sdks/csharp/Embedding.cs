using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Generates embeddings for text
    /// </summary>
    public async Task<EmbeddingResponse> EmbedTextAsync(
        EmbeddingRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<EmbeddingResponse>(
            "POST", "/embed", request, cancellationToken);
    }
}

