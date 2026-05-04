using System.Collections.Generic;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Vectorizer.Models;
using Xunit;

namespace Vectorizer.Tests;

public class FilterTests
{
    // -------------------------------------------------------------------------
    // Infrastructure (mirrors TierControlTests.FakeHandler)
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

    // -------------------------------------------------------------------------
    // Serialisation tests
    // -------------------------------------------------------------------------

    [Fact]
    public void Eq_serializes_to_match_value()
    {
        var condition = Filter.Eq("topic", "index");
        var json = JsonSerializer.Serialize(condition);

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;

        Assert.Equal("topic", root.GetProperty("key").GetString());
        Assert.Equal("index", root.GetProperty("match").GetProperty("value").GetString());
        Assert.False(root.GetProperty("match").TryGetProperty("any", out _));
    }

    [Fact]
    public void In_serializes_to_match_any()
    {
        var condition = Filter.In("status", new object[] { "draft", "review" });
        var json = JsonSerializer.Serialize(condition);

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;

        Assert.Equal("status", root.GetProperty("key").GetString());
        var any = root.GetProperty("match").GetProperty("any");
        Assert.Equal(2, any.GetArrayLength());
        Assert.Equal("draft", any[0].GetString());
        Assert.Equal("review", any[1].GetString());
        Assert.False(root.GetProperty("match").TryGetProperty("value", out _));
    }

    [Fact]
    public void Range_serializes_with_gte_lte()
    {
        var condition = Filter.Range("score", gte: 0.5, lte: 0.9);
        var json = JsonSerializer.Serialize(condition);

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;

        Assert.Equal("score", root.GetProperty("key").GetString());
        var range = root.GetProperty("range");
        Assert.Equal(0.5, range.GetProperty("gte").GetDouble());
        Assert.Equal(0.9, range.GetProperty("lte").GetDouble());
    }

    [Fact]
    public void MustOnly_omits_should_and_mustnot()
    {
        var filter = Filter.Must(Filter.Eq("topic", "index"));
        var json = JsonSerializer.Serialize(filter);

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;

        Assert.True(root.TryGetProperty("must", out _));
        Assert.False(root.TryGetProperty("should", out _));
        Assert.False(root.TryGetProperty("must_not", out _));
    }

    [Fact]
    public void Compound_must_and_mustnot()
    {
        var filter = new QdrantFilter
        {
            Must = new[] { Filter.Eq("tier", "hot") },
            MustNot = new[] { Filter.Eq("archived", true) }
        };
        var json = JsonSerializer.Serialize(filter);

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;

        Assert.True(root.TryGetProperty("must", out var must));
        Assert.Equal(1, must.GetArrayLength());
        Assert.True(root.TryGetProperty("must_not", out var mustNot));
        Assert.Equal(1, mustNot.GetArrayLength());
        Assert.False(root.TryGetProperty("should", out _));
    }

    [Fact]
    public void Nested_filter_round_trip()
    {
        var inner = Filter.Must(Filter.Eq("inner_key", "value"));
        var outer = Filter.Must(Filter.Nested(inner));
        var json = JsonSerializer.Serialize(outer);

        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;

        var nestedCond = root.GetProperty("must")[0];
        Assert.Equal("__nested__", nestedCond.GetProperty("key").GetString());
        var nestedFilter = nestedCond.GetProperty("filter");
        Assert.Equal("inner_key", nestedFilter.GetProperty("must")[0].GetProperty("key").GetString());
    }

    // -------------------------------------------------------------------------
    // IsEmpty tests
    // -------------------------------------------------------------------------

    [Fact]
    public void IsEmpty_true_when_no_conditions()
    {
        Assert.True(new QdrantFilter().IsEmpty());
        Assert.True(new QdrantFilter { Must = null, Should = null, MustNot = null }.IsEmpty());
        Assert.True(new QdrantFilter { Must = Array.Empty<FilterCondition>() }.IsEmpty());
    }

    [Fact]
    public void IsEmpty_false_when_must_populated()
    {
        var filter = Filter.Must(Filter.Eq("key", "val"));
        Assert.False(filter.IsEmpty());
    }

    // -------------------------------------------------------------------------
    // Client-side validation tests
    // -------------------------------------------------------------------------

    [Fact]
    public async Task DeleteByFilterAsync_typed_empty_throws()
    {
        // FakeHandler must NOT be invoked — ArgumentException fires before HTTP
        var (client, handler) = CreateClient(_ =>
        {
            Assert.Fail("HTTP should not be called for an empty typed filter");
            return new HttpResponseMessage(HttpStatusCode.OK);
        });

        var emptyFilter = Filter.Must(); // zero conditions → IsEmpty() == true

        await Assert.ThrowsAsync<ArgumentException>(
            () => client.DeleteByFilterAsync("col", emptyFilter));

        Assert.Empty(handler.Requests);
    }

    [Fact]
    public async Task DeleteByFilterAsync_typed_happy_path()
    {
        var fakeResponse = new
        {
            scanned = 5,
            matched = 2,
            deleted = 2,
            results = Array.Empty<object>()
        };

        string? capturedBody = null;
        var (client, handler) = CreateClient(req =>
        {
            capturedBody = req.Content!.ReadAsStringAsync().GetAwaiter().GetResult();
            return JsonOk(fakeResponse);
        });

        var filter = Filter.Must(Filter.Eq("topic", "index"));
        var report = await client.DeleteByFilterAsync("my-col", filter);

        // Verify exactly one request reached the handler
        var req = Assert.Single(handler.Requests);
        Assert.Equal(HttpMethod.Post, req.Method);
        Assert.Equal(
            "/collections/my-col/vectors/delete_by_filter",
            req.RequestUri!.AbsolutePath);

        // Verify the serialised body has the expected Qdrant filter shape
        Assert.NotNull(capturedBody);
        using var doc = JsonDocument.Parse(capturedBody!);
        var filterNode = doc.RootElement.GetProperty("filter");
        Assert.True(filterNode.TryGetProperty("must", out var must));
        Assert.Equal("topic", must[0].GetProperty("key").GetString());
        Assert.Equal("index", must[0].GetProperty("match").GetProperty("value").GetString());

        // Verify the deserialised response fields
        Assert.Equal(5, report.Scanned);
        Assert.Equal(2, report.Matched);
        Assert.Equal(2, report.Deleted);
    }
}
