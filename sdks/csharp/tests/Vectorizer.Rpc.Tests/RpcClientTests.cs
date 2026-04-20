using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;
using Vectorizer.Rpc;
using Xunit;

namespace Vectorizer.Rpc.Tests;

/// <summary>
/// Integration tests that exercise the full TCP reader/writer loop
/// against a minimal in-process server. No real Vectorizer server is
/// required.
/// </summary>
public class RpcClientTests
{
    [Fact]
    public async Task HelloAndPing_AgainstMockServer_Succeed()
    {
        await using var server = await MockRpcServer.StartAsync((req, id) =>
        {
            return req.Command switch
            {
                "HELLO" => new RpcResponse(id, RpcResult.Ok(VectorizerValue.OfMap(new[]
                {
                    new MapPair(VectorizerValue.OfStr("server_version"), VectorizerValue.OfStr("test")),
                    new MapPair(VectorizerValue.OfStr("protocol_version"), VectorizerValue.OfInt(1)),
                    new MapPair(VectorizerValue.OfStr("authenticated"), VectorizerValue.OfBool(true)),
                    new MapPair(VectorizerValue.OfStr("admin"), VectorizerValue.OfBool(false)),
                    new MapPair(VectorizerValue.OfStr("capabilities"),
                        VectorizerValue.OfArray(new[] { VectorizerValue.OfStr("PING") })),
                }))),
                "PING" => new RpcResponse(id, RpcResult.Ok(VectorizerValue.OfStr("PONG"))),
                _ => new RpcResponse(id, RpcResult.Err($"unknown command '{req.Command}'")),
            };
        });

        await using var client = await RpcClient.ConnectAsync("127.0.0.1", server.Port);

        var hello = await client.HelloAsync(new HelloPayload { ClientName = "csharp-test" });
        Assert.True(hello.Authenticated);
        Assert.Equal(1, hello.ProtocolVersion);

        var pong = await client.PingAsync();
        Assert.Equal("PONG", pong);
    }

    [Fact]
    public async Task Call_BeforeHello_ThrowsNotAuthenticated()
    {
        await using var server = await MockRpcServer.StartAsync((_, id) =>
            new RpcResponse(id, RpcResult.Err("should not reach server")));

        await using var client = await RpcClient.ConnectAsync("127.0.0.1", server.Port);

        await Assert.ThrowsAsync<RpcNotAuthenticatedException>(() =>
            client.CallAsync("collections.list", Array.Empty<VectorizerValue>()));
    }

    [Fact]
    public async Task Call_ServerReturnsErr_ThrowsRpcServerException()
    {
        await using var server = await MockRpcServer.StartAsync((req, id) =>
            req.Command == "HELLO"
                ? new RpcResponse(id, RpcResult.Ok(HelloResponseMap(authenticated: true)))
                : new RpcResponse(id, RpcResult.Err("explicit failure")));

        await using var client = await RpcClient.ConnectAsync("127.0.0.1", server.Port);
        await client.HelloAsync(new HelloPayload());

        var ex = await Assert.ThrowsAsync<RpcServerException>(() =>
            client.CallAsync("collections.list", Array.Empty<VectorizerValue>()));
        Assert.Equal("explicit failure", ex.Message);
    }

    [Fact]
    public async Task Call_WithCancelledToken_ObservesCancellation()
    {
        await using var server = await MockRpcServer.StartAsync((req, id) =>
            req.Command == "HELLO"
                ? new RpcResponse(id, RpcResult.Ok(HelloResponseMap(authenticated: true)))
                : null); // dropped → client waits

        await using var client = await RpcClient.ConnectAsync("127.0.0.1", server.Port);
        await client.HelloAsync(new HelloPayload());

        using var cts = new CancellationTokenSource();
        cts.CancelAfter(TimeSpan.FromMilliseconds(100));
        await Assert.ThrowsAsync<TaskCanceledException>(() =>
            client.CallAsync("never.replies", Array.Empty<VectorizerValue>(), cts.Token));
    }

    [Fact]
    public async Task ConcurrentCalls_AreDemultiplexedById()
    {
        await using var server = await MockRpcServer.StartAsync((req, id) =>
        {
            return req.Command switch
            {
                "HELLO" => new RpcResponse(id, RpcResult.Ok(HelloResponseMap(authenticated: true))),
                "echo" => new RpcResponse(id, RpcResult.Ok(req.Args[0])),
                _ => new RpcResponse(id, RpcResult.Err("unknown")),
            };
        });

        await using var client = await RpcClient.ConnectAsync("127.0.0.1", server.Port);
        await client.HelloAsync(new HelloPayload());

        var tasks = new List<Task<VectorizerValue>>();
        for (var i = 0; i < 32; i++)
        {
            tasks.Add(client.CallAsync("echo", new[] { VectorizerValue.OfInt(i) }));
        }
        var results = await Task.WhenAll(tasks);
        for (var i = 0; i < results.Length; i++)
        {
            Assert.Equal(i, results[i].AsInt());
        }
    }

    private static VectorizerValue HelloResponseMap(bool authenticated) =>
        VectorizerValue.OfMap(new[]
        {
            new MapPair(VectorizerValue.OfStr("server_version"), VectorizerValue.OfStr("test")),
            new MapPair(VectorizerValue.OfStr("protocol_version"), VectorizerValue.OfInt(1)),
            new MapPair(VectorizerValue.OfStr("authenticated"), VectorizerValue.OfBool(authenticated)),
            new MapPair(VectorizerValue.OfStr("admin"), VectorizerValue.OfBool(false)),
            new MapPair(VectorizerValue.OfStr("capabilities"),
                VectorizerValue.OfArray(Array.Empty<VectorizerValue>())),
        });
}

/// <summary>
/// Minimal in-process TCP server that speaks the Vectorizer RPC wire
/// format. The handler receives each decoded <see cref="RpcRequest"/>
/// and returns the response to send; returning <c>null</c> drops the
/// request silently (useful for cancellation tests).
/// </summary>
internal sealed class MockRpcServer : IAsyncDisposable
{
    private readonly TcpListener _listener;
    private readonly Func<RpcRequest, uint, RpcResponse?> _handler;
    private readonly CancellationTokenSource _cts = new();
    private readonly Task _acceptTask;

    private MockRpcServer(TcpListener listener, Func<RpcRequest, uint, RpcResponse?> handler)
    {
        _listener = listener;
        _handler = handler;
        _acceptTask = Task.Run(AcceptLoop);
    }

    public int Port => ((IPEndPoint)_listener.LocalEndpoint).Port;

    public static Task<MockRpcServer> StartAsync(Func<RpcRequest, uint, RpcResponse?> handler)
    {
        var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        return Task.FromResult(new MockRpcServer(listener, handler));
    }

    private async Task AcceptLoop()
    {
        try
        {
            while (!_cts.IsCancellationRequested)
            {
                var client = await _listener.AcceptTcpClientAsync(_cts.Token);
                _ = Task.Run(() => HandleClient(client));
            }
        }
        catch (OperationCanceledException) { }
        catch (ObjectDisposedException) { }
    }

    private async Task HandleClient(TcpClient client)
    {
        using (client)
        await using (var stream = client.GetStream())
        {
            try
            {
                while (!_cts.IsCancellationRequested)
                {
                    var raw = await FrameCodec.ReadFrameAsync(stream, _cts.Token);
                    var request = DecodeRequest(raw);
                    var response = _handler(request, request.Id);
                    if (response is null) continue;

                    var frame = FrameCodec.EncodeFrame(response.ToWire());
                    await stream.WriteAsync(frame, _cts.Token);
                    await stream.FlushAsync(_cts.Token);
                }
            }
            catch (OperationCanceledException) { }
            catch (EndOfStreamException) { }
            catch (IOException) { }
        }
    }

    private static RpcRequest DecodeRequest(object? raw)
    {
        if (raw is not object?[] arr || arr.Length != 3)
        {
            throw new InvalidDataException("invalid request frame shape");
        }
        var id = (uint)VectorizerValue.CoerceInt(arr[0], "Request.id");
        var command = VectorizerValue.CoerceStr(arr[1], "Request.command");
        var argsRaw = VectorizerValue.CoerceArray(arr[2], "Request.args");
        var args = new VectorizerValue[argsRaw.Length];
        for (var i = 0; i < argsRaw.Length; i++) args[i] = VectorizerValue.FromWire(argsRaw[i]);
        return new RpcRequest(id, command, args);
    }

    public async ValueTask DisposeAsync()
    {
        _cts.Cancel();
        try { _listener.Stop(); } catch { }
        try { await _acceptTask.WaitAsync(TimeSpan.FromSeconds(2)); } catch { }
        _cts.Dispose();
    }
}
