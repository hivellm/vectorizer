namespace Vectorizer.Models;

/// <summary>
/// Discovery request
/// </summary>
public class DiscoverRequest
{
    public string Query { get; set; } = string.Empty;
    public List<string>? IncludeCollections { get; set; }
    public List<string>? ExcludeCollections { get; set; }
    public int? MaxBullets { get; set; }
    public int? BroadK { get; set; }
    public int? FocusK { get; set; }
}

/// <summary>
/// Filter collections request
/// </summary>
public class FilterCollectionsRequest
{
    public string Query { get; set; } = string.Empty;
    public List<string>? Include { get; set; }
    public List<string>? Exclude { get; set; }
}

/// <summary>
/// Score collections request
/// </summary>
public class ScoreCollectionsRequest
{
    public string Query { get; set; } = string.Empty;
    public float? NameMatchWeight { get; set; }
    public float? TermBoostWeight { get; set; }
    public float? SignalBoostWeight { get; set; }
}

/// <summary>
/// Expand queries request
/// </summary>
public class ExpandQueriesRequest
{
    public string Query { get; set; } = string.Empty;
    public int? MaxExpansions { get; set; }
    public bool? IncludeDefinition { get; set; }
    public bool? IncludeFeatures { get; set; }
    public bool? IncludeArchitecture { get; set; }
}

