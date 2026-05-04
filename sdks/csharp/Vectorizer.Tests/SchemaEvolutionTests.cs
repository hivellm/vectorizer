using System.Net;
using System.Text;
using System.Text.Json;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

public class SchemaEvolutionTests
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

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    [Fact]
    public async Task RenameCollectionAsync_PostsToCorrectEndpoint()
    {
        var (client, handler) = CreateClient(_ => JsonOk(new { }));

        await client.RenameCollectionAsync("old_col", "new_col");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/old_col/rename", req.RequestUri!.AbsolutePath);
        var body = await req.Content!.ReadAsStringAsync();
        Assert.Contains("new_col", body);
    }

    [Fact]
    public async Task ReindexCollectionAsync_PostsParamsAndReturnsJob()
    {
        var job = new ReindexJob
        {
            JobId = "job-1",
            Collection = "my_col",
            State = "completed",
            Progress = 1.0
        };
        var (client, handler) = CreateClient(_ => JsonOk(job));

        var result = await client.ReindexCollectionAsync(
            "my_col",
            new ReindexParams { M = 16, EfConstruction = 200, EfSearch = 100 });

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/my_col/reindex", req.RequestUri!.AbsolutePath);
        Assert.Equal("job-1", result.JobId);
        Assert.Equal("completed", result.State);
    }

    [Fact]
    public async Task SnapshotCollectionNativeAsync_PostsEmptyBodyReturnsInfo()
    {
        var info = new NativeSnapshotInfo
        {
            Id = "snap-42",
            Collection = "my_col",
            CreatedAt = "2025-01-01T00:00:00Z",
            SizeBytes = 1024
        };
        var (client, handler) = CreateClient(_ => JsonOk(info));

        var result = await client.SnapshotCollectionNativeAsync("my_col");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/my_col/snapshot", req.RequestUri!.AbsolutePath);
        Assert.Equal("snap-42", result.Id);
        Assert.Equal(1024L, result.SizeBytes);
    }

    [Fact]
    public async Task ListCollectionSnapshotsNativeAsync_UnwrapsSnapshotsEnvelope()
    {
        var envelope = new
        {
            snapshots = new[]
            {
                new { id = "snap-1", collection = "my_col", created_at = "2025-01-01T00:00:00Z", size_bytes = 512 },
                new { id = "snap-2", collection = "my_col", created_at = "2025-01-02T00:00:00Z", size_bytes = 768 }
            }
        };
        var (client, handler) = CreateClient(_ => JsonOk(envelope));

        var results = await client.ListCollectionSnapshotsNativeAsync("my_col");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Get, req.Method);
        Assert.Equal("/collections/my_col/snapshots", req.RequestUri!.AbsolutePath);
        Assert.Equal(2, results.Count);
        Assert.Equal("snap-1", results[0].Id);
        Assert.Equal("snap-2", results[1].Id);
    }

    [Fact]
    public async Task RestoreCollectionSnapshotNativeAsync_PostsToRestoreEndpoint()
    {
        var (client, handler) = CreateClient(_ => JsonOk(new { }));

        await client.RestoreCollectionSnapshotNativeAsync("my_col", "snap-42");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/my_col/snapshots/snap-42/restore", req.RequestUri!.AbsolutePath);
    }

    [Fact]
    public async Task ExplainSearchAsync_PostsVectorAndKReturnsTrace()
    {
        var response = new
        {
            collection = "my_col",
            k = 5,
            results = Array.Empty<object>(),
            trace = new
            {
                visited_nodes = 42,
                ef_search = 100,
                hnsw_search_ms = 1.5,
                payload_filter_evals = 0,
                quantization_score_ms = 0.2,
                total_ms = 2.1
            }
        };
        var (client, handler) = CreateClient(_ => JsonOk(response));

        var vector = new List<float> { 0.1f, 0.2f, 0.3f };
        var result = await client.ExplainSearchAsync("my_col", vector, 5);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/my_col/explain", req.RequestUri!.AbsolutePath);
        var body = await req.Content!.ReadAsStringAsync();
        Assert.Contains("\"k\"", body);
        Assert.Equal("my_col", result.Collection);
        Assert.Equal(42, result.Trace.VisitedNodes);
    }

    [Fact]
    public async Task ListSlowQueriesAsync_UnwrapsEntriesEnvelope()
    {
        var envelope = new
        {
            entries = new[]
            {
                new { timestamp = "2025-01-01T00:00:00Z", collection = "my_col", k = 10, duration_ms = 120.5 }
            }
        };
        var (client, handler) = CreateClient(_ => JsonOk(envelope));

        var results = await client.ListSlowQueriesAsync();

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Get, req.Method);
        Assert.Equal("/slow_queries", req.RequestUri!.AbsolutePath);
        Assert.Single(results);
        Assert.Equal("my_col", results[0].Collection);
        Assert.Equal(120.5, results[0].DurationMs);
    }

    [Fact]
    public async Task SetSlowQueryConfigAsync_PostsConfigAndReturnsUpdated()
    {
        var returned = new SlowQueryConfig { ThresholdMs = 500, Capacity = 200 };
        var (client, handler) = CreateClient(_ => JsonOk(returned));

        var result = await client.SetSlowQueryConfigAsync(
            new SlowQueryConfig { ThresholdMs = 500, Capacity = 200 });

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/slow_queries/config", req.RequestUri!.AbsolutePath);
        var body = await req.Content!.ReadAsStringAsync();
        Assert.Contains("500", body);
        Assert.Equal(500L, result.ThresholdMs);
        Assert.Equal(200, result.Capacity);
    }
}
