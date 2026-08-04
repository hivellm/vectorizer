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

// Framing belongs to Thunder (`@hivehub/thunder`), the shared binary RPC
// package the server also runs. Re-exported here so tooling that needs to
// build or read frames directly (test servers, proxies) uses the same codec
// as the client instead of a second implementation.
export {
  DEFAULT_MAX_FRAME_BYTES,
  DecodeError,
  FrameReader,
  FrameTooLargeError,
  PUSH_ID,
  WIRE_VERSION,
  decodeRequest,
  decodeRequestBody,
  decodeResponse,
  decodeResponseBody,
  encodeRequest,
  encodeResponse,
} from '@hivehub/thunder';

// Importing commands has the side effect of attaching typed wrappers
// as methods on RpcClient. Must come AFTER client export.
import './commands';
export type {
  CollectionInfo,
  SearchHit,
  CreateCollectionResult,
  CleanupEmptyResult,
  VectorWriteResult,
  BatchItemResult,
  BatchInsertResult,
  BatchUpdateResult,
  BatchDeleteResult,
  BatchSearchResult,
  MoveRpcResult,
  CopyRpcResult,
  DeleteByFilterRpcResult,
  BulkUpdateMetadataRpcResult,
  SetExpiryResult,
  EmbedResult,
  VectorListResult,
  SearchTrace,
  SearchExplainResult,
  DiscoverResult,
  ScoredCollection,
  ExpandQueriesResult,
  DiscoveryChunk,
  CompressBullet,
  AnswerPlanSection,
  AnswerPlanResult,
  RenderPromptResult,
  GraphDiscoveryStatus,
  DiscoverEdgesResult,
  DiscoverEdgesForNodeResult,
  AdminStats,
  AdminStatus,
  SlowQueryConfigResult,
  AuthMeResult,
  RefreshTokenResult,
  ValidatePasswordResult,
  ApiKeyCreated,
  RotatedApiKey,
  ReplicationConfigureResult,
  RebalanceStatus,
} from './commands';

export type { ConnectOptions, HelloPayload, HelloResponse } from './client';
export {
  RpcClient,
  RpcClientError,
  RpcConnectionClosed,
  RpcNotAuthenticated,
  RpcProtocolError,
  RpcServerError,
  RpcTimeout,
  protocolConfig,
} from './client';

export {
  DEFAULT_HTTP_PORT,
  DEFAULT_RPC_PORT,
  Endpoint,
  EndpointParseError,
  parseEndpoint,
} from './endpoint';

export { PooledClient, RpcPool, RpcPoolConfig } from './pool';

export type { Request, ResponseResult, VectorizerValue } from './types';
export {
  Response,
  Value,
  asArray,
  asBool,
  asFloat,
  asInt,
  asMap,
  asStr,
  mapGet,
} from './types';
