using System.Text.Json.Serialization;
using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    // Private envelope types for unwrapping server responses.
    private sealed class LogsEnvelope
    {
        [JsonPropertyName("logs")]
        public List<LogEntry>? Logs { get; set; }
    }

    private sealed class BackupsEnvelope
    {
        [JsonPropertyName("backups")]
        public List<BackupInfo>? Backups { get; set; }
    }

    private sealed class WorkspacesEnvelope
    {
        [JsonPropertyName("workspaces")]
        public List<WorkspaceItem>? Workspaces { get; set; }
    }

    private sealed class CollectionsEnvelope
    {
        [JsonPropertyName("collections")]
        public List<string>? Collections { get; set; }
    }

    /// <summary>
    /// Returns aggregate server statistics (GET /stats).
    /// Named GetServerStatsAsync to disambiguate from GetStatsAsync which
    /// returns DatabaseStats.
    /// </summary>
    public async Task<Stats> GetServerStatsAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<Stats>("GET", "/stats", null, cancellationToken);
    }

    /// <summary>
    /// Returns the current liveness state of the server (GET /status).
    /// </summary>
    public async Task<ServerStatus> GetStatusAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ServerStatus>("GET", "/status", null, cancellationToken);
    }

    /// <summary>
    /// Returns the dashboard runtime snapshot (phase25): CPU, memory,
    /// active connections, rolling 60-second QPS, per-route p50/p99,
    /// 5xx error rate, and the WAL state. Requires admin auth on the
    /// server. Older servers without phase25 §4 may return zero-valued
    /// defaults instead of a populated payload.
    ///
    /// Calls GET /metrics/runtime.
    /// </summary>
    public async Task<RuntimeMetrics> GetRuntimeMetricsAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<RuntimeMetrics>("GET", "/metrics/runtime", null, cancellationToken);
    }

    /// <summary>
    /// Returns recent log entries (GET /logs).
    /// </summary>
    /// <param name="lines">Maximum number of lines to return; 0 means server default.</param>
    /// <param name="level">Minimum log level filter (e.g. "error", "warn", "info"). Empty string means all.</param>
    /// <param name="cancellationToken">Optional cancellation token.</param>
    public async Task<IReadOnlyList<LogEntry>> GetLogsAsync(
        int lines = 0,
        string level = "",
        CancellationToken cancellationToken = default)
    {
        var query = new List<string>();
        if (lines > 0)
            query.Add($"lines={lines}");
        if (!string.IsNullOrEmpty(level))
            query.Add($"level={Uri.EscapeDataString(level)}");

        var path = query.Count > 0 ? $"/logs?{string.Join("&", query)}" : "/logs";
        var envelope = await RequestAsync<LogsEnvelope>(path: path, method: "GET", body: null, cancellationToken: cancellationToken);
        return envelope.Logs ?? new List<LogEntry>();
    }

    /// <summary>
    /// Returns indexing progress for all collections (GET /indexing/progress).
    /// </summary>
    public async Task<IndexingProgress> GetIndexingProgressAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<IndexingProgress>("GET", "/indexing/progress", null, cancellationToken);
    }

    /// <summary>
    /// Forces an immediate save of a collection to disk (POST /collections/{name}/force-save).
    /// </summary>
    public async Task ForceSaveCollectionAsync(
        string name,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "POST",
            $"/collections/{Uri.EscapeDataString(name)}/force-save",
            null,
            cancellationToken);
    }

    /// <summary>
    /// Returns the names of collections that contain no vectors (GET /collections/empty).
    /// </summary>
    public async Task<IReadOnlyList<string>> ListEmptyCollectionsAsync(
        CancellationToken cancellationToken = default)
    {
        // The server may return a bare JSON array or a {collections:[...]} envelope.
        var raw = await RequestAsync<System.Text.Json.JsonElement>(
            "GET", "/collections/empty", null, cancellationToken);

        if (raw.ValueKind == System.Text.Json.JsonValueKind.Array)
        {
            var list = new List<string>();
            foreach (var item in raw.EnumerateArray())
                list.Add(item.GetString() ?? string.Empty);
            return list;
        }

        if (raw.TryGetProperty("collections", out var prop))
        {
            var list = new List<string>();
            foreach (var item in prop.EnumerateArray())
                list.Add(item.GetString() ?? string.Empty);
            return list;
        }

        return new List<string>();
    }

    /// <summary>
    /// Deletes all empty collections and returns a report (DELETE /collections/cleanup).
    /// </summary>
    public async Task<CleanupReport> CleanupEmptyCollectionsAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<CleanupReport>("DELETE", "/collections/cleanup", null, cancellationToken);
    }

    /// <summary>
    /// Returns the current server configuration (GET /config).
    /// </summary>
    public async Task<ConfigSnapshot> GetConfigAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ConfigSnapshot>("GET", "/config", null, cancellationToken);
    }

    /// <summary>
    /// Applies a partial configuration patch and returns the updated snapshot (POST /config).
    /// </summary>
    public async Task<ConfigSnapshot> UpdateConfigAsync(
        IDictionary<string, object> patch,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ConfigSnapshot>("POST", "/config", patch, cancellationToken);
    }

    /// <summary>
    /// Returns metadata for all available server-side backups (GET /backups).
    /// </summary>
    public async Task<IReadOnlyList<BackupInfo>> ListBackupsAsync(
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<BackupsEnvelope>("GET", "/backups", null, cancellationToken);
        return envelope.Backups ?? new List<BackupInfo>();
    }

    /// <summary>
    /// Creates a new server-side backup (POST /backups/create).
    /// </summary>
    public async Task<BackupInfo> CreateBackupAsync(
        CreateBackupRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<BackupInfo>("POST", "/backups/create", request, cancellationToken);
    }

    /// <summary>
    /// Restores data from an existing backup (POST /backups/restore).
    /// </summary>
    public async Task RestoreBackupAsync(
        RestoreBackupRequest request,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>("POST", "/backups/restore", request, cancellationToken);
    }

    /// <summary>
    /// Requests a graceful server restart (POST /admin/restart).
    /// </summary>
    public async Task RestartServerAsync(
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>("POST", "/admin/restart", null, cancellationToken);
    }

    /// <summary>
    /// Returns all configured workspaces (GET /workspace/list).
    /// </summary>
    public async Task<IReadOnlyList<WorkspaceItem>> ListWorkspacesAsync(
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<WorkspacesEnvelope>("GET", "/workspace/list", null, cancellationToken);
        return envelope.Workspaces ?? new List<WorkspaceItem>();
    }

    /// <summary>
    /// Returns the active workspace configuration (GET /workspace/config).
    /// </summary>
    public async Task<WorkspaceConfig> GetWorkspaceConfigAsync(
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<WorkspaceConfig>("GET", "/workspace/config", null, cancellationToken);
    }

    /// <summary>
    /// Adds a new workspace (POST /workspace/add).
    /// </summary>
    public async Task AddWorkspaceAsync(
        AddWorkspaceRequest request,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>("POST", "/workspace/add", request, cancellationToken);
    }

    /// <summary>
    /// Removes a workspace by name (POST /workspace/remove).
    /// </summary>
    public async Task RemoveWorkspaceAsync(
        string name,
        CancellationToken cancellationToken = default)
    {
        var body = new Dictionary<string, object> { ["path"] = name };
        await RequestAsync<object>("POST", "/workspace/remove", body, cancellationToken);
    }
}
