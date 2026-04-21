# Proposal: phase8_fix-csharp-sdk-xunit-refs

## Why

Probe 4.5 of `phase8_release-v3-runtime-verification` ran
`dotnet build sdks/csharp/Vectorizer.Tests/Vectorizer.Tests.csproj`.
Result: **169 compile errors**, all of the form:

```
The type or namespace name 'InlineDataAttribute' could not be found
  (are you missing a using directive or an assembly reference?)
The type or namespace name 'FactAttribute' could not be found ...
The type or namespace name 'TheoryAttribute' could not be found ...
```

in `sdks/csharp/tests/Vectorizer.Rpc.Tests/EndpointParserTests.cs`
and sibling test files. xunit's attributes are not resolving because
the test project's `.csproj` lacks a `PackageReference` to the xunit
+ xunit.runner.visualstudio + Microsoft.NET.Test.Sdk trio.

The root project `Vectorizer.csproj` IS pulling those packages
(phase5_refresh-all-dependencies moved them to 17.14 / 2.9 / 2.8)
but `Vectorizer.Tests.csproj` doesn't transitively inherit them —
probably a missing `ProjectReference` or `PackageReference` in the
test csproj that was lost in a merge.

Source: `docs/releases/v3.0.0-verification.md` probe 4.5.

## What Changes

Restore the xunit package refs on the C# test project so the 169
compile errors clear; then run `dotnet test` against the live server
and verify the suite passes.

1. Inspect `sdks/csharp/tests/Vectorizer.Rpc.Tests/*.csproj` +
   `sdks/csharp/Vectorizer.Tests/Vectorizer.Tests.csproj` for
   missing `PackageReference` entries.
2. Add the canonical xunit trio (matching the version pins already
   declared in the main `Vectorizer.csproj`):
   ```xml
   <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.0" />
   <PackageReference Include="xunit" Version="2.9.0" />
   <PackageReference Include="xunit.runner.visualstudio" Version="2.8.0" />
   <PackageReference Include="coverlet.collector" Version="6.0.4" />
   ```
3. Confirm `dotnet restore && dotnet build` both test projects
   cleanly.
4. Run `dotnet test` against the live release binary — capture the
   pass/fail counts per test class.

## Impact

- Affected specs: `docs/users/sdks/CSHARP.md` if documented.
- Affected code:
  - `sdks/csharp/Vectorizer.Tests/Vectorizer.Tests.csproj`
  - `sdks/csharp/tests/Vectorizer.Rpc.Tests/Vectorizer.Rpc.Tests.csproj`
  - possibly `sdks/csharp/Vectorizer.csproj` if a shared props file
    needs the refs centralized.
- Breaking change: NO (build-only fix).
- User benefit: C# test suite compiles + runs. Unblocks probe 4.5
  so the 54-test suite (framing, endpoint, value round-trip, RPC
  client, pool, DI) can actually execute against the v3 server.
