"""Unit tests for the phase12 AuthClient additions.

Tests: me, logout, refresh_token, validate_password, create_api_key,
list_api_keys, revoke_api_key, create_user, list_users, delete_user,
change_password (login was already covered by existing tests).
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import (  # type: ignore[import-not-found]
    ApiKey,
    CreateApiKeyRequest,
    CreateUserRequest,
    JwtToken,
    PasswordPolicyReport,
    User,
)
from vectorizer.auth import AuthClient  # type: ignore[import-not-found]
from vectorizer._base import AuthState, TransportRouter  # type: ignore[import-not-found]


def _make_auth() -> tuple[AuthClient, MagicMock]:
    transport = MagicMock()
    transport.get = AsyncMock()
    transport.post = AsyncMock()
    transport.put = AsyncMock()
    transport.delete = AsyncMock()
    client = AuthClient.__new__(AuthClient)
    client._transport = transport
    client._auth = AuthState()
    client._router = TransportRouter(primary=transport)
    client.base_url = "http://localhost:15002"
    return client, transport


class TestMe(unittest.TestCase):
    def test_returns_user(self):
        client, transport = _make_auth()
        transport.get.return_value = {"user_id": "u-1", "username": "alice", "roles": ["Admin"]}
        result = asyncio.run(client.me())
        transport.get.assert_awaited_once_with("/auth/me")
        self.assertIsInstance(result, User)
        self.assertEqual(result.username, "alice")
        self.assertEqual(result.roles, ["Admin"])

    def test_tolerates_empty_response(self):
        client, transport = _make_auth()
        transport.get.return_value = {}
        result = asyncio.run(client.me())
        self.assertEqual(result.user_id, "")


class TestLogout(unittest.TestCase):
    def test_posts_logout(self):
        client, transport = _make_auth()
        transport.post.return_value = {}
        asyncio.run(client.logout())
        transport.post.assert_awaited_once_with("/auth/logout")


class TestRefreshToken(unittest.TestCase):
    def test_returns_jwt_token(self):
        client, transport = _make_auth()
        transport.post.return_value = {
            "access_token": "eyJ...",
            "token_type": "Bearer",
            "expires_in": 3600,
        }
        result = asyncio.run(client.refresh_token())
        transport.post.assert_awaited_once_with("/auth/refresh", {})
        self.assertIsInstance(result, JwtToken)
        self.assertEqual(result.token_type, "Bearer")
        self.assertEqual(result.expires_in, 3600)


class TestValidatePassword(unittest.TestCase):
    def test_valid_password(self):
        client, transport = _make_auth()
        transport.post.return_value = {
            "valid": True, "errors": [], "strength": 80, "strength_label": "Strong"
        }
        result = asyncio.run(client.validate_password("S3cur3Pass!"))
        transport.post.assert_awaited_once_with(
            "/auth/validate-password", {"password": "S3cur3Pass!"}
        )
        self.assertIsInstance(result, PasswordPolicyReport)
        self.assertTrue(result.valid)
        self.assertEqual(result.strength, 80)

    def test_invalid_password(self):
        client, transport = _make_auth()
        transport.post.return_value = {
            "valid": False, "errors": ["too short"], "strength": 10, "strength_label": "Very Weak"
        }
        result = asyncio.run(client.validate_password("abc"))
        self.assertFalse(result.valid)
        self.assertIn("too short", result.errors)


class TestCreateApiKey(unittest.TestCase):
    def test_posts_and_returns_api_key(self):
        client, transport = _make_auth()
        transport.post.return_value = {
            "id": "k-1", "name": "ci-bot", "permissions": ["Read"],
            "api_key": "sk-abc123", "created_at": 1714608000, "active": True,
        }
        req = CreateApiKeyRequest(name="ci-bot", permissions=["Read"])
        result = asyncio.run(client.create_api_key(req))
        transport.post.assert_awaited_once_with(
            "/auth/keys", data={"name": "ci-bot", "permissions": ["Read"]}
        )
        self.assertIsInstance(result, ApiKey)
        self.assertEqual(result.api_key, "sk-abc123")
        self.assertTrue(result.active)

    def test_with_expires_in(self):
        client, transport = _make_auth()
        transport.post.return_value = {
            "id": "k-2", "name": "deploy", "permissions": [], "created_at": 0, "active": False
        }
        req = CreateApiKeyRequest(name="deploy", permissions=[], expires_in=86400)
        asyncio.run(client.create_api_key(req))
        call_kwargs = transport.post.call_args
        self.assertEqual(call_kwargs[1]["data"]["expires_in"], 86400)


class TestListApiKeys(unittest.TestCase):
    def test_returns_list(self):
        client, transport = _make_auth()
        transport.get.return_value = {
            "keys": [
                {"id": "k-1", "name": "ci", "permissions": [], "created_at": 0, "active": True},
                {"id": "k-2", "name": "deploy", "permissions": [], "created_at": 0, "active": False},
            ]
        }
        result = asyncio.run(client.list_api_keys())
        transport.get.assert_awaited_once_with("/auth/keys")
        self.assertEqual(len(result), 2)
        self.assertIsNone(result[0].api_key)  # not present in list responses


class TestRevokeApiKey(unittest.TestCase):
    def test_deletes_key(self):
        client, transport = _make_auth()
        transport.delete.return_value = {}
        asyncio.run(client.revoke_api_key("k-99"))
        transport.delete.assert_awaited_once_with("/auth/keys/k-99")


class TestCreateUser(unittest.TestCase):
    def test_posts_and_returns_user(self):
        client, transport = _make_auth()
        transport.post.return_value = {
            "user_id": "u-2", "username": "charlie", "roles": ["User"]
        }
        req = CreateUserRequest(username="charlie", password="P@ss!", roles=["User"])
        result = asyncio.run(client.create_user(req))
        transport.post.assert_awaited_once_with(
            "/auth/users",
            data={"username": "charlie", "password": "P@ss!", "roles": ["User"]},
        )
        self.assertIsInstance(result, User)
        self.assertEqual(result.username, "charlie")


class TestListUsers(unittest.TestCase):
    def test_returns_user_list(self):
        client, transport = _make_auth()
        transport.get.return_value = {
            "users": [
                {"user_id": "u-1", "username": "alice", "roles": ["Admin"]},
                {"user_id": "u-2", "username": "bob", "roles": ["User"]},
            ]
        }
        result = asyncio.run(client.list_users())
        transport.get.assert_awaited_once_with("/auth/users")
        self.assertEqual(len(result), 2)
        self.assertEqual(result[1].username, "bob")


class TestDeleteUser(unittest.TestCase):
    def test_deletes_user(self):
        client, transport = _make_auth()
        transport.delete.return_value = {}
        asyncio.run(client.delete_user("alice"))
        transport.delete.assert_awaited_once_with("/auth/users/alice")


class TestChangePassword(unittest.TestCase):
    def test_puts_password(self):
        client, transport = _make_auth()
        transport.put.return_value = {}
        asyncio.run(client.change_password("alice", "NewP@ss1!"))
        transport.put.assert_awaited_once_with(
            "/auth/users/alice/password",
            data={"new_password": "NewP@ss1!"},
        )


if __name__ == "__main__":
    unittest.main()
