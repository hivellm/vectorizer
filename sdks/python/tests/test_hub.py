"""Unit tests for HubClient (phase12).

Tests all 10 methods: list_user_backups, create_user_backup,
restore_user_backup, upload_user_backup, get_user_backup,
delete_user_backup, download_user_backup, get_usage_statistics,
get_quota_info, validate_hub_api_key.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import (  # type: ignore[import-not-found]
    CreateUserBackupRequest,
    HubApiKeyValidation,
    QuotaInfo,
    RestoreUserBackupRequest,
    UploadUserBackupRequest,
    UsageStatistics,
    UserBackup,
)
from vectorizer.hub import HubClient  # type: ignore[import-not-found]
from vectorizer._base import AuthState, TransportRouter  # type: ignore[import-not-found]


def _make_hub() -> tuple[HubClient, MagicMock]:
    transport = MagicMock()
    transport.get = AsyncMock()
    transport.post = AsyncMock()
    transport.delete = AsyncMock()
    client = HubClient.__new__(HubClient)
    client._transport = transport
    client._auth = AuthState()
    client._router = TransportRouter(primary=transport)
    client.base_url = "http://localhost:15002"
    return client, transport


_BACKUP_DICT = {
    "id": "b-1", "user_id": "u-1", "name": "my-backup",
    "collections": ["docs"], "created_at": "2026-05-02T00:00:00Z",
    "size": 8192, "status": "active",
}


class TestListUserBackups(unittest.TestCase):
    def test_returns_list(self):
        client, transport = _make_hub()
        transport.get.return_value = {"backups": [_BACKUP_DICT]}
        result = asyncio.run(client.list_user_backups("u-1"))
        transport.get.assert_awaited_once_with("/hub/backups?user_id=u-1")
        self.assertEqual(len(result), 1)
        self.assertIsInstance(result[0], UserBackup)
        self.assertEqual(result[0].id, "b-1")

    def test_empty_list(self):
        client, transport = _make_hub()
        transport.get.return_value = {"backups": []}
        result = asyncio.run(client.list_user_backups("u-2"))
        self.assertEqual(result, [])


class TestCreateUserBackup(unittest.TestCase):
    def test_posts_and_returns_backup(self):
        client, transport = _make_hub()
        transport.post.return_value = {"backup": _BACKUP_DICT}
        req = CreateUserBackupRequest(user_id="u-1", name="my-backup")
        result = asyncio.run(client.create_user_backup(req))
        transport.post.assert_awaited_once_with(
            "/hub/backups", data={"user_id": "u-1", "name": "my-backup"}
        )
        self.assertIsInstance(result, UserBackup)
        self.assertEqual(result.status, "active")

    def test_flat_response_without_backup_key(self):
        client, transport = _make_hub()
        transport.post.return_value = _BACKUP_DICT
        req = CreateUserBackupRequest(user_id="u-1", name="my-backup")
        result = asyncio.run(client.create_user_backup(req))
        self.assertEqual(result.id, "b-1")

    def test_description_and_collections_included(self):
        client, transport = _make_hub()
        transport.post.return_value = {"backup": _BACKUP_DICT}
        req = CreateUserBackupRequest(
            user_id="u-1", name="full", description="desc", collections=["docs"]
        )
        asyncio.run(client.create_user_backup(req))
        call_data = transport.post.call_args[1]["data"]
        self.assertEqual(call_data["description"], "desc")
        self.assertEqual(call_data["collections"], ["docs"])


class TestRestoreUserBackup(unittest.TestCase):
    def test_posts_restore(self):
        client, transport = _make_hub()
        transport.post.return_value = {}
        req = RestoreUserBackupRequest(user_id="u-1", backup_id="b-1", overwrite=True)
        asyncio.run(client.restore_user_backup(req))
        transport.post.assert_awaited_once_with(
            "/hub/backups/restore",
            data={"user_id": "u-1", "backup_id": "b-1", "overwrite": True},
        )


class TestUploadUserBackup(unittest.TestCase):
    def test_posts_with_query_params(self):
        client, transport = _make_hub()
        transport.post.return_value = {"backup": _BACKUP_DICT}
        req = UploadUserBackupRequest(user_id="u-1", name="weekly", data=b"\x01\x02")
        result = asyncio.run(client.upload_user_backup(req))
        call_args = transport.post.call_args
        self.assertIn("user_id=u-1", call_args[0][0])
        self.assertIn("name=weekly", call_args[0][0])
        self.assertIsInstance(result, UserBackup)

    def test_without_name(self):
        client, transport = _make_hub()
        transport.post.return_value = _BACKUP_DICT
        req = UploadUserBackupRequest(user_id="u-1", data=b"\x00")
        asyncio.run(client.upload_user_backup(req))
        url = transport.post.call_args[0][0]
        self.assertNotIn("name=", url)


class TestGetUserBackup(unittest.TestCase):
    def test_returns_backup(self):
        client, transport = _make_hub()
        transport.get.return_value = {"backup": _BACKUP_DICT}
        result = asyncio.run(client.get_user_backup("u-1", "b-1"))
        transport.get.assert_awaited_once_with("/hub/backups/b-1?user_id=u-1")
        self.assertIsInstance(result, UserBackup)
        self.assertEqual(result.size, 8192)


class TestDeleteUserBackup(unittest.TestCase):
    def test_deletes(self):
        client, transport = _make_hub()
        transport.delete.return_value = {}
        asyncio.run(client.delete_user_backup("u-1", "b-1"))
        transport.delete.assert_awaited_once_with("/hub/backups/b-1?user_id=u-1")


class TestDownloadUserBackup(unittest.TestCase):
    def test_returns_bytes_from_string(self):
        client, transport = _make_hub()
        transport.get.return_value = "binary-data"
        result = asyncio.run(client.download_user_backup("u-1", "b-1"))
        transport.get.assert_awaited_once_with("/hub/backups/b-1/download?user_id=u-1")
        self.assertIsInstance(result, bytes)
        self.assertEqual(result, b"binary-data")

    def test_returns_bytes_from_bytes(self):
        client, transport = _make_hub()
        transport.get.return_value = b"\x01\x02\x03"
        result = asyncio.run(client.download_user_backup("u-1", "b-1"))
        self.assertEqual(result, b"\x01\x02\x03")

    def test_returns_empty_for_none(self):
        client, transport = _make_hub()
        transport.get.return_value = None
        result = asyncio.run(client.download_user_backup("u-1", "b-1"))
        self.assertEqual(result, b"")


class TestGetUsageStatistics(unittest.TestCase):
    def test_returns_usage_statistics(self):
        client, transport = _make_hub()
        transport.get.return_value = {
            "success": True, "message": "ok",
            "stats": {"user_id": "u-1", "total_vectors": 500}
        }
        result = asyncio.run(client.get_usage_statistics("u-1"))
        transport.get.assert_awaited_once_with("/hub/usage/statistics?user_id=u-1")
        self.assertIsInstance(result, UsageStatistics)
        self.assertTrue(result.success)
        self.assertIsNotNone(result.stats)


class TestGetQuotaInfo(unittest.TestCase):
    def test_returns_quota_info(self):
        client, transport = _make_hub()
        transport.get.return_value = {
            "success": True, "message": "ok",
            "quota": {"tenant_id": "t-1", "storage": {"limit": 1000000, "used": 50000}}
        }
        result = asyncio.run(client.get_quota_info("u-1"))
        transport.get.assert_awaited_once_with("/hub/usage/quota?user_id=u-1")
        self.assertIsInstance(result, QuotaInfo)
        self.assertTrue(result.success)


class TestValidateHubApiKey(unittest.TestCase):
    def test_valid_key(self):
        client, transport = _make_hub()
        transport.post.return_value = {
            "valid": True, "tenant_id": "t-abc", "tenant_name": "Acme",
            "permissions": ["Read", "Write"], "validated_at": "2026-05-02T00:00:00Z",
        }
        result = asyncio.run(client.validate_hub_api_key("hub-key-123"))
        transport.post.assert_awaited_once_with(
            "/hub/validate-key", data={"key": "hub-key-123"}
        )
        self.assertIsInstance(result, HubApiKeyValidation)
        self.assertTrue(result.valid)
        self.assertEqual(result.tenant_id, "t-abc")
        self.assertEqual(len(result.permissions), 2)

    def test_invalid_key(self):
        client, transport = _make_hub()
        transport.post.return_value = {
            "valid": False, "tenant_id": "", "tenant_name": "",
            "permissions": [], "validated_at": "",
        }
        result = asyncio.run(client.validate_hub_api_key("bad-key"))
        self.assertFalse(result.valid)


if __name__ == "__main__":
    unittest.main()
