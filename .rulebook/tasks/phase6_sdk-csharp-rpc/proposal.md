# Proposal: phase6_sdk-csharp-rpc

## Why

`sdks/csharp/` serves .NET consumers. Add RPC transport so .NET users get the fast path by default. Reference: `../Synap/sdks/csharp/`.

## What Changes

Inside `sdks/csharp/`:

1. Add NuGet dependency `MessagePack` (official, high-performance).
2. New namespace `Vectorizer.Rpc` with `RpcClient` class using `System.Net.Sockets.TcpClient` + async/await.
3. Connection pool (`RpcClientPool`) with configurable max connections.
4. Typed API: `IVectorizerClient` interface + `RpcVectorizerClient` + `HttpVectorizerClient` implementations.
5. Extension method `services.AddVectorizerClient(...)` for ASP.NET Core DI with RPC default.
6. `.NET 8.0` minimum; nullable reference types enabled.
7. Update README quickstart to RPC.
8. Bump NuGet package major version.

## Impact

- Affected specs: SDK spec
- Affected code: `sdks/csharp/src/Vectorizer.Rpc/` (new), existing HTTP client, `*.csproj`, README, tests
- Breaking change: YES (default transport changes) — major version bump
- User benefit: .NET/Unity/Xamarin consumers get fast path; DI-friendly; tested under ASP.NET Core.

## Default URL scheme

`new VectorizerClient(string url)` parses the URL as follows:

- `vectorizer://host:15503` → RPC (binary MessagePack via the
  `MessagePack` NuGet package, see `docs/specs/VECTORIZER_RPC.md`).
- `vectorizer://host` → RPC on default port 15503.
- `host:15503` (no scheme) → RPC.
- `http://host:15002` / `https://host` → REST (legacy fallback).

`vectorizer://` is the canonical default per
`phase6_make-rpc-default-transport`. The
`services.AddVectorizerClient(url)` DI extension applies the same URL
parsing.
