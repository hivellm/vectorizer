namespace Vectorizer;

/// <summary>
/// Configuration for the Vectorizer client
/// </summary>
public class ClientConfig
{
    /// <summary>
    /// Base URL for the Vectorizer API (default: http://localhost:15002)
    /// </summary>
    public string BaseUrl { get; set; } = "http://localhost:15002";

    /// <summary>
    /// API key for authentication
    /// </summary>
    public string? ApiKey { get; set; }

    /// <summary>
    /// Request timeout in seconds (default: 30)
    /// </summary>
    public int TimeoutSeconds { get; set; } = 30;

    /// <summary>
    /// Custom HTTP client (optional)
    /// </summary>
    public HttpClient? HttpClient { get; set; }
}

