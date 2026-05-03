# Specification: API key usage metrics + permission update

Closes phase8 audit gaps 9A.6 (no usage counter), 9A.11 (no permission
update without revocation), and 9B.8 (no dashboard usage display).

## ADDED Requirements

### Requirement: API key usage counter
The system SHALL increment a per-key usage counter exactly once on
every successful credential validation, including rotated-grace
acceptances.

#### Scenario: Counter increments on validate_api_key
Given an API key created with `create_api_key`
When the same key is presented to `AuthManager::validate_api_key`
Then the key's `usage_count` is bumped by 1
And the bump is observable via `ApiKeyManager::current_usage`

#### Scenario: Concurrent validations are race-free
Given a single API key
When 100 concurrent tasks each call `increment_usage` once
Then the final `current_usage` reads exactly 100

#### Scenario: Counter survives persistence flush
Given a key with a non-zero in-memory counter
When `snapshot_usage_counts` is called and the keys are persisted
Then the next `register_key` (load from disk) seeds the atomic counter
from the persisted `usage_count`

#### Scenario: Read-only paths do not bump the counter
Given an API key with `usage_count = 0`
When `ApiKeyManager::validate_key` is called directly (without going
through `AuthManager::validate_api_key`)
Then `current_usage` remains 0
(this separation lets `AuthManager::introspect_token` observe a token
without consuming a usage credit)

### Requirement: Permission update endpoint
The system SHALL accept `PUT /auth/keys/{id}/permissions` from an
admin caller and replace `permissions` (and optionally `scopes`) on an
existing key without rotating the credential.

#### Scenario: Admin replaces permissions
Given an admin caller and an existing key
When the caller sends `PUT /auth/keys/{id}/permissions` with
`{"permissions": ["read", "write"]}`
Then the response is HTTP 200 with the updated `ApiKeyView`
And subsequent `validate_api_key` returns the new permission set
And `key_hash`, `id`, `user_id`, `created_at` are unchanged

#### Scenario: Empty permission list is rejected
Given an admin caller
When the request body is `{"permissions": []}`
Then the server responds with HTTP 400 and
`error="invalid_permissions"`

#### Scenario: Non-admin caller is rejected
Given a non-admin caller (Role::User or anonymous)
When the request hits the endpoint
Then the server responds with HTTP 403 (or 401 if unauthenticated)

#### Scenario: Unknown key id returns 404
Given a request for an id that does not exist
When the handler runs
Then the server responds with HTTP 404 and
`error="not_found"`

### Requirement: Per-key per-day usage time-series
The system SHALL maintain a per-key ring buffer of daily usage counts,
default 30 days, and expose the last `window` days via
`GET /auth/keys/{id}/usage?window=<n>`.

#### Scenario: Daily aggregates reflect recorded events
Given an `ApiKeyUsageRecorder` with a 7-day window
When the test records 30 events on day D-1 and 20 events on day D
Then `snapshot_at(key_id, today=D, days=2)` returns
`[{date: D-1, count: 30}, {date: D, count: 20}]`
And `total(key_id)` returns 50

#### Scenario: Missing days are zero-filled
Given a recorder with events only on D-2 and D
When the snapshot covers `[D-2, D-1, D]`
Then the D-1 bucket appears with `count: 0`
(consumer renders a continuous sparkline without gap-fill logic)

#### Scenario: Window beyond retention is clamped
Given the recorder default window is 30 days
When the caller asks for `window=90`
Then the handler clamps to 30 and returns 30 buckets

#### Scenario: GET /auth/keys/{id}/usage returns the live key snapshot
Given an admin caller and an existing key
When the caller calls `GET /auth/keys/{id}/usage?window=7`
Then the response is HTTP 200 with `{key, buckets, window_total}`
And `key.usage_count` reflects the in-memory atomic counter
And `buckets.len() == 7`
And `window_total == sum(buckets[*].count)`

### Requirement: SDK parity
The Rust, TypeScript, and Python SDKs SHALL expose
`update_api_key_permissions(id, request)` and
`get_api_key_usage(id, window_days?)` methods returning typed views of
the server response.

#### Scenario: Rust SDK methods exist on VectorizerClient
Given a `VectorizerClient` from `vectorizer-sdk`
When the caller invokes `update_api_key_permissions` or
`get_api_key_usage`
Then the methods compile, hit the right HTTP route, and return
`ApiKeyView` / `ApiKeyUsageReport` respectively

#### Scenario: TypeScript SDK methods exist on AuthClient
Given an `AuthClient` from `@hivellm/vectorizer-sdk`
When the caller invokes `updateApiKeyPermissions` or `getApiKeyUsage`
Then the methods compile (passing `tsc --noEmit`) and return the
corresponding typed promise

#### Scenario: Python SDK methods exist on AuthClient
Given an `AuthClient` from the Python `vectorizer` package
When the caller awaits `update_api_key_permissions` or
`get_api_key_usage`
Then the methods return the corresponding dataclass

### Requirement: Dashboard usage columns + sparkline
The dashboard SHALL render `Last 24h` and `Total` columns on the API
keys list and open a usage detail panel with a per-day sparkline when
the operator clicks the row's `Usage` button.

#### Scenario: List shows zero counts for fresh keys
Given a freshly created key
When the API keys list page loads
Then the row shows `Last 24h: 0` and `Total: 0`

#### Scenario: Counts come from the list endpoint without N+1 fetches
Given the page loads N keys
When the dashboard calls `GET /auth/keys` once
Then the response includes `usage_24h` and `usage_count` per row
(no per-row follow-up fetch is required for the columns)

#### Scenario: Usage panel renders the sparkline + bucket table
Given the operator clicks `Usage` on a row
When the dashboard fetches `GET /auth/keys/{id}/usage?window=14`
Then the panel shows the sparkline, the window total, the last-24h
count, and a table of each daily bucket
