"""Canonical URL parser for the SDK's connection string.

Mirrors the Rust SDK at ``sdks/rust/src/rpc/endpoint.rs`` so polyglot
projects share a single contract:

- ``vectorizer://host:port`` → RPC on the given port.
- ``vectorizer://host`` (no port) → RPC on default port 15503.
- ``host:port`` (no scheme) → RPC.
- ``http://host:port`` / ``https://host:port`` → REST (legacy fallback).
- Anything else → :class:`EndpointParseError`.

URLs that carry credentials in the userinfo section
(``user:pass@host``) are REJECTED. Credentials cross the wire in the
HELLO handshake, NOT in the URL. This avoids accidentally logging or
shell-history-saving a token-bearing URL.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional, Union

DEFAULT_RPC_PORT: int = 15503
"""Default RPC port. Matches the server's ``RpcConfig::default_port()``."""

DEFAULT_HTTP_PORT: int = 15002
"""Default REST port. Matches the server's ``ServerConfig::default()``."""


class EndpointParseError(ValueError):
    """Raised when :func:`parse_endpoint` cannot interpret the URL.

    Subclass of :class:`ValueError` so callers that want to handle URL
    errors generically don't need to import this class.
    """


@dataclass(frozen=True)
class _RpcEndpoint:
    host: str
    port: int

    def __repr__(self) -> str:  # pragma: no cover — trivial
        return f"Endpoint.Rpc(host={self.host!r}, port={self.port})"


@dataclass(frozen=True)
class _RestEndpoint:
    url: str

    def __repr__(self) -> str:  # pragma: no cover — trivial
        return f"Endpoint.Rest(url={self.url!r})"


# A parsed endpoint is one of two shapes. Using a Union (rather than
# inheritance) keeps pattern-matching readable: ``isinstance(ep, Endpoint.Rpc)``.
class Endpoint:
    """Namespace for the two endpoint variants returned by :func:`parse_endpoint`.

    Use :class:`Endpoint.Rpc` for VectorizerRPC endpoints and
    :class:`Endpoint.Rest` for HTTP(S) URLs. Both are frozen dataclasses
    with structural equality, so test assertions can compare them
    directly.
    """

    Rpc = _RpcEndpoint
    Rest = _RestEndpoint


EndpointType = Union[_RpcEndpoint, _RestEndpoint]


def parse_endpoint(url: str) -> EndpointType:
    """Parse a connection string into a typed endpoint.

    See the module docstring for the contract. Returns the first
    matching endpoint shape; never falls through silently.

    Raises :class:`EndpointParseError` for empty input, unsupported
    schemes, malformed authorities, and URLs carrying credentials in
    the userinfo section.
    """
    if not isinstance(url, str):
        raise EndpointParseError(f"endpoint URL must be a string, got {type(url).__name__}")
    trimmed = url.strip()
    if not trimmed:
        raise EndpointParseError("endpoint URL is empty")

    # Recognise an explicit scheme by splitting on the FIRST "://".
    if "://" in trimmed:
        scheme, _, rest = trimmed.partition("://")
        scheme_lower = scheme.lower()
        if scheme_lower == "vectorizer":
            return _parse_rpc_authority(rest)
        if scheme_lower in ("http", "https"):
            return _parse_rest(scheme_lower, rest, trimmed)
        raise EndpointParseError(
            f"unsupported URL scheme '{scheme}'; "
            f"expected 'vectorizer', 'http', or 'https'"
        )

    # No scheme — treat as bare host[:port] for RPC.
    return _parse_rpc_authority(trimmed)


def _parse_rpc_authority(authority: str) -> _RpcEndpoint:
    if not authority:
        raise EndpointParseError(f"invalid authority in URL '{authority}': missing host")
    if "@" in authority:
        raise EndpointParseError(
            "URL carries credentials in the userinfo section; "
            "pass credentials to the HELLO handshake instead of embedding them in the URL"
        )

    # Trim a trailing path; the RPC scheme has no notion of paths.
    host_port = authority
    for sep in ("/", "?", "#"):
        idx = host_port.find(sep)
        if idx >= 0:
            host_port = host_port[:idx]
    if not host_port:
        raise EndpointParseError(f"invalid authority in URL '{authority}': missing host")

    # IPv6 literal: [::1] or [::1]:port. Bracket-aware split.
    if host_port.startswith("["):
        close = host_port.find("]")
        if close < 0:
            raise EndpointParseError(
                f"invalid authority in URL '{authority}': unterminated IPv6 literal '['"
            )
        host = host_port[: close + 1]
        after = host_port[close + 1 :]
        if not after:
            return _RpcEndpoint(host=host, port=DEFAULT_RPC_PORT)
        if not after.startswith(":"):
            raise EndpointParseError(
                f"invalid authority in URL '{authority}': "
                f"expected ':<port>' after IPv6 literal, got '{after}'"
            )
        port = _parse_port(after[1:], authority)
        return _RpcEndpoint(host=host, port=port)

    # Hostname or IPv4. Split on the LAST colon (so a port-like suffix
    # wins over an unrelated colon in a future hostname extension).
    if ":" in host_port:
        host, _, port_str = host_port.rpartition(":")
        if not host:
            raise EndpointParseError(
                f"invalid authority in URL '{authority}': missing host before ':<port>'"
            )
        port = _parse_port(port_str, authority)
        return _RpcEndpoint(host=host, port=port)

    return _RpcEndpoint(host=host_port, port=DEFAULT_RPC_PORT)


def _parse_rest(scheme: str, rest: str, raw: str) -> _RestEndpoint:
    if not rest:
        raise EndpointParseError(f"invalid authority in URL '{raw}': missing host")
    if "@" in rest:
        raise EndpointParseError(
            "URL carries credentials in the userinfo section; "
            "pass credentials to the HELLO handshake instead of embedding them in the URL"
        )
    return _RestEndpoint(url=f"{scheme}://{rest}")


def _parse_port(port_str: str, authority: str) -> int:
    try:
        port = int(port_str)
    except ValueError as e:
        raise EndpointParseError(
            f"invalid authority in URL '{authority}': invalid port: {e}"
        ) from e
    if not (0 <= port <= 65535):
        raise EndpointParseError(
            f"invalid authority in URL '{authority}': "
            f"port {port} is out of range 0..65535"
        )
    return port
