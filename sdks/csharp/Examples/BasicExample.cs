using Vectorizer;
using Vectorizer.Models;
using Vectorizer.Exceptions;

namespace Vectorizer.Examples;

/// <summary>
/// Basic usage example for the Hive Vectorizer C# SDK.
/// This example demonstrates all core operations available in the SDK.
/// </summary>
public class BasicExample
{
    public static async Task RunAsync()
    {
        Console.WriteLine("🔷 Vectorizer C# SDK Basic Example");
        Console.WriteLine("==================================");

        // Create client
        var client = new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://localhost:15002",
            ApiKey = "your-api-key"
        });
        Console.WriteLine("✅ Client created successfully");

        var collectionName = "example-documents";

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
                Console.WriteLine($"📁 Found {collections.Count} collections:");
                foreach (var name in collections.Take(5))
                {
                    Console.WriteLine($"   - {name}");
                }
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
                Console.WriteLine($"⚠️ Collection creation failed (may already exist): {ex.Message}");
            }

            // Insert texts
            Console.WriteLine("\n📥 Inserting texts...");
            var texts = new[]
            {
                new { Id = "doc_1", Text = "Introduction to Machine Learning", Metadata = new Dictionary<string, object>
                {
                    { "source", "document1.pdf" },
                    { "title", "Introduction to Machine Learning" },
                    { "category", "AI" }
                }},
                new { Id = "doc_2", Text = "Deep Learning Fundamentals", Metadata = new Dictionary<string, object>
                {
                    { "source", "document2.pdf" },
                    { "title", "Deep Learning Fundamentals" },
                    { "category", "AI" }
                }},
                new { Id = "doc_3", Text = "Data Science Best Practices", Metadata = new Dictionary<string, object>
                {
                    { "source", "document3.pdf" },
                    { "title", "Data Science Best Practices" },
                    { "category", "Data" }
                }}
            };

            var inserted = 0;
            foreach (var text in texts)
            {
                try
                {
                    var result = await client.InsertTextAsync(collectionName, text.Text, text.Metadata);
                    Console.WriteLine($"✅ Inserted text: {text.Id} (ID: {result.Id})");
                    inserted++;
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"⚠️ Insert text failed for {text.Id}: {ex.Message}");
                }
            }
            Console.WriteLine($"✅ Total texts inserted: {inserted}");

            // Search for similar vectors
            Console.WriteLine("\n🔍 Searching for similar vectors...");
            try
            {
                var results = await client.SearchTextAsync(collectionName, "machine learning algorithms", new SearchOptions
                {
                    Limit = 3
                });
                Console.WriteLine("🎯 Search results:");
                for (int i = 0; i < results.Count; i++)
                {
                    var result = results[i];
                    Console.WriteLine($"   {i + 1}. Score: {result.Score:F4}");
                    if (result.Payload != null)
                    {
                        if (result.Payload.TryGetValue("title", out var title))
                        {
                            Console.WriteLine($"      Title: {title}");
                        }
                        if (result.Payload.TryGetValue("category", out var category))
                        {
                            Console.WriteLine($"      Category: {category}");
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"⚠️ Search failed: {ex.Message}");
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

            Console.WriteLine("\n🌐 All operations completed successfully!");

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
        }
        finally
        {
            client.Dispose();
            Console.WriteLine("\n👋 Client closed");
        }

        Console.WriteLine("\n👋 Example completed!");
    }
}
