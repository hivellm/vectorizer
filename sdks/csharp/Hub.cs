using System.Text;
using System.Text.Json.Serialization;
using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    // Private envelope types for unwrapping hub backup responses.
    private sealed class ListUserBackupsEnvelope
    {
        [JsonPropertyName("backups")]
        public List<UserBackup>? Backups { get; set; }
    }

    private sealed class SingleUserBackupEnvelope
    {
        [JsonPropertyName("backup")]
        public UserBackup? Backup { get; set; }
    }

    private sealed class UploadBackupBody
    {
        [JsonPropertyName("data")]
        public string Data { get; set; } = string.Empty;
    }

    private sealed class ValidateKeyBody
    {
        [JsonPropertyName("key")]
        public string Key { get; set; } = string.Empty;
    }

    /// <summary>
    /// Lists all backups owned by the given user.
    /// Calls GET /hub/backups?user_id={userId}.
    /// </summary>
    public async Task<IReadOnlyList<UserBackup>> ListUserBackupsAsync(
        string userId,
        CancellationToken cancellationToken = default)
    {
        var qs = $"user_id={Uri.EscapeDataString(userId)}";
        var envelope = await RequestAsync<ListUserBackupsEnvelope>(
            "GET", $"/hub/backups?{qs}", null, cancellationToken);
        return envelope.Backups ?? new List<UserBackup>();
    }

    /// <summary>
    /// Creates a new backup for a user.
    /// Calls POST /hub/backups with the request body containing user_id, name,
    /// optional description, and optional collections slice.
    /// </summary>
    public async Task<UserBackup> CreateUserBackupAsync(
        CreateUserBackupRequest request,
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<SingleUserBackupEnvelope>(
            "POST", "/hub/backups", request, cancellationToken);
        return envelope.Backup ?? throw new InvalidOperationException("Server returned no backup in response.");
    }

    /// <summary>
    /// Restores a previously created backup.
    /// Calls POST /hub/backups/restore with user_id, backup_id, and optional overwrite flag.
    /// </summary>
    public async Task RestoreUserBackupAsync(
        RestoreUserBackupRequest request,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>("POST", "/hub/backups/restore", request, cancellationToken);
    }

    /// <summary>
    /// Uploads raw backup data for a user.
    /// Calls POST /hub/backups/upload?user_id={userId}&amp;name={name}.
    /// The binary data is base64-encoded and sent as {"data":"&lt;base64&gt;"} in the request body.
    /// </summary>
    public async Task<UserBackup> UploadUserBackupAsync(
        string userId,
        string name,
        byte[] data,
        CancellationToken cancellationToken = default)
    {
        var qs = $"user_id={Uri.EscapeDataString(userId)}&name={Uri.EscapeDataString(name)}";
        var body = new UploadBackupBody { Data = Convert.ToBase64String(data) };
        var envelope = await RequestAsync<SingleUserBackupEnvelope>(
            "POST", $"/hub/backups/upload?{qs}", body, cancellationToken);
        return envelope.Backup ?? throw new InvalidOperationException("Server returned no backup in response.");
    }

    /// <summary>
    /// Fetches metadata for a single backup.
    /// Calls GET /hub/backups/{backupId}?user_id={userId}.
    /// </summary>
    public async Task<UserBackup> GetUserBackupAsync(
        string userId,
        string backupId,
        CancellationToken cancellationToken = default)
    {
        var qs = $"user_id={Uri.EscapeDataString(userId)}";
        var path = $"/hub/backups/{Uri.EscapeDataString(backupId)}?{qs}";
        var envelope = await RequestAsync<SingleUserBackupEnvelope>(
            "GET", path, null, cancellationToken);
        return envelope.Backup ?? throw new InvalidOperationException("Server returned no backup in response.");
    }

    /// <summary>
    /// Deletes a backup by ID.
    /// Calls DELETE /hub/backups/{backupId}?user_id={userId}.
    /// </summary>
    public async Task DeleteUserBackupAsync(
        string userId,
        string backupId,
        CancellationToken cancellationToken = default)
    {
        var qs = $"user_id={Uri.EscapeDataString(userId)}";
        var path = $"/hub/backups/{Uri.EscapeDataString(backupId)}?{qs}";
        await RequestAsync<object>("DELETE", path, null, cancellationToken);
    }

    /// <summary>
    /// Downloads the raw binary content of a backup.
    /// Calls GET /hub/backups/{backupId}/download?user_id={userId}.
    /// Bypasses RequestAsync (which JSON-decodes) by sending the request directly
    /// via the shared HttpClient and reading the response as raw bytes.
    /// Auth headers are already set on _httpClient at construction time.
    /// </summary>
    public async Task<byte[]> DownloadUserBackupAsync(
        string userId,
        string backupId,
        CancellationToken cancellationToken = default)
    {
        var qs = $"user_id={Uri.EscapeDataString(userId)}";
        var url = $"{_baseUrl}/hub/backups/{Uri.EscapeDataString(backupId)}/download?{qs}";

        var request = new HttpRequestMessage(HttpMethod.Get, url);
        var response = await _httpClient.SendAsync(request, cancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            var errContent = await response.Content.ReadAsStringAsync(cancellationToken);
            throw new HttpRequestException(
                $"Download failed with status {(int)response.StatusCode}: {errContent}");
        }

        return await response.Content.ReadAsByteArrayAsync(cancellationToken);
    }

    /// <summary>
    /// Returns aggregate usage statistics for a user.
    /// Calls GET /hub/usage/statistics?user_id={userId}.
    /// </summary>
    public async Task<UsageStatistics> GetUsageStatisticsAsync(
        string userId,
        CancellationToken cancellationToken = default)
    {
        var qs = $"user_id={Uri.EscapeDataString(userId)}";
        return await RequestAsync<UsageStatistics>(
            "GET", $"/hub/usage/statistics?{qs}", null, cancellationToken);
    }

    /// <summary>
    /// Returns quota information for a user.
    /// Calls GET /hub/usage/quota?user_id={userId}.
    /// </summary>
    public async Task<QuotaInfo> GetQuotaInfoAsync(
        string userId,
        CancellationToken cancellationToken = default)
    {
        var qs = $"user_id={Uri.EscapeDataString(userId)}";
        return await RequestAsync<QuotaInfo>(
            "GET", $"/hub/usage/quota?{qs}", null, cancellationToken);
    }

    /// <summary>
    /// Validates a HiveHub API key.
    /// Calls POST /hub/validate-key with {"key": key} in the request body.
    /// The key being validated is passed in the body and may differ from the
    /// credential configured on the client itself.
    /// </summary>
    public async Task<HubApiKeyValidation> ValidateHubApiKeyAsync(
        string key,
        CancellationToken cancellationToken = default)
    {
        var body = new ValidateKeyBody { Key = key };
        return await RequestAsync<HubApiKeyValidation>(
            "POST", "/hub/validate-key", body, cancellationToken);
    }
}
