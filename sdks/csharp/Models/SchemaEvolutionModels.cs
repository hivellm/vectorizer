using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Parameters for POST /collections/{name}/reindex.
/// </summary>
public class ReindexParams
{
    [JsonPropertyName("m")]
    public int M { get; set; }

    [JsonPropertyName("ef_construction")]
    public int EfConstruction { get; set; }

    [JsonPropertyName("ef_search")]
    public int EfSearch { get; set; }
}

/// <summary>
/// Job descriptor returned by POST /collections/{name}/reindex.
/// </summary>
public class ReindexJob
{
    [JsonPropertyName("job_id")]
    public string JobId { get; set; } = string.Empty;

    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("state")]
    public string State { get; set; } = string.Empty;

    [JsonPropertyName("params")]
    public Dictionary<string, object> Params { get; set; } = new();

    [JsonPropertyName("progress")]
    public double Progress { get; set; }
}

/// <summary>
/// Metadata for a native collection snapshot.
/// </summary>
public class NativeSnapshotInfo
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("created_at")]
    public string CreatedAt { get; set; } = string.Empty;

    [JsonPropertyName("size_bytes")]
    public long SizeBytes { get; set; }
}

/// <summary>
/// HNSW execution trace from POST /collections/{name}/explain.
/// </summary>
public class ExplainTrace
{
    [JsonPropertyName("visited_nodes")]
    public int VisitedNodes { get; set; }

    [JsonPropertyName("ef_search")]
    public int EfSearch { get; set; }

    [JsonPropertyName("hnsw_search_ms")]
    public double HnswSearchMs { get; set; }

    [JsonPropertyName("payload_filter_evals")]
    public int PayloadFilterEvals { get; set; }

    [JsonPropertyName("quantization_score_ms")]
    public double QuantizationScoreMs { get; set; }

    [JsonPropertyName("total_ms")]
    public double TotalMs { get; set; }
}

/// <summary>
/// Response from POST /collections/{name}/explain.
/// </summary>
public class ExplainResponse
{
    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("k")]
    public int K { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<Dictionary<string, object>> Results { get; set; } = Array.Empty<Dictionary<string, object>>();

    [JsonPropertyName("trace")]
    public ExplainTrace Trace { get; set; } = new();
}

/// <summary>
/// One entry in the slow-query ring buffer.
/// </summary>
public class SlowQueryEntry
{
    [JsonPropertyName("timestamp")]
    public string Timestamp { get; set; } = string.Empty;

    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("k")]
    public int K { get; set; }

    [JsonPropertyName("duration_ms")]
    public double DurationMs { get; set; }
}

/// <summary>
/// Slow-query ring-buffer configuration.
/// </summary>
public class SlowQueryConfig
{
    [JsonPropertyName("threshold_ms")]
    public long ThresholdMs { get; set; }

    [JsonPropertyName("capacity")]
    public int Capacity { get; set; }
}
