using Vectorizer;
using Vectorizer.Models;
using Vectorizer.Exceptions;

namespace ExampleTest;

class Program
{
    static async Task Main(string[] args)
    {
        Console.WriteLine("🔷 Vectorizer C# SDK Test");
        Console.WriteLine("=========================");

        // Test Master/Replica configuration
        Console.WriteLine("\n=== Testing Master/Replica Topology ===\n");

        // Create client with master/replica configuration
        var replicaClient = new VectorizerClient(new ClientConfig
        {
            Hosts = new HostConfig
            {
                Master = "http://localhost:15002",
                Replicas = new List<string> { "http://localhost:17780" }
            },
            ReadPreference = ReadPreference.Replica,
            ApiKey = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiYWRtaW4iLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZXMiOlsiQWRtaW4iXSwiaWF0IjoxNzY0Mzk0MzI3LCJleHAiOjE3NjQzOTc5Mjd9.AnLPTgdHRfCFMdMp6VFemhcIZPUfpzjwB5r6xOkCxNQ"
        });

        Console.WriteLine("1. Testing health check on replica...");
        try
        {
            await replicaClient.HealthAsync();
            Console.WriteLine("   ✅ Replica health: OK");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"   ❌ Replica health failed: {ex.Message}");
        }

        Console.WriteLine("2. Testing WithMaster()...");
        try
        {
            var masterClient = replicaClient.WithMaster();
            await masterClient.HealthAsync();
            Console.WriteLine("   ✅ Master health: OK");
            masterClient.Dispose();
        }
        catch (Exception ex)
        {
            Console.WriteLine($"   ❌ WithMaster failed: {ex.Message}");
        }

        replicaClient.Dispose();
        Console.WriteLine("\n=== Master/Replica Test Complete ===\n");

        // Create client for the rest of tests
        var client = new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://localhost:15002",
            ApiKey = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiYWRtaW4iLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZXMiOlsiQWRtaW4iXSwiaWF0IjoxNzY0Mzk0MzI3LCJleHAiOjE3NjQzOTc5Mjd9.AnLPTgdHRfCFMdMp6VFemhcIZPUfpzjwB5r6xOkCxNQ"
        });
        Console.WriteLine("✅ Client created successfully");

        var collectionName = "test-collection-" + Guid.NewGuid().ToString("N")[..8];

        try
        {
            // Health check
            Console.WriteLine("\n🔍 Checking server health...");
            try
            {
                await client.HealthAsync();
                Console.WriteLine("✅ Server is healthy");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Health check failed: {ex.Message}");
                return;
            }

            // Get database stats
            Console.WriteLine("\n📊 Getting database statistics...");
            try
            {
                var stats = await client.GetStatsAsync();
                Console.WriteLine("📈 Database stats:");
                Console.WriteLine($"   Collections: {stats.Collections}");
                Console.WriteLine($"   Vectors: {stats.Vectors}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Get stats failed: {ex.Message}");
            }

            // List existing collections
            Console.WriteLine("\n📋 Listing collections...");
            try
            {
                var collections = await client.ListCollectionsAsync();
                Console.WriteLine($"📁 Found {collections.Count} collections");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Error listing collections: {ex.Message}");
            }

            // Create a new collection
            Console.WriteLine("\n🆕 Creating collection...");
            try
            {
                var collection = await client.CreateCollectionAsync(new CreateCollectionRequest
                {
                    Name = collectionName,
                    Config = new CollectionConfig
                    {
                        Dimension = 384,
                        Metric = DistanceMetric.Cosine
                    }
                });
                Console.WriteLine($"✅ Collection created: {collection.Name}");
                if (collection.Config != null)
                {
                    Console.WriteLine($"   Dimension: {collection.Config.Dimension}");
                    Console.WriteLine($"   Metric: {collection.Config.Metric}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Collection creation failed: {ex.Message}");
                return;
            }

            // Insert text
            Console.WriteLine("\n📥 Inserting text...");
            try
            {
                var result = await client.InsertTextAsync(
                    collectionName,
                    "Machine learning algorithms",
                    new Dictionary<string, object> { { "source", "test" } }
                );
                Console.WriteLine($"✅ Text inserted (ID: {result.Id})");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Insert text failed: {ex.Message}");
            }

            // Wait a bit for indexing
            await Task.Delay(1000);

            // Search
            Console.WriteLine("\n🔍 Searching...");
            try
            {
                var results = await client.SearchTextAsync(collectionName, "machine learning", new SearchOptions
                {
                    Limit = 5
                });
                Console.WriteLine($"🎯 Found {results.Count} results");
                foreach (var result in results.Take(3))
                {
                    Console.WriteLine($"   - Score: {result.Score:F4}, ID: {result.Id}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Search failed: {ex.Message}");
            }

            // Test Embedding
            Console.WriteLine("\n🧠 Testing embedding...");
            try
            {
                var embedding = await client.EmbedTextAsync(new EmbeddingRequest
                {
                    Text = "test embedding",
                    Model = null
                });
                Console.WriteLine($"✅ Embedding generated: {embedding.Embedding.Length} dimensions, Model: {embedding.Model}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Embedding failed: {ex.Message}");
            }

            // Test Intelligent Search
            Console.WriteLine("\n🔍 Testing intelligent search...");
            try
            {
                var intResults = await client.IntelligentSearchAsync(new IntelligentSearchRequest
                {
                    Query = "machine learning",
                    Collections = new List<string> { collectionName },
                    MaxResults = 3
                });
                Console.WriteLine($"✅ Intelligent search found {intResults.Count} results");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Intelligent search failed: {ex.Message}");
            }

            // Test Semantic Search
            Console.WriteLine("\n🔍 Testing semantic search...");
            try
            {
                var semResults = await client.SemanticSearchAsync(new SemanticSearchRequest
                {
                    Query = "machine learning",
                    Collection = collectionName,
                    MaxResults = 3
                });
                Console.WriteLine($"✅ Semantic search found {semResults.Results.Count} results");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Semantic search failed: {ex.Message}");
            }

            // Test Batch Insert
            Console.WriteLine("\n📦 Testing batch insert...");
            try
            {
                var batchResult = await client.BatchInsertTextsAsync(collectionName, new BatchInsertRequest
                {
                    Texts = new List<BatchTextRequest>
                    {
                        new BatchTextRequest { Id = "batch1", Text = "Batch text 1", Metadata = new Dictionary<string, object> { { "type", "batch" } } },
                        new BatchTextRequest { Id = "batch2", Text = "Batch text 2", Metadata = new Dictionary<string, object> { { "type", "batch" } } }
                    }
                });
                Console.WriteLine($"✅ Batch insert: {batchResult.SuccessfulOperations} successful, {batchResult.FailedOperations} failed");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Batch insert failed: {ex.Message}");
            }

            // Get collection info
            Console.WriteLine("\n📊 Getting collection information...");
            try
            {
                var info = await client.GetCollectionInfoAsync(collectionName);
                Console.WriteLine("📈 Collection info:");
                Console.WriteLine($"   Name: {info.Name}");
                Console.WriteLine($"   Dimension: {info.Dimension}");
                Console.WriteLine($"   Vector count: {info.VectorCount}");
                Console.WriteLine($"   Metric: {info.Metric}");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Get collection info failed: {ex.Message}");
            }

            Console.WriteLine("\n🌐 All tests completed!");

            // Clean up
            Console.WriteLine("\n🧹 Cleaning up...");
            try
            {
                await client.DeleteCollectionAsync(collectionName);
                Console.WriteLine("✅ Collection deleted");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Delete collection failed: {ex.Message}");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"❌ Error: {ex.Message}");
            if (ex is VectorizerException vex)
            {
                Console.WriteLine($"   Error Type: {vex.ErrorType}");
            }
            Console.WriteLine($"   Stack Trace: {ex.StackTrace}");
        }
        finally
        {
            client.Dispose();
            Console.WriteLine("\n👋 Client closed");
        }

        Console.WriteLine("\n✅ Test completed!");
    }
}
