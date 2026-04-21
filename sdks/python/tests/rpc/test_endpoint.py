"""Unit tests for ``rpc.endpoint.parse_endpoint``.

The contract is shared with every other Vectorizer SDK; the golden
URL forms here are the same ones tested in
``sdks/rust/src/rpc/endpoint.rs``. Keeping them aligned across SDKs
prevents subtle behaviour drift between language runtimes.
"""

from __future__ import annotations

import os
import sys

import pytest

# Add SDK root to sys.path so `from rpc import …` resolves when tests
# are run via plain `pytest sdks/python/tests/rpc/` from any cwd.
_SDK_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
if _SDK_ROOT not in sys.path:
    sys.path.insert(0, _SDK_ROOT)

from rpc.endpoint import (  # noqa: E402  (path manipulation must precede import)
    DEFAULT_RPC_PORT,
    Endpoint,
    EndpointParseError,
    parse_endpoint,
)


class TestParseEndpoint:
    """Cover every branch of the canonical URL parser."""

    def test_rpc_with_explicit_host_and_port(self):
        ep = parse_endpoint("vectorizer://example.com:9000")
        assert ep == Endpoint.Rpc(host="example.com", port=9000)

    def test_rpc_without_port_defaults_to_15503(self):
        ep = parse_endpoint("vectorizer://example.com")
        assert ep == Endpoint.Rpc(host="example.com", port=DEFAULT_RPC_PORT)
        assert DEFAULT_RPC_PORT == 15503

    def test_bare_host_port_without_scheme_is_rpc(self):
        ep = parse_endpoint("localhost:15503")
        assert ep == Endpoint.Rpc(host="localhost", port=15503)

    def test_http_url_routes_to_rest_endpoint(self):
        assert parse_endpoint("http://localhost:15002") == Endpoint.Rest(
            url="http://localhost:15002"
        )
        assert parse_endpoint("https://api.example.com") == Endpoint.Rest(
            url="https://api.example.com"
        )

    def test_unsupported_scheme_is_rejected_by_name(self):
        with pytest.raises(EndpointParseError) as exc_info:
            parse_endpoint("ftp://server.example.com")
        assert "ftp" in str(exc_info.value)
        assert "vectorizer" in str(exc_info.value)

    def test_empty_string_is_rejected(self):
        with pytest.raises(EndpointParseError, match="empty"):
            parse_endpoint("")
        with pytest.raises(EndpointParseError, match="empty"):
            parse_endpoint("   ")

    def test_url_with_userinfo_credentials_is_rejected(self):
        # Both schemes must reject credentials in the URL — they go
        # through the HELLO handshake instead. This avoids accidentally
        # logging or shell-history-saving a token-bearing URL.
        with pytest.raises(EndpointParseError, match="credentials"):
            parse_endpoint("vectorizer://user:pass@host:15503")
        with pytest.raises(EndpointParseError, match="credentials"):
            parse_endpoint("https://user:secret@api.example.com")

    def test_malformed_port_is_rejected(self):
        with pytest.raises(EndpointParseError, match="invalid port"):
            parse_endpoint("vectorizer://host:not-a-port")

    def test_ipv6_literal_with_port(self):
        ep = parse_endpoint("vectorizer://[::1]:15503")
        assert ep == Endpoint.Rpc(host="[::1]", port=15503)

    def test_ipv6_literal_without_port_defaults(self):
        ep = parse_endpoint("vectorizer://[::1]")
        assert ep == Endpoint.Rpc(host="[::1]", port=DEFAULT_RPC_PORT)

    def test_non_string_input_is_rejected(self):
        with pytest.raises(EndpointParseError, match="must be a string"):
            parse_endpoint(15503)  # type: ignore[arg-type]
