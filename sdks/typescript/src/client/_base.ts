/**
 * Shared transport plumbing for every per-surface client in this folder.
 *
 * `BaseClient` owns the routing-aware `transport` field, the master/replica
 * helpers, the logger, and the config — everything per-surface classes need
 * to issue requests without knowing which transport is wired up. The
 * upcoming RPC client (`phase6_sdk-typescript-rpc`) plugs in by
 * implementing the same `Transport` shape and replacing the value returned
 * by `TransportFactory.create`.
 */

import { HttpClientConfig } from '../utils/http-client';
import { UMICPClientConfig } from '../utils/umicp-client';
import {
  ITransport,
  TransportFactory,
  TransportProtocol,
  parseConnectionString,
} from '../utils/transport';
import { createLogger, Logger, LoggerConfig } from '../utils/logger';
import { HostConfig, ReadOptions, ReadPreference } from '../models';

/** The transport contract every per-surface client routes through. */
export type Transport = ITransport;

export interface VectorizerClientConfig {
  /** Base URL for the Vectorizer API (HTTP/HTTPS) - for single node deployments */
  baseURL?: string;
  /** Host configuration for master/replica topology */
  hosts?: HostConfig;
  /** Read preference for routing read operations (default: 'replica' if hosts configured, otherwise N/A) */
  readPreference?: ReadPreference;
  /** Connection string (supports http://, https://, umicp://) */
  connectionString?: string;
  /** Transport protocol to use */
  protocol?: TransportProtocol;
  /** API key for authentication */
  apiKey?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Custom headers for requests (HTTP only) */
  headers?: Record<string, string>;
  /** UMICP-specific configuration */
  umicp?: Partial<UMICPClientConfig>;
  /** Logger configuration */
  logger?: LoggerConfig;
  /**
   * Pre-built transport. When supplied, all factory/connection-string
   * logic is skipped — used by tests (and by the upcoming RPC client) to
   * inject a custom `Transport` implementation.
   */
  transport?: ITransport;
}

export class BaseClient {
  protected transport!: ITransport;
  protected masterTransport?: ITransport;
  protected replicaTransports: ITransport[] = [];
  protected replicaIndex: number = 0;
  protected logger: Logger;
  protected config: VectorizerClientConfig;
  protected protocol!: TransportProtocol;
  protected readPreference: ReadPreference = 'replica';
  protected isReplicaMode: boolean = false;

  constructor(config: VectorizerClientConfig = {}) {
    this.config = {
      baseURL: 'http://localhost:15002',
      timeout: 30000,
      headers: {},
      logger: { level: 'info', enabled: true },
      readPreference: 'replica',
      ...config,
    };

    this.logger = createLogger(this.config.logger);
    this.readPreference = this.config.readPreference || 'replica';

    // Test/RPC injection point: caller supplied an already-built transport.
    if (config.transport) {
      this.transport = config.transport;
      this.protocol = (config.protocol || 'http') as TransportProtocol;
      return;
    }

    if (this.config.hosts) {
      this.initializeReplicaMode();
      return;
    }

    if (this.config.connectionString) {
      const transportConfig = parseConnectionString(
        this.config.connectionString,
        this.config.apiKey,
      );
      this.transport = TransportFactory.create(transportConfig);
      this.protocol = transportConfig.protocol!;

      this.logger.info('VectorizerClient initialized from connection string', {
        protocol: this.protocol,
        connectionString: this.config.connectionString,
        hasApiKey: !!this.config.apiKey,
      });
      return;
    }

    this.protocol = this.config.protocol || 'http';

    if (this.protocol === 'http') {
      const httpConfig: HttpClientConfig = {
        baseURL: this.config.baseURL!,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...(this.config.headers && { headers: this.config.headers }),
        ...(this.config.apiKey && { apiKey: this.config.apiKey }),
      };
      this.transport = TransportFactory.create({ protocol: 'http', http: httpConfig });

      this.logger.info('VectorizerClient initialized with HTTP', {
        baseURL: this.config.baseURL,
        hasApiKey: !!this.config.apiKey,
      });
    } else if (this.protocol === 'umicp') {
      if (!this.config.umicp) {
        throw new Error('UMICP configuration is required when using UMICP protocol');
      }

      const umicpConfig: UMICPClientConfig = {
        host: this.config.umicp.host || 'localhost',
        port: this.config.umicp.port || 15003,
        ...(this.config.apiKey && { apiKey: this.config.apiKey }),
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...this.config.umicp,
      };
      this.transport = TransportFactory.create({ protocol: 'umicp', umicp: umicpConfig });

      this.logger.info('VectorizerClient initialized with UMICP', {
        host: umicpConfig.host,
        port: umicpConfig.port,
        hasApiKey: !!this.config.apiKey,
      });
    }
  }

  private initializeReplicaMode(): void {
    const { hosts } = this.config;
    if (!hosts) return;

    this.isReplicaMode = true;
    this.protocol = 'http';

    const masterHttpConfig: HttpClientConfig = {
      baseURL: hosts.master,
      ...(this.config.timeout && { timeout: this.config.timeout }),
      ...(this.config.headers && { headers: this.config.headers }),
      ...(this.config.apiKey && { apiKey: this.config.apiKey }),
    };
    this.masterTransport = TransportFactory.create({ protocol: 'http', http: masterHttpConfig });

    this.replicaTransports = hosts.replicas.map((replicaUrl) => {
      const replicaHttpConfig: HttpClientConfig = {
        baseURL: replicaUrl,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...(this.config.headers && { headers: this.config.headers }),
        ...(this.config.apiKey && { apiKey: this.config.apiKey }),
      };
      return TransportFactory.create({ protocol: 'http', http: replicaHttpConfig });
    });

    this.transport = this.masterTransport;

    this.logger.info('VectorizerClient initialized with master/replica topology', {
      master: hosts.master,
      replicas: hosts.replicas,
      readPreference: this.readPreference,
      hasApiKey: !!this.config.apiKey,
    });
  }

  /** Transport for write operations (always master in replica mode). */
  protected getWriteTransport(): ITransport {
    if (this.isReplicaMode && this.masterTransport) {
      return this.masterTransport;
    }
    return this.transport;
  }

  /** Transport for read operations, honouring readPreference. */
  protected getReadTransport(options?: ReadOptions): ITransport {
    if (!this.isReplicaMode) {
      return this.transport;
    }

    const preference = options?.readPreference || this.readPreference;

    switch (preference) {
      case 'master':
        return this.masterTransport!;

      case 'replica': {
        if (this.replicaTransports.length === 0) {
          return this.masterTransport!;
        }
        const transport = this.replicaTransports[this.replicaIndex]!;
        this.replicaIndex = (this.replicaIndex + 1) % this.replicaTransports.length;
        return transport;
      }

      case 'nearest': {
        if (this.replicaTransports.length === 0) {
          return this.masterTransport!;
        }
        const nearestTransport = this.replicaTransports[this.replicaIndex]!;
        this.replicaIndex = (this.replicaIndex + 1) % this.replicaTransports.length;
        return nearestTransport;
      }

      default:
        return this.masterTransport!;
    }
  }

  /** Current transport protocol. */
  public getProtocol(): TransportProtocol {
    return this.protocol;
  }

  /** Snapshot of the active configuration. */
  public getConfig(): Readonly<VectorizerClientConfig> {
    return { ...this.config };
  }

  /** Update the API key in-place by rebuilding the underlying transport. */
  public setApiKey(apiKey: string): void {
    this.config.apiKey = apiKey;

    if (this.protocol === 'http' && this.config.baseURL) {
      const httpConfig: HttpClientConfig = {
        baseURL: this.config.baseURL,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...(this.config.headers && { headers: this.config.headers }),
        apiKey: this.config.apiKey,
      };
      this.transport = TransportFactory.create({ protocol: 'http', http: httpConfig });
    } else if (this.protocol === 'umicp' && this.config.umicp) {
      const umicpConfig: UMICPClientConfig = {
        host: this.config.umicp.host || 'localhost',
        port: this.config.umicp.port || 15003,
        apiKey: this.config.apiKey,
        ...(this.config.timeout && { timeout: this.config.timeout }),
        ...this.config.umicp,
      };
      this.transport = TransportFactory.create({ protocol: 'umicp', umicp: umicpConfig });
    }

    this.logger.info('API key updated');
  }

  /** Release any resources held by the client. */
  public async close(): Promise<void> {
    this.logger.info('VectorizerClient closed');
  }
}
