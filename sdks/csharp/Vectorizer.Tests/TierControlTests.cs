using System.Collections.Generic;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Vectorizer.Exceptions;
using Vectorizer.Models;
using Xunit;

namespace Vectorizer.Tests;

public class TierControlTests
{
    // -------------------------------------------------------------------------
    // Infrastructure
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Happy-path tests (6 — one per method)
    // -------------------------------------------------------------------------

    [Fact]
    public async Task DeleteByFilterAsync_ValidFilter_PostsToCorrectEndpoint()
    {
        var expected = new { scanned = 10, matched = 3, deleted = 3, results = Array.Empty<object>() };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var filter = new Dictionary<string, object> { ["category"] = "news" };
        var report = await client.DeleteByFilterAsync("my-col", filter);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/my-col/vectors/delete_by_filter", req.RequestUri!.AbsolutePath);
        Assert.Equal(10, report.Scanned);
        Assert.Equal(3, report.Matched);
        Assert.Equal(3, report.Deleted);
    }

    [Fact]
    public async Task BulkUpdateMetadataAsync_ValidFilter_PostsToCorrectEndpoint()
    {
        var expected = new { scanned = 20, matched = 5, updated = 5, results = Array.Empty<object>() };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var filter = new Dictionary<string, object> { ["status"] = "draft" };
        var patch = new Dictionary<string, object> { ["status"] = "published" };
        var report = await client.BulkUpdateMetadataAsync("articles", filter, patch);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/articles/vectors/bulk_update_metadata", req.RequestUri!.AbsolutePath);
        Assert.Equal(20, report.Scanned);
        Assert.Equal(5, report.Matched);
        Assert.Equal(5, report.Updated);
    }

    [Fact]
    public async Task CopyVectorsAsync_ValidIds_PostsToCorrectEndpoint()
    {
        var expected = new
        {
            src = "source-col",
            dst = "dest-col",
            requested = 2,
            copied = 2,
            failed = 0,
            results = Array.Empty<object>()
        };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var ids = new[] { "id-1", "id-2" };
        var report = await client.CopyVectorsAsync("source-col", "dest-col", ids);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/source-col/vectors/copy", req.RequestUri!.AbsolutePath);

        var bodyJson = await req.Content!.ReadAsStringAsync();
        using var doc = JsonDocument.Parse(bodyJson);
        Assert.Equal("dest-col", doc.RootElement.GetProperty("destination").GetString());

        Assert.Equal("source-col", report.Src);
        Assert.Equal("dest-col", report.Dst);
        Assert.Equal(2, report.Copied);
    }

    [Fact]
    public async Task ReencodeCollectionAsync_ValidEncoding_PostsToCorrectEndpoint()
    {
        var expected = new
        {
            job_id = "job-abc",
            collection = "embeddings",
            state = "completed",
            target_encoding = "sq8",
            progress = 1.0
        };
        var (client, handler) = CreateClient(_ => JsonOk(expected));

        var job = await client.ReencodeCollectionAsync("embeddings", "sq8");

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/embeddings/reencode", req.RequestUri!.AbsolutePath);

        var bodyJson = await req.Content!.ReadAsStringAsync();
        using var doc = JsonDocument.Parse(bodyJson);
        Assert.Equal("sq8", doc.RootElement.GetProperty("target_encoding").GetString());

        Assert.Equal("job-abc", job.JobId);
        Assert.Equal("completed", job.State);
        Assert.Equal("sq8", job.TargetEncoding);
    }

    [Fact]
    public async Task SetCollectionTtlAsync_NullTtl_PostsNullToCorrectEndpoint()
    {
        var (client, handler) = CreateClient(_ => NoContent());

        await client.SetCollectionTtlAsync("my-col", null);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal("/collections/my-col/ttl", req.RequestUri!.AbsolutePath);

        var bodyJson = await req.Content!.ReadAsStringAsync();
        using var doc = JsonDocument.Parse(bodyJson);
        Assert.Equal(JsonValueKind.Null, doc.RootElement.GetProperty("ttl_secs").ValueKind);
    }

    [Fact]
    public async Task SetVectorExpiryAsync_ValidExpiry_PatchesToCorrectEndpoint()
    {
        var (client, handler) = CreateClient(_ => NoContent());

        await client.SetVectorExpiryAsync("col-x", "vec-1", 9999999L);

        var req = Assert.Single(handler.Requests);
        Assert.Equal(new HttpMethod("PATCH"), req.Method);
        Assert.Equal("/collections/col-x/vectors/vec-1/expiry", req.RequestUri!.AbsolutePath);

        var bodyJson = await req.Content!.ReadAsStringAsync();
        using var doc = JsonDocument.Parse(bodyJson);
        Assert.Equal(9999999L, doc.RootElement.GetProperty("expires_at").GetInt64());
    }

    // -------------------------------------------------------------------------
    // Validation tests — server FakeHandler must NOT be called
    // -------------------------------------------------------------------------

    [Fact]
    public async Task DeleteByFilterAsync_EmptyFilter_ThrowsBeforeHttp()
    {
        var (client, _) = CreateClient(_ =>
        {
            Assert.Fail("server should not be called");
            return new HttpResponseMessage(HttpStatusCode.OK);
        });

        var ex = await Assert.ThrowsAsync<VectorizerException>(
            () => client.DeleteByFilterAsync("col", new Dictionary<string, object>()));

        Assert.Equal("validation_error", ex.ErrorType);
        Assert.Equal(0, ex.StatusCode);
    }

    [Fact]
    public async Task BulkUpdateMetadataAsync_EmptyFilter_ThrowsBeforeHttp()
    {
        var (client, _) = CreateClient(_ =>
        {
            Assert.Fail("server should not be called");
            return new HttpResponseMessage(HttpStatusCode.OK);
        });

        var ex = await Assert.ThrowsAsync<VectorizerException>(
            () => client.BulkUpdateMetadataAsync(
                "col",
                new Dictionary<string, object>(),
                new Dictionary<string, object> { ["key"] = "val" }));

        Assert.Equal("validation_error", ex.ErrorType);
        Assert.Equal(0, ex.StatusCode);
    }
}
