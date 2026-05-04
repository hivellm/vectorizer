using System.Text.Json.Serialization;
using Vectorizer.Models;

namespace Vectorizer;

public partial class VectorizerClient
{
    // Envelope for GET /auth/keys
    private sealed record KeysEnvelope([property: JsonPropertyName("keys")] List<ApiKey>? Keys);

    // Envelope for GET /auth/users
    private sealed record UsersEnvelope([property: JsonPropertyName("users")] List<User>? Users);

    /// <summary>
    /// Returns the current authenticated user's claims (GET /auth/me).
    /// </summary>
    public async Task<User> MeAsync(CancellationToken cancellationToken = default)
    {
        return await RequestAsync<User>("GET", "/auth/me", null, cancellationToken);
    }

    /// <summary>
    /// Invalidates the current session token (POST /auth/logout).
    /// </summary>
    public async Task LogoutAsync(CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>("POST", "/auth/logout", null, cancellationToken);
    }

    /// <summary>
    /// Exchanges the current token for a fresh JWT (POST /auth/refresh).
    /// </summary>
    public async Task<JwtToken> RefreshTokenAsync(CancellationToken cancellationToken = default)
    {
        return await RequestAsync<JwtToken>("POST", "/auth/refresh", new { }, cancellationToken);
    }

    /// <summary>
    /// Checks a password against the server's policy without creating an account
    /// (POST /auth/validate-password).
    /// </summary>
    public async Task<PasswordPolicyReport> ValidatePasswordAsync(
        string password,
        CancellationToken cancellationToken = default)
    {
        var body = new { password };
        return await RequestAsync<PasswordPolicyReport>(
            "POST", "/auth/validate-password", body, cancellationToken);
    }

    /// <summary>
    /// Creates a new API key for the calling user (POST /auth/keys).
    /// The <see cref="ApiKey.ApiKeyValue"/> field is only present in the creation response.
    /// </summary>
    public async Task<ApiKey> CreateApiKeyAsync(
        CreateApiKeyRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<ApiKey>("POST", "/auth/keys", request, cancellationToken);
    }

    /// <summary>
    /// Returns the API keys belonging to the calling user (GET /auth/keys).
    /// </summary>
    public async Task<IReadOnlyList<ApiKey>> ListApiKeysAsync(
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<KeysEnvelope>("GET", "/auth/keys", null, cancellationToken);
        return envelope.Keys ?? new List<ApiKey>();
    }

    /// <summary>
    /// Revokes an API key by id (DELETE /auth/keys/{id}).
    /// </summary>
    public async Task RevokeApiKeyAsync(
        string id,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "DELETE", $"/auth/keys/{Uri.EscapeDataString(id)}", null, cancellationToken);
    }

    /// <summary>
    /// Creates a new user (POST /auth/users). Requires admin role.
    /// </summary>
    public async Task<User> CreateUserAsync(
        CreateUserRequest request,
        CancellationToken cancellationToken = default)
    {
        return await RequestAsync<User>("POST", "/auth/users", request, cancellationToken);
    }

    /// <summary>
    /// Returns all users (GET /auth/users). Requires admin role.
    /// </summary>
    public async Task<IReadOnlyList<User>> ListUsersAsync(
        CancellationToken cancellationToken = default)
    {
        var envelope = await RequestAsync<UsersEnvelope>("GET", "/auth/users", null, cancellationToken);
        return envelope.Users ?? new List<User>();
    }

    /// <summary>
    /// Deletes a user by username (DELETE /auth/users/{username}). Requires admin role.
    /// The server refuses to delete self or the last admin.
    /// </summary>
    public async Task DeleteUserAsync(
        string username,
        CancellationToken cancellationToken = default)
    {
        await RequestAsync<object>(
            "DELETE", $"/auth/users/{Uri.EscapeDataString(username)}", null, cancellationToken);
    }

    /// <summary>
    /// Sets a new password for the given user
    /// (PUT /auth/users/{username}/password).
    /// Admins can change any password; non-admins must also supply their current
    /// password at the server level.
    /// </summary>
    public async Task ChangePasswordAsync(
        string username,
        string newPassword,
        CancellationToken cancellationToken = default)
    {
        var body = new { new_password = newPassword };
        await RequestAsync<object>(
            "PUT", $"/auth/users/{Uri.EscapeDataString(username)}/password", body, cancellationToken);
    }
}
