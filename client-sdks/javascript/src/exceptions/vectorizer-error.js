/**
 * Base exception class for all Vectorizer-related errors.
 */

export class VectorizerError extends Error {
  constructor(message, errorCode = 'VECTORIZER_ERROR', details = {}) {
    super(message);
    this.name = 'VectorizerError';
    this.errorCode = errorCode;
    this.details = details;

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, VectorizerError);
    }
  }

  /**
   * Returns a string representation of the error.
   */
  toString() {
    if (this.errorCode) {
      return `[${this.errorCode}] ${this.message}`;
    }
    return this.message;
  }

  /**
   * Returns a JSON representation of the error.
   */
  toJSON() {
    return {
      name: this.name,
      message: this.message,
      errorCode: this.errorCode,
      details: this.details,
      stack: this.stack,
    };
  }
}
