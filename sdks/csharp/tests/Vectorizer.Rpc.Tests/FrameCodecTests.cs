using System;
using System.Buffers.Binary;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Vectorizer.Rpc;
using Xunit;

namespace Vectorizer.Rpc.Tests;

/// <summary>
/// Framing round-trip + golden-vector checks from
/// <c>docs/specs/VECTORIZER_RPC.md § 11</c>.
/// </summary>
public class FrameCodecTests
{
    [Fact]
    public void EncodeFrame_PrependsLittleEndianU32Length()
    {
        // value=1 encodes as a single byte (positive fixint) in msgpack.
        var frame = FrameCodec.EncodeFrame(1);
        Assert.Equal(5, frame.Length); // 4-byte header + 1-byte body
        var declared = BinaryPrimitives.ReadUInt32LittleEndian(frame.AsSpan(0, 4));
        Assert.Equal(1u, declared);
    }

    [Fact]
    public void EncodeFrame_RejectsBodyLargerThanMaxBodySize()
    {
        // 65 MiB payload overshoots the 64 MiB cap — encoder must reject.
        var huge = new byte[FrameCodec.MaxBodySize + 1];
        Assert.Throws<FrameTooLargeException>(() => FrameCodec.EncodeFrame(huge));
    }

    [Fact]
    public async Task ReadFrameAsync_ReadsHeaderAndBody()
    {
        var frame = FrameCodec.EncodeFrame("hello");
        using var ms = new MemoryStream(frame);
        var decoded = await FrameCodec.ReadFrameAsync(ms, CancellationToken.None);
        Assert.Equal("hello", decoded);
    }

    [Fact]
    public async Task ReadFrameAsync_ThrowsOnLengthOverCap()
    {
        var badHeader = new byte[4];
        BinaryPrimitives.WriteUInt32LittleEndian(badHeader, (uint)(FrameCodec.MaxBodySize + 1));
        using var ms = new MemoryStream(badHeader);
        await Assert.ThrowsAsync<FrameTooLargeException>(
            () => FrameCodec.ReadFrameAsync(ms, CancellationToken.None));
    }

    [Fact]
    public async Task ReadFrameAsync_ThrowsOnTruncatedBody()
    {
        var frame = FrameCodec.EncodeFrame("hello");
        // Drop the last byte — reader should surface EndOfStreamException.
        using var ms = new MemoryStream(frame, 0, frame.Length - 1);
        await Assert.ThrowsAsync<EndOfStreamException>(
            () => FrameCodec.ReadFrameAsync(ms, CancellationToken.None));
    }

    /// <summary>
    /// Wire spec § 11 golden vector:
    ///   Request { id: 1, command: "PING", args: [] }
    ///   → 08 00 00 00  93  01  a4 50 49 4e 47  90
    ///
    /// We check the length prefix and every byte of the body so any
    /// drift in MessagePack compact-int / string encoding surfaces
    /// immediately.
    /// </summary>
    [Fact]
    public void EncodeFrame_PingRequest_MatchesGoldenVector()
    {
        var request = new RpcRequest(1, "PING", System.Array.Empty<VectorizerValue>());
        var frame = FrameCodec.EncodeFrame(request.ToWire());

        var header = BinaryPrimitives.ReadUInt32LittleEndian(frame.AsSpan(0, 4));
        Assert.Equal(8u, header);

        var body = frame.AsSpan(4).ToArray();
        var expectedBody = new byte[]
        {
            0x93,                                           // array(3)
            0x01,                                           // id = 1
            0xa4, 0x50, 0x49, 0x4e, 0x47,                   // "PING"
            0x90,                                           // args = array(0)
        };
        Assert.Equal(expectedBody, body);
    }

    /// <summary>
    /// Wire spec § 11 golden vector:
    ///   Response { id: 1, result: Ok(Str("PONG")) }
    ///   → 10 00 00 00  92  01  81 a2 4f 6b  81 a3 53 74 72  a4 50 4f 4e 47
    /// </summary>
    [Fact]
    public void EncodeFrame_PongResponse_MatchesGoldenVector()
    {
        var response = new RpcResponse(1, RpcResult.Ok(VectorizerValue.OfStr("PONG")));
        var frame = FrameCodec.EncodeFrame(response.ToWire());

        var header = BinaryPrimitives.ReadUInt32LittleEndian(frame.AsSpan(0, 4));
        Assert.Equal(16u, header);

        var body = frame.AsSpan(4).ToArray();
        var expectedBody = new byte[]
        {
            0x92,                                           // array(2)
            0x01,                                           // id = 1
            0x81, 0xa2, 0x4f, 0x6b,                         // map(1) "Ok":
            0x81, 0xa3, 0x53, 0x74, 0x72,                   //   map(1) "Str":
            0xa4, 0x50, 0x4f, 0x4e, 0x47,                   //     "PONG"
        };
        Assert.Equal(expectedBody, body);
    }

    [Fact]
    public async Task EncodeFrame_PongResponse_RoundTripsThroughFromWire()
    {
        var response = new RpcResponse(42, RpcResult.Ok(VectorizerValue.OfStr("PONG")));
        var frame = FrameCodec.EncodeFrame(response.ToWire());

        using var ms = new MemoryStream(frame);
        var raw = await FrameCodec.ReadFrameAsync(ms, CancellationToken.None);
        var decoded = RpcResponse.FromWire(raw);

        Assert.Equal(42u, decoded.Id);
        Assert.True(decoded.Result.IsOk);
        Assert.True(decoded.Result.Value.TryAsStr(out var s));
        Assert.Equal("PONG", s);
    }

    [Fact]
    public async Task EncodeFrame_ErrResponse_RoundTrips()
    {
        var response = new RpcResponse(7, RpcResult.Err("boom"));
        var frame = FrameCodec.EncodeFrame(response.ToWire());

        using var ms = new MemoryStream(frame);
        var raw = await FrameCodec.ReadFrameAsync(ms, CancellationToken.None);
        var decoded = RpcResponse.FromWire(raw);

        Assert.Equal(7u, decoded.Id);
        Assert.False(decoded.Result.IsOk);
        Assert.Equal("boom", decoded.Result.ErrorMessage);
    }
}
