/**
 * Base exception class for all Vectorizer-related errors.
 */

export interface ErrorDetails {
  [key: string]: unknown;
}

export class VectorizerError extends Error {
  public readonly errorCode: string;
  public readonly details: ErrorDetails;

  constructor(message: string, errorCode?: string, details?: ErrorDetails) {
    super(message);
    this.name = 'VectorizerError';
    this.errorCode = errorCode || 'VECTORIZER_ERROR';
    this.details = details || {};

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, VectorizerError);
    }
  }

  /**
   * Returns a string representation of the error.
   */
  public override toString(): string {
    if (this.errorCode) {
      return `[${this.errorCode}] ${this.message}`;
    }
    return this.message;
  }

  /**
   * Returns a JSON representation of the error.
   */
  public toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      errorCode: this.errorCode,
      details: this.details,
      stack: this.stack,
    };
  }
}
