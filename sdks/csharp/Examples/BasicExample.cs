using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Examples;

public class BasicExample
{
    public static async Task RunAsync()
    {
        // Create client
        var client = new VectorizerClient(new ClientConfig
        {
            BaseUrl = "http://localhost:15002",
            ApiKey = "your-api-key"
        });

        try
        {
            // Check health
            await client.HealthAsync();
            Console.WriteLine("✓ Server is healthy");

            // Create collection
            var collection = await client.CreateCollectionAsync(new CreateCollectionRequest
            {
                Name = "documents",
                Config = new CollectionConfig
                {
                    Dimension = 384,
                    Metric = DistanceMetric.Cosine
                }
            });
            Console.WriteLine($"✓ Created collection: {collection.Name}");

            // Insert text
            var result = await client.InsertTextAsync("documents", "Hello, world!", null);
            Console.WriteLine($"✓ Inserted vector ID: {result.Id}");

            // Search
            var results = await client.SearchTextAsync("documents", "hello", new SearchOptions
            {
                Limit = 10
            });
            Console.WriteLine($"✓ Found {results.Count} results");

            // List collections
            var collections = await client.ListCollectionsAsync();
            Console.WriteLine($"✓ Collections: {string.Join(", ", collections)}");
        }
        catch (Vectorizer.Exceptions.VectorizerException ex)
        {
            Console.WriteLine($"Error: {ex.ErrorType} - {ex.Message}");
        }
        finally
        {
            client.Dispose();
        }
    }
}

