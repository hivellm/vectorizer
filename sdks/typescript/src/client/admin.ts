/**
 * Admin / dashboard surface.
 *
 * Wraps the GUI-facing endpoints used by the dashboard: server status,
 * logs, workspaces, backups, server config, and the destructive
 * `restartServer` admin call.
 */

import { BaseClient } from './_base';

export class AdminClient extends BaseClient {
  /** Server status (uptime, collection / vector counts). */
  public async getStatus(): Promise<{
    status: string;
    version: string;
    uptime: number;
    collections: number;
    total_vectors: number;
  }> {
    this.logger.debug('Getting server status');
    return this.transport.get('/status');
  }

  /** Recent log lines. */
  public async getLogs(params?: { lines?: number; level?: string }): Promise<{ logs: string[] }> {
    this.logger.debug('Getting logs', params);
    return this.transport.get('/logs', params ? { params } : undefined);
  }

  /** Force-flush a collection to disk. */
  public async forceSaveCollection(
    name: string,
  ): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Force saving collection', { name });
    return this.transport.post(`/collections/${name}/force-save`, {});
  }

  /** Register a workspace folder for indexing. */
  public async addWorkspace(params: {
    name: string;
    path: string;
    collections: Array<{
      name: string;
      path: string;
      exclude_patterns?: string[];
    }>;
  }): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Adding workspace', params);
    return this.transport.post('/workspace/add', params);
  }

  /** Detach a workspace. */
  public async removeWorkspace(params: {
    name: string;
  }): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Removing workspace', params);
    return this.transport.post('/workspace/remove', params);
  }

  /** List configured workspaces. */
  public async listWorkspaces(): Promise<{
    workspaces: Array<{
      name: string;
      path: string;
      collections: number;
    }>;
  }> {
    this.logger.debug('Listing workspaces');
    return this.transport.get('/workspace/list');
  }

  /** Read-only server config snapshot. */
  public async getServerConfig(): Promise<Record<string, unknown>> {
    this.logger.debug('Getting server configuration');
    return this.transport.get('/config');
  }

  /** Update server config (subset of fields). */
  public async updateConfig(
    config: Record<string, unknown>,
  ): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Updating server configuration', config);
    return this.transport.post('/config', config);
  }

  /** Restart the running server (admin-only). */
  public async restartServer(): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Requesting server restart');
    return this.transport.post('/admin/restart', {});
  }

  /** List backup archives on disk. */
  public async listBackups(): Promise<{
    backups: Array<{
      filename: string;
      size: number;
      created_at: string;
    }>;
  }> {
    this.logger.debug('Listing backups');
    return this.transport.get('/backups/list');
  }

  /** Trigger a new backup. */
  public async createBackup(params?: { name?: string }): Promise<{
    success: boolean;
    message: string;
    filename?: string;
  }> {
    this.logger.debug('Creating backup', params);
    return this.transport.post('/backups/create', params || {});
  }

  /** Restore from a backup archive. */
  public async restoreBackup(params: {
    filename: string;
  }): Promise<{ success: boolean; message: string }> {
    this.logger.debug('Restoring backup', params);
    return this.transport.post('/backups/restore', params);
  }

  /** Where backups are written. */
  public async getBackupDirectory(): Promise<{ directory: string }> {
    this.logger.debug('Getting backup directory');
    return this.transport.get('/backups/directory');
  }
}
