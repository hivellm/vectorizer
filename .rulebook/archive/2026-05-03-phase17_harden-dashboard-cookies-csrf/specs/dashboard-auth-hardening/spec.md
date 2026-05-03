# Specification: Dashboard auth hardening (cookies + CSRF)

Phase17 closes the cookie + CSRF gaps phase8 enumerated (audit sections
1.9 + 6.5–6.6).

## ADDED Requirements

### Requirement: Hardened session cookie attributes
The system SHALL emit every dashboard session cookie with the
`HttpOnly`, `Secure`, `SameSite=Strict`, `Path=/`, and `Max-Age=<jwt_exp>`
attributes when the operator has not opted into `auth.cookies.insecure_dev`.

#### Scenario: Production cookie carries all four attributes
Given an `AuthConfig` with `cookies.insecure_dev=false`
When the server emits a session cookie via `build_session_cookie`
Then the resulting `Set-Cookie` value contains `HttpOnly`, `Secure`,
`SameSite=Strict`, `Path=/`, and `Max-Age=<jwt_exp>`

#### Scenario: Dev-mode cookie omits Secure
Given an `AuthConfig` with `cookies.insecure_dev=true`
When the server emits a session cookie via `build_session_cookie`
Then the resulting `Set-Cookie` value omits `Secure` while keeping
`HttpOnly`, `SameSite=Strict`, `Path=/`, and `Max-Age=<jwt_exp>`

### Requirement: CSRF token cookie + header
The system SHALL emit a non-`HttpOnly` `XSRF-TOKEN` cookie at login and
require the same value to be echoed in the `X-CSRF-Token` header on
every mutating (`POST`/`PUT`/`PATCH`/`DELETE`) request under `/auth/*`
and `/admin/*`.

#### Scenario: Login emits the CSRF cookie alongside the session cookie
Given a successful `POST /auth/login`
When the server returns the response
Then it carries two `Set-Cookie` headers: one for `vectorizer_session`
(HttpOnly) and one for `XSRF-TOKEN` (non-HttpOnly), both with
`SameSite=Strict; Path=/; Max-Age=<jwt_exp>`

#### Scenario: Mutating request without X-CSRF-Token returns 403
Given the dashboard makes `POST /auth/users`
And the request omits the `X-CSRF-Token` header
When the CSRF middleware processes the request
Then it returns HTTP 403 with `error="missing_csrf_token"`

#### Scenario: Mutating request with valid X-CSRF-Token returns 200
Given the dashboard makes `POST /auth/users`
And the request carries `Cookie: vectorizer_session=<jwt>`
And the request carries `X-CSRF-Token: <token bound to that jwt at login>`
When the CSRF middleware processes the request
Then it forwards the request to the handler, which returns HTTP 200/201

#### Scenario: Mutating request with mismatched X-CSRF-Token returns 403
Given the dashboard makes `POST /auth/users`
And the request carries a valid session cookie
And the request carries an `X-CSRF-Token` header that differs from
the token bound to the session
When the CSRF middleware processes the request
Then it returns HTTP 403 with `error="invalid_csrf_token"`

#### Scenario: Read methods bypass CSRF
Given the dashboard makes `GET /auth/users`
When the CSRF middleware processes the request
Then it forwards the request unconditionally — `GET`/`HEAD`/`OPTIONS`
do not require the CSRF header

#### Scenario: API-key requests bypass CSRF
Given a programmatic client makes `POST /auth/users` with `X-API-Key`
When the CSRF middleware processes the request
Then it forwards the request unconditionally — header-bearer API keys
are not subject to the cross-origin attack the CSRF token defends
against

#### Scenario: /auth/login is exempt from CSRF
Given the dashboard makes `POST /auth/login` (no session yet)
When the CSRF middleware processes the request
Then it forwards the request unconditionally — login is the path that
mints the CSRF token in the first place

### Requirement: Boot guard against insecure_dev on non-loopback bind
The system SHALL refuse to start when `auth.cookies.insecure_dev=true`
and the bind host is not a loopback address (`127.0.0.1`, `::1`,
`localhost`).

#### Scenario: insecure_dev=true plus 0.0.0.0 fails boot
Given an `AuthConfig` with `cookies.insecure_dev=true`
When `VectorizerServer::start` is called with host `"0.0.0.0"`
Then the call returns an `Err` whose message references
`auth.cookies.insecure_dev` and refuses to open any listener

#### Scenario: insecure_dev=true plus 127.0.0.1 boots
Given an `AuthConfig` with `cookies.insecure_dev=true`
When `VectorizerServer::start` is called with host `"127.0.0.1"`
Then the boot guard passes and the server proceeds to bind

### Requirement: Logout scrubs cookies and CSRF binding
The system SHALL emit `Set-Cookie` headers that expire both
`vectorizer_session` and `XSRF-TOKEN` immediately, and SHALL drop the
CSRF binding for the logged-out JWT, on `POST /auth/logout`.

#### Scenario: Logout response carries two expired Set-Cookie headers
Given an authenticated user makes `POST /auth/logout`
When the server processes the request
Then the response carries `Set-Cookie: vectorizer_session=; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT; …`
and `Set-Cookie: XSRF-TOKEN=; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT; …`
and the JWT is added to the blacklist
and the CSRF binding for that JWT is removed

### Requirement: Refresh rotates cookies but preserves the CSRF token value
The system SHALL emit fresh `Set-Cookie` headers for both cookies on
`POST /auth/refresh`, binding the existing CSRF value to the newly
minted JWT so the SPA does not need to re-read the cookie.

#### Scenario: Refresh re-binds CSRF to the new JWT
Given an authenticated user with JWT `J1` and CSRF binding `C1`
When the user makes `POST /auth/refresh`
Then the response includes a new JWT `J2`
and the server's CSRF map contains `J2 -> C1`
and the CSRF entry for `J1` is removed
and the response carries `Set-Cookie: vectorizer_session=J2; …`
and `Set-Cookie: XSRF-TOKEN=C1; …`
