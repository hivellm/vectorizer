/**
 * Logger utility for the Hive Vectorizer SDK.
 */

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface Logger {
  debug(message: string, ...args: unknown[]): void;
  info(message: string, ...args: unknown[]): void;
  warn(message: string, ...args: unknown[]): void;
  error(message: string, ...args: unknown[]): void;
}

export interface LoggerConfig {
  level?: LogLevel;
  enabled?: boolean;
  prefix?: string;
}

class DefaultLogger implements Logger {
  private config: Required<LoggerConfig>;
  private levels: Record<LogLevel, number> = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3,
  };

  constructor(config: LoggerConfig = {}) {
    this.config = {
      level: 'info',
      enabled: true,
      prefix: '[Vectorizer]',
      ...config,
    };
  }

  private shouldLog(level: LogLevel): boolean {
    return this.config.enabled && this.levels[level] >= this.levels[this.config.level];
  }

  private formatMessage(level: LogLevel, message: string): string {
    const timestamp = new Date().toISOString();
    return `${timestamp} ${this.config.prefix} [${level.toUpperCase()}] ${message}`;
  }

  debug(message: string, ...args: unknown[]): void {
    if (this.shouldLog('debug')) {
      console.debug(this.formatMessage('debug', message), ...args);
    }
  }

  info(message: string, ...args: unknown[]): void {
    if (this.shouldLog('info')) {
      console.info(this.formatMessage('info', message), ...args);
    }
  }

  warn(message: string, ...args: unknown[]): void {
    if (this.shouldLog('warn')) {
      console.warn(this.formatMessage('warn', message), ...args);
    }
  }

  error(message: string, ...args: unknown[]): void {
    if (this.shouldLog('error')) {
      console.error(this.formatMessage('error', message), ...args);
    }
  }
}

class NoOpLogger implements Logger {
  debug(): void {
    // No-op
  }

  info(): void {
    // No-op
  }

  warn(): void {
    // No-op
  }

  error(): void {
    // No-op
  }
}

/**
 * Creates a logger instance.
 */
export function createLogger(config?: LoggerConfig): Logger {
  if (config?.enabled === false) {
    return new NoOpLogger();
  }
  return new DefaultLogger(config);
}

/**
 * Default logger instance.
 */
export const logger = createLogger();

