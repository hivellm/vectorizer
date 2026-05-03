/**
 * HiveHub surface.
 *
 * Covers user-scoped backup management (`/hub/backups/*`), usage
 * statistics (`/hub/usage/*`), and API key validation
 * (`/hub/validate-key`).
 *
 * These endpoints are only meaningful when the server is running in
 * HiveHub cluster mode. Calling them on a standalone instance returns
 * a 503.
 */

import { BaseClient } from './_base';
import type {
  CreateUserBackupRequest,
  HubApiKeyValidation,
  QuotaInfo,
  RestoreUserBackupRequest,
  UploadUserBackupRequest,
  UsageStatistics,
  UserBackup,
} from '../models';

export class HubClient extends BaseClient {
  /**
   * List all backups owned by a user.
   * Calls `GET /hub/backups?user_id={userId}`.
   */
  public async listUserBackups(userId: string): Promise<UserBackup[]> {
    this.logger.debug('Listing user backups', { userId });
    const response = await this.transport.get<{ backups: UserBackup[] }>(
      `/hub/backups?user_id=${encodeURIComponent(userId)}`,
    );
    return response.backups ?? [];
  }

  /**
   * Create a new backup for a user.
   * Calls `POST /hub/backups`.
   */
  public async createUserBackup(request: CreateUserBackupRequest): Promise<UserBackup> {
    this.logger.debug('Creating user backup', { userId: request.user_id, name: request.name });
    const response = await this.transport.post<{ backup?: UserBackup } & UserBackup>(
      '/hub/backups',
      request,
    );
    return (response as { backup?: UserBackup }).backup ?? (response as unknown as UserBackup);
  }

  /**
   * Restore a previously created user backup.
   * Calls `POST /hub/backups/restore`.
   */
  public async restoreUserBackup(request: RestoreUserBackupRequest): Promise<void> {
    this.logger.debug('Restoring user backup', {
      userId: request.user_id,
      backupId: request.backup_id,
    });
    await this.transport.post('/hub/backups/restore', request);
  }

  /**
   * Upload a backup file.
   * Calls `POST /hub/backups/upload?user_id={userId}&name={name}`.
   *
   * The `data` field in `UploadUserBackupRequest` should be a base64-encoded
   * string of the binary backup content.
   */
  public async uploadUserBackup(request: UploadUserBackupRequest): Promise<UserBackup> {
    let qs = `user_id=${encodeURIComponent(request.user_id)}`;
    if (request.name) {
      qs += `&name=${encodeURIComponent(request.name)}`;
    }
    this.logger.debug('Uploading user backup', { userId: request.user_id });
    const response = await this.transport.post<{ backup?: UserBackup } & UserBackup>(
      `/hub/backups/upload?${qs}`,
      { data: request.data },
    );
    return (response as { backup?: UserBackup }).backup ?? (response as unknown as UserBackup);
  }

  /**
   * Fetch metadata for a single backup.
   * Calls `GET /hub/backups/{backupId}?user_id={userId}`.
   */
  public async getUserBackup(userId: string, backupId: string): Promise<UserBackup> {
    this.logger.debug('Getting user backup', { userId, backupId });
    const response = await this.transport.get<{ backup?: UserBackup } & UserBackup>(
      `/hub/backups/${encodeURIComponent(backupId)}?user_id=${encodeURIComponent(userId)}`,
    );
    return (response as { backup?: UserBackup }).backup ?? (response as unknown as UserBackup);
  }

  /**
   * Delete a user backup by id.
   * Calls `DELETE /hub/backups/{backupId}?user_id={userId}`.
   */
  public async deleteUserBackup(userId: string, backupId: string): Promise<void> {
    this.logger.debug('Deleting user backup', { userId, backupId });
    await this.transport.delete(
      `/hub/backups/${encodeURIComponent(backupId)}?user_id=${encodeURIComponent(userId)}`,
    );
  }

  /**
   * Download the raw binary data for a backup.
   * Calls `GET /hub/backups/{backupId}/download?user_id={userId}`.
   *
   * Returns the response body as a string (the raw binary may need
   * further decoding by the caller for compressed backup archives).
   */
  public async downloadUserBackup(userId: string, backupId: string): Promise<string> {
    this.logger.debug('Downloading user backup', { userId, backupId });
    return this.transport.get<string>(
      `/hub/backups/${encodeURIComponent(backupId)}/download?user_id=${encodeURIComponent(userId)}`,
    );
  }

  /**
   * Get aggregate usage statistics for a user.
   * Calls `GET /hub/usage/statistics?user_id={userId}`.
   */
  public async getUsageStatistics(userId: string): Promise<UsageStatistics> {
    this.logger.debug('Getting usage statistics', { userId });
    return this.transport.get<UsageStatistics>(
      `/hub/usage/statistics?user_id=${encodeURIComponent(userId)}`,
    );
  }

  /**
   * Get quota information for a user.
   * Calls `GET /hub/usage/quota?user_id={userId}`.
   */
  public async getQuotaInfo(userId: string): Promise<QuotaInfo> {
    this.logger.debug('Getting quota info', { userId });
    return this.transport.get<QuotaInfo>(
      `/hub/usage/quota?user_id=${encodeURIComponent(userId)}`,
    );
  }

  /**
   * Validate a HiveHub API key.
   * Calls `POST /hub/validate-key`. The `key` parameter is forwarded
   * in the request body.
   */
  public async validateHubApiKey(key: string): Promise<HubApiKeyValidation> {
    this.logger.debug('Validating HiveHub API key');
    return this.transport.post<HubApiKeyValidation>('/hub/validate-key', { key });
  }
}
