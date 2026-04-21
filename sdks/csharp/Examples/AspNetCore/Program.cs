using Vectorizer.Rpc;

// Minimal-API sample showing services.AddVectorizerClient(...) under
// ASP.NET Core. Run with `VECTORIZER_URL=vectorizer://localhost:15503 dotnet run`.

var builder = WebApplication.CreateBuilder(args);

var vectorizerUrl = builder.Configuration["Vectorizer:Url"]
    ?? $"vectorizer://localhost:{EndpointParser.DefaultRpcPort}";

builder.Services.AddVectorizerClient(options =>
{
    options.Url = vectorizerUrl;
    options.Hello = new HelloPayload
    {
        ClientName = $"vectorizer-aspnetcore/{builder.Environment.EnvironmentName}",
        ApiKey = builder.Configuration["Vectorizer:ApiKey"],
    };
});

var app = builder.Build();

app.MapGet("/health", async (IVectorizerClient client) =>
    new { transport = client.Transport.ToString(), pong = await client.PingAsync() });

app.MapGet("/collections", async (IVectorizerClient client) =>
    await client.ListCollectionsAsync());

app.MapGet("/search/{collection}", async (string collection, string q, int? limit, IVectorizerClient client) =>
    await client.SearchBasicAsync(collection, q, limit ?? 10));

app.Run();
