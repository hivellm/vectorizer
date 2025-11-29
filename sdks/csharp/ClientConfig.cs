namespace Vectorizer;

/// <summary>
/// Read preference for routing read operations.
/// Similar to MongoDB's read preferences.
/// </summary>
public enum ReadPreference
{
    /// <summary>Route all reads to master</summary>
    Master,
    /// <summary>Route reads to replicas (round-robin)</summary>
    Replica,
    /// <summary>Route to the node with lowest latency</summary>
    Nearest
}

/// <summary>
/// Host configuration for master/replica topology.
/// </summary>
public class HostConfig
{
    /// <summary>
    /// Master node URL (receives all write operations)
    /// </summary>
    public string Master { get; set; } = string.Empty;

    /// <summary>
    /// Replica node URLs (receive read operations based on ReadPreference)
    /// </summary>
    public List<string> Replicas { get; set; } = new();
}

/// <summary>
/// Options that can be passed to read operations for per-operation override.
/// </summary>
public class ReadOptions
{
    /// <summary>
    /// Override the default read preference for this operation
    /// </summary>
    public ReadPreference? ReadPreference { get; set; }
}

/// <summary>
/// Configuration for the Vectorizer client
/// </summary>
public class ClientConfig
{
    /// <summary>
    /// Base URL for the Vectorizer API (default: http://localhost:15002)
    /// For single-node deployments.
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

    /// <summary>
    /// Master/replica host configuration for read/write routing
    /// </summary>
    public HostConfig? Hosts { get; set; }

    /// <summary>
    /// Default read preference for read operations (default: Replica)
    /// </summary>
    public ReadPreference ReadPreference { get; set; } = ReadPreference.Replica;
}

