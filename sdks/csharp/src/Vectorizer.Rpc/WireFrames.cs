using System;
using System.Collections.Generic;

// Aliased rather than imported: HiveLLM.Thunder declares its own `Request`
// and `Response` frame types, which these convert to and from.
using Thunder = HiveLLM.Thunder;

namespace Vectorizer.Rpc;

/// <summary>
/// One frame from client to server. Wire spec § 2. The bytes are Thunder's
/// (a 3-element MessagePack array <c>[id, command, args]</c>, matching
/// <c>rmp-serde</c>'s default struct representation); this type is the SDK's
/// shape for them.
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

    /// <summary>Converts to a Thunder request frame, ready for
    /// <c>FrameCodec.EncodeRequest</c>.</summary>
    public Thunder.Request ToThunder()
    {
        var args = new Thunder.Value[Args.Count];
        for (var i = 0; i < args.Length; i++)
        {
            args[i] = Args[i].ToThunder();
        }
        return new Thunder.Request(Id, Command, args);
    }

    /// <summary>Converts a decoded Thunder request frame. Used by test servers
    /// and proxies that read requests off the wire.</summary>
    public static RpcRequest FromThunder(Thunder.Request request)
    {
        ArgumentNullException.ThrowIfNull(request);
        var args = new VectorizerValue[request.Args.Count];
        for (var i = 0; i < args.Length; i++)
        {
            args[i] = VectorizerValue.FromThunder(request.Args[i]);
        }
        return new RpcRequest(request.Id, request.Command, args);
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

    /// <summary>Converts to a Thunder response frame, ready for
    /// <c>FrameCodec.EncodeResponse</c>.</summary>
    public Thunder.Response ToThunder() =>
        Result.IsOk
            ? Thunder.Response.Ok(Id, Result.Value.ToThunder())
            : Thunder.Response.Err(Id, Result.ErrorMessage ?? "unknown server error");

    /// <summary>Converts a decoded Thunder response frame.</summary>
    public static RpcResponse FromThunder(Thunder.Response response)
    {
        ArgumentNullException.ThrowIfNull(response);
        return response.IsOk
            ? new RpcResponse(response.Id, RpcResult.Ok(VectorizerValue.FromThunder(response.Value!)))
            : new RpcResponse(response.Id, RpcResult.Err(response.Error ?? "unknown server error"));
    }
}
