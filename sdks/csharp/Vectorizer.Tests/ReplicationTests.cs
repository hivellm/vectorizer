using System.Net;
using System.Text;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

/// <summary>
/// Unit tests for the replication surface of <see cref="VectorizerClient"/>.
/// Uses an in-process <see cref="FakeHandler"/> so no live server is required.
/// </summary>
public class ReplicationTests
{
    // -------------------------------------------------------------------------
    // Inline test helpers (FakeHandler / CreateClient / JsonOk)
    // -------------------------------------------------------------------------

    /// <summary>
    /// An <see cref="HttpMessageHandler"/> that returns a pre-configured
    /// <see cref="HttpResponseMessage"/> for every request without hitting
    /// the network.
    /// </summary>
    private sealed class FakeHandler : HttpMessageHandler
    {
        private readonly HttpResponseMessage _response;

        public FakeHandler(HttpResponseMessage response)
        {
            _response = response;
        }

        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            return Task.FromResult(_response);
        }
    }

    /// <summary>
    /// Builds a <see cref="VectorizerClient"/> whose underlying
    /// <see cref="HttpClient"/> is backed by the given handler.
    /// </summary>
    private static VectorizerClient CreateClient(HttpMessageHandler handler)
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

    /// <summary>
    /// Creates a 200 OK <see cref="HttpResponseMessage"/> containing
    /// <paramref name="json"/> as UTF-8 application/json content.
    /// </summary>
    private static HttpResponseMessage JsonOk(string json)
    {
        return new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent(json, Encoding.UTF8, "application/json")
        };
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    [Fact]
    public async Task GetReplicationStatusAsync_ReturnsStatusWithNonEmptyReplicas()
    {
        // Arrange — spec scenario: enabled master with two connected replicas.
        const string json = """
            {
              "role": "master",
              "enabled": true,
              "replicas": [
                {
                  "replica_id": "r-1",
                  "host": "10.0.0.2",
                  "port": 15003,
                  "status": "synced",
                  "last_heartbeat": "2026-05-03T00:00:00Z",
                  "operations_synced": 1024
                },
                {
                  "replica_id": "r-2",
                  "host": "10.0.0.3",
                  "port": 15003,
                  "status": "synced",
                  "last_heartbeat": "2026-05-03T00:00:01Z",
                  "operations_synced": 1020
                }
              ]
            }
            """;

        using var client = CreateClient(new FakeHandler(JsonOk(json)));

        // Act
        var status = await client.GetReplicationStatusAsync();

        // Assert
        Assert.Equal("master", status.Role);
        Assert.True(status.Enabled);
        Assert.NotNull(status.Replicas);
        Assert.True(status.Replicas!.Count > 0, "Replicas slice must be non-empty");
        Assert.Equal("r-1", status.Replicas[0].ReplicaId);
    }

    [Fact]
    public async Task ConfigureReplicationAsync_SendsRequestWithoutThrowing()
    {
        // Arrange — server responds with 200 and empty body.
        using var client = CreateClient(new FakeHandler(JsonOk("{}")));

        var config = new ReplicationConfig
        {
            Role = "master",
            BindAddress = "0.0.0.0:15004",
            HeartbeatInterval = 5000,
            LogSize = 100
        };

        // Act + Assert — must complete without exception.
        await client.ConfigureReplicationAsync(config);
    }

    [Fact]
    public async Task GetReplicationStatsAsync_ReturnsParsedStats()
    {
        // Arrange
        const string json = """
            {
              "role": "master",
              "bytes_sent": 204800,
              "bytes_received": 1024,
              "operations_pending": 3,
              "connected_replicas": 2,
              "master_offset": 500,
              "replica_offset": 497,
              "lag_operations": 3,
              "total_replicated": 1024
            }
            """;

        using var client = CreateClient(new FakeHandler(JsonOk(json)));

        // Act
        var stats = await client.GetReplicationStatsAsync();

        // Assert
        Assert.Equal("master", stats.Role);
        Assert.Equal(204800, stats.BytesSent);
        Assert.Equal(2, stats.ConnectedReplicas);
        Assert.Equal(3, stats.LagOperations);
    }

    [Fact]
    public async Task ListReplicasAsync_UnwrapsEnvelopeAndReturnsList()
    {
        // Arrange — response uses the {replicas: [...]} envelope shape.
        const string json = """
            {
              "replicas": [
                {
                  "replica_id": "r-1",
                  "host": "10.0.0.2",
                  "port": 15003,
                  "status": "synced",
                  "last_heartbeat": "2026-05-03T00:00:00Z",
                  "operations_synced": 512,
                  "offset": 1000,
                  "lag": 2
                }
              ]
            }
            """;

        using var client = CreateClient(new FakeHandler(JsonOk(json)));

        // Act
        var replicas = await client.ListReplicasAsync();

        // Assert
        Assert.NotNull(replicas);
        Assert.Single(replicas);
        Assert.Equal("r-1", replicas[0].ReplicaId);
        Assert.Equal("synced", replicas[0].Status);
        Assert.Equal(1000, replicas[0].Offset);
    }
}
