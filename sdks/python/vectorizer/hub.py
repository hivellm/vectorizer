"""HiveHub surface.

Covers user-scoped backup management (``/hub/backups/*``), usage
statistics (``/hub/usage/*``), and API key validation
(``/hub/validate-key``).

These endpoints are only meaningful when the server is running in
HiveHub cluster mode. Calling them on a standalone instance returns a
503 that surfaces as a server error.
"""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

try:
    from ..models import (
        CreateUserBackupRequest,
        HubApiKeyValidation,
        QuotaInfo,
        RestoreUserBackupRequest,
        UploadUserBackupRequest,
        UsageStatistics,
        UserBackup,
    )
except ImportError:  # pragma: no cover
    from models import (  # type: ignore[import-not-found]
        CreateUserBackupRequest,
        HubApiKeyValidation,
        QuotaInfo,
        RestoreUserBackupRequest,
        UploadUserBackupRequest,
        UsageStatistics,
        UserBackup,
    )

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class HubClient(_ApiBase):
    """HiveHub backup management, usage statistics, and API key validation."""

    async def list_user_backups(self, user_id: str) -> List[UserBackup]:
        """List all backups owned by a user.

        Calls ``GET /hub/backups?user_id={user_id}``.

        Args:
            user_id: User UUID whose backups to list.

        Returns:
            List of :class:`UserBackup` entries.
        """
        data = await self._transport.get(f"/hub/backups?user_id={user_id}")
        raw = data.get("backups", []) if isinstance(data, dict) else []
        return [UserBackup.from_dict(b) for b in raw]

    async def create_user_backup(
        self, request: CreateUserBackupRequest
    ) -> UserBackup:
        """Create a new backup for a user.

        Calls ``POST /hub/backups`` with ``{user_id, name, description?, collections?}``.

        Args:
            request: :class:`CreateUserBackupRequest` with user_id, name, etc.

        Returns:
            :class:`UserBackup` for the newly created backup.
        """
        payload: Dict[str, Any] = {
            "user_id": request.user_id,
            "name": request.name,
        }
        if request.description is not None:
            payload["description"] = request.description
        if request.collections is not None:
            payload["collections"] = request.collections
        data = await self._transport.post("/hub/backups", data=payload)
        if isinstance(data, dict):
            backup_data = data.get("backup", data)
        else:
            backup_data = {}
        return UserBackup.from_dict(backup_data)

    async def restore_user_backup(
        self, request: RestoreUserBackupRequest
    ) -> None:
        """Restore a previously created user backup.

        Calls ``POST /hub/backups/restore``.

        Args:
            request: :class:`RestoreUserBackupRequest` with user_id, backup_id, overwrite.
        """
        payload: Dict[str, Any] = {
            "user_id": request.user_id,
            "backup_id": request.backup_id,
            "overwrite": request.overwrite,
        }
        await self._transport.post("/hub/backups/restore", data=payload)

    async def upload_user_backup(
        self, request: UploadUserBackupRequest
    ) -> UserBackup:
        """Upload a backup file (raw bytes via JSON-encoded payload).

        Calls ``POST /hub/backups/upload?user_id={user_id}&name={name}``.

        The SDK sends the binary data as a JSON ``{data: [...]}`` body
        because the transport abstraction's ``post`` takes JSON.
        For production uploads of large binaries use the raw HTTP client.

        Args:
            request: :class:`UploadUserBackupRequest` with user_id, optional name,
                and raw bytes in ``data``.

        Returns:
            :class:`UserBackup` metadata for the uploaded backup.
        """
        qs = f"user_id={request.user_id}"
        if request.name is not None:
            qs += f"&name={request.name}"
        endpoint = f"/hub/backups/upload?{qs}"
        payload: Dict[str, Any] = {"data": list(request.data)}
        data = await self._transport.post(endpoint, data=payload)
        if isinstance(data, dict):
            backup_data = data.get("backup", data)
        else:
            backup_data = {}
        return UserBackup.from_dict(backup_data)

    async def get_user_backup(self, user_id: str, backup_id: str) -> UserBackup:
        """Fetch metadata for a single backup.

        Calls ``GET /hub/backups/{backup_id}?user_id={user_id}``.

        Args:
            user_id: User UUID who owns the backup.
            backup_id: Backup UUID to fetch.

        Returns:
            :class:`UserBackup` metadata.
        """
        data = await self._transport.get(
            f"/hub/backups/{backup_id}?user_id={user_id}"
        )
        if isinstance(data, dict):
            backup_data = data.get("backup", data)
        else:
            backup_data = {}
        return UserBackup.from_dict(backup_data)

    async def delete_user_backup(self, user_id: str, backup_id: str) -> None:
        """Delete a user backup by id.

        Calls ``DELETE /hub/backups/{backup_id}?user_id={user_id}``.

        Args:
            user_id: User UUID who owns the backup.
            backup_id: Backup UUID to delete.
        """
        await self._transport.delete(
            f"/hub/backups/{backup_id}?user_id={user_id}"
        )

    async def download_user_backup(
        self, user_id: str, backup_id: str
    ) -> bytes:
        """Download the raw binary data for a backup.

        Calls ``GET /hub/backups/{backup_id}/download?user_id={user_id}``.

        The transport returns the response as a dict/str; the SDK
        re-encodes as UTF-8 bytes. For compressed binary backups use
        the raw HTTP client.

        Args:
            user_id: User UUID who owns the backup.
            backup_id: Backup UUID to download.

        Returns:
            Response body as bytes.
        """
        data = await self._transport.get(
            f"/hub/backups/{backup_id}/download?user_id={user_id}"
        )
        if isinstance(data, (bytes, bytearray)):
            return bytes(data)
        if isinstance(data, str):
            return data.encode()
        return b""

    async def get_usage_statistics(self, user_id: str) -> UsageStatistics:
        """Get aggregate usage statistics for a user.

        Calls ``GET /hub/usage/statistics?user_id={user_id}``.

        Args:
            user_id: User UUID whose statistics to retrieve.

        Returns:
            :class:`UsageStatistics` with success flag and stats payload.
        """
        data = await self._transport.get(
            f"/hub/usage/statistics?user_id={user_id}"
        )
        return UsageStatistics.from_dict(data if isinstance(data, dict) else {})

    async def get_quota_info(self, user_id: str) -> QuotaInfo:
        """Get quota information for a user.

        Calls ``GET /hub/usage/quota?user_id={user_id}``.

        Args:
            user_id: User UUID whose quota to retrieve.

        Returns:
            :class:`QuotaInfo` with success flag and quota payload.
        """
        data = await self._transport.get(
            f"/hub/usage/quota?user_id={user_id}"
        )
        return QuotaInfo.from_dict(data if isinstance(data, dict) else {})

    async def validate_hub_api_key(self, key: str) -> HubApiKeyValidation:
        """Validate a HiveHub API key.

        Calls ``POST /hub/validate-key`` with ``{key}``.

        The ``key`` parameter is the key to validate — it may differ
        from the key configured on the transport.

        Args:
            key: The HiveHub API key string to validate.

        Returns:
            :class:`HubApiKeyValidation` with valid flag, tenant info, and permissions.
        """
        data = await self._transport.post("/hub/validate-key", data={"key": key})
        return HubApiKeyValidation.from_dict(data if isinstance(data, dict) else {})
