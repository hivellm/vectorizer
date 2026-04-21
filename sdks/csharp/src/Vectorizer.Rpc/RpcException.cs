using System;

namespace Vectorizer.Rpc;

/// <summary>Base type for RPC transport errors surfaced by the SDK.</summary>
public class VectorizerRpcException : Exception
{
    public VectorizerRpcException(string message) : base(message) { }
    public VectorizerRpcException(string message, Exception inner) : base(message, inner) { }
}

/// <summary>
/// Wraps a <c>Result::Err(message)</c> returned by the server. In
/// protocol v1 the message is a human-readable string; do not branch
/// on its contents.
/// </summary>
public sealed class RpcServerException : VectorizerRpcException
{
    public RpcServerException(string message) : base(message) { }
}

/// <summary>
/// Raised when the reader loop exits before a response arrived — the
/// connection is unusable and must be replaced.
/// </summary>
public sealed class RpcConnectionClosedException : VectorizerRpcException
{
    public RpcConnectionClosedException() : base("connection closed before response") { }

    public RpcConnectionClosedException(string message) : base(message) { }

    public RpcConnectionClosedException(string message, Exception inner) : base(message, inner) { }
}

/// <summary>
/// Raised when a data-plane command is issued before HELLO has
/// succeeded. The local gate saves a network round-trip that the
/// server would fail anyway.
/// </summary>
public sealed class RpcNotAuthenticatedException : VectorizerRpcException
{
    public RpcNotAuthenticatedException()
        : base("HELLO must succeed before any data-plane command can be issued")
    {
    }
}
