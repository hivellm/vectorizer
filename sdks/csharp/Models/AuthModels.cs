using System.Text.Json.Serialization;

namespace Vectorizer.Models;

/// <summary>
/// A user record returned by auth endpoints.
/// </summary>
public class User
{
    [JsonPropertyName("user_id")]
    public string UserId { get; set; } = string.Empty;

    [JsonPropertyName("username")]
    public string Username { get; set; } = string.Empty;

    [JsonPropertyName("roles")]
    public IReadOnlyList<string> Roles { get; set; } = Array.Empty<string>();
}

/// <summary>
/// A JWT token returned by POST /auth/refresh.
/// </summary>
public class JwtToken
{
    [JsonPropertyName("access_token")]
    public string AccessToken { get; set; } = string.Empty;

    [JsonPropertyName("token_type")]
    public string TokenType { get; set; } = string.Empty;

    [JsonPropertyName("expires_in")]
    public long ExpiresIn { get; set; }
}

/// <summary>
/// An API key returned by POST /auth/keys or GET /auth/keys.
/// </summary>
public class ApiKey
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("permissions")]
    public IReadOnlyList<string> Permissions { get; set; } = Array.Empty<string>();

    [JsonPropertyName("api_key")]
    public string? ApiKeyValue { get; set; }

    [JsonPropertyName("created_at")]
    public long CreatedAt { get; set; }

    [JsonPropertyName("expires_at")]
    public long? ExpiresAt { get; set; }

    [JsonPropertyName("active")]
    public bool Active { get; set; }

    [JsonPropertyName("warning")]
    public string? Warning { get; set; }

    [JsonPropertyName("usage_count")]
    public long UsageCount { get; set; }
}

/// <summary>
/// Per-collection permission scope on an API key.
/// </summary>
public class ApiKeyScope
{
    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("permissions")]
    public List<string> Permissions { get; set; } = new();
}

/// <summary>
/// Flattened key view returned by PUT /auth/keys/{id}/permissions and
/// GET /auth/keys/{id}/usage.
/// </summary>
public class ApiKeyView
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("user_id")]
    public string UserId { get; set; } = string.Empty;

    [JsonPropertyName("permissions")]
    public IReadOnlyList<string> Permissions { get; set; } = Array.Empty<string>();

    [JsonPropertyName("scopes")]
    public IReadOnlyList<ApiKeyScope> Scopes { get; set; } = Array.Empty<ApiKeyScope>();

    [JsonPropertyName("created_at")]
    public long CreatedAt { get; set; }

    [JsonPropertyName("last_used")]
    public long? LastUsed { get; set; }

    [JsonPropertyName("expires_at")]
    public long? ExpiresAt { get; set; }

    [JsonPropertyName("active")]
    public bool Active { get; set; }

    [JsonPropertyName("usage_count")]
    public long UsageCount { get; set; }
}

/// <summary>
/// One day's usage bucket from GET /auth/keys/{id}/usage.
/// </summary>
public class ApiKeyUsageBucket
{
    [JsonPropertyName("date")]
    public string Date { get; set; } = string.Empty;

    [JsonPropertyName("count")]
    public long Count { get; set; }
}

/// <summary>
/// Response from GET /auth/keys/{id}/usage.
/// </summary>
public class ApiKeyUsageReport
{
    [JsonPropertyName("key")]
    public ApiKeyView Key { get; set; } = new();

    [JsonPropertyName("buckets")]
    public IReadOnlyList<ApiKeyUsageBucket> Buckets { get; set; } = Array.Empty<ApiKeyUsageBucket>();

    [JsonPropertyName("window_total")]
    public long WindowTotal { get; set; }
}

/// <summary>
/// Password policy report returned by POST /auth/validate-password.
/// </summary>
public class PasswordPolicyReport
{
    [JsonPropertyName("valid")]
    public bool Valid { get; set; }

    [JsonPropertyName("errors")]
    public IReadOnlyList<string> Errors { get; set; } = Array.Empty<string>();

    [JsonPropertyName("strength")]
    public int Strength { get; set; }

    [JsonPropertyName("strength_label")]
    public string StrengthLabel { get; set; } = string.Empty;
}

/// <summary>
/// Request body for POST /auth/keys.
/// </summary>
public class CreateApiKeyRequest
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("permissions")]
    public List<string>? Permissions { get; set; }

    [JsonPropertyName("expires_in")]
    public long? ExpiresIn { get; set; }
}

/// <summary>
/// Per-collection permission scope used in scoped key requests.
/// </summary>
public class TokenScope
{
    [JsonPropertyName("collection")]
    public string Collection { get; set; } = string.Empty;

    [JsonPropertyName("permissions")]
    public List<string> Permissions { get; set; } = new();
}

/// <summary>
/// Request body for POST /auth/keys with per-collection scopes.
/// </summary>
public class CreateScopedApiKeyRequest
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("permissions")]
    public List<string>? Permissions { get; set; }

    [JsonPropertyName("expires_in")]
    public long? ExpiresIn { get; set; }

    [JsonPropertyName("scopes")]
    public List<TokenScope>? Scopes { get; set; }
}

/// <summary>
/// Request body for PUT /auth/keys/{id}/permissions.
/// </summary>
public class UpdateApiKeyPermissionsRequest
{
    [JsonPropertyName("permissions")]
    public List<string> Permissions { get; set; } = new();

    [JsonPropertyName("scopes")]
    public List<TokenScope>? Scopes { get; set; }
}

/// <summary>
/// Request body for POST /auth/users.
/// </summary>
public class CreateUserRequest
{
    [JsonPropertyName("username")]
    public string Username { get; set; } = string.Empty;

    [JsonPropertyName("password")]
    public string Password { get; set; } = string.Empty;

    [JsonPropertyName("roles")]
    public List<string>? Roles { get; set; }
}

/// <summary>
/// Result of POST /auth/keys/{id}/rotate.
/// </summary>
public class RotatedKey
{
    [JsonPropertyName("old_key_id")]
    public string OldKeyId { get; set; } = string.Empty;

    [JsonPropertyName("new_key_id")]
    public string NewKeyId { get; set; } = string.Empty;

    [JsonPropertyName("new_token")]
    public string NewToken { get; set; } = string.Empty;

    [JsonPropertyName("grace_until")]
    public long GraceUntil { get; set; }
}

/// <summary>
/// RFC 7662 introspection response from POST /auth/introspect.
/// </summary>
public class TokenIntrospection
{
    [JsonPropertyName("active")]
    public bool Active { get; set; }

    [JsonPropertyName("scope")]
    public string? Scope { get; set; }

    [JsonPropertyName("sub")]
    public string? Sub { get; set; }

    [JsonPropertyName("exp")]
    public long? Exp { get; set; }

    [JsonPropertyName("username")]
    public string? Username { get; set; }
}

/// <summary>
/// One entry in the admin audit log.
/// </summary>
public class AuditEntry
{
    [JsonPropertyName("actor")]
    public string Actor { get; set; } = string.Empty;

    [JsonPropertyName("action")]
    public string Action { get; set; } = string.Empty;

    [JsonPropertyName("target")]
    public string Target { get; set; } = string.Empty;

    [JsonPropertyName("at")]
    public string At { get; set; } = string.Empty;

    [JsonPropertyName("correlation_id")]
    public string? CorrelationId { get; set; }
}
