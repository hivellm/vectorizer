using Xunit;
using Vectorizer;

namespace Vectorizer.Tests;

public class QdrantAdvancedTests
{
    private readonly VectorizerClient _client;

    public QdrantAdvancedTests()
    {
        _client = new VectorizerClient("http://localhost:15002");
    }

    [Fact]
    public async Task QdrantListCollectionSnapshots_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantListCollectionSnapshotsAsync("test_collection");
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            // Skip if server not running
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantCreateCollectionSnapshot_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantCreateCollectionSnapshotAsync("test_collection");
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantDeleteCollectionSnapshot_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantDeleteCollectionSnapshotAsync("test_collection", "test_snapshot");
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantRecoverCollectionSnapshot_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantRecoverCollectionSnapshotAsync("test_collection", "snapshots/test.snapshot");
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantListAllSnapshots_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantListAllSnapshotsAsync();
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantCreateFullSnapshot_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantCreateFullSnapshotAsync();
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantListShardKeys_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantListShardKeysAsync("test_collection");
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantCreateShardKey_ShouldReturnResult()
    {
        try
        {
            var shardKey = new Dictionary<string, object> { { "shard_key", "test_key" } };
            var result = await _client.QdrantCreateShardKeyAsync("test_collection", shardKey);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantDeleteShardKey_ShouldReturnResult()
    {
        try
        {
            var shardKey = new Dictionary<string, object> { { "shard_key", "test_key" } };
            var result = await _client.QdrantDeleteShardKeyAsync("test_collection", shardKey);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantGetClusterStatus_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantGetClusterStatusAsync();
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantClusterRecover_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantClusterRecoverAsync();
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantRemovePeer_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantRemovePeerAsync("test_peer_123");
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantListMetadataKeys_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantListMetadataKeysAsync();
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantGetMetadataKey_ShouldReturnResult()
    {
        try
        {
            var result = await _client.QdrantGetMetadataKeyAsync("test_key");
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantUpdateMetadataKey_ShouldReturnResult()
    {
        try
        {
            var value = new Dictionary<string, object> { { "value", "test_value" } };
            var result = await _client.QdrantUpdateMetadataKeyAsync("test_key", value);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantQueryPoints_ShouldReturnResult()
    {
        try
        {
            var request = new Dictionary<string, object>
            {
                { "query", new Dictionary<string, object> { { "vector", new float[384] } } },
                { "limit", 10 }
            };
            var result = await _client.QdrantQueryPointsAsync("test_collection", request);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantBatchQueryPoints_ShouldReturnResult()
    {
        try
        {
            var request = new Dictionary<string, object>
            {
                {
                    "searches", new[]
                    {
                        new Dictionary<string, object>
                        {
                            { "query", new Dictionary<string, object> { { "vector", new float[384] } } },
                            { "limit", 10 }
                        }
                    }
                }
            };
            var result = await _client.QdrantBatchQueryPointsAsync("test_collection", request);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantQueryPointsGroups_ShouldReturnResult()
    {
        try
        {
            var request = new Dictionary<string, object>
            {
                { "query", new Dictionary<string, object> { { "vector", new float[384] } } },
                { "group_by", "category" },
                { "group_size", 3 },
                { "limit", 10 }
            };
            var result = await _client.QdrantQueryPointsGroupsAsync("test_collection", request);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantSearchPointsGroups_ShouldReturnResult()
    {
        try
        {
            var request = new Dictionary<string, object>
            {
                { "vector", new float[384] },
                { "group_by", "category" },
                { "group_size", 3 },
                { "limit", 10 }
            };
            var result = await _client.QdrantSearchPointsGroupsAsync("test_collection", request);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantSearchMatrixPairs_ShouldReturnResult()
    {
        try
        {
            var request = new Dictionary<string, object>
            {
                { "sample", 10 },
                { "limit", 5 }
            };
            var result = await _client.QdrantSearchMatrixPairsAsync("test_collection", request);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }

    [Fact]
    public async Task QdrantSearchMatrixOffsets_ShouldReturnResult()
    {
        try
        {
            var request = new Dictionary<string, object>
            {
                { "sample", 10 },
                { "limit", 5 }
            };
            var result = await _client.QdrantSearchMatrixOffsetsAsync("test_collection", request);
            Assert.NotNull(result);
        }
        catch (Exception ex)
        {
            if (ex.Message.Contains("ECONNREFUSED") || ex.Message.Contains("No connection"))
            {
                return;
            }
            throw;
        }
    }
}

