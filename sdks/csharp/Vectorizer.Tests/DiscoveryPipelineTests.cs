using System.Net;
using System.Text;
using System.Text.Json;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

public class DiscoveryPipelineTests
{
    // ── test helpers ────────────────────────────────────────────────────────

    private sealed class FakeHandler : HttpMessageHandler
    {
        public HttpRequestMessage? LastRequest { get; private set; }
        private readonly HttpResponseMessage _response;

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

    private static (VectorizerClient client, FakeHandler handler) CreateClient(object responseBody)
    {
        var handler = new FakeHandler(JsonOk(responseBody));
        var httpClient = new HttpClient(handler);
        var client = new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://fake",
            HttpClient = httpClient
        });
        return (client, handler);
    }

    private static HttpResponseMessage JsonOk(object body)
    {
        var json = JsonSerializer.Serialize(body);
        return new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        };
    }

    private static async Task<T> ReadRequestBody<T>(HttpRequestMessage request)
    {
        var json = await request.Content!.ReadAsStringAsync();
        return JsonSerializer.Deserialize<T>(json,
            new JsonSerializerOptions { PropertyNameCaseInsensitive = true })!;
    }

    // ── tests ────────────────────────────────────────────────────────────────

    [Fact]
    public async Task BroadDiscoveryAsync_PostsToCorrectPath_AndReturnsResponse()
    {
        var mockResponse = new { chunks = new[] { new { text = "result1" } }, count = 1 };
        var (client, handler) = CreateClient(mockResponse);

        var request = new BroadDiscoveryRequest
        {
            Queries = new List<string> { "find authentication" },
            K = 10
        };

        var result = await client.BroadDiscoveryAsync(request);

        Assert.NotNull(handler.LastRequest);
        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal("/discovery/broad_discovery", handler.LastRequest.RequestUri!.AbsolutePath);

        var sent = await ReadRequestBody<BroadDiscoveryRequest>(handler.LastRequest);
        Assert.Contains("find authentication", sent.Queries);

        Assert.Equal(1, result.Count);
    }

    [Fact]
    public async Task SemanticFocusAsync_PostsToCorrectPath_AndReturnsResponse()
    {
        var mockResponse = new { chunks = new[] { new { text = "focused result" } }, count = 1 };
        var (client, handler) = CreateClient(mockResponse);

        var request = new SemanticFocusRequest
        {
            Collection = "docs",
            Queries = new List<string> { "semantic query" },
            K = 5
        };

        var result = await client.SemanticFocusAsync(request);

        Assert.NotNull(handler.LastRequest);
        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal("/discovery/semantic_focus", handler.LastRequest.RequestUri!.AbsolutePath);

        var sent = await ReadRequestBody<SemanticFocusRequest>(handler.LastRequest);
        Assert.Equal("docs", sent.Collection);

        Assert.Equal(1, result.Count);
    }

    [Fact]
    public async Task PromoteReadmeAsync_PostsToCorrectPath_AndReturnsResponse()
    {
        var mockResponse = new
        {
            promoted_chunks = new[] { new { text = "README content" } },
            count = 1
        };
        var (client, handler) = CreateClient(mockResponse);

        var request = new PromoteReadmeRequest
        {
            Chunks = new List<Dictionary<string, object>>
            {
                new() { ["text"] = "some chunk" }
            }
        };

        var result = await client.PromoteReadmeAsync(request);

        Assert.NotNull(handler.LastRequest);
        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal("/discovery/promote_readme", handler.LastRequest.RequestUri!.AbsolutePath);

        Assert.Equal(1, result.Count);
        Assert.Single(result.PromotedChunks);
    }

    [Fact]
    public async Task CompressEvidenceAsync_PostsToCorrectPath_AndReturnsResponse()
    {
        var mockResponse = new
        {
            bullets = new[] { new { text = "• bullet point" } },
            count = 1
        };
        var (client, handler) = CreateClient(mockResponse);

        var request = new CompressEvidenceRequest
        {
            Chunks = new List<Dictionary<string, object>>
            {
                new() { ["text"] = "long chunk" }
            },
            MaxBullets = 5,
            MaxPerDoc = 2
        };

        var result = await client.CompressEvidenceAsync(request);

        Assert.NotNull(handler.LastRequest);
        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal("/discovery/compress_evidence", handler.LastRequest.RequestUri!.AbsolutePath);

        var sent = await ReadRequestBody<CompressEvidenceRequest>(handler.LastRequest);
        Assert.Equal(5, sent.MaxBullets);

        Assert.Equal(1, result.Count);
        Assert.Single(result.Bullets);
    }

    [Fact]
    public async Task BuildAnswerPlanAsync_PostsToCorrectPath_AndReturnsResponse()
    {
        var mockResponse = new
        {
            sections = new[] { new { title = "Overview" } },
            total_bullets = 3,
            sources = new[] { "doc1.md" }
        };
        var (client, handler) = CreateClient(mockResponse);

        var request = new AnswerPlanRequest
        {
            Bullets = new List<Dictionary<string, object>>
            {
                new() { ["text"] = "bullet one" },
                new() { ["text"] = "bullet two" },
                new() { ["text"] = "bullet three" }
            }
        };

        var result = await client.BuildAnswerPlanAsync(request);

        Assert.NotNull(handler.LastRequest);
        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal("/discovery/build_answer_plan", handler.LastRequest.RequestUri!.AbsolutePath);

        var sent = await ReadRequestBody<AnswerPlanRequest>(handler.LastRequest);
        Assert.Equal(3, sent.Bullets.Count);

        Assert.Equal(3, result.TotalBullets);
        Assert.Single(result.Sources);
        Assert.Equal("doc1.md", result.Sources[0]);
    }

    [Fact]
    public async Task RenderLlmPromptAsync_PostsToCorrectPath_AndReturnsResponse()
    {
        var mockResponse = new
        {
            prompt = "You are an assistant...",
            length = 24,
            estimated_tokens = 6
        };
        var (client, handler) = CreateClient(mockResponse);

        var request = new RenderPromptRequest
        {
            Plan = new AnswerPlan
            {
                TotalBullets = 2,
                Sources = new List<string> { "readme.md" },
                Sections = new List<Dictionary<string, object>>()
            }
        };

        var result = await client.RenderLlmPromptAsync(request);

        Assert.NotNull(handler.LastRequest);
        Assert.Equal(HttpMethod.Post, handler.LastRequest!.Method);
        Assert.Equal("/discovery/render_llm_prompt", handler.LastRequest.RequestUri!.AbsolutePath);

        var sent = await ReadRequestBody<RenderPromptRequest>(handler.LastRequest);
        Assert.Equal(2, sent.Plan.TotalBullets);

        Assert.Equal("You are an assistant...", result.Prompt);
        Assert.Equal(24, result.Length);
        Assert.Equal(6, result.EstimatedTokens);
    }
}
