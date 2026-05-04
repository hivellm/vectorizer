using System.Net;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

public class AuthTests
{
    // ---------------------------------------------------------------------------
    // Infrastructure
    // ---------------------------------------------------------------------------

    private sealed class FakeHandler : HttpMessageHandler
    {
        public Func<HttpRequestMessage, HttpResponseMessage> Responder { get; set; }
            = _ => new HttpResponseMessage(HttpStatusCode.NotFound);

        public List<HttpRequestMessage> Requests { get; } = new();

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken ct)
        {
            Requests.Add(request);
            return Task.FromResult(Responder(request));
        }
    }

    private static (VectorizerClient client, FakeHandler handler) CreateClient(
        Func<HttpRequestMessage, HttpResponseMessage> responder)
    {
        var handler = new FakeHandler { Responder = responder };
        var http = new HttpClient(handler);
        return (new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://test.local",
            HttpClient = http
        }), handler);
    }

    private static HttpResponseMessage JsonOk(object body) => new(HttpStatusCode.OK)
    {
        Content = new StringContent(
            JsonSerializer.Serialize(body),
            Encoding.UTF8,
            "application/json")
    };

    private static HttpResponseMessage NoContent() =>
        new(HttpStatusCode.NoContent);

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    [Fact]
    public async Task MeAsync_CallsGetAuthMe_ReturnsUser()
    {
        var expected = new { user_id = "u1", username = "alice", roles = new[] { "admin" } };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var user = await client.MeAsync();

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Get, req.Method);
        Assert.Equal("/auth/me", req.RequestUri!.AbsolutePath);
        Assert.Equal("u1", user.UserId);
        Assert.Equal("alice", user.Username);
        Assert.Contains("admin", user.Roles);
    }

    [Fact]
    public async Task LogoutAsync_CallsPostAuthLogout_Completes()
    {
        var (client, handler) = CreateClient(_ => NoContent());

        await client.LogoutAsync();

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/auth/logout", req.RequestUri!.AbsolutePath);
    }

    [Fact]
    public async Task RefreshTokenAsync_CallsPostAuthRefresh_ReturnsJwtToken()
    {
        var expected = new { access_token = "tok123", token_type = "bearer", expires_in = 3600L };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var token = await client.RefreshTokenAsync();

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/auth/refresh", req.RequestUri!.AbsolutePath);
        Assert.Equal("tok123", token.AccessToken);
        Assert.Equal("bearer", token.TokenType);
        Assert.Equal(3600L, token.ExpiresIn);
    }

    [Fact]
    public async Task ValidatePasswordAsync_CallsPostAuthValidatePassword_ReturnsReport()
    {
        var expected = new
        {
            valid = true,
            errors = Array.Empty<string>(),
            strength = 4,
            strength_label = "strong"
        };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var report = await client.ValidatePasswordAsync("S3cur3Pass!");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/auth/validate-password", req.RequestUri!.AbsolutePath);

        // Verify the body contained the password field
        var bodyJson = await req.Content!.ReadAsStringAsync();
        using var doc = JsonDocument.Parse(bodyJson);
        Assert.Equal("S3cur3Pass!", doc.RootElement.GetProperty("password").GetString());

        Assert.True(report.Valid);
        Assert.Equal(4, report.Strength);
        Assert.Equal("strong", report.StrengthLabel);
    }

    [Fact]
    public async Task CreateApiKeyAsync_CallsPostAuthKeys_ReturnsApiKey()
    {
        var expected = new
        {
            id = "key1",
            name = "mykey",
            permissions = new[] { "read" },
            api_key = "rawtoken",
            created_at = 1000L,
            expires_at = (long?)null,
            active = true,
            warning = (string?)null,
            usage_count = 0L
        };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var createReq = new CreateApiKeyRequest { Name = "mykey", Permissions = new List<string> { "read" } };
        var key = await client.CreateApiKeyAsync(createReq);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/auth/keys", req.RequestUri!.AbsolutePath);
        Assert.Equal("key1", key.Id);
        Assert.Equal("mykey", key.Name);
        Assert.Equal("rawtoken", key.ApiKeyValue);
        Assert.True(key.Active);
    }

    [Fact]
    public async Task ListApiKeysAsync_CallsGetAuthKeys_ReturnsKeyList()
    {
        var expected = new
        {
            keys = new[]
            {
                new { id = "k1", name = "first", permissions = new[] { "read" }, api_key = (string?)null,
                      created_at = 1000L, expires_at = (long?)null, active = true, warning = (string?)null, usage_count = 0L },
                new { id = "k2", name = "second", permissions = new[] { "write" }, api_key = (string?)null,
                      created_at = 2000L, expires_at = (long?)null, active = true, warning = (string?)null, usage_count = 5L }
            }
        };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var keys = await client.ListApiKeysAsync();

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Get, req.Method);
        Assert.Equal("/auth/keys", req.RequestUri!.AbsolutePath);
        Assert.Equal(2, keys.Count);
        Assert.Equal("k1", keys[0].Id);
        Assert.Equal("k2", keys[1].Id);
    }

    [Fact]
    public async Task RevokeApiKeyAsync_CallsDeleteAuthKeysId_Completes()
    {
        var (client, handler) = CreateClient(_ => NoContent());

        await client.RevokeApiKeyAsync("key-abc");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Delete, req.Method);
        Assert.Equal("/auth/keys/key-abc", req.RequestUri!.AbsolutePath);
    }

    [Fact]
    public async Task CreateUserAsync_CallsPostAuthUsers_ReturnsUser()
    {
        var expected = new { user_id = "u2", username = "bob", roles = new[] { "user" } };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var createReq = new CreateUserRequest { Username = "bob", Password = "pass123" };
        var user = await client.CreateUserAsync(createReq);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/auth/users", req.RequestUri!.AbsolutePath);
        Assert.Equal("u2", user.UserId);
        Assert.Equal("bob", user.Username);
    }

    [Fact]
    public async Task ListUsersAsync_CallsGetAuthUsers_ReturnsUserList()
    {
        var expected = new
        {
            users = new[]
            {
                new { user_id = "u1", username = "alice", roles = new[] { "admin" } },
                new { user_id = "u2", username = "bob", roles = new[] { "user" } }
            }
        };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var users = await client.ListUsersAsync();

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Get, req.Method);
        Assert.Equal("/auth/users", req.RequestUri!.AbsolutePath);
        Assert.Equal(2, users.Count);
        Assert.Equal("alice", users[0].Username);
        Assert.Equal("bob", users[1].Username);
    }

    [Fact]
    public async Task DeleteUserAsync_CallsDeleteAuthUsersUsername_Completes()
    {
        var (client, handler) = CreateClient(_ => NoContent());

        await client.DeleteUserAsync("bob");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Delete, req.Method);
        Assert.Equal("/auth/users/bob", req.RequestUri!.AbsolutePath);
    }

    [Fact]
    public async Task ChangePasswordAsync_CallsPutAuthUsersUsernamePassword_Completes()
    {
        var (client, handler) = CreateClient(_ => NoContent());

        await client.ChangePasswordAsync("alice", "N3wP@ss!");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Put, req.Method);
        Assert.Equal("/auth/users/alice/password", req.RequestUri!.AbsolutePath);

        var bodyJson = await req.Content!.ReadAsStringAsync();
        using var doc = JsonDocument.Parse(bodyJson);
        Assert.Equal("N3wP@ss!", doc.RootElement.GetProperty("new_password").GetString());
    }

    // ---------------------------------------------------------------------------
    // Integration-style: stateful Create → List → Revoke lifecycle
    // ---------------------------------------------------------------------------

    [Fact]
    public async Task TestAuthApiKeyLifecycle_CreateListRevoke()
    {
        var store = new Dictionary<string, object>(StringComparer.Ordinal);

        HttpResponseMessage StatefulResponder(HttpRequestMessage req)
        {
            var path = req.RequestUri!.AbsolutePath;

            if (req.Method == HttpMethod.Post && path == "/auth/keys")
            {
                var body = req.Content!.ReadAsStringAsync().GetAwaiter().GetResult();
                using var doc = JsonDocument.Parse(body);
                var name = doc.RootElement.GetProperty("name").GetString() ?? "unnamed";
                var keyId = Guid.NewGuid().ToString("N");
                var entry = new
                {
                    id = keyId,
                    name,
                    permissions = new[] { "read" },
                    api_key = "tok-" + keyId,
                    created_at = DateTimeOffset.UtcNow.ToUnixTimeSeconds(),
                    expires_at = (long?)null,
                    active = true,
                    warning = (string?)null,
                    usage_count = 0L
                };
                store[keyId] = entry;
                return JsonOk(entry);
            }

            if (req.Method == HttpMethod.Get && path == "/auth/keys")
            {
                return JsonOk(new { keys = store.Values.ToArray() });
            }

            if (req.Method == HttpMethod.Delete && path.StartsWith("/auth/keys/"))
            {
                var id = path["/auth/keys/".Length..];
                store.Remove(id);
                return NoContent();
            }

            return new HttpResponseMessage(HttpStatusCode.NotFound);
        }

        var handler = new FakeHandler { Responder = StatefulResponder };
        var http = new HttpClient(handler);
        var client = new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://test.local",
            HttpClient = http
        });

        // 1. Create
        var createReq = new CreateApiKeyRequest { Name = "lifecycle-key", Permissions = new List<string> { "read" } };
        var created = await client.CreateApiKeyAsync(createReq);
        Assert.NotEmpty(created.Id);
        Assert.Equal("lifecycle-key", created.Name);
        Assert.NotEmpty(created.ApiKeyValue!);

        // 2. List — key should appear
        var listed = await client.ListApiKeysAsync();
        Assert.Single(listed);
        Assert.Equal(created.Id, listed[0].Id);

        // 3. Revoke
        await client.RevokeApiKeyAsync(created.Id);

        // 4. List after revocation — store should be empty
        var afterRevoke = await client.ListApiKeysAsync();
        Assert.Empty(afterRevoke);

        // Verify request count: Create + List + Delete + List = 4
        Assert.Equal(4, handler.Requests.Count);
    }
}
