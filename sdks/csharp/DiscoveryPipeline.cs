using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    /// <summary>
    /// Runs a multi-query broad search across all collections.
    /// POST /discovery/broad_discovery
    /// </summary>
    public async Task<BroadDiscoveryResponse> BroadDiscoveryAsync(
        BroadDiscoveryRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<BroadDiscoveryResponse>(
            "POST", "/discovery/broad_discovery", request, cancellationToken);
    }

    /// <summary>
    /// Runs a focused semantic search within a single collection.
    /// POST /discovery/semantic_focus
    /// </summary>
    public async Task<SemanticFocusResponse> SemanticFocusAsync(
        SemanticFocusRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<SemanticFocusResponse>(
            "POST", "/discovery/semantic_focus", request, cancellationToken);
    }

    /// <summary>
    /// Promotes README-quality chunks to the top of a result set.
    /// POST /discovery/promote_readme
    /// </summary>
    public async Task<PromoteReadmeResponse> PromoteReadmeAsync(
        PromoteReadmeRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<PromoteReadmeResponse>(
            "POST", "/discovery/promote_readme", request, cancellationToken);
    }

    /// <summary>
    /// Compresses a chunk set into a concise bullet list.
    /// POST /discovery/compress_evidence
    /// </summary>
    public async Task<CompressEvidenceResponse> CompressEvidenceAsync(
        CompressEvidenceRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<CompressEvidenceResponse>(
            "POST", "/discovery/compress_evidence", request, cancellationToken);
    }

    /// <summary>
    /// Organises bullets into a structured answer plan.
    /// POST /discovery/build_answer_plan
    /// </summary>
    public async Task<AnswerPlan> BuildAnswerPlanAsync(
        AnswerPlanRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<AnswerPlan>(
            "POST", "/discovery/build_answer_plan", request, cancellationToken);
    }

    /// <summary>
    /// Renders an answer plan into a final LLM prompt string.
    /// POST /discovery/render_llm_prompt
    /// </summary>
    public async Task<LlmPrompt> RenderLlmPromptAsync(
        RenderPromptRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<LlmPrompt>(
            "POST", "/discovery/render_llm_prompt", request, cancellationToken);
    }
}
