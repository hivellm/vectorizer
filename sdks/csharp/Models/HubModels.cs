using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// A user-scoped backup entry returned by GET /hub/backups.
/// </summary>
public class UserBackup
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("user_id")]
    public string UserId { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("description")]
    public string? Description { get; set; }

    [JsonPropertyName("collections")]
    public IReadOnlyList<string> Collections { get; set; } = Array.Empty<string>();

    [JsonPropertyName("created_at")]
    public string CreatedAt { get; set; } = string.Empty;

    [JsonPropertyName("size")]
    public long Size { get; set; }

    [JsonPropertyName("status")]
    public string Status { get; set; } = string.Empty;
}

/// <summary>
/// Request body for POST /hub/backups.
/// </summary>
public class CreateUserBackupRequest
{
    [JsonPropertyName("user_id")]
    public string UserId { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("description")]
    public string? Description { get; set; }

    [JsonPropertyName("collections")]
    public List<string>? Collections { get; set; }
}

/// <summary>
/// Request body for POST /hub/backups/restore.
/// </summary>
public class RestoreUserBackupRequest
{
    [JsonPropertyName("user_id")]
    public string UserId { get; set; } = string.Empty;

    [JsonPropertyName("backup_id")]
    public string BackupId { get; set; } = string.Empty;

    [JsonPropertyName("overwrite")]
    public bool? Overwrite { get; set; }
}

/// <summary>
/// Usage statistics from GET /hub/usage/statistics.
/// </summary>
public class UsageStatistics
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;

    [JsonPropertyName("stats")]
    public Dictionary<string, object>? Stats { get; set; }
}

/// <summary>
/// Quota information from GET /hub/usage/quota.
/// </summary>
public class QuotaInfo
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;

    [JsonPropertyName("quota")]
    public Dictionary<string, object>? Quota { get; set; }
}

/// <summary>
/// Result of POST /hub/validate-key.
/// </summary>
public class HubApiKeyValidation
{
    [JsonPropertyName("valid")]
    public bool Valid { get; set; }

    [JsonPropertyName("tenant_id")]
    public string TenantId { get; set; } = string.Empty;

    [JsonPropertyName("tenant_name")]
    public string TenantName { get; set; } = string.Empty;

    [JsonPropertyName("permissions")]
    public IReadOnlyList<string> Permissions { get; set; } = Array.Empty<string>();

    [JsonPropertyName("validated_at")]
    public string ValidatedAt { get; set; } = string.Empty;
}
