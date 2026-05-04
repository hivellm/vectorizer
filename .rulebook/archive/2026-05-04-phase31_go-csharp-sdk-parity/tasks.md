## 1. Go SDK

- [x] 1.1 `UpdateApiKeyPermissions(id, request) → *ApiKeyView` (`PUT /auth/keys/{id}/permissions`)
- [x] 1.2 `GetApiKeyUsage(id, windowDays) → *ApiKeyUsageReport` (`GET /auth/keys/{id}/usage[?window=N]`)
- [x] 1.3 `DeleteVectors(collection, ids) → *DeleteReport` (`POST /batch_delete`)
- [x] 1.4 `MoveToCollection(src, dst, ids) → *MoveReport` (`POST /collections/{src}/vectors/move`)

## 2. C# SDK

- [x] 2.1 `UpdateApiKeyPermissionsAsync(id, request) → ApiKeyView`
- [x] 2.2 `GetApiKeyUsageAsync(id, windowDays?) → ApiKeyUsageReport`
- [x] 2.3 `DeleteVectorsAsync(collection, ids) → DeleteReport`
- [x] 2.4 `MoveToCollectionAsync(src, dst, ids) → MoveReport`

## 3. Tests

- [x] 3.1 Go: 5 unit tests using `httptest.NewServer` (`phase31_sdk_parity_test.go`).
- [x] 3.2 C#: 5 unit tests using an in-process FakeHandler (`Phase31SdkParityTests.cs`).

## 4. Tail (mandatory — enforced by rulebook v5.3.0)

- [x] 4.1 Update or create documentation covering the implementation (CHANGELOG)
- [x] 4.2 Write tests covering the new behavior
- [x] 4.3 Run tests and confirm they pass (Go: `go test ./...` ok; C#: 199 passed, 4 pre-existing ignored unrelated to phase31)
