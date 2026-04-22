"""Authentication surface.

The legacy :class:`client.VectorizerClient` carried the bearer token on
``self.api_key`` but didn't expose dedicated auth endpoints — auth was
always a passive header wired up by the transport layer. This module
preserves that behavior while giving the package a named home for auth
state, so future login / rotation / token-refresh endpoints have a
canonical surface to land in without growing the facade again.
"""

from __future__ import annotations

from typing import Optional

from ._base import AuthState, Transport, _ApiBase


class AuthClient(_ApiBase):
    """Authentication state holder.

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
