using System;
using System.Collections.Generic;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;
using Vectorizer.Rpc;
using Xunit;

namespace Vectorizer.Rpc.Tests;

/// <summary>
/// Wire-shape tests for the phase 16 RPC typed wrappers.
/// Each test spins up a minimal in-process mock server that returns a
/// predetermined response, then asserts that the typed wrapper decodes
/// every field correctly. No real Vectorizer server is required.
/// </summary>
public class Phase16Tests
{
    // ── helpers ──────────────────────────────────────────────────────────

    private static VectorizerValue HelloMap() =>
        VectorizerValue.OfMap(new[]
        {
            new MapPair(VectorizerValue.OfStr("server_version"), VectorizerValue.OfStr("test")),
            new MapPair(VectorizerValue.OfStr("protocol_version"), VectorizerValue.OfInt(1)),
            new MapPair(VectorizerValue.OfStr("authenticated"), VectorizerValue.OfBool(true)),
            new MapPair(VectorizerValue.OfStr("admin"), VectorizerValue.OfBool(false)),
            new MapPair(VectorizerValue.OfStr("capabilities"),
                VectorizerValue.OfArray(Array.Empty<VectorizerValue>())),
        });

    private static VectorizerValue Map(params (string, VectorizerValue)[] pairs)
    {
        var mapPairs = new MapPair[pairs.Length];
        for (var i = 0; i < pairs.Length; i++)
            mapPairs[i] = new MapPair(VectorizerValue.OfStr(pairs[i].Item1), pairs[i].Item2);
        return VectorizerValue.OfMap(mapPairs);
    }

    private static async Task<(MockRpcServer server, RpcClient client)> SetupAsync(
        Func<RpcRequest, uint, RpcResponse?> handler)
    {
        var server = await MockRpcServer.StartAsync((req, id) =>
            req.Command == "HELLO"
                ? new RpcResponse(id, RpcResult.Ok(HelloMap()))
                : handler(req, id));

        var client = await RpcClient.ConnectAsync("127.0.0.1", server.Port);
        await client.HelloAsync(new HelloPayload { ClientName = "phase16-test" });
        return (server, client);
    }

    // ── Collections ──────────────────────────────────────────────────────

    [Fact]
    public async Task Collections_CreateCollection_DecodesAllFields()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("collections.create", req.Command);
            var resp = Map(
                ("name", VectorizerValue.OfStr("my-col")),
                ("dimension", VectorizerValue.OfInt(128)),
                ("metric", VectorizerValue.OfStr("cosine")),
                ("success", VectorizerValue.OfBool(true)));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var result = await client.CreateCollectionAsync("my-col");

            Assert.Equal("my-col", result.Name);
            Assert.Equal(128, result.Dimension);
            Assert.Equal("cosine", result.Metric);
            Assert.True(result.Success);
        }
    }

    [Fact]
    public async Task Collections_CleanupEmpty_DecodesRemovedAndDryRun()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("collections.cleanup_empty", req.Command);
            var resp = Map(
                ("removed", VectorizerValue.OfInt(3)),
                ("dry_run", VectorizerValue.OfBool(false)));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var result = await client.CleanupEmptyCollectionsAsync(dryRun: false);

            Assert.Equal(3, result.Removed);
            Assert.False(result.DryRun);
        }
    }

    // ── Vectors ──────────────────────────────────────────────────────────

    [Fact]
    public async Task Vectors_InsertVector_ReturnsIdAndSuccess()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("vectors.insert", req.Command);
            var resp = Map(
                ("id", VectorizerValue.OfStr("vec-1")),
                ("success", VectorizerValue.OfBool(true)));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var result = await client.InsertVectorAsync("col", "vec-1", new[] { 0.1f, 0.2f, 0.3f });

            Assert.Equal("vec-1", result.Id);
            Assert.True(result.Success);
        }
    }

    [Fact]
    public async Task Vectors_BatchInsertTexts_DecodesInsertedAndFailed()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("vectors.batch_insert_texts", req.Command);
            var resp = Map(
                ("inserted", VectorizerValue.OfInt(5)),
                ("failed", VectorizerValue.OfInt(0)),
                ("results", VectorizerValue.OfArray(Array.Empty<VectorizerValue>())));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var items = new VectorizerValue[]
            {
                Map(("text", VectorizerValue.OfStr("hello"))),
                Map(("text", VectorizerValue.OfStr("world"))),
            };
            var result = await client.BatchInsertTextsAsync("col", items);

            Assert.Equal(5, result.Inserted);
            Assert.Equal(0, result.Failed);
        }
    }

    [Fact]
    public async Task Vectors_EmbedText_DecodesEmbeddingAndDimension()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("vectors.embed", req.Command);
            var embValues = new VectorizerValue[]
            {
                VectorizerValue.OfFloat(0.1),
                VectorizerValue.OfFloat(0.2),
                VectorizerValue.OfFloat(0.3),
            };
            var resp = Map(
                ("embedding", VectorizerValue.OfArray(embValues)),
                ("model", VectorizerValue.OfStr("bm25")),
                ("dimension", VectorizerValue.OfInt(3)));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var result = await client.EmbedTextAsync("test sentence");

            Assert.Equal(3, result.Embedding.Count);
            Assert.Equal(0.1, result.Embedding[0], precision: 6);
            Assert.Equal("bm25", result.Model);
            Assert.Equal(3, result.Dimension);
        }
    }

    // ── Search ───────────────────────────────────────────────────────────

    [Fact]
    public async Task Search_SearchByText_DecodesHits()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("search.by_text", req.Command);
            var hit = Map(
                ("id", VectorizerValue.OfStr("doc-1")),
                ("score", VectorizerValue.OfFloat(0.95)));
            var resp = Map(("results", VectorizerValue.OfArray(new[] { hit })));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var hits = await client.SearchByTextAsync("col", "query", 10);

            Assert.Single(hits);
            Assert.Equal("doc-1", hits[0].Id);
            Assert.Equal(0.95, hits[0].Score, precision: 6);
        }
    }

    [Fact]
    public async Task Search_SearchExplain_DecodesTraceFields()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("search.explain", req.Command);
            var trace = Map(
                ("visited_nodes", VectorizerValue.OfInt(42)),
                ("ef_search", VectorizerValue.OfInt(100)),
                ("hnsw_search_ms", VectorizerValue.OfFloat(1.5)),
                ("total_ms", VectorizerValue.OfFloat(2.0)));
            var resp = Map(
                ("hits", VectorizerValue.OfArray(Array.Empty<VectorizerValue>())),
                ("collection", VectorizerValue.OfStr("col")),
                ("k", VectorizerValue.OfInt(10)),
                ("trace", trace));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var request = Map(("k", VectorizerValue.OfInt(10)));
            var result = await client.SearchExplainAsync("col", request);

            Assert.Equal("col", result.Collection);
            Assert.Equal(10, result.K);
            Assert.Equal(42, result.Trace.VisitedNodes);
            Assert.Equal(1.5, result.Trace.HnswSearchMs, precision: 6);
        }
    }

    // ── Graph ────────────────────────────────────────────────────────────

    [Fact]
    public async Task Graph_DiscoverEdges_DecodesAllFields()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("graph.discover_edges", req.Command);
            var resp = Map(
                ("success", VectorizerValue.OfBool(true)),
                ("total_nodes", VectorizerValue.OfInt(100)),
                ("nodes_processed", VectorizerValue.OfInt(100)),
                ("nodes_with_edges", VectorizerValue.OfInt(80)),
                ("total_edges_created", VectorizerValue.OfInt(240)));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var request = Map(("similarity_threshold", VectorizerValue.OfFloat(0.8)));
            var result = await client.GraphDiscoverEdgesAsync("col", request);

            Assert.True(result.Success);
            Assert.Equal(100, result.TotalNodes);
            Assert.Equal(240, result.TotalEdgesCreated);
        }
    }

    [Fact]
    public async Task Graph_DiscoveryStatus_DecodesProgressPercentage()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("graph.discovery_status", req.Command);
            var resp = Map(
                ("total_nodes", VectorizerValue.OfInt(50)),
                ("nodes_with_edges", VectorizerValue.OfInt(25)),
                ("total_edges", VectorizerValue.OfInt(75)),
                ("progress_percentage", VectorizerValue.OfFloat(50.0)));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var result = await client.GraphDiscoveryStatusAsync("col");

            Assert.Equal(50, result.TotalNodes);
            Assert.Equal(50.0, result.ProgressPercentage, precision: 6);
        }
    }

    // ── Admin ────────────────────────────────────────────────────────────

    [Fact]
    public async Task Admin_Stats_DecodesCollectionsAndVectors()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("admin.stats", req.Command);
            var resp = Map(
                ("collections_count", VectorizerValue.OfInt(12)),
                ("total_vectors", VectorizerValue.OfInt(500000)),
                ("version", VectorizerValue.OfStr("3.8.0")));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var result = await client.AdminStatsAsync();

            Assert.Equal(12, result.CollectionsCount);
            Assert.Equal(500000, result.TotalVectors);
            Assert.Equal("3.8.0", result.Version);
        }
    }

    // ── Replication ──────────────────────────────────────────────────────

    [Fact]
    public async Task Replication_Configure_DecodesSuccessRoleMessage()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("replication.configure", req.Command);
            var resp = Map(
                ("success", VectorizerValue.OfBool(true)),
                ("role", VectorizerValue.OfStr("master")),
                ("message", VectorizerValue.OfStr("Replication configured. Server restart required.")));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var request = Map(("role", VectorizerValue.OfStr("master")));
            var result = await client.ReplicationConfigureAsync(request);

            Assert.True(result.Success);
            Assert.Equal("master", result.Role);
            Assert.Contains("restart", result.Message, StringComparison.OrdinalIgnoreCase);
        }
    }

    // ── Cluster ──────────────────────────────────────────────────────────

    [Fact]
    public async Task Cluster_RebalanceStatus_DecodesIdleStatus()
    {
        var (server, client) = await SetupAsync((req, id) =>
        {
            Assert.Equal("cluster.rebalance_status", req.Command);
            var resp = Map(
                ("status", VectorizerValue.OfStr("idle")),
                ("message", VectorizerValue.OfStr("No rebalance has been triggered on this node")));
            return new RpcResponse(id, RpcResult.Ok(resp));
        });

        await using (server)
        await using (client)
        {
            var result = await client.ClusterRebalanceStatusAsync();

            Assert.Equal("idle", result.Status);
            Assert.NotNull(result.Message);
        }
    }
}
