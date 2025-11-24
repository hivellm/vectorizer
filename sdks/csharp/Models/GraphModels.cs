namespace Vectorizer.Models;

/// <summary>
/// Graph node representing a document/file
/// </summary>
public class GraphNode
{
    public string Id { get; set; } = string.Empty;
    public string NodeType { get; set; } = string.Empty;
    public Dictionary<string, object> Metadata { get; set; } = new();
}

/// <summary>
/// Graph edge representing a relationship between nodes
/// </summary>
public class GraphEdge
{
    public string Id { get; set; } = string.Empty;
    public string Source { get; set; } = string.Empty;
    public string Target { get; set; } = string.Empty;
    public string RelationshipType { get; set; } = string.Empty;
    public float Weight { get; set; }
    public Dictionary<string, object> Metadata { get; set; } = new();
    public string CreatedAt { get; set; } = string.Empty;
}

/// <summary>
/// Neighbor information
/// </summary>
public class NeighborInfo
{
    public GraphNode Node { get; set; } = new();
    public GraphEdge Edge { get; set; } = new();
}

/// <summary>
/// Related node information
/// </summary>
public class RelatedNodeInfo
{
    public GraphNode Node { get; set; } = new();
    public int Distance { get; set; }
    public float Weight { get; set; }
}

/// <summary>
/// Request to find related nodes
/// </summary>
public class FindRelatedRequest
{
    public int? MaxHops { get; set; }
    public string? RelationshipType { get; set; }

    /// <summary>
    /// Validates the request
    /// </summary>
    public void Validate()
    {
        if (MaxHops.HasValue && (MaxHops.Value < 1))
        {
            throw new ArgumentException("MaxHops must be a positive integer", nameof(MaxHops));
        }

        if (!string.IsNullOrWhiteSpace(RelationshipType) && string.IsNullOrEmpty(RelationshipType.Trim()))
        {
            throw new ArgumentException("RelationshipType must be a non-empty string", nameof(RelationshipType));
        }
    }
}

/// <summary>
/// Response for finding related nodes
/// </summary>
public class FindRelatedResponse
{
    public List<RelatedNodeInfo> Related { get; set; } = new();
}

/// <summary>
/// Request to find path between nodes
/// </summary>
public class FindPathRequest
{
    public string Collection { get; set; } = string.Empty;
    public string Source { get; set; } = string.Empty;
    public string Target { get; set; } = string.Empty;

    /// <summary>
    /// Validates the request
    /// </summary>
    public void Validate()
    {
        if (string.IsNullOrWhiteSpace(Collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(Collection));
        }

        if (string.IsNullOrWhiteSpace(Source))
        {
            throw new ArgumentException("Source must be a non-empty string", nameof(Source));
        }

        if (string.IsNullOrWhiteSpace(Target))
        {
            throw new ArgumentException("Target must be a non-empty string", nameof(Target));
        }
    }
}

/// <summary>
/// Response for finding path
/// </summary>
public class FindPathResponse
{
    public List<GraphNode> Path { get; set; } = new();
    public bool Found { get; set; }
}

/// <summary>
/// Request to create an edge
/// </summary>
public class CreateEdgeRequest
{
    public string Collection { get; set; } = string.Empty;
    public string Source { get; set; } = string.Empty;
    public string Target { get; set; } = string.Empty;
    public string RelationshipType { get; set; } = string.Empty;
    public float? Weight { get; set; }

    /// <summary>
    /// Validates the request
    /// </summary>
    public void Validate()
    {
        if (string.IsNullOrWhiteSpace(Collection))
        {
            throw new ArgumentException("Collection must be a non-empty string", nameof(Collection));
        }

        if (string.IsNullOrWhiteSpace(Source))
        {
            throw new ArgumentException("Source must be a non-empty string", nameof(Source));
        }

        if (string.IsNullOrWhiteSpace(Target))
        {
            throw new ArgumentException("Target must be a non-empty string", nameof(Target));
        }

        if (string.IsNullOrWhiteSpace(RelationshipType))
        {
            throw new ArgumentException("RelationshipType must be a non-empty string", nameof(RelationshipType));
        }

        if (Weight.HasValue && (Weight.Value < 0.0f || Weight.Value > 1.0f))
        {
            throw new ArgumentException("Weight must be between 0.0 and 1.0", nameof(Weight));
        }
    }
}

/// <summary>
/// Response for creating an edge
/// </summary>
public class CreateEdgeResponse
{
    public string EdgeId { get; set; } = string.Empty;
    public bool Success { get; set; }
    public string Message { get; set; } = string.Empty;
}

/// <summary>
/// Response for listing nodes
/// </summary>
public class ListNodesResponse
{
    public List<GraphNode> Nodes { get; set; } = new();
    public int Count { get; set; }
}

/// <summary>
/// Response for getting neighbors
/// </summary>
public class GetNeighborsResponse
{
    public List<NeighborInfo> Neighbors { get; set; } = new();
}

/// <summary>
/// Response for listing edges
/// </summary>
public class ListEdgesResponse
{
    public List<GraphEdge> Edges { get; set; } = new();
    public int Count { get; set; }
}

/// <summary>
/// Request to discover edges
/// </summary>
public class DiscoverEdgesRequest
{
    public float? SimilarityThreshold { get; set; }
    public int? MaxPerNode { get; set; }

    /// <summary>
    /// Validates the request
    /// </summary>
    public void Validate()
    {
        if (SimilarityThreshold.HasValue && (SimilarityThreshold.Value < 0.0f || SimilarityThreshold.Value > 1.0f))
        {
            throw new ArgumentException("SimilarityThreshold must be between 0.0 and 1.0", nameof(SimilarityThreshold));
        }

        if (MaxPerNode.HasValue && MaxPerNode.Value < 1)
        {
            throw new ArgumentException("MaxPerNode must be a positive integer", nameof(MaxPerNode));
        }
    }
}

/// <summary>
/// Response for discovering edges
/// </summary>
public class DiscoverEdgesResponse
{
    public bool Success { get; set; }
    public int EdgesCreated { get; set; }
    public string Message { get; set; } = string.Empty;
}

/// <summary>
/// Response for discovery status
/// </summary>
public class DiscoveryStatusResponse
{
    public int TotalNodes { get; set; }
    public int NodesWithEdges { get; set; }
    public int TotalEdges { get; set; }
    public double ProgressPercentage { get; set; }
}

