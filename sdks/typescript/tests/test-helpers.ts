/**
 * Test Helper: Server Availability Check + Live-gate for integration suites.
 *
 * Every test file that exercises the live Vectorizer REST surface
 * should route through `runIfServer(...)` / `runItIfServer(...)` so
 * the suite runs when a server is reachable and cleanly skips
 * (with a "requires VECTORIZER_LIVE_TESTS=1 or reachable live
 * server at VECTORIZER_BASE_URL" reason) when it is not.
 *
 * Pattern introduced by phase8_enable-typescript-sdk-live-tests
 * (probe 4.1): replaced 3 unconditional `describe.skip(...)` /
 * `it.skip(...)` gates that were silently masking 46 regression
 * tests — now those tests run against the v3 server whenever a
 * developer (or CI) boots one, and still skip noiselessly when
 * none is present.
 */

import { describe, it } from 'vitest';
import { VectorizerClient } from '../src/client';

/**
 * Default base URL for live-gated tests. Override with
 * `VECTORIZER_BASE_URL=http://host:port` in the test environment.
 */
export const VECTORIZER_BASE_URL =
  process.env['VECTORIZER_BASE_URL'] ?? 'http://127.0.0.1:15002';

/**
 * Environment override to force-enable the live-gated suites even
 * when the health-check probe returns false (e.g. when a proxy
 * intercepts `/health` but the real endpoints work).
 * Set `VECTORIZER_LIVE_TESTS=1` in CI to lock-in the live path.
 */
const LIVE_FORCED = process.env['VECTORIZER_LIVE_TESTS'] === '1';

/**
 * Async health-check probe. Returns true when the server at `baseURL`
 * answers `healthCheck()` inside 3 seconds.
 */
export async function checkServerAvailability(baseURL: string): Promise<boolean> {
  const client = new VectorizerClient({
    baseURL,
    timeout: 3000,
  });

  try {
    await client.healthCheck();
    return true;
  } catch {
    return false;
  }
}

/**
 * Cached live-server probe. Vitest supports top-level await, so each
 * test file evaluates this once at module load time and the `describe`
 * / `it` gates below pick up the resolved value.
 */
let liveServerProbeCache: Promise<boolean> | null = null;
export function isLiveServerAvailable(
  baseURL: string = VECTORIZER_BASE_URL
): Promise<boolean> {
  if (LIVE_FORCED) return Promise.resolve(true);
  if (liveServerProbeCache === null) {
    liveServerProbeCache = checkServerAvailability(baseURL);
  }
  return liveServerProbeCache;
}

/**
 * Backwards-compat wrapper kept for existing callers.
 */
export function skipIfServerNotAvailable(
  serverAvailable: boolean,
  testFn: () => void | Promise<void>
) {
  if (!serverAvailable) {
    return;
  }
  return testFn();
}

/**
 * Run `describe(name, fn)` when the live server is reachable, else
 * skip it (vitest still shows the suite as skipped instead of the
 * test file reporting "no tests found").
 *
 * Usage:
 *   const live = await isLiveServerAvailable();
 *   runIfServer(live, 'File Operations', () => {
 *     it('should list files', async () => { ... });
 *   });
 */
export function runIfServer(
  live: boolean,
  name: string,
  fn: () => void | Promise<void>
): void {
  if (live) {
    describe(name, fn);
  } else {
    describe.skip(
      `${name} (skipped — set VECTORIZER_LIVE_TESTS=1 or boot ${VECTORIZER_BASE_URL})`,
      fn
    );
  }
}

/**
 * Per-test variant of `runIfServer`. Use inside a `describe` block
 * when only one case needs the live-server gate.
 */
export function runItIfServer(
  live: boolean,
  name: string,
  fn: () => void | Promise<void>
): void {
  if (live) {
    it(name, fn);
  } else {
    it.skip(
      `${name} (skipped — set VECTORIZER_LIVE_TESTS=1 or boot ${VECTORIZER_BASE_URL})`,
      fn
    );
  }
}

