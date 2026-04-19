/**
 * VectorizerRPC client for TypeScript.
 *
 * Implements the binary VectorizerRPC transport (port 15503/tcp)
 * documented in `docs/specs/VECTORIZER_RPC.md`. Default transport in
 * v3.x; the legacy REST `VectorizerClient` stays available for
 * browsers, scripting, and ops tooling that already targets HTTP.
 *
 * Quickstart::
 *
 *     import { RpcClient } from '@hivehub/vectorizer-sdk/rpc';
 *
 *     const client = await RpcClient.connectUrl('vectorizer://127.0.0.1:15503');
 *     await client.hello({ clientName: 'my-app' });
 *     const cols = await client.listCollections();
 *
 * The shapes mirror the Rust + Python SDKs at `sdks/rust/src/rpc/`
 * and `sdks/python/rpc/` so polyglot codebases share a single mental
 * model.
 */

export {
  HEADER_SIZE,
  MAX_BODY_SIZE,
  FrameDecodeError,
  FrameReader,
  FrameTooLargeError,
  decodeBody,
  encodeFrame,
} from './codec';

// Importing commands has the side effect of attaching typed wrappers
// as methods on RpcClient. Must come AFTER client export.
import './commands';
export { CollectionInfo, SearchHit } from './commands';

export type { ConnectOptions, HelloPayload, HelloResponse } from './client';
export {
  RpcClient,
  RpcClientError,
  RpcConnectionClosed,
  RpcNotAuthenticated,
  RpcServerError,
} from './client';

export {
  DEFAULT_HTTP_PORT,
  DEFAULT_RPC_PORT,
  Endpoint,
  EndpointParseError,
  parseEndpoint,
} from './endpoint';

export { PooledClient, RpcPool, RpcPoolConfig } from './pool';

export type {
  Request,
  Response,
  ResponseResult,
  VectorizerValue,
} from './types';
export {
  Value,
  asArray,
  asBool,
  asFloat,
  asInt,
  asMap,
  asStr,
  mapGet,
  requestToMsgpack,
  responseErr,
  responseFromMsgpack,
  responseOk,
  responseToMsgpack,
  valueFromMsgpack,
  valueToMsgpack,
} from './types';
