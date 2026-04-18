## 1. Prerequisites

- [ ] 1.1 `phase6_add-rpc-protocol-server` merged
- [ ] 1.2 Read `../Synap/sdks/csharp/` for reference

## 2. Transport layer

- [ ] 2.1 Add `MessagePack` NuGet dep to `sdks/csharp/src/Vectorizer.Rpc/Vectorizer.Rpc.csproj`
- [ ] 2.2 Implement `FrameCodec.cs` (u32 LE length-prefix encode/decode)
- [ ] 2.3 Implement `RpcClient.cs` with async `ConnectAsync`, `CallAsync<T>`, `CloseAsync`; use `TcpClient` + `NetworkStream`
- [ ] 2.4 Implement `RpcClientPool.cs` with configurable size + per-connection timeout

## 3. Client API

- [ ] 3.1 Define `IVectorizerClient` interface aligned to capability registry
- [ ] 3.2 Implement `RpcVectorizerClient` and `HttpVectorizerClient`
- [ ] 3.3 Implement HELLO + AUTH handshake per spec

## 4. DI integration

- [ ] 4.1 Add `IServiceCollection.AddVectorizerClient(options)` extension defaulting to RPC
- [ ] 4.2 Support `options.Transport = TransportKind.Http` opt-out

## 5. Examples + docs

- [ ] 5.1 Update `sdks/csharp/examples/Quickstart/Program.cs` to RPC
- [ ] 5.2 Add ASP.NET Core minimal-API example showing DI
- [ ] 5.3 Rewrite `sdks/csharp/README.md` with RPC-first usage

## 6. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 6.1 Publish DocFX docs and update the NuGet package description
- [ ] 6.2 Integration tests in `sdks/csharp/tests/Vectorizer.Rpc.Tests/` covering CRUD, search, streaming, pool, cancellation tokens
- [ ] 6.3 Run `dotnet test` and confirm all pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
