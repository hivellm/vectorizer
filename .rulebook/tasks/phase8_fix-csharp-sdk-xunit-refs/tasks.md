## 1. Implementation

- [ ] 1.1 Diff the `PackageReference` block of
  `sdks/csharp/Vectorizer.csproj` vs the two test csprojs
  (`Vectorizer.Tests/Vectorizer.Tests.csproj` +
  `tests/Vectorizer.Rpc.Tests/Vectorizer.Rpc.Tests.csproj`); confirm
  the xunit trio is missing from the test csprojs.
- [ ] 1.2 Add the canonical xunit references (versions matched to
  the main csproj pins):
  ```xml
  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.0" />
    <PackageReference Include="xunit" Version="2.9.0" />
    <PackageReference Include="xunit.runner.visualstudio" Version="2.8.0">
      <IncludeAssets>runtime; build; native; contentfiles; analyzers</IncludeAssets>
      <PrivateAssets>all</PrivateAssets>
    </PackageReference>
    <PackageReference Include="coverlet.collector" Version="6.0.4">
      <IncludeAssets>runtime; build; native; contentfiles; analyzers</IncludeAssets>
      <PrivateAssets>all</PrivateAssets>
    </PackageReference>
  </ItemGroup>
  ```
- [ ] 1.3 `dotnet restore && dotnet build` both test projects —
  confirm 0 errors.
- [ ] 1.4 `dotnet test` against the live release binary — record
  per-project pass/fail counts.

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Update or create documentation covering the implementation
  (`sdks/csharp/CHANGELOG.md` under `3.0.0 > Fixed`; build-prereq
  note in `sdks/csharp/README.md#testing`).
- [ ] 2.2 Write tests covering the new behavior (no new tests — the
  existing 54-test suite IS the gate once compilation succeeds).
- [ ] 2.3 Run tests and confirm they pass (`dotnet test` on both
  test projects — target 0 failures).
