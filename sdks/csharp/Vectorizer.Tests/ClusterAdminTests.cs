using System.Net;
using System.Text;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

/// <summary>
/// Unit tests for the cluster-admin and auth-admin surface of
/// <see cref="VectorizerClient"/>.
/// Uses an in-process <see cref="FakeHandler"/> so no live server is required.
/// </summary>
public class ClusterAdminTests
{
    // -------------------------------------------------------------------------
    // Inline test helpers (FakeHandler / CreateClient / JsonOk)
    // -------------------------------------------------------------------------

    /// <summary>
    /// Captures the last request sent so tests can inspect method, URI, and body.
    /// </summary>
    private sealed class FakeHandler : HttpMessageHandler
    {
        private readonly Func<HttpRequestMessage, HttpResponseMessage> _respond;

        public HttpRequestMessage? LastRequest { get; private set; }

        public FakeHandler(HttpResponseMessage response)
            : this(_ => response) { }

        public FakeHandler(Func<HttpRequestMessage, HttpResponseMessage> respond)
        {
            _respond = respond;
        }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            LastRequest = request;
            return Task.FromResult(_respond(request));
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

    private static HttpResponseMessage JsonOk(string json)
    {
        return new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        };
    }

    // -------------------------------------------------------------------------
    // ClusterFailoverAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ClusterFailoverAsync_ReturnsParsedFailoverReport()
    {
        const string json = """
            {
              "promoted_replica_id": "r-1",
              "master_offset_at_promotion": 1000,
              "replica_offset_at_promotion": 998,
              "residual_lag_operations": 2
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var report = await client.ClusterFailoverAsync("r-1");

        Assert.Equal("r-1", report.PromotedReplicaId);
        Assert.Equal(1000, report.MasterOffsetAtPromotion);
        Assert.Equal(2, report.ResidualLagOperations);
    }

    // -------------------------------------------------------------------------
    // ClusterResyncReplicaAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ClusterResyncReplicaAsync_ReturnsParsedResyncJob()
    {
        const string json = """
            {
              "replica_id": "r-2",
              "snapshot_offset": 500,
              "full_snapshot": true
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var job = await client.ClusterResyncReplicaAsync("r-2");

        Assert.Equal("r-2", job.ReplicaId);
        Assert.Equal(500, job.SnapshotOffset);
        Assert.True(job.FullSnapshot);
    }

    // -------------------------------------------------------------------------
    // ClusterAddPeerAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ClusterAddPeerAsync_ReturnsParsedPeerInfo()
    {
        const string json = """
            {
              "node_id": "node-42",
              "address": "10.0.0.5:15002",
              "role": "replica"
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var peer = await client.ClusterAddPeerAsync(
            new AddPeerRequest { Address = "10.0.0.5:15002", Role = "replica" });

        Assert.Equal("node-42", peer.NodeId);
        Assert.Equal("replica", peer.Role);
    }

    // -------------------------------------------------------------------------
    // ClusterRebalanceAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ClusterRebalanceAsync_ReturnsParsedRebalanceJob()
    {
        const string json = """
            {
              "job_id": "job-99",
              "status": "running",
              "shards_to_move": 4,
              "shards_moved": 0,
              "message": "rebalance started"
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var job = await client.ClusterRebalanceAsync();

        Assert.Equal("job-99", job.JobId);
        Assert.Equal("running", job.Status);
        Assert.Equal(4, job.ShardsToMove);
    }

    // -------------------------------------------------------------------------
    // ClusterRebalanceStatusAsync — active
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ClusterRebalanceStatus_Active_ReturnsNonNullJob()
    {
        // Server returns a real job descriptor — client must return a populated object.
        const string json = """
            {
              "job_id": "job-77",
              "status": "running",
              "shards_to_move": 3,
              "shards_moved": 1,
              "message": "in progress"
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var job = await client.ClusterRebalanceStatusAsync();

        Assert.NotNull(job);
        Assert.Equal("job-77", job!.JobId);
        Assert.Equal("running", job.Status);
        Assert.Equal(3, job.ShardsToMove);
        Assert.Equal(1, job.ShardsMoved);
    }

    // -------------------------------------------------------------------------
    // ClusterRebalanceStatusAsync — idle sentinel
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ClusterRebalanceStatus_Idle_ReturnsNull()
    {
        // Server returns the idle sentinel — client must return null.
        const string json = """{"status":"idle"}""";

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var job = await client.ClusterRebalanceStatusAsync();

        Assert.Null(job);
    }

    // -------------------------------------------------------------------------
    // RotateApiKeyAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task RotateApiKeyAsync_ReturnsParsedRotatedKey()
    {
        const string json = """
            {
              "old_key_id": "key-old",
              "new_key_id": "key-new",
              "new_token": "tok-abc123",
              "grace_until": 1746057600
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var rotated = await client.RotateApiKeyAsync("key-old");

        Assert.Equal("key-old", rotated.OldKeyId);
        Assert.Equal("key-new", rotated.NewKeyId);
        Assert.Equal("tok-abc123", rotated.NewToken);
        Assert.Equal(1746057600, rotated.GraceUntil);
    }

    // -------------------------------------------------------------------------
    // CreateScopedApiKeyAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task CreateScopedApiKeyAsync_ReturnsParsedApiKey()
    {
        const string json = """
            {
              "id": "k-1",
              "name": "scoped-key",
              "permissions": ["read"],
              "api_key": "raw-token-value",
              "created_at": 1746000000,
              "active": true,
              "usage_count": 0
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var key = await client.CreateScopedApiKeyAsync(
            new CreateScopedApiKeyRequest
            {
                Name = "scoped-key",
                Permissions = new List<string> { "read" }
            });

        Assert.Equal("k-1", key.Id);
        Assert.Equal("raw-token-value", key.ApiKeyValue);
        Assert.True(key.Active);
    }

    // -------------------------------------------------------------------------
    // IntrospectTokenAsync
    // -------------------------------------------------------------------------

    [Fact]
    public async Task IntrospectTokenAsync_ReturnsParsedIntrospection()
    {
        const string json = """
            {
              "active": true,
              "scope": "read write",
              "sub": "user-7",
              "exp": 9999999999,
              "username": "alice"
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var result = await client.IntrospectTokenAsync("some-opaque-token");

        Assert.True(result.Active);
        Assert.Equal("user-7", result.Sub);
        Assert.Equal("alice", result.Username);
    }

    // -------------------------------------------------------------------------
    // ListAuditLogAsync — verifies query params (skip empty)
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ListAuditLogAsync_VerifiesQueryParams_SkipsEmptyFields()
    {
        const string json = """
            {
              "entries": [
                {
                  "actor": "alice",
                  "action": "create_key",
                  "target": "k-1",
                  "at": "2026-05-03T00:00:00Z"
                }
              ]
            }
            """;

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        // Only actor and limit are set; action/since/until are omitted.
        var entries = await client.ListAuditLogAsync(new AuditQuery
        {
            Actor = "alice",
            Limit = 50
        });

        Assert.NotNull(handler.LastRequest);
        var uri = handler.LastRequest!.RequestUri!.ToString();

        // actor and limit must appear in the query string.
        Assert.Contains("actor=alice", uri);
        Assert.Contains("limit=50", uri);

        // action / since / until must NOT appear because they were not set.
        Assert.DoesNotContain("action=", uri);
        Assert.DoesNotContain("since=", uri);
        Assert.DoesNotContain("until=", uri);

        Assert.Single(entries);
        Assert.Equal("alice", entries[0].Actor);
        Assert.Equal("create_key", entries[0].Action);
    }

    // -------------------------------------------------------------------------
    // ListAuditLogAsync — empty entries list
    // -------------------------------------------------------------------------

    [Fact]
    public async Task ListAuditLogAsync_NoQuery_ReturnsEmptyListWhenEntriesNull()
    {
        // Server returns envelope with null entries field.
        const string json = """{"entries":null}""";

        var handler = new FakeHandler(JsonOk(json));
        using var client = CreateClient(handler);

        var entries = await client.ListAuditLogAsync(new AuditQuery());

        Assert.NotNull(entries);
        Assert.Empty(entries);
    }
}
