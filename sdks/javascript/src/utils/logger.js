/**
 * Logger utility for the Hive Vectorizer SDK.
 */

/* eslint-disable no-console */
export class DefaultLogger {
  constructor(config = {}) {
    this.config = {
      level: 'info',
      enabled: true,
      prefix: '[Vectorizer]',
      ...config,
    };
    
    this.levels = {
      debug: 0,
      info: 1,
      warn: 2,
      error: 3,
    };
  }

  shouldLog(level) {
    return this.config.enabled && this.levels[level] >= this.levels[this.config.level];
  }

  formatMessage(level, message) {
    const timestamp = new Date().toISOString();
    return `${timestamp} ${this.config.prefix} [${level.toUpperCase()}] ${message}`;
  }

  debug(message, ...args) {
    if (this.shouldLog('debug')) {
      console.debug(this.formatMessage('debug', message), ...args);
    }
  }

  info(message, ...args) {
    if (this.shouldLog('info')) {
      console.info(this.formatMessage('info', message), ...args);
    }
  }

  warn(message, ...args) {
    if (this.shouldLog('warn')) {
      console.warn(this.formatMessage('warn', message), ...args);
    }
  }

  error(message, ...args) {
    if (this.shouldLog('error')) {
      console.error(this.formatMessage('error', message), ...args);
    }
  }
}

export class NoOpLogger {
  debug() {
    // No-op
  }

  info() {
    // No-op
  }

  warn() {
    // No-op
  }

  error() {
    // No-op
  }
}

/**
 * Creates a logger instance.
 */
export function createLogger(config = {}) {
  if (config.enabled === false) {
    return new NoOpLogger();
  }
  return new DefaultLogger(config);
}

/**
 * Default logger instance.
 */
export const logger = createLogger();
