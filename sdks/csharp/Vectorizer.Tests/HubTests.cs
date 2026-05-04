using System.Net;
using System.Text;
using System.Text.Json;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

/// <summary>
/// Unit tests for Hub methods on VectorizerClient.
/// Uses an in-process FakeHandler to avoid network dependencies.
/// </summary>
public class HubTests
{
    // -------------------------------------------------------------------------
    // Test infrastructure helpers
    // -------------------------------------------------------------------------

    /// <summary>
    /// An HttpMessageHandler that returns a fixed HttpResponseMessage for every request,
    /// and records the last request for assertion.
    /// </summary>
    private sealed class FakeHandler : HttpMessageHandler
    {
        private readonly HttpResponseMessage _response;

        public HttpRequestMessage? LastRequest { get; private set; }

        public FakeHandler(HttpResponseMessage response)
        {
            _response = response;
        }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            LastRequest = request;
            return Task.FromResult(_response);
        }
    }

    /// <summary>
    /// Builds a VectorizerClient wired to the given FakeHandler so no
    /// real HTTP connection is required.
    /// </summary>
    private static (VectorizerClient client, FakeHandler handler) CreateClient(
        HttpResponseMessage response)
    {
        var handler = new FakeHandler(response);
        var httpClient = new HttpClient(handler) { BaseAddress = new Uri("http://localhost") };
        var client = new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://localhost",
            HttpClient = httpClient
        });
        return (client, handler);
    }

    /// <summary>
    /// Creates an HTTP 200 response with a JSON-serialised body.
    /// </summary>
    private static HttpResponseMessage JsonOk(object body)
    {
        var json = JsonSerializer.Serialize(body);
        return new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        };
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ListUserBackupsAsync_ReturnsMappedBackups()
    {
        var payload = new
        {
            backups = new[]
            {
                new { id = "b1", user_id = "u1", name = "backup-one", created_at = "2026-01-01", size = 1024L, status = "complete" }
            }
        };
        var (client, handler) = CreateClient(JsonOk(payload));

        var result = await client.ListUserBackupsAsync("u1");

        Assert.Single(result);
        Assert.Equal("b1", result[0].Id);
        Assert.Equal("u1", result[0].UserId);
        Assert.Contains("user_id=u1", handler.LastRequest!.RequestUri!.Query);
        Assert.Equal(HttpMethod.Get, handler.LastRequest.Method);
    }

    [Fact]
    public async Task CreateUserBackupAsync_ReturnsCreatedBackup()
    {
        var payload = new
        {
            backup = new { id = "b2", user_id = "u2", name = "new-backup", created_at = "2026-02-01", size = 0L, status = "pending" }
        };
        var (client, _) = CreateClient(JsonOk(payload));

        var req = new CreateUserBackupRequest { UserId = "u2", Name = "new-backup" };
        var result = await client.CreateUserBackupAsync(req);

        Assert.Equal("b2", result.Id);
        Assert.Equal("new-backup", result.Name);
    }

    [Fact]
    public async Task RestoreUserBackupAsync_CompletesWithoutException()
    {
        var (client, handler) = CreateClient(new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent("{}", Encoding.UTF8, "application/json")
        });

        var req = new RestoreUserBackupRequest { UserId = "u3", BackupId = "b3" };
        await client.RestoreUserBackupAsync(req); // must not throw

        Assert.Equal("/hub/backups/restore", handler.LastRequest!.RequestUri!.AbsolutePath);
        Assert.Equal(HttpMethod.Post, handler.LastRequest.Method);
    }

    [Fact]
    public async Task UploadUserBackupAsync_Base64EncodesData()
    {
        var payload = new
        {
            backup = new { id = "b4", user_id = "u4", name = "uploaded", created_at = "2026-03-01", size = 3L, status = "complete" }
        };
        var (client, handler) = CreateClient(JsonOk(payload));

        var bytes = new byte[] { 0xDE, 0xAD, 0xBE };
        var result = await client.UploadUserBackupAsync("u4", "uploaded", bytes);

        Assert.Equal("b4", result.Id);
        // Verify the request path contains query params
        Assert.Contains("user_id=u4", handler.LastRequest!.RequestUri!.Query);
        Assert.Contains("name=uploaded", handler.LastRequest.RequestUri.Query);
        // Verify body has base64-encoded data field. JSON serializer escapes
        // '+' as + by default, so decode the payload before asserting.
        var body = await handler.LastRequest.Content!.ReadAsStringAsync();
        using var doc = System.Text.Json.JsonDocument.Parse(body);
        var actual = doc.RootElement.GetProperty("data").GetString();
        Assert.Equal(Convert.ToBase64String(bytes), actual);
    }

    [Fact]
    public async Task GetUserBackupAsync_ReturnsSingleBackup()
    {
        var payload = new
        {
            backup = new { id = "b5", user_id = "u5", name = "fetched", created_at = "2026-04-01", size = 512L, status = "complete" }
        };
        var (client, handler) = CreateClient(JsonOk(payload));

        var result = await client.GetUserBackupAsync("u5", "b5");

        Assert.Equal("b5", result.Id);
        Assert.Contains("/hub/backups/b5", handler.LastRequest!.RequestUri!.AbsolutePath);
        Assert.Contains("user_id=u5", handler.LastRequest.RequestUri.Query);
    }

    [Fact]
    public async Task DeleteUserBackupAsync_SendsDeleteRequest()
    {
        var (client, handler) = CreateClient(new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent("{}", Encoding.UTF8, "application/json")
        });

        await client.DeleteUserBackupAsync("u6", "b6");

        Assert.Equal(HttpMethod.Delete, handler.LastRequest!.Method);
        Assert.Contains("/hub/backups/b6", handler.LastRequest.RequestUri!.AbsolutePath);
        Assert.Contains("user_id=u6", handler.LastRequest.RequestUri.Query);
    }

    [Fact]
    public async Task DownloadUserBackupAsync_ReturnsByteEqualContent()
    {
        var expectedBytes = new byte[] { 0x50, 0x4B, 0x03, 0x04, 0xFF, 0x00 };
        var (client, handler) = CreateClient(new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new ByteArrayContent(expectedBytes)
        });

        var result = await client.DownloadUserBackupAsync("u7", "b7");

        Assert.Equal(expectedBytes, result);
        Assert.Contains("/hub/backups/b7/download", handler.LastRequest!.RequestUri!.AbsolutePath);
        Assert.Contains("user_id=u7", handler.LastRequest.RequestUri.Query);
        Assert.Equal(HttpMethod.Get, handler.LastRequest.Method);
    }

    [Fact]
    public async Task GetUsageStatisticsAsync_ReturnsStatistics()
    {
        var payload = new { success = true, message = "ok", stats = new Dictionary<string, object> { ["total_vectors"] = 42 } };
        var (client, handler) = CreateClient(JsonOk(payload));

        var result = await client.GetUsageStatisticsAsync("u8");

        Assert.True(result.Success);
        Assert.Contains("user_id=u8", handler.LastRequest!.RequestUri!.Query);
    }

    [Fact]
    public async Task GetQuotaInfoAsync_ReturnsQuotaInfo()
    {
        var payload = new { success = true, message = "ok", quota = new Dictionary<string, object> { ["max_backups"] = 10 } };
        var (client, handler) = CreateClient(JsonOk(payload));

        var result = await client.GetQuotaInfoAsync("u9");

        Assert.True(result.Success);
        Assert.Contains("user_id=u9", handler.LastRequest!.RequestUri!.Query);
        Assert.Equal(HttpMethod.Get, handler.LastRequest.Method);
    }

    [Fact]
    public async Task ValidateHubApiKeyAsync_ReturnsValidationResult()
    {
        var payload = new
        {
            valid = true,
            tenant_id = "tenant-1",
            tenant_name = "MyTenant",
            permissions = new[] { "read", "write" },
            validated_at = "2026-05-01T00:00:00Z"
        };
        var (client, handler) = CreateClient(JsonOk(payload));

        var result = await client.ValidateHubApiKeyAsync("my-hub-key");

        Assert.True(result.Valid);
        Assert.Equal("tenant-1", result.TenantId);
        Assert.Equal(2, result.Permissions.Count);
        Assert.Equal("/hub/validate-key", handler.LastRequest!.RequestUri!.AbsolutePath);
        Assert.Equal(HttpMethod.Post, handler.LastRequest.Method);
        var body = await handler.LastRequest.Content!.ReadAsStringAsync();
        Assert.Contains("my-hub-key", body);
    }
}
