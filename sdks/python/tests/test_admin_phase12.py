"""Unit tests for the phase12 AdminClient additions.

Tests: get_stats, get_status, get_logs, force_save_collection,
list_empty_collections, cleanup_empty_collections, get_config,
update_config, list_backups, create_backup, restore_backup,
restart_server, list_workspaces, get_workspace_config,
add_workspace, remove_workspace.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import (  # type: ignore[import-not-found]
    AddWorkspaceRequest,
    BackupInfo,
    CleanupReport,
    CreateBackupRequest,
    LogEntry,
    LogsQuery,
    RestoreBackupRequest,
    ServerStatus,
    Stats,
)
from vectorizer.admin import AdminClient  # type: ignore[import-not-found]


def _make_admin() -> tuple[AdminClient, MagicMock]:
    client = AdminClient.__new__(AdminClient)
    transport = MagicMock()
    transport.get = AsyncMock()
    transport.post = AsyncMock()
    transport.delete = AsyncMock()
    client._transport = transport
    return client, transport


class TestGetStats(unittest.TestCase):
    def test_returns_stats_dataclass(self):
        client, transport = _make_admin()
        transport.get.return_value = {
            "collections": 5,
            "total_vectors": 1000,
            "uptime_seconds": 3600,
            "version": "3.4.0",
        }
        result = asyncio.run(client.get_stats())
        transport.get.assert_awaited_once_with("/stats")
        self.assertIsInstance(result, Stats)
        self.assertEqual(result.collections, 5)
        self.assertEqual(result.total_vectors, 1000)
        self.assertEqual(result.version, "3.4.0")

    def test_tolerates_empty_response(self):
        client, transport = _make_admin()
        transport.get.return_value = {}
        result = asyncio.run(client.get_stats())
        self.assertEqual(result.collections, 0)
        self.assertEqual(result.version, "")


class TestGetStatus(unittest.TestCase):
    def test_returns_server_status(self):
        client, transport = _make_admin()
        transport.get.return_value = {
            "online": True,
            "version": "3.4.0",
            "uptime_seconds": 120,
            "collections_count": 3,
        }
        result = asyncio.run(client.get_status())
        transport.get.assert_awaited_once_with("/status")
        self.assertIsInstance(result, ServerStatus)
        self.assertTrue(result.online)
        self.assertEqual(result.collections_count, 3)


class TestGetLogs(unittest.TestCase):
    def test_returns_log_entries(self):
        client, transport = _make_admin()
        transport.get.return_value = {
            "logs": [
                {"timestamp": "T", "level": "INFO", "message": "ok", "source": "srv"},
            ]
        }
        result = asyncio.run(client.get_logs())
        transport.get.assert_awaited_once_with("/logs")
        self.assertEqual(len(result), 1)
        self.assertIsInstance(result[0], LogEntry)
        self.assertEqual(result[0].level, "INFO")

    def test_with_params(self):
        client, transport = _make_admin()
        transport.get.return_value = {"logs": []}
        asyncio.run(client.get_logs(LogsQuery(lines=50, level="ERROR")))
        transport.get.assert_awaited_once_with("/logs?lines=50&level=ERROR")

    def test_tolerates_non_dict_response(self):
        client, transport = _make_admin()
        transport.get.return_value = None
        result = asyncio.run(client.get_logs())
        self.assertEqual(result, [])


class TestForceSave(unittest.TestCase):
    def test_calls_correct_endpoint(self):
        client, transport = _make_admin()
        transport.post.return_value = {}
        asyncio.run(client.force_save_collection("my_col"))
        transport.post.assert_awaited_once_with("/collections/my_col/force-save")


class TestListEmptyCollections(unittest.TestCase):
    def test_dict_response(self):
        client, transport = _make_admin()
        transport.get.return_value = {"collections": ["empty1", "empty2"]}
        result = asyncio.run(client.list_empty_collections())
        self.assertEqual(result, ["empty1", "empty2"])

    def test_list_response(self):
        client, transport = _make_admin()
        transport.get.return_value = ["empty1"]
        result = asyncio.run(client.list_empty_collections())
        self.assertEqual(result, ["empty1"])


class TestCleanupEmptyCollections(unittest.TestCase):
    def test_returns_cleanup_report(self):
        client, transport = _make_admin()
        transport.delete.return_value = {
            "success": True,
            "removed": 2,
            "collections": ["a", "b"],
            "message": "done",
        }
        result = asyncio.run(client.cleanup_empty_collections())
        transport.delete.assert_awaited_once_with("/collections/cleanup")
        self.assertIsInstance(result, CleanupReport)
        self.assertTrue(result.success)
        self.assertEqual(result.removed, 2)


class TestGetConfig(unittest.TestCase):
    def test_returns_dict(self):
        client, transport = _make_admin()
        transport.get.return_value = {"server": {"port": 15002}}
        result = asyncio.run(client.get_config())
        transport.get.assert_awaited_once_with("/config")
        self.assertEqual(result["server"]["port"], 15002)


class TestUpdateConfig(unittest.TestCase):
    def test_posts_and_returns_dict(self):
        client, transport = _make_admin()
        patch = {"embedding": {"provider": "fastembed"}}
        transport.post.return_value = patch
        result = asyncio.run(client.update_config(patch))
        transport.post.assert_awaited_once_with("/config", data=patch)
        self.assertEqual(result, patch)


class TestListBackups(unittest.TestCase):
    def test_returns_backup_info_list(self):
        client, transport = _make_admin()
        transport.get.return_value = {
            "backups": [
                {"id": "b1", "name": "weekly", "date": "2026-05-02", "size": 4096, "collections": ["docs"]},
            ]
        }
        result = asyncio.run(client.list_backups())
        transport.get.assert_awaited_once_with("/backups")
        self.assertEqual(len(result), 1)
        self.assertIsInstance(result[0], BackupInfo)
        self.assertEqual(result[0].id, "b1")


class TestCreateBackup(unittest.TestCase):
    def test_posts_and_returns_backup_info(self):
        client, transport = _make_admin()
        transport.post.return_value = {
            "id": "b2", "name": "nightly", "date": "2026-05-02", "size": 0, "collections": []
        }
        req = CreateBackupRequest(name="nightly", collections=[])
        result = asyncio.run(client.create_backup(req))
        transport.post.assert_awaited_once_with(
            "/backups/create", data={"name": "nightly", "collections": []}
        )
        self.assertIsInstance(result, BackupInfo)
        self.assertEqual(result.name, "nightly")


class TestRestoreBackup(unittest.TestCase):
    def test_posts_restore(self):
        client, transport = _make_admin()
        transport.post.return_value = {}
        asyncio.run(client.restore_backup(RestoreBackupRequest(backup_id="b99")))
        transport.post.assert_awaited_once_with("/backups/restore", data={"backup_id": "b99"})


class TestRestartServer(unittest.TestCase):
    def test_posts_restart(self):
        client, transport = _make_admin()
        transport.post.return_value = {}
        asyncio.run(client.restart_server())
        transport.post.assert_awaited_once_with("/admin/restart")


class TestListWorkspaces(unittest.TestCase):
    def test_returns_list(self):
        client, transport = _make_admin()
        transport.get.return_value = {"workspaces": [{"path": "/home/user"}]}
        result = asyncio.run(client.list_workspaces())
        transport.get.assert_awaited_once_with("/workspace/list")
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0]["path"], "/home/user")


class TestGetWorkspaceConfig(unittest.TestCase):
    def test_returns_dict(self):
        client, transport = _make_admin()
        transport.get.return_value = {"projects": []}
        result = asyncio.run(client.get_workspace_config())
        transport.get.assert_awaited_once_with("/workspace/config")
        self.assertIn("projects", result)


class TestAddWorkspace(unittest.TestCase):
    def test_posts_add_workspace(self):
        client, transport = _make_admin()
        transport.post.return_value = {}
        req = AddWorkspaceRequest(path="/home/user/proj", collection_name="proj_docs")
        asyncio.run(client.add_workspace(req))
        transport.post.assert_awaited_once_with(
            "/workspace/add",
            data={"path": "/home/user/proj", "collection_name": "proj_docs"},
        )


class TestRemoveWorkspace(unittest.TestCase):
    def test_posts_remove_workspace(self):
        client, transport = _make_admin()
        transport.post.return_value = {}
        asyncio.run(client.remove_workspace("/home/user/proj"))
        transport.post.assert_awaited_once_with(
            "/workspace/remove", data={"path": "/home/user/proj"}
        )


if __name__ == "__main__":
    unittest.main()
