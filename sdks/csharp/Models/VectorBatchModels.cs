using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Paginated vector listing from GET /collections/{name}/vectors.
/// </summary>
public class VectorPage
{
    [JsonPropertyName("total")]
    public int Total { get; set; }

    [JsonPropertyName("vectors")]
    public IReadOnlyList<Dictionary<string, object>> Vectors { get; set; } = Array.Empty<Dictionary<string, object>>();

    [JsonPropertyName("limit")]
    public int Limit { get; set; }

    [JsonPropertyName("offset")]
    public int Offset { get; set; }
}

/// <summary>
/// One item in a batch_insert_texts call.
/// </summary>
public class BatchInsertItem
{
    [JsonPropertyName("id")]
    public string? Id { get; set; }

    [JsonPropertyName("text")]
    public string Text { get; set; } = string.Empty;

    [JsonPropertyName("metadata")]
    public Dictionary<string, object>? Metadata { get; set; }
}

/// <summary>
/// Result of POST /batch_insert or POST /insert_vectors.
/// </summary>
public class BatchInsertReport
{
    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("count")]
    public int Total { get; set; }

    [JsonPropertyName("inserted")]
    public int Inserted { get; set; }

    [JsonPropertyName("failed")]
    public int Failed { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<Dictionary<string, object>>? Results { get; set; }
}

/// <summary>
/// Result of POST /batch_update.
/// </summary>
public class BatchUpdateReport
{
    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("count")]
    public int Total { get; set; }

    [JsonPropertyName("updated")]
    public int Updated { get; set; }

    [JsonPropertyName("failed")]
    public int Failed { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<Dictionary<string, object>>? Results { get; set; }
}

/// <summary>
/// Result of POST /batch_delete.
/// </summary>
public class DeleteReport
{
    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("count")]
    public int Count { get; set; }

    [JsonPropertyName("deleted")]
    public int Deleted { get; set; }

    [JsonPropertyName("failed")]
    public int Failed { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<VectorOpResult> Results { get; set; } = Array.Empty<VectorOpResult>();
}

/// <summary>
/// Result of POST /collections/{src}/vectors/move.
/// </summary>
public class MoveReport
{
    [JsonPropertyName("src")]
    public string Src { get; set; } = string.Empty;

    [JsonPropertyName("dst")]
    public string Dst { get; set; } = string.Empty;

    [JsonPropertyName("requested")]
    public int Requested { get; set; }

    [JsonPropertyName("moved")]
    public int Moved { get; set; }

    [JsonPropertyName("failed")]
    public int Failed { get; set; }

    [JsonPropertyName("results")]
    public IReadOnlyList<VectorOpResult> Results { get; set; } = Array.Empty<VectorOpResult>();
}
