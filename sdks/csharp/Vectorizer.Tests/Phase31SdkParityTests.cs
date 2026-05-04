using System.Net;
using System.Text;
using System.Text.Json;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

/// <summary>
/// Unit tests for the four phase31 client methods that close the v3.3.0
/// parity gap between the C# SDK and the Rust / TypeScript / Python SDKs:
/// <list type="bullet">
///   <item><see cref="VectorizerClient.UpdateApiKeyPermissionsAsync"/></item>
///   <item><see cref="VectorizerClient.GetApiKeyUsageAsync"/></item>
///   <item><see cref="VectorizerClient.DeleteVectorsAsync"/></item>
///   <item><see cref="VectorizerClient.MoveToCollectionAsync"/></item>
/// </list>
/// Uses an in-process FakeHandler so no live server is required.
/// </summary>
public class Phase31SdkParityTests
{
    private sealed class FakeHandler : HttpMessageHandler
    {
        private readonly Func<HttpRequestMessage, HttpResponseMessage> _respond;

        public HttpRequestMessage? LastRequest { get; private set; }
        public string? LastBody { get; private set; }

        public FakeHandler(HttpResponseMessage response)
            : this(_ => response) { }

        public FakeHandler(Func<HttpRequestMessage, HttpResponseMessage> respond)
        {
            _respond = respond;
        }

        protected override async Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            LastRequest = request;
            if (request.Content is not null)
            {
                LastBody = await request.Content.ReadAsStringAsync(cancellationToken);
            }
            return _respond(request);
        }
    }

    private static VectorizerClient CreateClient(FakeHandler handler)
    {
        var httpClient = new HttpClient(handler)
        {
            BaseAddress = new Uri("http://localhost:15002")
        };
        return new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://localhost:15002",
            HttpClient = httpClient
        });
    }

    private static HttpResponseMessage JsonOk(string json) =>
        new(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        };

    // -------------------------------------------------------------------------
    // UpdateApiKeyPermissionsAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task UpdateApiKeyPermissionsAsync_TargetsCorrectRouteAndDecodesView()
    {
        const string json = """
            {
              "id": "key-42",
              "name": "ci-bot",
              "user_id": "user-1",
              "permissions": ["read", "write"],
              "scopes": [],
              "created_at": 1700000000,
              "active": true,
              "usage_count": 17
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var view = await client.UpdateApiKeyPermissionsAsync(
            "key-42",
            new UpdateApiKeyPermissionsRequest
            {
                Permissions = new List<string> { "read", "write" }
            });

        Assert.Equal(HttpMethod.Put, handler.LastRequest!.Method);
        Assert.Equal(
            "http://localhost:15002/auth/keys/key-42/permissions",
            handler.LastRequest.RequestUri!.ToString());
        Assert.Equal("key-42", view.Id);
        Assert.Equal(2, view.Permissions.Count);
        Assert.Equal(17, view.UsageCount);
    }

    // -------------------------------------------------------------------------
    // GetApiKeyUsageAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task GetApiKeyUsageAsync_DefaultWindow_OmitsQueryParameter()
    {
        const string json = """
            {
              "key": {
                "id": "key-42",
                "name": "ci-bot",
                "user_id": "u1",
                "permissions": [],
                "scopes": [],
                "created_at": 1700000000,
                "active": true,
                "usage_count": 0
              },
              "buckets": [
                {"date": "2026-04-28", "count": 10},
                {"date": "2026-04-29", "count": 0}
              ],
              "window_total": 10
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var report = await client.GetApiKeyUsageAsync("key-42");

        Assert.Equal(HttpMethod.Get, handler.LastRequest!.Method);
        Assert.Equal(
            "http://localhost:15002/auth/keys/key-42/usage",
            handler.LastRequest.RequestUri!.ToString());
        Assert.Equal(2, report.Buckets.Count);
        Assert.Equal(10, report.WindowTotal);
    }

    [Fact]
    public async Task GetApiKeyUsageAsync_CustomWindow_AppendsQueryParameter()
    {
        var handler = new FakeHandler(JsonOk("""
            {"key": {"id": "key-42", "name": "n", "user_id": "u", "permissions": [], "scopes": [], "created_at": 0, "active": true, "usage_count": 0}, "buckets": [], "window_total": 0}
            """));
        using var client = CreateClient(handler);

        await client.GetApiKeyUsageAsync("key-42", windowDays: 14);

        Assert.Equal(
            "http://localhost:15002/auth/keys/key-42/usage?window=14",
            handler.LastRequest!.RequestUri!.ToString());
    }

    // -------------------------------------------------------------------------
    // DeleteVectorsAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task DeleteVectorsAsync_PostsBatchDeleteAndDecodesReport()
    {
        const string json = """
            {
              "collection": "docs",
              "count": 3,
              "deleted": 2,
              "failed": 1,
              "results": [
                {"id": "a", "status": "ok"},
                {"id": "b", "status": "ok"},
                {"id": "c", "status": "missing_in_src"}
              ]
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var report = await client.DeleteVectorsAsync("docs", new[] { "a", "b", "c" });

        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal(
            "http://localhost:15002/batch_delete",
            handler.LastRequest.RequestUri!.ToString());

        // Body must carry both the collection name and the id list verbatim.
        using var bodyDoc = JsonDocument.Parse(handler.LastBody!);
        Assert.Equal("docs", bodyDoc.RootElement.GetProperty("collection").GetString());
        Assert.Equal(3, bodyDoc.RootElement.GetProperty("ids").GetArrayLength());

        Assert.Equal(2, report.Deleted);
        Assert.Equal(1, report.Failed);
        Assert.Equal(3, report.Results.Count);
    }

    // -------------------------------------------------------------------------
    // MoveToCollectionAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task MoveToCollectionAsync_TargetsSourceMoveRouteAndDecodesReport()
    {
        const string json = """
            {
              "src": "hot",
              "dst": "cold",
              "requested": 2,
              "moved": 2,
              "failed": 0,
              "results": [
                {"id": "x", "status": "ok"},
                {"id": "y", "status": "ok"}
              ]
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var report = await client.MoveToCollectionAsync("hot", "cold", new[] { "x", "y" });

        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal(
            "http://localhost:15002/collections/hot/vectors/move",
            handler.LastRequest.RequestUri!.ToString());

        using var bodyDoc = JsonDocument.Parse(handler.LastBody!);
        Assert.Equal("cold", bodyDoc.RootElement.GetProperty("destination").GetString());
        Assert.Equal(2, bodyDoc.RootElement.GetProperty("ids").GetArrayLength());

        Assert.Equal("hot", report.Src);
        Assert.Equal("cold", report.Dst);
        Assert.Equal(2, report.Moved);
        Assert.Equal(2, report.Results.Count);
    }
}
