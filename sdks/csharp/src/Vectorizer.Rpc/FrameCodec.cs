using System;
using System.Buffers;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using MessagePack;

namespace Vectorizer.Rpc;

/// <summary>
/// Wire-format codec for VectorizerRPC frames per
/// <c>docs/specs/VECTORIZER_RPC.md</c> § 1. Every frame is a 4-byte
/// little-endian unsigned length followed by a MessagePack-encoded body.
/// Bodies larger than <see cref="MaxBodySize"/> are rejected.
/// </summary>
public static class FrameCodec
{
    /// <summary>Wire-spec cap on a single frame body (64 MiB).</summary>
    public const int MaxBodySize = 64 * 1024 * 1024;

    /// <summary>Length prefix (u32 LE) in bytes.</summary>
    public const int HeaderSize = 4;

    /// <summary>
    /// Encodes <paramref name="value"/> (a tree of plain
    /// <see cref="IDictionary{TKey, TValue}"/>, arrays, and primitives)
    /// into a single complete wire frame (header + body).
    /// </summary>
    public static byte[] EncodeFrame(object? value)
    {
        var bufferWriter = new ArrayBufferWriter<byte>(256);
        var writer = new MessagePackWriter(bufferWriter);
        WriteValue(ref writer, value);
        writer.Flush();

        if (bufferWriter.WrittenCount > MaxBodySize)
        {
            throw new FrameTooLargeException(bufferWriter.WrittenCount);
        }

        var frame = new byte[HeaderSize + bufferWriter.WrittenCount];
        BinaryPrimitives.WriteUInt32LittleEndian(
            frame.AsSpan(0, HeaderSize), (uint)bufferWriter.WrittenCount);
        bufferWriter.WrittenSpan.CopyTo(frame.AsSpan(HeaderSize));
        return frame;
    }

    /// <summary>
    /// Decodes a single MessagePack body (no header) into a plain
    /// object graph (<see cref="IDictionary{TKey, TValue}"/>,
    /// <see cref="object"/>[], primitives).
    /// </summary>
    public static object? DecodeBody(ReadOnlyMemory<byte> body)
    {
        try
        {
            var reader = new MessagePackReader(body);
            return ReadValue(ref reader);
        }
        catch (Exception ex) when (ex is not FrameTooLargeException)
        {
            throw new FrameDecodeException(ex);
        }
    }

    /// <summary>
    /// Blocks until one complete frame has been read from
    /// <paramref name="stream"/>. Returns the decoded body. A clean EOF
    /// between frames surfaces as <see cref="EndOfStreamException"/>.
    /// </summary>
    public static async Task<object?> ReadFrameAsync(Stream stream, CancellationToken ct)
    {
        ArgumentNullException.ThrowIfNull(stream);

        var header = new byte[HeaderSize];
        await ReadExactAsync(stream, header, ct).ConfigureAwait(false);
        var length = BinaryPrimitives.ReadUInt32LittleEndian(header);
        if (length > MaxBodySize)
        {
            throw new FrameTooLargeException((int)Math.Min(length, int.MaxValue));
        }

        if (length == 0)
        {
            return DecodeBody(ReadOnlyMemory<byte>.Empty);
        }

        var buffer = ArrayPool<byte>.Shared.Rent((int)length);
        try
        {
            await ReadExactAsync(stream, buffer.AsMemory(0, (int)length), ct).ConfigureAwait(false);
            return DecodeBody(buffer.AsMemory(0, (int)length));
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }

    // ── Writer ────────────────────────────────────────────────────────
    // We write MessagePack bytes by hand rather than delegate to
    // MessagePackSerializer because the standard resolver emits Int32 as
    // the fixed 5-byte 0xd2 form (and Int64 as the 9-byte 0xd3 form),
    // which clashes with rmp-serde's default of packing each integer in
    // its smallest representation. The wire-spec golden vectors require
    // compact packing, so we invoke MessagePackWriter's Write(long) /
    // Write(ulong) overloads directly — those DO pick the tightest form.

    private static void WriteValue(ref MessagePackWriter writer, object? value)
    {
        switch (value)
        {
            case null:
                writer.WriteNil();
                return;
            case bool b:
                writer.Write(b);
                return;
            case sbyte sb:
                writer.Write(sb);
                return;
            case byte b:
                writer.Write(b);
                return;
            case short s:
                writer.Write(s);
                return;
            case ushort us:
                writer.Write(us);
                return;
            case int i:
                writer.Write(i);
                return;
            case uint ui:
                writer.Write(ui);
                return;
            case long l:
                writer.Write(l);
                return;
            case ulong ul:
                writer.Write(ul);
                return;
            case float f:
                writer.Write(f);
                return;
            case double d:
                writer.Write(d);
                return;
            case string s:
                writer.Write(s);
                return;
            case byte[] buf:
                writer.Write(buf);
                return;
            case object?[] arr:
                writer.WriteArrayHeader(arr.Length);
                foreach (var item in arr) WriteValue(ref writer, item);
                return;
            case IDictionary<string, object?> dict:
                writer.WriteMapHeader(dict.Count);
                foreach (var kvp in dict)
                {
                    writer.Write(kvp.Key);
                    WriteValue(ref writer, kvp.Value);
                }
                return;
            case IDictionary<object, object?> objDict:
                writer.WriteMapHeader(objDict.Count);
                foreach (var kvp in objDict)
                {
                    WriteValue(ref writer, kvp.Key);
                    WriteValue(ref writer, kvp.Value);
                }
                return;
            default:
                throw new InvalidOperationException(
                    $"unsupported wire value type: {value.GetType().FullName}");
        }
    }

    private static object? ReadValue(ref MessagePackReader reader)
    {
        switch (reader.NextMessagePackType)
        {
            case MessagePackType.Nil:
                reader.ReadNil();
                return null;
            case MessagePackType.Boolean:
                return reader.ReadBoolean();
            case MessagePackType.Integer:
                var code = reader.NextCode;
                if (code == MessagePackCode.UInt64)
                {
                    return reader.ReadUInt64();
                }
                return reader.ReadInt64();
            case MessagePackType.Float:
                if (reader.NextCode == MessagePackCode.Float32)
                {
                    return reader.ReadSingle();
                }
                return reader.ReadDouble();
            case MessagePackType.String:
                return reader.ReadString();
            case MessagePackType.Binary:
            {
                var seq = reader.ReadBytes();
                return seq.HasValue ? seq.Value.ToArray() : Array.Empty<byte>();
            }
            case MessagePackType.Array:
            {
                var len = reader.ReadArrayHeader();
                var arr = new object?[len];
                for (var i = 0; i < len; i++) arr[i] = ReadValue(ref reader);
                return arr;
            }
            case MessagePackType.Map:
            {
                var len = reader.ReadMapHeader();
                var dict = new Dictionary<object, object?>(len);
                for (var i = 0; i < len; i++)
                {
                    var k = ReadValue(ref reader)
                        ?? throw new FrameDecodeException(new InvalidDataException("map key is nil"));
                    dict[k] = ReadValue(ref reader);
                }
                return dict;
            }
            case MessagePackType.Extension:
            {
                var ext = reader.ReadExtensionFormat();
                return ext;
            }
            default:
                throw new FrameDecodeException(
                    new InvalidDataException($"unsupported msgpack type: {reader.NextMessagePackType}"));
        }
    }

    private static async Task ReadExactAsync(Stream stream, byte[] buffer, CancellationToken ct)
        => await ReadExactAsync(stream, buffer.AsMemory(), ct).ConfigureAwait(false);

    private static async Task ReadExactAsync(Stream stream, Memory<byte> buffer, CancellationToken ct)
    {
        var offset = 0;
        while (offset < buffer.Length)
        {
            var n = await stream.ReadAsync(buffer.Slice(offset), ct).ConfigureAwait(false);
            if (n == 0)
            {
                if (offset == 0)
                {
                    throw new EndOfStreamException("connection closed between frames");
                }
                throw new EndOfStreamException(
                    $"connection closed mid-frame after {offset} of {buffer.Length} bytes");
            }
            offset += n;
        }
    }
}

/// <summary>Thrown when a frame's declared length exceeds <see cref="FrameCodec.MaxBodySize"/>.</summary>
public sealed class FrameTooLargeException : Exception
{
    public int Size { get; }

    public FrameTooLargeException(int size)
        : base($"frame body is {size} bytes, exceeds 64 MiB cap")
    {
        Size = size;
    }
}

/// <summary>Thrown when a frame's body is not valid MessagePack.</summary>
public sealed class FrameDecodeException : Exception
{
    public FrameDecodeException(Exception inner)
        : base($"frame body is not valid MessagePack: {inner.Message}", inner)
    {
    }
}
