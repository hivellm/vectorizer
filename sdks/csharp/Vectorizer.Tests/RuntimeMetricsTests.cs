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

namespace Vectorizer.Tests;

// Phase25 §7 — RuntimeMetrics + extended Stats / CollectionInfo wire shape.

public class RuntimeMetricsTests
{
    private sealed class FakeHandler : HttpMessageHandler
    {
        public System.Func<HttpRequestMessage, HttpResponseMessage> Responder { get; set; } =
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
        System.Func<HttpRequestMessage, HttpResponseMessage> responder)
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

    [Fact]
    public async Task GetRuntimeMetricsAsync_TargetsMetricsRuntimeRoute()
    {
        var (client, handler) = CreateClient(req =>
        {
            Assert.Equal(HttpMethod.Get, req.Method);
            Assert.Equal("/metrics/runtime", req.RequestUri!.AbsolutePath);
            return JsonOk(new { });
        });

        await client.GetRuntimeMetricsAsync();
        Assert.Single(handler.Requests);
    }

    [Fact]
    public async Task GetRuntimeMetricsAsync_DecodesFullSnapshot()
    {
        var (client, _) = CreateClient(_ => JsonOk(new
        {
            cpu_percent = 12.4,
            memory_rss_bytes = 124857600UL,
            memory_total_bytes = 17179869184UL,
            memory_percent = 0.73,
            active_connections = 8,
            uptime_seconds = 3712UL,
            qps_window_60s = 142.3,
            error_rate_5xx_60s = 0.001,
            throughput_by_route = new[]
            {
                new { route = "/insert_texts", qps = 12.0, p50_ms = 8.2, p99_ms = 41.0 }
            },
            wal = new
            {
                current_seq = 482919UL,
                size_bytes = 12582912UL,
                last_checkpoint_at = 1714828800UL,
                last_checkpoint_seq = 482800UL,
            }
        }));

        var m = await client.GetRuntimeMetricsAsync();

        Assert.Equal(12.4, m.CpuPercent);
        Assert.Equal(8, m.ActiveConnections);
        Assert.Single(m.ThroughputByRoute);
        Assert.Equal("/insert_texts", m.ThroughputByRoute[0].Route);
        Assert.Equal(41.0, m.ThroughputByRoute[0].P99Ms);
        Assert.Equal(482919UL, m.Wal.CurrentSeq);
        Assert.Equal(482800UL, m.Wal.LastCheckpointSeq);
    }

    [Fact]
    public async Task GetRuntimeMetricsAsync_TolerantPartialPayload()
    {
        // Older / standalone server: omit routes + wal block.
        var (client, _) = CreateClient(_ => JsonOk(new
        {
            cpu_percent = 1.0,
            memory_total_bytes = 8000000000UL,
        }));

        var m = await client.GetRuntimeMetricsAsync();

        Assert.Equal(1.0, m.CpuPercent);
        Assert.Empty(m.ThroughputByRoute);
        Assert.Equal(0UL, m.Wal.CurrentSeq);
        Assert.Equal(0UL, m.Wal.LastCheckpointSeq);
    }

    [Fact]
    public void Stats_DecodesPhase25QuantizationFields()
    {
        const string raw = """
        {
          "collections": 3,
          "total_vectors": 12000,
          "uptime_seconds": 60,
          "version": "3.4.0",
          "default_quantization": "sq-8bit",
          "compression_ratio": 4.0
        }
        """;
        var s = JsonSerializer.Deserialize<Stats>(raw)!;
        Assert.Equal("sq-8bit", s.DefaultQuantization);
        Assert.Equal(4.0f, s.CompressionRatio);
    }

    [Fact]
    public void Stats_FallsBackForOlderServers()
    {
        const string raw = """
        {
          "collections": 0,
          "total_vectors": 0,
          "uptime_seconds": 0,
          "version": "3.3.0"
        }
        """;
        var s = JsonSerializer.Deserialize<Stats>(raw)!;
        // Default field initializers fire when the JSON omits the field.
        Assert.Equal("none", s.DefaultQuantization);
        Assert.Equal(1.0f, s.CompressionRatio);
    }

    [Fact]
    public void CollectionInfo_DecodesVectorCountHistory()
    {
        const string raw = """
        {
          "Name": "docs",
          "VectorCount": 482919,
          "Dimension": 768,
          "Metric": "cosine",
          "vector_count_history": [
            {"at": 1714828740, "count": 482900},
            {"at": 1714828800, "count": 482919}
          ]
        }
        """;
        var ci = JsonSerializer.Deserialize<CollectionInfo>(raw)!;
        Assert.Equal(2, ci.VectorCountHistory.Count);
        Assert.Equal(482900, ci.VectorCountHistory[0].Count);
        Assert.Equal(1714828800L, ci.VectorCountHistory[1].At);
    }

    [Fact]
    public void CollectionInfo_OlderServerHasEmptyHistory()
    {
        const string raw = """
        {
          "Name": "older",
          "VectorCount": 0,
          "Dimension": 384,
          "Metric": "cosine"
        }
        """;
        var ci = JsonSerializer.Deserialize<CollectionInfo>(raw)!;
        Assert.Empty(ci.VectorCountHistory);
    }
}
