using System.Collections.Generic;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Xunit;
using Vectorizer;
using Vectorizer.Models;
using Vectorizer.Exceptions;

namespace Vectorizer.Tests;

public class AdminTests
{
    // ---------------------------------------------------------------------------
    // Infrastructure
    // ---------------------------------------------------------------------------

    private sealed class FakeHandler : HttpMessageHandler
    {
        public Func<HttpRequestMessage, HttpResponseMessage> Responder { get; set; } =
            _ => new HttpResponseMessage(HttpStatusCode.NotFound);

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
        var client = new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://test.local",
            HttpClient = http
        });
        return (client, handler);
    }

    private static HttpResponseMessage JsonOk(object body) =>
        new(HttpStatusCode.OK)
        {
            Content = new StringContent(
                JsonSerializer.Serialize(body),
                Encoding.UTF8,
                "application/json")
        };

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    [Fact]
    public async Task GetServerStatsAsync_DecodesAllFields()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/stats", req.RequestUri!.AbsolutePath);
            return JsonOk(new
            {
                collections = 3,
                total_vectors = 100L,
                uptime_seconds = 60L,
                version = "3.3.0"
            });
        });

        var result = await client.GetServerStatsAsync();

        Assert.Equal(3, result.Collections);
        Assert.Equal(100L, result.TotalVectors);
        Assert.Equal(60L, result.UptimeSeconds);
        Assert.Equal("3.3.0", result.Version);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task GetStatusAsync_DecodesAllFields()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/status", req.RequestUri!.AbsolutePath);
            return JsonOk(new
            {
                online = true,
                version = "3.3.0",
                uptime_seconds = 120L,
                collections_count = 5
            });
        });

        var result = await client.GetStatusAsync();

        Assert.True(result.Online);
        Assert.Equal("3.3.0", result.Version);
        Assert.Equal(120L, result.UptimeSeconds);
        Assert.Equal(5, result.CollectionsCount);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task GetLogsAsync_UnwrapsEnvelopeAndForwardsQueryParams()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/logs", req.RequestUri!.AbsolutePath);
            Assert.Contains("lines=50", req.RequestUri.Query);
            Assert.Contains("level=error", req.RequestUri.Query);
            return JsonOk(new
            {
                logs = new[]
                {
                    new { timestamp = "2026-01-01T00:00:00Z", level = "error", message = "boom", source = "core" }
                }
            });
        });

        var result = await client.GetLogsAsync(lines: 50, level: "error");

        Assert.Single(result);
        Assert.Equal("error", result[0].Level);
        Assert.Equal("boom", result[0].Message);
    }

    [Fact]
    public async Task GetLogsAsync_OmitsEmptyQueryParams()
    {
        var (client, _) = CreateClient(req =>
        {
            // Neither "lines" nor "level" should appear when using defaults
            Assert.Equal("/logs", req.RequestUri!.PathAndQuery);
            return JsonOk(new { logs = new object[0] });
        });

        var result = await client.GetLogsAsync();

        Assert.Empty(result);
    }

    [Fact]
    public async Task GetIndexingProgressAsync_DecodesAllFields()
    {
        var (client, _) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/indexing/progress", req.RequestUri!.AbsolutePath);
            return JsonOk(new
            {
                is_indexing = true,
                overall_status = "running",
                collections = new[]
                {
                    new
                    {
                        collection_name = "docs",
                        status = "indexing",
                        progress = 0.75,
                        vector_count = 500,
                        error_message = (string?)null,
                        last_updated = "2026-01-01T00:00:00Z"
                    }
                }
            });
        });

        var result = await client.GetIndexingProgressAsync();

        Assert.True(result.IsIndexing);
        Assert.Equal("running", result.OverallStatus);
        Assert.Single(result.Collections);
        Assert.Equal("docs", result.Collections[0].CollectionName);
        Assert.Equal(0.75, result.Collections[0].Progress);
    }

    [Fact]
    public async Task ForceSaveCollectionAsync_PostsCorrectPath()
    {
        const string col = "my-col";
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Post, req.Method);
            Assert.Equal($"/collections/{col}/force-save", req.RequestUri!.AbsolutePath);
            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{}", Encoding.UTF8, "application/json")
            };
        });

        await client.ForceSaveCollectionAsync(col);

        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task ListEmptyCollectionsAsync_HandlesBareArray()
    {
        var (client, _) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/collections/empty", req.RequestUri!.AbsolutePath);
            return JsonOk(new[] { "col_a", "col_b" });
        });

        var result = await client.ListEmptyCollectionsAsync();

        Assert.Equal(2, result.Count);
        Assert.Contains("col_a", result);
        Assert.Contains("col_b", result);
    }

    [Fact]
    public async Task ListEmptyCollectionsAsync_HandlesEnvelope()
    {
        var (client, _) = CreateClient(_ =>
            JsonOk(new { collections = new[] { "col_x" } }));

        var result = await client.ListEmptyCollectionsAsync();

        Assert.Single(result);
        Assert.Equal("col_x", result[0]);
    }

    [Fact]
    public async Task CleanupEmptyCollectionsAsync_DecodesReport()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Delete, req.Method);
            Assert.Equal("/collections/cleanup", req.RequestUri!.AbsolutePath);
            return JsonOk(new
            {
                success = true,
                removed = 2,
                collections = new[] { "old_a", "old_b" },
                message = "done"
            });
        });

        var result = await client.CleanupEmptyCollectionsAsync();

        Assert.True(result.Success);
        Assert.Equal(2, result.Removed);
        Assert.Equal(2, result.Collections.Count);
        Assert.Equal("done", result.Message);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task GetConfigAsync_ReturnsConfigSnapshot()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/config", req.RequestUri!.AbsolutePath);
            return JsonOk(new Dictionary<string, object>
            {
                ["log_level"] = "info",
                ["max_connections"] = 100
            });
        });

        var result = await client.GetConfigAsync();

        Assert.NotNull(result);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task UpdateConfigAsync_SendsPatchBodyAndReturnsSnapshot()
    {
        var patch = new Dictionary<string, object> { ["log_level"] = "debug" };

        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Post, req.Method);
            Assert.Equal("/config", req.RequestUri!.AbsolutePath);

            var bodyJson = req.Content!.ReadAsStringAsync().GetAwaiter().GetResult();
            using var doc = JsonDocument.Parse(bodyJson);
            Assert.Equal("debug", doc.RootElement.GetProperty("log_level").GetString());

            return JsonOk(patch);
        });

        var result = await client.UpdateConfigAsync(patch);

        Assert.NotNull(result);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task UpdateConfigAsync_403_ThrowsVectorizerException()
    {
        // The client deserializes errors with a camelCase JsonNamingPolicy, so the
        // wire field is "errorType" (camelCase), not "error_type" (snake_case).
        var (client, _) = CreateClient(_ => new HttpResponseMessage(HttpStatusCode.Forbidden)
        {
            Content = new StringContent(
                "{\"errorType\":\"forbidden\",\"message\":\"admin only\"}",
                Encoding.UTF8,
                "application/json")
        });

        var ex = await Assert.ThrowsAsync<VectorizerException>(
            () => client.UpdateConfigAsync(new Dictionary<string, object>()));

        Assert.Equal("forbidden", ex.ErrorType);
        Assert.Equal(403, ex.StatusCode);
    }

    [Fact]
    public async Task ListBackupsAsync_UnwrapsEnvelope()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/backups", req.RequestUri!.AbsolutePath);
            return JsonOk(new
            {
                backups = new[]
                {
                    new { id = "bk-1", name = "daily", date = "2026-01-01", size = 1024L, collections = new[] { "docs" } }
                }
            });
        });

        var result = await client.ListBackupsAsync();

        Assert.Single(result);
        Assert.Equal("bk-1", result[0].Id);
        Assert.Equal("daily", result[0].Name);
        Assert.Equal(1024L, result[0].Size);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task CreateBackupAsync_SendsRequestBodyAndDecodesResponse()
    {
        var req = new CreateBackupRequest { Name = "manual-backup", Collections = new List<string> { "docs" } };

        var (client, handler) = CreateClient(httpReq =>
        {
            Assert.Equal(HttpMethod.Post, httpReq.Method);
            Assert.Equal("/backups/create", httpReq.RequestUri!.AbsolutePath);

            var bodyJson = httpReq.Content!.ReadAsStringAsync().GetAwaiter().GetResult();
            using var doc = JsonDocument.Parse(bodyJson);
            Assert.Equal("manual-backup", doc.RootElement.GetProperty("name").GetString());

            return JsonOk(new { id = "bk-99", name = "manual-backup", date = "2026-01-01", size = 512L, collections = new[] { "docs" } });
        });

        var result = await client.CreateBackupAsync(req);

        Assert.Equal("bk-99", result.Id);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task RestoreBackupAsync_PostsCorrectBody()
    {
        var req = new RestoreBackupRequest { BackupId = "bk-99" };

        var (client, handler) = CreateClient(httpReq =>
        {
            Assert.Equal(HttpMethod.Post, httpReq.Method);
            Assert.Equal("/backups/restore", httpReq.RequestUri!.AbsolutePath);

            var bodyJson = httpReq.Content!.ReadAsStringAsync().GetAwaiter().GetResult();
            using var doc = JsonDocument.Parse(bodyJson);
            Assert.Equal("bk-99", doc.RootElement.GetProperty("backup_id").GetString());

            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{}", Encoding.UTF8, "application/json")
            };
        });

        await client.RestoreBackupAsync(req);

        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task RestartServerAsync_PostsCorrectPath()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Post, req.Method);
            Assert.Equal("/admin/restart", req.RequestUri!.AbsolutePath);
            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{}", Encoding.UTF8, "application/json")
            };
        });

        await client.RestartServerAsync();

        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task ListWorkspacesAsync_UnwrapsEnvelope()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/workspace/list", req.RequestUri!.AbsolutePath);
            return JsonOk(new
            {
                workspaces = new[]
                {
                    new { name = "default", path = "/data/ws", collections = new[] { "docs" } }
                }
            });
        });

        var result = await client.ListWorkspacesAsync();

        Assert.Single(result);
        Assert.Equal("default", result[0].Name);
        Assert.Equal("/data/ws", result[0].Path);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task GetWorkspaceConfigAsync_ReturnsConfig()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/workspace/config", req.RequestUri!.AbsolutePath);
            return JsonOk(new Dictionary<string, object> { ["active"] = "default" });
        });

        var result = await client.GetWorkspaceConfigAsync();

        Assert.NotNull(result);
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task AddWorkspaceAsync_PostsRequestBody()
    {
        var req = new AddWorkspaceRequest { Path = "/data/new-ws", CollectionName = "docs" };

        var (client, handler) = CreateClient(httpReq =>
        {
            Assert.Equal(HttpMethod.Post, httpReq.Method);
            Assert.Equal("/workspace/add", httpReq.RequestUri!.AbsolutePath);

            var bodyJson = httpReq.Content!.ReadAsStringAsync().GetAwaiter().GetResult();
            using var doc = JsonDocument.Parse(bodyJson);
            Assert.Equal("/data/new-ws", doc.RootElement.GetProperty("path").GetString());
            Assert.Equal("docs", doc.RootElement.GetProperty("collection_name").GetString());

            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{}", Encoding.UTF8, "application/json")
            };
        });

        await client.AddWorkspaceAsync(req);

        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task RemoveWorkspaceAsync_PostsNameAsPath()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Post, req.Method);
            Assert.Equal("/workspace/remove", req.RequestUri!.AbsolutePath);

            var bodyJson = req.Content!.ReadAsStringAsync().GetAwaiter().GetResult();
            using var doc = JsonDocument.Parse(bodyJson);
            Assert.Equal("default", doc.RootElement.GetProperty("path").GetString());

            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{}", Encoding.UTF8, "application/json")
            };
        });

        await client.RemoveWorkspaceAsync("default");

        Assert.Single(handler.Requests);
    }
}
