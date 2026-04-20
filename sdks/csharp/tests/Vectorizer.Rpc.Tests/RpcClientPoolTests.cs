using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using Vectorizer.Rpc;
using Xunit;

namespace Vectorizer.Rpc.Tests;

public class RpcClientPoolTests
{
    [Fact]
    public async Task Acquire_ReturnsConnection_HelloIsAutomatic()
    {
        await using var server = await MockRpcServer.StartAsync((req, id) =>
            req.Command switch
            {
                "HELLO" => new RpcResponse(id, RpcResult.Ok(VectorizerValue.OfMap(new[]
                {
                    new MapPair(VectorizerValue.OfStr("server_version"), VectorizerValue.OfStr("test")),
                    new MapPair(VectorizerValue.OfStr("protocol_version"), VectorizerValue.OfInt(1)),
                    new MapPair(VectorizerValue.OfStr("authenticated"), VectorizerValue.OfBool(true)),
                    new MapPair(VectorizerValue.OfStr("admin"), VectorizerValue.OfBool(false)),
                    new MapPair(VectorizerValue.OfStr("capabilities"),
                        VectorizerValue.OfArray(Array.Empty<VectorizerValue>())),
                }))),
                "PING" => new RpcResponse(id, RpcResult.Ok(VectorizerValue.OfStr("PONG"))),
                _ => new RpcResponse(id, RpcResult.Err("unknown")),
            });

        await using var pool = new RpcClientPool(new RpcClientPoolOptions
        {
            Endpoint = Endpoint.Rpc("127.0.0.1", server.Port),
            MaxConnections = 2,
        });

        await using var leased = await pool.AcquireAsync();
        Assert.True(leased.Client.IsAuthenticated);
        Assert.Equal("PONG", await leased.Client.PingAsync());
    }

    [Fact]
    public async Task Acquire_Release_ReusesIdleConnection()
    {
        await using var server = await MockRpcServer.StartAsync(DefaultHandler);
        await using var pool = new RpcClientPool(new RpcClientPoolOptions
        {
            Endpoint = Endpoint.Rpc("127.0.0.1", server.Port),
            MaxConnections = 1,
        });

        // First acquire dials + HELLO.
        await using (var first = await pool.AcquireAsync())
        {
            Assert.True(first.Client.IsAuthenticated);
        }
        Assert.Equal(1, pool.IdleCount);

        // Second acquire should hand back the same idle client without a
        // second HELLO round-trip.
        await using var second = await pool.AcquireAsync();
        Assert.True(second.Client.IsAuthenticated);
    }

    [Fact]
    public async Task Acquire_WhenPoolFull_BlocksUntilRelease()
    {
        await using var server = await MockRpcServer.StartAsync(DefaultHandler);
        await using var pool = new RpcClientPool(new RpcClientPoolOptions
        {
            Endpoint = Endpoint.Rpc("127.0.0.1", server.Port),
            MaxConnections = 1,
        });

        var first = await pool.AcquireAsync();

        using var cts = new CancellationTokenSource();
        cts.CancelAfter(TimeSpan.FromMilliseconds(200));
        var blockedTask = pool.AcquireAsync(cts.Token);

        await Assert.ThrowsAnyAsync<OperationCanceledException>(async () => await blockedTask);

        await first.DisposeAsync();

        // Now another acquire should succeed.
        await using var resumed = await pool.AcquireAsync();
        Assert.NotNull(resumed.Client);
    }

    private static RpcResponse DefaultHandler(RpcRequest req, uint id) =>
        req.Command switch
        {
            "HELLO" => new RpcResponse(id, RpcResult.Ok(VectorizerValue.OfMap(new[]
            {
                new MapPair(VectorizerValue.OfStr("server_version"), VectorizerValue.OfStr("test")),
                new MapPair(VectorizerValue.OfStr("protocol_version"), VectorizerValue.OfInt(1)),
                new MapPair(VectorizerValue.OfStr("authenticated"), VectorizerValue.OfBool(true)),
                new MapPair(VectorizerValue.OfStr("admin"), VectorizerValue.OfBool(false)),
                new MapPair(VectorizerValue.OfStr("capabilities"),
                    VectorizerValue.OfArray(Array.Empty<VectorizerValue>())),
            }))),
            _ => new RpcResponse(id, RpcResult.Err("unknown")),
        };
}
