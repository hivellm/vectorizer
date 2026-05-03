# Specification: Dev-mode auth skip on loopback

Closes phase8 audit gap 7.5: there is no per-bind toggle that lets a
local developer skip credential validation without also disabling
auth in production builds.

## ADDED Requirements

### Requirement: Config flag `auth.dev_mode_skip_loopback`
The system SHALL accept a `dev_mode_skip_loopback: bool` field on
`AuthConfig`, defaulting to `false`. When `true`, the auth subsystem
short-circuits credential validation with a synthetic
`local-dev-admin` principal.

#### Scenario: Default config keeps auth enforcement unchanged
Given an `AuthConfig` with the default `dev_mode_skip_loopback: false`
When a request arrives without any credentials
Then the auth middleware returns HTTP 401

#### Scenario: Flag persists across config round-trips
Given a YAML payload that omits `dev_mode_skip_loopback`
When the deserializer parses the payload
Then `dev_mode_skip_loopback == false` (serde default)

### Requirement: Middleware short-circuit
When `dev_mode_skip_loopback` is `true`, every protected route SHALL
admit the request as the `local-dev-admin` principal and stamp the
response with `X-Vectorizer-Dev-Mode: true`.

#### Scenario: GET on a protected route succeeds without credentials
Given the flag is on
When a GET request hits a route gated by `require_auth_middleware`
Then the response status is 200
And the response carries `X-Vectorizer-Dev-Mode: true`

#### Scenario: POST on an admin route succeeds without credentials
Given the flag is on
When a POST request hits a route gated by `require_admin_middleware`
Then the response status is 200
And the response carries `X-Vectorizer-Dev-Mode: true`

#### Scenario: CSRF gate is bypassed in dev mode
Given the flag is on
When a POST request hits a route guarded by `require_csrf_middleware`
without an `X-CSRF-Token` header
Then the request is forwarded to the handler (no 403)

#### Scenario: Synthetic principal carries Role::Admin
Given the flag is on
When the middleware injects the principal as a request extension
Then the extension's `user_claims.username == "local-dev-admin"`
And `user_claims.roles` contains `Role::Admin`

### Requirement: Boot guard against non-loopback bind
The system SHALL refuse to start when `dev_mode_skip_loopback` is
`true` and the bind host is not one of `127.0.0.1`, `::1`, or
`localhost`.

#### Scenario: Boot fails with `dev_mode_skip_loopback=true` + `0.0.0.0`
Given an `AuthConfig` with `dev_mode_skip_loopback: true`
When `VectorizerServer::start` is called with host `"0.0.0.0"`
Then the call returns `Err(...)` referencing
`auth.dev_mode_skip_loopback`
And no listener is opened

#### Scenario: Boot succeeds with `dev_mode_skip_loopback=true` + `127.0.0.1`
Given an `AuthConfig` with `dev_mode_skip_loopback: true`
When `VectorizerServer::start` is called with host `"127.0.0.1"`
Then the boot guard passes
And the boot log emits the multi-line `WARN` banner referencing the
disabled-auth posture

### Requirement: Loopback predicate
The system SHALL provide a single `is_loopback_host(host)` predicate
shared by both the cookie boot guard and the dev-mode boot guard so
the two guards apply the same loopback definition.

#### Scenario: Predicate accepts the three canonical aliases
Given `host` ∈ {`127.0.0.1`, `::1`, `localhost`, `LocalHost`,
`  127.0.0.1  `}
When `is_loopback_host(host)` is called
Then the predicate returns `true`

#### Scenario: Predicate rejects 0.0.0.0 and LAN IPs
Given `host` ∈ {`0.0.0.0`, `192.168.1.10`, `10.0.0.1`}
When `is_loopback_host(host)` is called
Then the predicate returns `false`
