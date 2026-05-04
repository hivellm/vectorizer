using System;
using System.Collections.Generic;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

/// <summary>
/// Unit tests for the 9 phase-8 vector surface methods.
/// Each test uses a FakeHandler that intercepts the HTTP call, asserts the
/// outbound wire shape, and returns a canned JSON response — no live server needed.
/// </summary>
public class VectorsPhase8Tests
{
    // ── helpers ────────────────────────────────────────────────────────────────

    /// <summary>
    /// In-memory <see cref="HttpMessageHandler"/> that returns a fixed response.
    /// Callers may supply an optional assertion delegate that runs against the
    /// outbound <see cref="HttpRequestMessage"/> before the response is returned.
    /// </summary>
    private sealed class FakeHandler : HttpMessageHandler
    {
        private readonly HttpResponseMessage _response;
        private readonly Action<HttpRequestMessage>? _assert;

        public FakeHandler(HttpResponseMessage response, Action<HttpRequestMessage>? assert = null)
        {
            _response = response;
            _assert = assert;
        }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            _assert?.Invoke(request);
            return Task.FromResult(_response);
        }
    }

    /// <summary>Returns an <see cref="HttpResponseMessage"/> with 200 OK and the supplied JSON body.</summary>
    private static HttpResponseMessage JsonOk(string json) =>
        new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        };

    /// <summary>Creates a <see cref="VectorizerClient"/> wired to the supplied handler.</summary>
    private static VectorizerClient CreateClient(FakeHandler handler) =>
        new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://fake-host",
            HttpClient = new HttpClient(handler)
        });

    // ── 1. UpdateVectorPayloadAsync ─────────────────────────────────────────

    [Fact]
    public async Task UpdateVectorPayloadAsync_PostsToUpdateEndpoint()
    {
        var handler = new FakeHandler(
            JsonOk(@"{""message"":""updated""}"),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/update", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var meta = new Dictionary<string, object> { ["tag"] = "test" };
        var result = await client.UpdateVectorPayloadAsync("my-col", "vec-1", meta);

        Assert.NotNull(result);
        Assert.Equal("vec-1", result.Id);
    }

    // ── 2. InsertTextWithIdAsync ────────────────────────────────────────────

    [Fact]
    public async Task InsertTextWithIdAsync_PostsToInsertAndReturnsServerId()
    {
        var handler = new FakeHandler(
            JsonOk(@"{""id"":""server-id""}"),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/insert", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var result = await client.InsertTextWithIdAsync(
            "col1", "client-id", "hello world",
            new Dictionary<string, object> { ["lang"] = "en" });

        Assert.NotNull(result);
        Assert.Equal("server-id", result.Id);
    }

    // ── 3. ListVectorsAsync ─────────────────────────────────────────────────

    [Fact]
    public async Task ListVectorsAsync_GetWithLimitAndOffsetQueryParams()
    {
        var handler = new FakeHandler(
            JsonOk(@"{""total"":42,""limit"":10,""offset"":5,""vectors"":[{""id"":""v1""},{""id"":""v2""}]}"),
            req =>
            {
                Assert.Equal(HttpMethod.Get, req.Method);
                Assert.StartsWith("/collections/myCol/vectors", req.RequestUri!.AbsolutePath);
                Assert.Contains("limit=10", req.RequestUri.Query);
                Assert.Contains("offset=5", req.RequestUri.Query);
            });

        var client = CreateClient(handler);
        var page = await client.ListVectorsAsync("myCol", 10, 5);

        Assert.NotNull(page);
        Assert.Equal(42, page.Total);
        Assert.Equal(2, page.Vectors.Count);
    }

    // ── 4. BatchInsertRawTextsAsync ─────────────────────────────────────────

    [Fact]
    public async Task BatchInsertRawTextsAsync_PostsToBatchInsert()
    {
        var handler = new FakeHandler(
            JsonOk(@"{""collection"":""batch-col"",""count"":2,""inserted"":2,""failed"":0}"),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/batch_insert", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var items = new List<IDictionary<string, object>>
        {
            new Dictionary<string, object> { ["text"] = "first doc" },
            new Dictionary<string, object> { ["text"] = "second doc" }
        };
        var report = await client.BatchInsertRawTextsAsync("batch-col", items);

        Assert.NotNull(report);
        Assert.Equal(2, report.Inserted);
        Assert.Equal(0, report.Failed);
    }

    // ── 5. InsertVectorsAsync ───────────────────────────────────────────────

    [Fact]
    public async Task InsertVectorsAsync_PostsToInsertVectors()
    {
        var handler = new FakeHandler(
            JsonOk(@"{""collection"":""vec-col"",""count"":2,""inserted"":2,""failed"":0}"),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/insert_vectors", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var vectors = new List<Vector>
        {
            new Vector { Id = "v1", Data = new float[] { 0.1f, 0.2f } },
            new Vector { Id = "v2", Data = new float[] { 0.3f, 0.4f } }
        };
        var report = await client.InsertVectorsAsync("vec-col", vectors);

        Assert.NotNull(report);
        Assert.Equal(2, report.Inserted);
        Assert.Equal("vec-col", report.Collection);
    }

    // ── 6. BatchSearchQueriesAsync ──────────────────────────────────────────

    [Fact]
    public async Task BatchSearchQueriesAsync_PostsToBatchSearchAndUnpacksResults()
    {
        const string json = @"{
            ""collection"":""search-col"",
            ""count"":2,
            ""results"":[
                {""query"":""query one"",""collection"":""search-col"",""total_results"":0,""results"":[]},
                {""query"":""query two"",""collection"":""search-col"",""total_results"":0,""results"":[]}
            ]
        }";

        var handler = new FakeHandler(
            JsonOk(json),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/batch_search", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var queries = new List<IDictionary<string, object>>
        {
            new Dictionary<string, object> { ["query"] = "query one" },
            new Dictionary<string, object> { ["query"] = "query two" }
        };
        var responses = await client.BatchSearchQueriesAsync("search-col", queries);

        Assert.Equal(2, responses.Count);
        Assert.Equal("query one", responses[0].Query);
        Assert.Equal("query two", responses[1].Query);
    }

    // ── 7. BatchUpdateRawVectorsAsync ───────────────────────────────────────

    [Fact]
    public async Task BatchUpdateRawVectorsAsync_PostsToBatchUpdate()
    {
        var handler = new FakeHandler(
            JsonOk(@"{""collection"":""upd-col"",""count"":3,""updated"":3,""failed"":0}"),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/batch_update", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var updates = new List<IDictionary<string, object>>
        {
            new Dictionary<string, object> { ["id"] = "v1", ["metadata"] = new Dictionary<string, object> { ["k"] = "a" } },
            new Dictionary<string, object> { ["id"] = "v2", ["metadata"] = new Dictionary<string, object> { ["k"] = "b" } },
            new Dictionary<string, object> { ["id"] = "v3", ["metadata"] = new Dictionary<string, object> { ["k"] = "c" } }
        };
        var report = await client.BatchUpdateRawVectorsAsync("upd-col", updates);

        Assert.NotNull(report);
        Assert.Equal(3, report.Updated);
        Assert.Equal(0, report.Failed);
    }

    // ── 8. SearchByTextAsync ────────────────────────────────────────────────

    [Fact]
    public async Task SearchByTextAsync_PostsToSearchTextEndpoint()
    {
        const string json = @"{
            ""collection"":""txt-col"",
            ""query"":""find me"",
            ""limit"":7,
            ""total_results"":1,
            ""results"":[{""id"":""r1"",""score"":0.99}]
        }";

        var handler = new FakeHandler(
            JsonOk(json),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/collections/txt-col/search/text", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var resp = await client.SearchByTextAsync("txt-col", "find me", 7);

        Assert.NotNull(resp);
        Assert.Equal(1, resp.TotalResults);
        Assert.Single(resp.Results);
        Assert.Equal("r1", resp.Results[0].Id);
    }

    // ── 9. SearchByFileAsync ────────────────────────────────────────────────

    [Fact]
    public async Task SearchByFileAsync_PostsToSearchFileEndpoint()
    {
        const string json = @"{
            ""collection"":""file-col"",
            ""total_results"":2,
            ""results"":[
                {""id"":""f1"",""score"":0.9},
                {""id"":""f2"",""score"":0.8}
            ]
        }";

        var handler = new FakeHandler(
            JsonOk(json),
            req =>
            {
                Assert.Equal(HttpMethod.Post, req.Method);
                Assert.Equal("/collections/file-col/search/file", req.RequestUri!.AbsolutePath);
            });

        var client = CreateClient(handler);
        var resp = await client.SearchByFileAsync("file-col", "/docs/readme.md", 5);

        Assert.NotNull(resp);
        Assert.Equal(2, resp.TotalResults);
        Assert.Equal(2, resp.Results.Count);
        Assert.Equal("f1", resp.Results[0].Id);
    }
}
