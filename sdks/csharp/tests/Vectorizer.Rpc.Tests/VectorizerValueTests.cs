using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using Vectorizer.Rpc;
using Xunit;

namespace Vectorizer.Rpc.Tests;

public class VectorizerValueTests
{
    [Fact]
    public void Null_EncodesAsBareString()
    {
        var wire = VectorizerValue.Null.ToWire();
        Assert.Equal("Null", wire);
    }

    [Fact]
    public async Task Int_RoundTripsThroughMsgPack()
    {
        var v = VectorizerValue.OfInt(42);
        var frame = FrameCodec.EncodeFrame(v.ToWire());
        using var ms = new MemoryStream(frame);
        var raw = await FrameCodec.ReadFrameAsync(ms, CancellationToken.None);
        var decoded = VectorizerValue.FromWire(raw);

        Assert.True(decoded.TryAsInt(out var i));
        Assert.Equal(42, i);
    }

    [Fact]
    public async Task Bytes_RoundTripsWithoutBase64()
    {
        var bytes = new byte[] { 1, 2, 3, 4, 5 };
        var v = VectorizerValue.OfBytes(bytes);
        var frame = FrameCodec.EncodeFrame(v.ToWire());

        using var ms = new MemoryStream(frame);
        var raw = await FrameCodec.ReadFrameAsync(ms, CancellationToken.None);
        var decoded = VectorizerValue.FromWire(raw);

        Assert.True(decoded.TryAsBytes(out var got));
        Assert.Equal(bytes, got);
    }

    [Fact]
    public async Task Map_PreservesInsertionOrder()
    {
        var pairs = new List<MapPair>
        {
            new(VectorizerValue.OfStr("zzz"), VectorizerValue.OfInt(1)),
            new(VectorizerValue.OfStr("aaa"), VectorizerValue.OfInt(2)),
            new(VectorizerValue.OfStr("mmm"), VectorizerValue.OfInt(3)),
        };
        var v = VectorizerValue.OfMap(pairs);
        var frame = FrameCodec.EncodeFrame(v.ToWire());

        using var ms = new MemoryStream(frame);
        var raw = await FrameCodec.ReadFrameAsync(ms, CancellationToken.None);
        var decoded = VectorizerValue.FromWire(raw);

        Assert.True(decoded.TryAsMap(out var got));
        Assert.Equal(3, got.Count);
        Assert.Equal("zzz", got[0].Key.AsStr());
        Assert.Equal("aaa", got[1].Key.AsStr());
        Assert.Equal("mmm", got[2].Key.AsStr());
    }

    [Fact]
    public void MapGet_ReturnsExistingKey_MissingKeyReturnsFalse()
    {
        var v = VectorizerValue.OfMap(new[]
        {
            new MapPair(VectorizerValue.OfStr("found"), VectorizerValue.OfInt(7)),
        });

        Assert.True(v.TryMapGet("found", out var got));
        Assert.Equal(7, got.AsInt());

        Assert.False(v.TryMapGet("missing", out _));
    }

    [Fact]
    public async Task Array_OfNestedValues_RoundTrips()
    {
        var v = VectorizerValue.OfArray(new[]
        {
            VectorizerValue.OfInt(1),
            VectorizerValue.OfStr("two"),
            VectorizerValue.OfFloat(3.14),
            VectorizerValue.Null,
        });
        var frame = FrameCodec.EncodeFrame(v.ToWire());

        using var ms = new MemoryStream(frame);
        var raw = await FrameCodec.ReadFrameAsync(ms, CancellationToken.None);
        var decoded = VectorizerValue.FromWire(raw);

        Assert.True(decoded.TryAsArray(out var arr));
        Assert.Equal(4, arr.Count);
        Assert.Equal(1, arr[0].AsInt());
        Assert.Equal("two", arr[1].AsStr());
        Assert.InRange(arr[2].AsFloat(), 3.13, 3.15);
        Assert.Equal(ValueKind.Null, arr[3].Kind);
    }

    [Fact]
    public void HelloResponse_ParsesServerMap()
    {
        var value = VectorizerValue.OfMap(new[]
        {
            new MapPair(VectorizerValue.OfStr("server_version"), VectorizerValue.OfStr("3.0.0")),
            new MapPair(VectorizerValue.OfStr("protocol_version"), VectorizerValue.OfInt(1)),
            new MapPair(VectorizerValue.OfStr("authenticated"), VectorizerValue.OfBool(true)),
            new MapPair(VectorizerValue.OfStr("admin"), VectorizerValue.OfBool(false)),
            new MapPair(VectorizerValue.OfStr("capabilities"), VectorizerValue.OfArray(new[]
            {
                VectorizerValue.OfStr("collections.list"),
                VectorizerValue.OfStr("search.basic"),
            })),
        });

        var hello = HelloResponse.FromValue(value);

        Assert.Equal("3.0.0", hello.ServerVersion);
        Assert.Equal(1, hello.ProtocolVersion);
        Assert.True(hello.Authenticated);
        Assert.False(hello.Admin);
        Assert.Equal(new[] { "collections.list", "search.basic" }, hello.Capabilities);
    }
}
