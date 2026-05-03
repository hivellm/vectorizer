/**
 * Admin / dashboard surface.
 *
 * Wraps the GUI-facing endpoints used by the dashboard: server status,
 * logs, workspaces, backups, server config, and the destructive
 * `restartServer` admin call. Phase12 adds: getStats, getIndexingProgress,
 * listEmptyCollections, cleanupEmptyCollections, getWorkspaceConfig.
 */

import { BaseClient } from './_base';
import type {
  AddWorkspaceRequest,
  BackupInfo,
  CleanupReport,
  CreateBackupRequest,
  IndexingProgress,
  LogEntry,
  RestoreBackupRequest,
  SlowQueryConfig,
  SlowQueryEntry,
  Stats,
} from '../models';

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

  /** Aggregate collection + vector counts. Calls `GET /stats`. */
  public async getStats(): Promise<Stats> {
    this.logger.debug('Getting server stats');
    return this.transport.get<Stats>('/stats');
  }

  /** Per-collection indexing progress. Calls `GET /indexing/progress`. */
  public async getIndexingProgress(): Promise<IndexingProgress> {
    this.logger.debug('Getting indexing progress');
    return this.transport.get<IndexingProgress>('/indexing/progress');
  }

  /** List collections that contain zero vectors. Calls `GET /collections/empty`. */
  public async listEmptyCollections(): Promise<string[]> {
    this.logger.debug('Listing empty collections');
    const response = await this.transport.get<string[] | { collections: string[] }>(
      '/collections/empty',
    );
    if (Array.isArray(response)) {
      return response;
    }
    return (response as { collections: string[] }).collections ?? [];
  }

  /** Delete all empty collections in one call. Calls `DELETE /collections/cleanup`. */
  public async cleanupEmptyCollections(): Promise<CleanupReport> {
    this.logger.debug('Cleaning up empty collections');
    return this.transport.delete<CleanupReport>('/collections/cleanup');
  }

  /** Read the workspace configuration file. Calls `GET /workspace/config`. */
  public async getWorkspaceConfig(): Promise<Record<string, unknown>> {
    this.logger.debug('Getting workspace config');
    return this.transport.get<Record<string, unknown>>('/workspace/config');
  }

  /** Typed log entries. Calls `GET /logs?lines=N&level=LEVEL`. */
  public async getLogEntries(params?: {
    lines?: number;
    level?: string;
  }): Promise<LogEntry[]> {
    this.logger.debug('Getting log entries', params);
    const qs = new URLSearchParams();
    if (params?.lines !== undefined) qs.set('lines', String(params.lines));
    if (params?.level) qs.set('level', params.level);
    const endpoint = qs.toString() ? `/logs?${qs}` : '/logs';
    const response = await this.transport.get<{ logs: LogEntry[] }>(endpoint);
    return response.logs ?? [];
  }

  /** List server-side backup files. Calls `GET /backups`. */
  public async listBackupInfos(): Promise<BackupInfo[]> {
    this.logger.debug('Listing backup infos');
    const response = await this.transport.get<{ backups: BackupInfo[] }>('/backups');
    return response.backups ?? [];
  }

  /** Create a new backup with typed request/response. Calls `POST /backups/create`. */
  public async createBackupTyped(request: CreateBackupRequest): Promise<BackupInfo> {
    this.logger.debug('Creating typed backup', request);
    return this.transport.post<BackupInfo>('/backups/create', request);
  }

  /** Restore a backup with typed request. Calls `POST /backups/restore`. */
  public async restoreBackupTyped(request: RestoreBackupRequest): Promise<void> {
    this.logger.debug('Restoring typed backup', request);
    await this.transport.post('/backups/restore', request);
  }

  /** Register a workspace directory with typed request. Calls `POST /workspace/add`. */
  public async addWorkspaceTyped(request: AddWorkspaceRequest): Promise<void> {
    this.logger.debug('Adding workspace', request);
    await this.transport.post('/workspace/add', request);
  }

  // ── Phase-14: observability ────────────────────────────────────────────────

  /**
   * List slow-query ring-buffer entries (phase14).
   *
   * Calls `GET /slow_queries`. Returns entries in the order they were
   * recorded (oldest first). Use {@link setSlowQueryConfig} to tune the
   * threshold and capacity.
   */
  public async listSlowQueries(): Promise<SlowQueryEntry[]> {
    this.logger.debug('Listing slow queries');
    const response = await this.transport.get<{
      entries: SlowQueryEntry[];
      total: number;
      config: SlowQueryConfig;
    }>('/slow_queries');
    return response.entries ?? [];
  }

  /**
   * Reconfigure the slow-query ring buffer (phase14).
   *
   * Calls `POST /slow_queries/config` with `{ threshold_ms, capacity }`.
   * Existing entries are retained; if the new capacity is smaller than the
   * current entry count the oldest entries are evicted by the server.
   */
  public async setSlowQueryConfig(config: SlowQueryConfig): Promise<SlowQueryConfig> {
    this.logger.debug('Setting slow query config', config);
    const response = await this.transport.post<SlowQueryConfig & { status: string }>(
      '/slow_queries/config',
      { threshold_ms: config.threshold_ms, capacity: config.capacity },
    );
    return { threshold_ms: response.threshold_ms, capacity: response.capacity };
  }
}
