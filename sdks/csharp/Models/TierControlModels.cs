using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Per-vector outcome of a batch operation.
/// </summary>
public class VectorOpResult
{
    [JsonPropertyName("id")]
    public string? Id { get; set; }

    [JsonPropertyName("status")]
    public string Status { get; set; } = string.Empty;

    [JsonPropertyName("error")]
    public string? Error { get; set; }

    [JsonPropertyName("index")]
    public int? Index { get; set; }
}

/// <summary>
/// Result of POST /collections/{name}/vectors/delete_by_filter.
/// </summary>
public class DeleteByFilterReport
{
    [JsonPropertyName("scanned")]
    public int Scanned { get; set; }

    [JsonPropertyName("matched")]
    public int Matched { get; set; }

    [JsonPropertyName("deleted")]
    public int Deleted { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<VectorOpResult> Results { get; set; } = Array.Empty<VectorOpResult>();
}

/// <summary>
/// Result of POST /collections/{name}/vectors/bulk_update_metadata.
/// </summary>
public class BulkUpdateReport
{
    [JsonPropertyName("scanned")]
    public int Scanned { get; set; }

    [JsonPropertyName("matched")]
    public int Matched { get; set; }

    [JsonPropertyName("updated")]
    public int Updated { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<VectorOpResult> Results { get; set; } = Array.Empty<VectorOpResult>();
}

/// <summary>
/// Result of POST /collections/{src}/vectors/copy.
/// </summary>
public class CopyReport
{
    [JsonPropertyName("src")]
    public string Src { get; set; } = string.Empty;

    [JsonPropertyName("dst")]
    public string Dst { get; set; } = string.Empty;

    [JsonPropertyName("requested")]
    public int Requested { get; set; }

    [JsonPropertyName("copied")]
    public int Copied { get; set; }

    [JsonPropertyName("failed")]
    public int Failed { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<VectorOpResult> Results { get; set; } = Array.Empty<VectorOpResult>();
}

/// <summary>
/// Job descriptor returned by POST /collections/{name}/reencode.
/// </summary>
public class ReencodeJob
{
    [JsonPropertyName("job_id")]
    public string JobId { get; set; } = string.Empty;

    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("state")]
    public string State { get; set; } = string.Empty;

    [JsonPropertyName("target_encoding")]
    public string TargetEncoding { get; set; } = string.Empty;

    [JsonPropertyName("progress")]
    public double Progress { get; set; }
}

/// <summary>
/// Per-collection TTL configuration.
/// </summary>
public class TtlConfig
{
    [JsonPropertyName("ttl_secs")]
    public long? TtlSecs { get; set; }
}

/// <summary>
/// Request body for PATCH /collections/{name}/vectors/{id}/expiry.
/// </summary>
public class VectorExpiryRequest
{
    [JsonPropertyName("expires_at")]
    public long? ExpiresAt { get; set; }
}
