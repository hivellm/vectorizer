## 1. Prerequisites

- [x] 1.1 `phase6_add-rpc-protocol-server` merged
- [x] 1.2 Read `../Synap/sdks/csharp/` for reference

## 2. Transport layer

- [x] 2.1 Add `MessagePack` NuGet dep to `sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj`
- [x] 2.2 Implement `FrameCodec.cs` (u32 LE length-prefix encode/decode)
- [x] 2.3 Implement `RpcClient.cs` with async `ConnectAsync`, `CallAsync<T>`, `CloseAsync`; use `TcpClient` + `NetworkStream`
- [x] 2.4 Implement `RpcClientPool.cs` with configurable size + per-connection timeout

## 3. Client API

- [x] 3.1 Define `IVectorizerClient` interface aligned to capability registry
- [x] 3.2 Implement `RpcVectorizerClient` and `HttpVectorizerClient`
- [x] 3.3 Implement HELLO + AUTH handshake per spec

## 4. DI integration

- [x] 4.1 Add `IServiceCollection.AddVectorizerClient(string url)` extension defaulting to RPC
- [x] 4.2 Support `options.Transport = TransportKind.Http` opt-out
- [x] 4.3 Implement the canonical URL parser as `EndpointParser.Parse(string url)` returning a discriminated union (`Endpoint.Rpc(host, port)` / `Endpoint.Rest(uri)`): `vectorizer://host:port` → RPC on the given port; `vectorizer://host` (no port) → RPC on default port 15503; `host:port` (no scheme) → RPC; `http(s)://host:port` → REST. Throw `ArgumentException` for any other scheme. Both `new VectorizerClient(string url)` and `AddVectorizerClient(string url)` route URL parsing through this single helper.
- [x] 4.4 Unit tests in `tests/EndpointParserTests.cs` covering: each of the 4 valid forms, the default-port branch (15503), an invalid scheme (`ftp://`), `null` / empty string, and a URL with userinfo (which MUST be rejected — credentials go in HELLO, not the URL).

## 5. Examples + docs

- [x] 5.1 Update `sdks/csharp/examples/Quickstart/Program.cs` to RPC
- [x] 5.2 Add ASP.NET Core minimal-API example showing DI
- [x] 5.3 Rewrite `sdks/csharp/README.md` with RPC-first usage

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 6.1 Publish DocFX docs and update the NuGet package description
- [x] 6.2 Integration tests in `sdks/csharp/tests/Vectorizer.Rpc.Tests/` covering CRUD, search, streaming, pool, cancellation tokens
- [x] 6.3 Run `dotnet test` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [x] Update or create documentation covering the implementation
- [x] Write tests covering the new behavior
- [x] Run tests and confirm they pass
