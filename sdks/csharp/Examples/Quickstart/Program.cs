using System;
using System.Threading.Tasks;
using Vectorizer.Rpc;

// Vectorizer.Rpc quickstart — connect over the binary fast path,
// list collections, and run a basic search. Override the connection
// URL with VECTORIZER_URL (e.g. http://localhost:15002 to use REST).

var url = Environment.GetEnvironmentVariable("VECTORIZER_URL")
    ?? $"vectorizer://localhost:{EndpointParser.DefaultRpcPort}";

var apiKey = Environment.GetEnvironmentVariable("VECTORIZER_API_KEY");

await using var client = VectorizerClientFactory.Create(new VectorizerClientOptions
{
    Url = url,
    ApiKey = apiKey,
    Hello = new HelloPayload
    {
        ClientName = "vectorizer-csharp-quickstart/3.0.0",
        ApiKey = apiKey,
    },
});

Console.WriteLine($"Connected via transport: {client.Transport}");
Console.WriteLine($"Server PING → {await client.PingAsync()}");

var collections = await client.ListCollectionsAsync();
Console.WriteLine($"Found {collections.Count} collection(s):");
foreach (var name in collections)
{
    Console.WriteLine($"  - {name}");
}

if (collections.Count == 0)
{
    Console.WriteLine("No collections available; create one to run a search.");
    return;
}

var first = collections[0];
Console.WriteLine($"Running search.basic on '{first}'...");
var hits = await client.SearchBasicAsync(first, query: "hello world", limit: 5);
foreach (var hit in hits)
{
    Console.WriteLine($"  {hit.Id,-48}  score={hit.Score:F4}");
}
