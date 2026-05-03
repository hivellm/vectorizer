"""Authentication surface.

The legacy :class:`client.VectorizerClient` carried the bearer token on
``self.api_key`` but didn't expose dedicated auth endpoints — auth was
always a passive header wired up by the transport layer. This module
preserves that behavior while giving the package a named home for auth
state, so future login / rotation / token-refresh endpoints have a
canonical surface to land in without growing the facade again.

Phase12 additions: me, logout, refresh_token, validate_password,
create_api_key, list_api_keys, revoke_api_key, create_user, list_users,
delete_user, change_password.

Phase15 additions: rotate_api_key, create_scoped_api_key,
introspect_token, list_audit_log.
"""

from __future__ import annotations

from typing import Dict, List, Optional

try:
    from ..models import (
        ApiKey,
        AuditEntry,
        AuditQuery,
        CreateApiKeyRequest,
        CreateScopedApiKeyRequest,
        CreateUserRequest,
        JwtToken,
        PasswordPolicyReport,
        RotatedKey,
        TokenIntrospection,
        User,
    )
except ImportError:  # pragma: no cover
    from models import (  # type: ignore[import-not-found]
        ApiKey,
        AuditEntry,
        AuditQuery,
        CreateApiKeyRequest,
        CreateScopedApiKeyRequest,
        CreateUserRequest,
        JwtToken,
        PasswordPolicyReport,
        RotatedKey,
        TokenIntrospection,
        User,
    )

from ._base import AuthState, Transport, _ApiBase


class AuthClient(_ApiBase):
    """Authentication state holder and auth-endpoint surface.

    Standalone usage::

        from vectorizer import RestTransport, AuthClient
        auth = AuthClient(RestTransport("http://localhost:15002"), api_key="sk-...")
        print(auth.headers())  # raw API key → {"X-API-Key": "sk-..."}
        #                      # JWT            → {"Authorization": "Bearer <jwt>"}

    In the :class:`vectorizer.VectorizerClient` facade this is the
    surface that owns the API key and surfaces it via :attr:`api_key`
    and :meth:`headers`.
    """

    def __init__(
        self,
        transport: Transport,
        *,
        api_key: Optional[str] = None,
        **kwargs,
    ) -> None:
        super().__init__(transport, **kwargs)
        if api_key is not None:
            self._auth = AuthState(api_key=api_key)

    def headers(self) -> dict:
        """Return the bearer-token header set (empty if no key)."""
        return self._auth.headers()

    def set_api_key(self, api_key: Optional[str]) -> None:
        """Replace the current API key — affects future request headers."""
        self._auth.api_key = api_key

    async def login(self, username: str, password: str) -> str:
        """Exchange ``(username, password)`` for a JWT via
        ``POST /auth/login`` and return the raw ``access_token``.

        The token is **not** stored on ``self`` — to use it for
        subsequent requests, call :meth:`set_api_key` with the token
        (``AuthState.headers`` sniffs the JWT shape and routes it onto
        ``Authorization: Bearer …`` automatically).

        When the server runs with ``auth.enabled: false`` this
        endpoint returns 404 — dev servers without auth don't need
        to call :meth:`login` at all.
        """
        data = await self._transport.post(
            "/auth/login",
            {"username": username, "password": password},
        )
        token = data.get("access_token") if isinstance(data, dict) else None
        if not isinstance(token, str) or not token:
            raise ValueError(
                "login response missing `access_token`; server payload: "
                f"{data!r}"
            )
        return token

    async def me(self) -> User:
        """Return the authenticated user's claims.

        Calls ``GET /auth/me``. Requires a valid JWT / API key.

        Returns:
            :class:`User` with user_id, username, and roles.
        """
        data = await self._transport.get("/auth/me")
        return User.from_dict(data if isinstance(data, dict) else {})

    async def logout(self) -> None:
        """Invalidate the current session token.

        Calls ``POST /auth/logout``. The token is blacklisted until its
        natural expiry.
        """
        await self._transport.post("/auth/logout")

    async def refresh_token(self) -> JwtToken:
        """Exchange the current token for a fresh one with an extended TTL.

        Calls ``POST /auth/refresh``.

        Returns:
            :class:`JwtToken` with access_token, token_type, and expires_in.
        """
        data = await self._transport.post("/auth/refresh", {})
        return JwtToken.from_dict(data if isinstance(data, dict) else {})

    async def validate_password(self, password: str) -> PasswordPolicyReport:
        """Validate a password against the server's password policy.

        Calls ``POST /auth/validate-password`` with ``{password}``.

        Args:
            password: Password string to validate.

        Returns:
            :class:`PasswordPolicyReport` with valid flag, errors, and strength.
        """
        data = await self._transport.post("/auth/validate-password", {"password": password})
        return PasswordPolicyReport.from_dict(data if isinstance(data, dict) else {})

    async def create_api_key(self, request: CreateApiKeyRequest) -> ApiKey:
        """Create a new API key for the calling user.

        Calls ``POST /auth/keys``. The ``api_key`` field in the returned
        :class:`ApiKey` is only present at creation time — store it securely.

        Args:
            request: :class:`CreateApiKeyRequest` with name, permissions, expires_in.

        Returns:
            :class:`ApiKey` including the raw key value (creation time only).
        """
        payload = {
            "name": request.name,
            "permissions": request.permissions,
        }
        if request.expires_in is not None:
            payload["expires_in"] = request.expires_in
        data = await self._transport.post("/auth/keys", data=payload)
        return ApiKey.from_dict(data if isinstance(data, dict) else {})

    async def list_api_keys(self) -> List[ApiKey]:
        """List the API keys belonging to the calling user.

        Calls ``GET /auth/keys``. The ``api_key`` field is omitted in
        list responses for security.

        Returns:
            List of :class:`ApiKey` entries without raw key values.
        """
        data = await self._transport.get("/auth/keys")
        raw = data.get("keys", []) if isinstance(data, dict) else []
        return [ApiKey.from_dict(k) for k in raw]

    async def revoke_api_key(self, key_id: str) -> None:
        """Revoke an API key by id.

        Calls ``DELETE /auth/keys/{id}``. Admin can revoke any key;
        regular users can only revoke their own.

        Args:
            key_id: The key UUID to revoke.
        """
        await self._transport.delete(f"/auth/keys/{key_id}")

    async def create_user(self, request: CreateUserRequest) -> User:
        """Create a new user (admin only).

        Calls ``POST /auth/users``. Requires Admin role.

        Args:
            request: :class:`CreateUserRequest` with username, password, roles.

        Returns:
            :class:`User` for the newly created user.
        """
        payload = {
            "username": request.username,
            "password": request.password,
            "roles": request.roles,
        }
        data = await self._transport.post("/auth/users", data=payload)
        return User.from_dict(data if isinstance(data, dict) else {})

    async def list_users(self) -> List[User]:
        """List all users (admin only).

        Calls ``GET /auth/users``. Requires Admin role.

        Returns:
            List of :class:`User` entries.
        """
        data = await self._transport.get("/auth/users")
        raw = data.get("users", []) if isinstance(data, dict) else []
        return [User.from_dict(u) for u in raw]

    async def delete_user(self, username: str) -> None:
        """Delete a user (admin only).

        Calls ``DELETE /auth/users/{username}``. Requires Admin role.
        The server refuses to delete self or the last admin.

        Args:
            username: Username to delete.
        """
        await self._transport.delete(f"/auth/users/{username}")

    async def change_password(self, username: str, new_password: str) -> None:
        """Change a user's password.

        Calls ``PUT /auth/users/{username}/password`` with ``{new_password}``.
        Admins can change any password; non-admins must also supply
        ``current_password`` (passed via ``new_password`` field by the server).

        Args:
            username: The user whose password to change.
            new_password: The new password value.
        """
        await self._transport.put(
            f"/auth/users/{username}/password",
            data={"new_password": new_password},
        )

    # ── phase15 admin endpoints ───────────────────────────────────────────────

    async def rotate_api_key(self, key_id: str) -> RotatedKey:
        """Atomically rotate an API key (admin only).

        Calls ``POST /auth/keys/{id}/rotate`` with an empty body.
        Returns both tokens plus a grace window during which the old token
        remains valid.

        Args:
            key_id: UUID of the key to rotate.

        Returns:
            :class:`RotatedKey` with old_key_id, new_key_id, new_token,
            and grace_until.
        """
        data = await self._transport.post(f"/auth/keys/{key_id}/rotate", data={})
        return RotatedKey.from_dict(data if isinstance(data, dict) else {})

    async def create_scoped_api_key(self, request: CreateScopedApiKeyRequest) -> ApiKey:
        """Create an API key with optional per-collection scopes.

        Calls ``POST /auth/keys``. When ``scopes`` is non-empty the key is
        restricted to the listed collections.

        Args:
            request: :class:`CreateScopedApiKeyRequest` with name, permissions,
                expires_in, and scopes.

        Returns:
            :class:`ApiKey` including the raw key value (creation time only).
        """
        payload: Dict = {
            "name": request.name,
            "permissions": request.permissions,
            "scopes": [
                {"collection": s.collection, "permissions": s.permissions}
                for s in request.scopes
            ],
        }
        if request.expires_in is not None:
            payload["expires_in"] = request.expires_in
        data = await self._transport.post("/auth/keys", data=payload)
        return ApiKey.from_dict(data if isinstance(data, dict) else {})

    async def introspect_token(self, token: str) -> TokenIntrospection:
        """Introspect a token — RFC 7662.

        Calls ``POST /auth/introspect`` with ``{token}``. Requires
        authentication but not admin. Returns ``active: False`` for any
        unrecognized token.

        Args:
            token: JWT or API key token string to inspect.

        Returns:
            :class:`TokenIntrospection` with active flag, sub, scope, exp,
            and username.
        """
        data = await self._transport.post("/auth/introspect", data={"token": token})
        return TokenIntrospection.from_dict(data if isinstance(data, dict) else {})

    async def list_audit_log(self, query: Optional[AuditQuery] = None) -> List[AuditEntry]:
        """Query the admin audit log (admin only).

        Calls ``GET /auth/audit`` with optional query parameters.
        Returns entries newest-first, bounded by ``query.limit``
        (server default 200).

        Args:
            query: Optional :class:`AuditQuery` filter (actor, action,
                since, until, limit).

        Returns:
            List of :class:`AuditEntry` records.
        """
        params: Dict = {}
        if query is not None:
            if query.actor is not None:
                params["actor"] = query.actor
            if query.action is not None:
                params["action"] = query.action
            if query.since is not None:
                params["since"] = query.since
            if query.until is not None:
                params["until"] = query.until
            if query.limit is not None:
                params["limit"] = query.limit
        path = "/auth/audit"
        if params:
            qs = "&".join(f"{k}={v}" for k, v in params.items())
            path = f"/auth/audit?{qs}"
        data = await self._transport.get(path)
        raw = data.get("entries", []) if isinstance(data, dict) else []
        return [AuditEntry.from_dict(e) for e in raw]
