namespace Vectorizer.Models;

/// <summary>
/// Embedding request parameters
/// </summary>
public class EmbeddingParameters
{
    public int? MaxLength { get; set; }
    public bool? Normalize { get; set; }
    public string? Prefix { get; set; }
}

/// <summary>
/// Embedding request
/// </summary>
public class EmbeddingRequest
{
    public string Text { get; set; } = string.Empty;
    public string? Model { get; set; }
    public EmbeddingParameters? Parameters { get; set; }
}

/// <summary>
/// Embedding response
/// </summary>
public class EmbeddingResponse
{
    public float[] Embedding { get; set; } = Array.Empty<float>();
    public string Model { get; set; } = string.Empty;
    public string Text { get; set; } = string.Empty;
    public string? Provider { get; set; }
    public int Dimension { get; set; }
    public EmbeddingParameters? Parameters { get; set; }
}

