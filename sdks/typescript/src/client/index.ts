/**
 * `VectorizerClient` is the public facade — every per-surface client
 * is mixed into one class so the established
 * `new VectorizerClient(config).searchVectors(...)` ergonomics keep
 * working, while users who want a smaller import can pull a single
 * surface (`new SearchClient(config)`) directly.
 *
 * The mixin pattern is the standard TypeScript recipe for combining
 * many class-shaped behaviours into one (see TS handbook ➜ Mixins).
 * `applyMixins` copies each surface's methods onto the facade's
 * prototype at module load.
 *
 * Cross-reference: when `phase6_sdk-typescript-rpc` lands, the RPC
 * client implements the same `Transport` interface and is injected via
 * `new VectorizerClient({ transport: new RpcTransport(...) })` — every
 * surface keeps working unchanged.
 */

import { BaseClient, VectorizerClientConfig } from './_base';
import { CoreClient } from './core';
import { CollectionsClient } from './collections';
import { VectorsClient } from './vectors';
import { SearchClient } from './search';
import { DiscoveryClient } from './discovery';
import { FilesClient } from './files';
import { GraphClient } from './graph';
import { QdrantClient } from './qdrant';
import { AdminClient } from './admin';

export { BaseClient, VectorizerClientConfig, Transport } from './_base';
export { CoreClient } from './core';
export { CollectionsClient } from './collections';
export { VectorsClient } from './vectors';
export { SearchClient } from './search';
export { DiscoveryClient } from './discovery';
export { FilesClient } from './files';
export { GraphClient } from './graph';
export { QdrantClient } from './qdrant';
export { AdminClient } from './admin';

// Standard TS mixin recipe: declare the merged interface so callers
// see every surface's methods, then copy the prototypes at runtime.
// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface VectorizerClient
  extends CoreClient,
    CollectionsClient,
    VectorsClient,
    SearchClient,
    DiscoveryClient,
    FilesClient,
    GraphClient,
    QdrantClient,
    AdminClient {}

export class VectorizerClient extends BaseClient {
  constructor(config: VectorizerClientConfig = {}) {
    super(config);
  }

  /**
   * Run `callback` against a copy of this client pinned to the master
   * transport — used for read-your-writes flows where the read must
   * see a write you just issued.
   */
  public async withMaster<T>(
    callback: (client: VectorizerClient) => Promise<T>,
  ): Promise<T> {
    const masterClient = new VectorizerClient({
      ...this.config,
      readPreference: 'master',
    });
    return callback(masterClient);
  }
}

function applyMixins(derivedCtor: Function, constructors: Function[]): void {
  for (const baseCtor of constructors) {
    for (const name of Object.getOwnPropertyNames(baseCtor.prototype)) {
      if (name === 'constructor') continue;
      const descriptor = Object.getOwnPropertyDescriptor(baseCtor.prototype, name);
      if (descriptor) {
        Object.defineProperty(derivedCtor.prototype, name, descriptor);
      }
    }
  }
}

applyMixins(VectorizerClient, [
  CoreClient,
  CollectionsClient,
  VectorsClient,
  SearchClient,
  DiscoveryClient,
  FilesClient,
  GraphClient,
  QdrantClient,
  AdminClient,
]);
