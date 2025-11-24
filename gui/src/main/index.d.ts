/// <reference types="electron" />

declare module 'electron-store' {
  class Store {
    constructor(options?: { 
      name?: string; 
      defaults?: Record<string, unknown>;
      encryptionKey?: string;
      schema?: Record<string, any>;
    });
    get(key: string): unknown;
    set(key: string, value: unknown): void;
    has(key: string): boolean;
    delete(key: string): void;
    clear(): void;
    path: string;
  }
  export = Store;
}

