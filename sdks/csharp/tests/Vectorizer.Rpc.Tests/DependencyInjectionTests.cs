using System;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;
using Vectorizer.Rpc;
using Xunit;

namespace Vectorizer.Rpc.Tests;

public class DependencyInjectionTests
{
    [Fact]
    public void AddVectorizerClient_WithRpcUrl_RegistersRpcImpl()
    {
        var services = new ServiceCollection();
        services.AddVectorizerClient("vectorizer://localhost:15503");
        using var provider = services.BuildServiceProvider();

        var client = provider.GetRequiredService<IVectorizerClient>();
        Assert.Equal(EndpointKind.Rpc, client.Transport);
        Assert.IsType<RpcVectorizerClient>(client);
    }

    [Fact]
    public void AddVectorizerClient_WithHttpUrl_RegistersHttpImpl()
    {
        var services = new ServiceCollection();
        services.AddVectorizerClient("http://localhost:15002");
        using var provider = services.BuildServiceProvider();

        var client = provider.GetRequiredService<IVectorizerClient>();
        Assert.Equal(EndpointKind.Rest, client.Transport);
        Assert.IsType<HttpVectorizerClient>(client);
    }

    [Fact]
    public void AddVectorizerClient_ForcedHttpOnRpcUrl_Throws()
    {
        var services = new ServiceCollection();
        services.AddVectorizerClient(opts =>
        {
            opts.Url = "vectorizer://localhost:15503";
            opts.Transport = TransportKind.Http;
        });
        using var provider = services.BuildServiceProvider();

        Assert.Throws<ArgumentException>(() => provider.GetRequiredService<IVectorizerClient>());
    }

    [Fact]
    public void AddVectorizerClient_Resolves_Singleton()
    {
        var services = new ServiceCollection();
        services.AddVectorizerClient("vectorizer://localhost");
        using var provider = services.BuildServiceProvider();

        var a = provider.GetRequiredService<IVectorizerClient>();
        var b = provider.GetRequiredService<IVectorizerClient>();
        Assert.Same(a, b);
    }

    [Fact]
    public async Task ClientFactory_Create_ByUrl_SelectsTransportByScheme()
    {
        await using (var rpc = VectorizerClientFactory.Create("vectorizer://host"))
        {
            Assert.Equal(EndpointKind.Rpc, rpc.Transport);
        }
        await using (var rest = VectorizerClientFactory.Create("http://host"))
        {
            Assert.Equal(EndpointKind.Rest, rest.Transport);
        }
    }
}
