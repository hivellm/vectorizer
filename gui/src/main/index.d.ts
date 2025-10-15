/// <reference types="electron" />

declare module 'electron-store' {
  class Store {
    constructor(options?: { name?: string; defaults?: Record<string, unknown> });
    get(key: string): unknown;
    set(key: string, value: unknown): void;
    has(key: string): boolean;
    delete(key: string): void;
    clear(): void;
  }
  export = Store;
}

