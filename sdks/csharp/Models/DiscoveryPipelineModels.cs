using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Request body for POST /discovery/broad_discovery.
/// </summary>
public class BroadDiscoveryRequest
{
    [JsonPropertyName("queries")]
    public List<string> Queries { get; set; } = new();

    [JsonPropertyName("k")]
    public int? K { get; set; }
}

/// <summary>
/// Response from POST /discovery/broad_discovery.
/// </summary>
public class BroadDiscoveryResponse
{
    [JsonPropertyName("chunks")]
    public IReadOnlyList<Dictionary<string, object>> Chunks { get; set; } = Array.Empty<Dictionary<string, object>>();

    [JsonPropertyName("count")]
    public int Count { get; set; }
}

/// <summary>
/// Request body for POST /discovery/semantic_focus.
/// </summary>
public class SemanticFocusRequest
{
    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("queries")]
    public List<string> Queries { get; set; } = new();

    [JsonPropertyName("k")]
    public int? K { get; set; }
}

/// <summary>
/// Response from POST /discovery/semantic_focus.
/// </summary>
public class SemanticFocusResponse
{
    [JsonPropertyName("chunks")]
    public IReadOnlyList<Dictionary<string, object>> Chunks { get; set; } = Array.Empty<Dictionary<string, object>>();

    [JsonPropertyName("count")]
    public int Count { get; set; }
}

/// <summary>
/// Request body for POST /discovery/promote_readme.
/// </summary>
public class PromoteReadmeRequest
{
    [JsonPropertyName("chunks")]
    public List<Dictionary<string, object>> Chunks { get; set; } = new();
}

/// <summary>
/// Response from POST /discovery/promote_readme.
/// </summary>
public class PromoteReadmeResponse
{
    [JsonPropertyName("promoted_chunks")]
    public IReadOnlyList<Dictionary<string, object>> PromotedChunks { get; set; } = Array.Empty<Dictionary<string, object>>();

    [JsonPropertyName("count")]
    public int Count { get; set; }
}

/// <summary>
/// Request body for POST /discovery/compress_evidence.
/// </summary>
public class CompressEvidenceRequest
{
    [JsonPropertyName("chunks")]
    public List<Dictionary<string, object>> Chunks { get; set; } = new();

    [JsonPropertyName("max_bullets")]
    public int? MaxBullets { get; set; }

    [JsonPropertyName("max_per_doc")]
    public int? MaxPerDoc { get; set; }
}

/// <summary>
/// Response from POST /discovery/compress_evidence.
/// </summary>
public class CompressEvidenceResponse
{
    [JsonPropertyName("bullets")]
    public IReadOnlyList<Dictionary<string, object>> Bullets { get; set; } = Array.Empty<Dictionary<string, object>>();

    [JsonPropertyName("count")]
    public int Count { get; set; }
}

/// <summary>
/// Request body for POST /discovery/build_answer_plan.
/// </summary>
public class AnswerPlanRequest
{
    [JsonPropertyName("bullets")]
    public List<Dictionary<string, object>> Bullets { get; set; } = new();
}

/// <summary>
/// Structured answer plan returned by POST /discovery/build_answer_plan.
/// </summary>
public class AnswerPlan
{
    [JsonPropertyName("sections")]
    public IReadOnlyList<Dictionary<string, object>> Sections { get; set; } = Array.Empty<Dictionary<string, object>>();

    [JsonPropertyName("total_bullets")]
    public int TotalBullets { get; set; }

    [JsonPropertyName("sources")]
    public IReadOnlyList<string> Sources { get; set; } = Array.Empty<string>();
}

/// <summary>
/// Request body for POST /discovery/render_llm_prompt.
/// </summary>
public class RenderPromptRequest
{
    [JsonPropertyName("plan")]
    public AnswerPlan Plan { get; set; } = new();
}

/// <summary>
/// Rendered LLM prompt returned by POST /discovery/render_llm_prompt.
/// </summary>
public class LlmPrompt
{
    [JsonPropertyName("prompt")]
    public string Prompt { get; set; } = string.Empty;

    [JsonPropertyName("length")]
    public int Length { get; set; }

    [JsonPropertyName("estimated_tokens")]
    public int EstimatedTokens { get; set; }
}
