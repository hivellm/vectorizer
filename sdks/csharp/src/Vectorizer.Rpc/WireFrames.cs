using System;
using System.Collections.Generic;

namespace Vectorizer.Rpc;

/// <summary>
/// One frame from client to server. Wire spec § 2.
/// Serialises as a 3-element MessagePack array <c>[id, command, args]</c>
/// to match <c>rmp-serde</c>'s default struct representation.
/// </summary>
public sealed class RpcRequest
{
    public uint Id { get; }
    public string Command { get; }
    public IReadOnlyList<VectorizerValue> Args { get; }

    public RpcRequest(uint id, string command, IReadOnlyList<VectorizerValue> args)
    {
        Command = command ?? throw new ArgumentNullException(nameof(command));
        Args = args ?? throw new ArgumentNullException(nameof(args));
        Id = id;
    }

    /// <summary>Returns the wire-shaped value (a 3-element object[]).</summary>
    public object?[] ToWire()
    {
        var args = new object?[Args.Count];
        for (var i = 0; i < Args.Count; i++)
        {
            args[i] = Args[i].ToWire();
        }
        // Wire spec § 2 packs id as the smallest unsigned representation.
        // FrameCodec.EncodeFrame dispatches uint through
        // MessagePackWriter.Write(uint), which picks the compact form.
        return new object?[] { Id, Command, args };
    }
}

/// <summary>
/// Discriminated result mirroring Rust's <c>Result&lt;Value, String&gt;</c>
/// nested inside the response envelope.
/// </summary>
public readonly struct RpcResult
{
    public bool IsOk { get; }
    public VectorizerValue Value { get; }
    public string? ErrorMessage { get; }

    private RpcResult(bool ok, VectorizerValue value, string? err)
    {
        IsOk = ok;
        Value = value;
        ErrorMessage = err;
    }

    public static RpcResult Ok(VectorizerValue value) => new(true, value, null);

    public static RpcResult Err(string message) =>
        new(false, VectorizerValue.Null, message ?? throw new ArgumentNullException(nameof(message)));
}

/// <summary>One frame from server to client. Wire spec § 2.</summary>
public sealed class RpcResponse
{
    public uint Id { get; }
    public RpcResult Result { get; }

    public RpcResponse(uint id, RpcResult result)
    {
        Id = id;
        Result = result;
    }

    /// <summary>Returns the wire-shaped value (a 2-element object[]).</summary>
    public object?[] ToWire()
    {
        // See RpcRequest.ToWire for why Id is a uint (FrameCodec emits compact form).
        if (Result.IsOk)
        {
            return new object?[]
            {
                Id,
                new Dictionary<string, object?> { [VectorizerValue.ResultOk] = Result.Value.ToWire() },
            };
        }
        return new object?[]
        {
            Id,
            new Dictionary<string, object?> { [VectorizerValue.ResultErr] = Result.ErrorMessage },
        };
    }

    /// <summary>Decodes a <see cref="RpcResponse"/> from an on-wire payload.</summary>
    public static RpcResponse FromWire(object? raw)
    {
        if (raw is not object?[] arr || arr.Length != 2)
        {
            throw new InvalidOperationException(
                $"Response frame must be a 2-element array, got {DescribeType(raw)}");
        }

        var id = (uint)VectorizerValue.CoerceInt(arr[0], "Response.id");

        var resultObj = arr[1];
        if (resultObj is null)
        {
            throw new InvalidOperationException("Response.result is null");
        }

        IDictionary<object, object?> resultMap;
        if (resultObj is IDictionary<object, object?> objMap)
        {
            resultMap = objMap;
        }
        else if (resultObj is IDictionary<string, object?> strMap)
        {
            resultMap = new Dictionary<object, object?>(strMap.Count);
            foreach (var kvp in strMap) resultMap[kvp.Key] = kvp.Value;
        }
        else
        {
            throw new InvalidOperationException(
                $"Response.result must be a single-key map, got {DescribeType(resultObj)}");
        }

        if (resultMap.Count != 1)
        {
            throw new InvalidOperationException(
                $"Response.result must be a single-key map, got {resultMap.Count} keys");
        }

        foreach (var kvp in resultMap)
        {
            var tag = kvp.Key as string
                ?? throw new InvalidOperationException(
                    $"Response.result key must be string, got {DescribeType(kvp.Key)}");
            switch (tag)
            {
                case VectorizerValue.ResultOk:
                    return new RpcResponse(id, RpcResult.Ok(VectorizerValue.FromWire(kvp.Value)));
                case VectorizerValue.ResultErr:
                    return new RpcResponse(id, RpcResult.Err(
                        kvp.Value as string
                            ?? throw new InvalidOperationException(
                                $"Err payload must be string, got {DescribeType(kvp.Value)}")));
                default:
                    throw new InvalidOperationException($"unknown Result tag: '{tag}'");
            }
        }

        throw new InvalidOperationException("unreachable");
    }

    private static string DescribeType(object? v) => v?.GetType().FullName ?? "null";
}
