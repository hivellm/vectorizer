/**
 * Minimal RPC connection pool.
 *
 * A bounded pool of {@link RpcClient} connections. {@link acquire}
 * returns an idle client (or builds a new one if the pool isn't at
 * capacity); the returned guard returns the client to the pool when
 * `release()` is called or when used with `using` (TC39 disposable).
 *
 * Intentionally NOT a full retry/health-check pool — those bring
 * complexity that the v1 SDK doesn't need. If a future workload
 * requires fancier pooling (per-connection health checks, idle
 * eviction), swap to a real pool implementation at that point.
 */

import { HelloPayload, RpcClient, RpcClientError } from './client';

/** Configuration for {@link RpcPool}. */
export interface RpcPoolConfig {
  /** `host:port` every connection in the pool dials. */
  address: string;
  /**
   * Maximum number of concurrent open connections. Calls block on
   * `acquire()` once this many are checked out. Defaults to 8.
   */
  maxConnections?: number;
  /** HELLO payload sent on every newly-built connection. */
  hello?: HelloPayload;
  /** Per-connection connect timeout in ms. */
  connectTimeoutMs?: number;
}

/** Guard returned by {@link RpcPool.acquire}. */
export class PooledClient {
  private released = false;

  constructor(
    private pool: RpcPool,
    private inner: RpcClient,
  ) {}

  /** Borrow the underlying client. Throws if `release()` has run. */
  client(): RpcClient {
    if (this.released) {
      throw new RpcClientError('PooledClient already released');
    }
    return this.inner;
  }

  /** Return the client to the pool. Subsequent calls are no-ops. */
  release(): void {
    if (this.released) return;
    this.released = true;
    this.pool._return(this.inner);
  }
}

/**
 * Bounded pool of {@link RpcClient} connections.
 *
 * Does NOT open any connections eagerly; the first `acquire()`
 * dials the first connection. `maxConnections` is enforced so
 * simultaneous acquires beyond the cap wait until a slot frees.
 */
export class RpcPool {
  private readonly maxConnections: number;
  private readonly hello: HelloPayload;
  private readonly idle: RpcClient[] = [];
  private inUse = 0;
  private readonly waiters: Array<() => void> = [];

  constructor(private readonly config: RpcPoolConfig) {
    this.maxConnections = Math.max(1, config.maxConnections ?? 8);
    this.hello = config.hello ?? {};
  }

  /**
   * Acquire a client from the pool. Resolves once a slot is free.
   * Returned {@link PooledClient}'s `release()` puts the connection
   * back in the pool for reuse.
   */
  async acquire(): Promise<PooledClient> {
    while (this.inUse >= this.maxConnections) {
      await new Promise<void>((resolve) => this.waiters.push(resolve));
    }
    this.inUse += 1;
    let client = this.idle.pop();
    if (client === undefined) {
      try {
        client = await RpcClient.connect(this.config.address, {
          timeoutMs: this.config.connectTimeoutMs ?? 10_000,
        });
        await client.hello(this.hello);
      } catch (err) {
        // Building failed — release the slot so other acquires don't
        // hang waiting for a connection that's not actually held.
        this.inUse -= 1;
        this.wakeNextWaiter();
        throw err;
      }
    }
    return new PooledClient(this, client);
  }

  /** Number of idle clients currently sitting in the pool. */
  idleCount(): number {
    return this.idle.length;
  }

  /**
   * Close every idle connection. In-flight clients (held by
   * callers) close on their own `release()` path or when the caller
   * drops the reference.
   */
  close(): void {
    while (this.idle.length > 0) {
      const c = this.idle.pop();
      c?.close();
    }
  }

  /** @internal — used by {@link PooledClient.release}. */
  _return(client: RpcClient): void {
    this.idle.push(client);
    this.inUse -= 1;
    this.wakeNextWaiter();
  }

  private wakeNextWaiter(): void {
    const next = this.waiters.shift();
    if (next !== undefined) next();
  }
}
