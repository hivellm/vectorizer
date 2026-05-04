using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// Top-level Qdrant-style filter accepted by <c>DeleteByFilterAsync</c> and
/// <c>BulkUpdateMetadataAsync</c>.  At least one clause with at least one
/// condition must be present — use <see cref="IsEmpty"/> to guard before
/// sending.
/// </summary>
public sealed class QdrantFilter
{
    /// <summary>All conditions must be true (AND semantics).</summary>
    [JsonPropertyName("must")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public IReadOnlyList<FilterCondition>? Must { get; init; }

    /// <summary>At least one condition must be true (OR semantics).</summary>
    [JsonPropertyName("should")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public IReadOnlyList<FilterCondition>? Should { get; init; }

    /// <summary>All conditions must be false (NOT semantics).</summary>
    [JsonPropertyName("must_not")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public IReadOnlyList<FilterCondition>? MustNot { get; init; }

    /// <summary>
    /// Returns <c>true</c> when all three clauses are absent or empty —
    /// i.e. the filter would be rejected by the server as having no conditions.
    /// </summary>
    public bool IsEmpty() =>
        (Must?.Count ?? 0) == 0 &&
        (Should?.Count ?? 0) == 0 &&
        (MustNot?.Count ?? 0) == 0;
}

/// <summary>A single filter condition applied to a payload field.</summary>
public sealed class FilterCondition
{
    /// <summary>Payload field path (dot-separated for nested fields).</summary>
    [JsonPropertyName("key")]
    public string Key { get; init; } = "";

    /// <summary>Exact-value or any-of-values match.</summary>
    [JsonPropertyName("match")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public FilterMatch? Match { get; init; }

    /// <summary>Numeric range bounds.</summary>
    [JsonPropertyName("range")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public FilterRange? Range { get; init; }

    /// <summary>Nested sub-filter applied to a nested object field.</summary>
    [JsonPropertyName("filter")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public QdrantFilter? Filter { get; init; }
}

/// <summary>
/// Match discriminator: either a single <see cref="Value"/> or a list
/// of <see cref="Any"/> values (in-set check).
/// </summary>
public sealed class FilterMatch
{
    /// <summary>Exact value to match (string, number, or bool).</summary>
    [JsonPropertyName("value")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public object? Value { get; init; }

    /// <summary>Set of values — the field must equal at least one entry.</summary>
    [JsonPropertyName("any")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public IReadOnlyList<object>? Any { get; init; }
}

/// <summary>Numeric range bounds for a payload field (both bounds optional).</summary>
public sealed class FilterRange
{
    /// <summary>Greater-than-or-equal lower bound.</summary>
    [JsonPropertyName("gte")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public double? Gte { get; init; }

    /// <summary>Less-than-or-equal upper bound.</summary>
    [JsonPropertyName("lte")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public double? Lte { get; init; }
}

/// <summary>Static builder helpers for typed Vectorizer filters.</summary>
public static class Filter
{
    /// <summary>
    /// Creates an exact-value match condition:
    /// <c>{"key":"&lt;key&gt;","match":{"value":&lt;value&gt;}}</c>.
    /// </summary>
    public static FilterCondition Eq(string key, object value) =>
        new() { Key = key, Match = new FilterMatch { Value = value } };

    /// <summary>
    /// Creates an in-set match condition:
    /// <c>{"key":"&lt;key&gt;","match":{"any":[...]}}</c>.
    /// </summary>
    public static FilterCondition In(string key, IEnumerable<object> values) =>
        new() { Key = key, Match = new FilterMatch { Any = values.ToList() } };

    /// <summary>
    /// Creates a numeric range condition. At least one bound must be supplied.
    /// </summary>
    public static FilterCondition Range(string key, double? gte = null, double? lte = null) =>
        new() { Key = key, Range = new FilterRange { Gte = gte, Lte = lte } };

    /// <summary>
    /// Creates a nested sub-filter condition that applies <paramref name="filter"/>
    /// to a nested object field.
    /// </summary>
    public static FilterCondition Nested(QdrantFilter filter) =>
        new() { Key = "__nested__", Filter = filter };

    /// <summary>
    /// Builds a filter requiring ALL supplied conditions (AND semantics).
    /// </summary>
    public static QdrantFilter Must(params FilterCondition[] conditions) =>
        new() { Must = conditions };

    /// <summary>
    /// Builds a filter requiring AT LEAST ONE of the supplied conditions (OR semantics).
    /// </summary>
    public static QdrantFilter Should(params FilterCondition[] conditions) =>
        new() { Should = conditions };

    /// <summary>
    /// Builds a filter requiring ALL conditions to be false (NOT semantics).
    /// </summary>
    public static QdrantFilter MustNot(params FilterCondition[] conditions) =>
        new() { MustNot = conditions };
}
