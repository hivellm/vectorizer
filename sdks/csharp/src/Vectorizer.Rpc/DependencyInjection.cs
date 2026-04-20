using System;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;

namespace Vectorizer.Rpc;

/// <summary>
/// ASP.NET Core DI extension methods for registering a shared
/// <see cref="IVectorizerClient"/>. Route every registration through
/// <see cref="VectorizerClientFactory"/> so the URL contract stays
/// identical to <c>new VectorizerClient(string url)</c>.
/// </summary>
public static class VectorizerClientServiceCollectionExtensions
{
    /// <summary>
    /// Registers <see cref="IVectorizerClient"/> as a singleton backed by
    /// the RPC fast path by default.
    /// </summary>
    /// <param name="services">DI container.</param>
    /// <param name="url">Connection URL (same grammar as the factory).</param>
    /// <returns>The same <paramref name="services"/> for chaining.</returns>
    public static IServiceCollection AddVectorizerClient(this IServiceCollection services, string url)
    {
        ArgumentNullException.ThrowIfNull(services);
        ArgumentNullException.ThrowIfNull(url);
        return services.AddVectorizerClient(opts => opts.Url = url);
    }

    /// <summary>
    /// Registers <see cref="IVectorizerClient"/> as a singleton with the
    /// supplied configuration action.
    /// </summary>
    public static IServiceCollection AddVectorizerClient(
        this IServiceCollection services,
        Action<VectorizerClientOptions> configure)
    {
        ArgumentNullException.ThrowIfNull(services);
        ArgumentNullException.ThrowIfNull(configure);

        services.Configure(configure);
        services.TryAddSingleton<IVectorizerClient>(sp =>
        {
            var options = ResolveOptions(sp);
            return VectorizerClientFactory.Create(options);
        });
        return services;
    }

    private static VectorizerClientOptions ResolveOptions(IServiceProvider sp)
    {
        var snapshot = sp.GetService(typeof(Microsoft.Extensions.Options.IOptions<VectorizerClientOptions>))
            as Microsoft.Extensions.Options.IOptions<VectorizerClientOptions>;
        return snapshot?.Value ?? new VectorizerClientOptions();
    }
}
