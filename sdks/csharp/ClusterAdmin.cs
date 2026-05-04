using System.Text.Json;
using System.Text.Json.Serialization;
using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    // Envelope for GET /auth/audit
    private sealed record AuditLogEnvelope(
        [property: JsonPropertyName("entries")] List<AuditEntry>? Entries);

    // Intermediate decode target used to detect the idle sentinel.
    private sealed record RebalanceStatusRaw(
        [property: JsonPropertyName("status")] string Status);

    /// <summary>
    /// Promotes a replica to primary (POST /cluster/failover).
    /// The server returns HTTP 409 when the replica's WAL lag exceeds the
    /// configured threshold.
    /// </summary>
    /// <param name="replicaId">The ID of the replica to promote.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="FailoverReport"/> describing the promotion result.</returns>
    public async Task<FailoverReport> ClusterFailoverAsync(
        string replicaId,
        CancellationToken cancellationToken = default)
    {
        var body = new { replica_id = replicaId };
        return await RequestAsync<FailoverReport>("POST", "/cluster/failover", body, cancellationToken);
    }

    /// <summary>
    /// Forces a full resync on the given replica
    /// (POST /cluster/replicas/{id}/resync).
    /// </summary>
    /// <param name="replicaId">The ID of the replica to resync.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="ResyncJob"/> describing the initiated resync.</returns>
    public async Task<ResyncJob> ClusterResyncReplicaAsync(
        string replicaId,
        CancellationToken cancellationToken = default)
    {
        var path = $"/cluster/replicas/{Uri.EscapeDataString(replicaId)}/resync";
        return await RequestAsync<ResyncJob>("POST", path, new { }, cancellationToken);
    }

    /// <summary>
    /// Registers a new peer in the cluster (POST /cluster/peers).
    /// </summary>
    /// <param name="request">Peer address and optional role.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="PeerInfo"/> describing the newly registered peer.</returns>
    public async Task<PeerInfo> ClusterAddPeerAsync(
        AddPeerRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<PeerInfo>("POST", "/cluster/peers", request, cancellationToken);
    }

    /// <summary>
    /// Triggers a shard rebalance across all active cluster nodes
    /// (POST /cluster/rebalance).
    /// The server returns HTTP 400 when fewer than two active nodes are present
    /// or when a rebalance is already in progress.
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="RebalanceJob"/> describing the initiated rebalance.</returns>
    public async Task<RebalanceJob> ClusterRebalanceAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<RebalanceJob>("POST", "/cluster/rebalance", new { }, cancellationToken);
    }

    /// <summary>
    /// Returns progress of the active or last completed rebalance job
    /// (GET /cluster/rebalance/status).
    /// Returns <see langword="null"/> when no rebalance has been triggered on
    /// this node (server returns <c>{"status":"idle"}</c>).
    /// </summary>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>
    /// A <see cref="RebalanceJob"/>, or <see langword="null"/> when idle.
    /// </returns>
    public async Task<RebalanceJob?> ClusterRebalanceStatusAsync(
        CancellationToken cancellationToken = default)
    {
        // Capture the raw JSON so we can probe the idle sentinel and then
        // decode into the typed struct without an extra HTTP round-trip.
        var raw = await RequestAsync<JsonElement>(
            "GET", "/cluster/rebalance/status", null, cancellationToken);

        if (raw.TryGetProperty("status", out var statusEl) &&
            statusEl.GetString() == "idle")
        {
            return null;
        }

        return JsonSerializer.Deserialize<RebalanceJob>(raw.GetRawText(), _jsonOptions);
    }

    /// <summary>
    /// Atomically rotates an API key (POST /auth/keys/{id}/rotate).
    /// Returns both the old and new tokens plus a grace window during which
    /// the old token remains valid. Requires admin role.
    /// </summary>
    /// <param name="id">The API key ID to rotate.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="RotatedKey"/> containing the new token and grace period.</returns>
    public async Task<RotatedKey> RotateApiKeyAsync(
        string id,
        CancellationToken cancellationToken = default)
    {
        var path = $"/auth/keys/{Uri.EscapeDataString(id)}/rotate";
        return await RequestAsync<RotatedKey>("POST", path, new { }, cancellationToken);
    }

    /// <summary>
    /// Creates an API key with optional per-collection scopes (POST /auth/keys).
    /// When <see cref="CreateScopedApiKeyRequest.Scopes"/> is non-empty the key
    /// is restricted to the listed collections.
    /// The <see cref="ApiKey.ApiKeyValue"/> field is only present at creation time
    /// — store it securely.
    /// </summary>
    /// <param name="request">Key name, permissions, expiry, and optional scopes.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The newly created <see cref="ApiKey"/>.</returns>
    public async Task<ApiKey> CreateScopedApiKeyAsync(
        CreateScopedApiKeyRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ApiKey>("POST", "/auth/keys", request, cancellationToken);
    }

    /// <summary>
    /// Introspects a token per RFC 7662 (POST /auth/introspect).
    /// Returns <c>active: false</c> for any unrecognised token.
    /// Does not require admin role.
    /// </summary>
    /// <param name="token">The raw token string to introspect.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A <see cref="TokenIntrospection"/> describing the token's state.</returns>
    public async Task<TokenIntrospection> IntrospectTokenAsync(
        string token,
        CancellationToken cancellationToken = default)
    {
        var body = new { token };
        return await RequestAsync<TokenIntrospection>("POST", "/auth/introspect", body, cancellationToken);
    }

    /// <summary>
    /// Queries the admin audit log (GET /auth/audit).
    /// Empty string fields and a null/zero <see cref="AuditQuery.Limit"/> are
    /// omitted from the query string.  Returns entries newest-first, bounded by
    /// <see cref="AuditQuery.Limit"/> (server default 200).
    /// Requires admin role.
    /// </summary>
    /// <param name="query">Optional filter parameters.</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>A read-only list of <see cref="AuditEntry"/> records.</returns>
    public async Task<IReadOnlyList<AuditEntry>> ListAuditLogAsync(
        AuditQuery query,
        CancellationToken cancellationToken = default)
    {
        var qs = BuildAuditQueryString(query);
        var path = string.IsNullOrEmpty(qs) ? "/auth/audit" : $"/auth/audit?{qs}";
        var envelope = await RequestAsync<AuditLogEnvelope>("GET", path, null, cancellationToken);
        return envelope.Entries ?? new List<AuditEntry>();
    }

    /// <summary>
    /// Replaces the permission set (and optionally the scopes) of an API key
    /// without rotating its credential (PUT /auth/keys/{id}/permissions).
    /// Requires admin role. The <c>key_hash</c>, <c>id</c>, <c>user_id</c>
    /// and <c>created_at</c> fields stay immutable. The server rejects an
    /// empty permissions list with HTTP 400 and an unknown id with HTTP 404.
    /// </summary>
    /// <param name="id">The API key ID to update.</param>
    /// <param name="request">New permissions (and optional scopes).</param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>The updated <see cref="ApiKeyView"/>.</returns>
    public async Task<ApiKeyView> UpdateApiKeyPermissionsAsync(
        string id,
        UpdateApiKeyPermissionsRequest request,
        CancellationToken cancellationToken = default)
    {
        var path = $"/auth/keys/{Uri.EscapeDataString(id)}/permissions";
        return await RequestAsync<ApiKeyView>("PUT", path, request, cancellationToken);
    }

    /// <summary>
    /// Returns the per-day usage time-series for an API key
    /// (GET /auth/keys/{id}/usage[?window=N]). Requires admin role.
    /// </summary>
    /// <param name="id">The API key ID to sample.</param>
    /// <param name="windowDays">
    /// Optional window in days (server clamps to 1..=30, default 7 when null).
    /// </param>
    /// <param name="cancellationToken">Cancellation token.</param>
    /// <returns>
    /// An <see cref="ApiKeyUsageReport"/> with the live key view, the bucket
    /// array (oldest first, including zero-count days), and the window total.
    /// </returns>
    public async Task<ApiKeyUsageReport> GetApiKeyUsageAsync(
        string id,
        int? windowDays = null,
        CancellationToken cancellationToken = default)
    {
        var path = windowDays.HasValue
            ? $"/auth/keys/{Uri.EscapeDataString(id)}/usage?window={windowDays.Value}"
            : $"/auth/keys/{Uri.EscapeDataString(id)}/usage";
        return await RequestAsync<ApiKeyUsageReport>("GET", path, null, cancellationToken);
    }

    private static string BuildAuditQueryString(AuditQuery query)
    {
        var parts = new List<string>(5);
        if (!string.IsNullOrEmpty(query.Actor))
            parts.Add($"actor={Uri.EscapeDataString(query.Actor)}");
        if (!string.IsNullOrEmpty(query.Action))
            parts.Add($"action={Uri.EscapeDataString(query.Action)}");
        if (!string.IsNullOrEmpty(query.Since))
            parts.Add($"since={Uri.EscapeDataString(query.Since)}");
        if (!string.IsNullOrEmpty(query.Until))
            parts.Add($"until={Uri.EscapeDataString(query.Until)}");
        if (query.Limit.HasValue && query.Limit.Value > 0)
            parts.Add($"limit={query.Limit.Value}");
        return string.Join("&", parts);
    }
}
