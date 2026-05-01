# Bulk-Upsert Backpressure Specification

Source issue: https://github.com/hivellm/vectorizer/issues/263

## ADDED Requirements

### Requirement: Bounded BM25 Vocabulary-Build Concurrency
The system SHALL bound the number of concurrent BM25 vocabulary-build
operations to a configurable maximum.

#### Scenario: Permit limit honored under burst
Given `backpressure.max_concurrent_vocab_builds = 4`
And 32 concurrent upsert requests target collections needing vocabulary build
When the requests are dispatched
Then at any instant the number of in-flight vocabulary builds MUST be ≤ 4
And remaining builds MUST wait on a semaphore until a permit is released

#### Scenario: Default sizing
Given `backpressure.max_concurrent_vocab_builds = 0`
When the server starts
Then the permit count MUST default to `num_cpus::get()`

#### Scenario: Permit released on error
Given a vocabulary-build holds a permit
When the build returns an error
Then the permit MUST be released before the error propagates to the caller

### Requirement: Per-Collection Upsert Queue Limits
The system SHALL track per-collection in-flight upsert depth and reject
work that exceeds a configured hard limit.

#### Scenario: Soft watermark — accept and log
Given `backpressure.upsert_queue_high_water = 256`
And a collection's in-flight depth is 257
When a new upsert arrives
Then the system MUST accept the upsert
And the system MUST emit a structured warn log "queue high-water exceeded"
And the gauge `vectorizer_upsert_queue_depth{collection}` MUST report 258

#### Scenario: Hard limit — reject with 429
Given `backpressure.upsert_queue_hard_limit = 1024`
And a collection's in-flight depth is 1024
When a new HTTP upsert arrives
Then the system MUST respond with status `429 Too Many Requests`
And the response MUST include header `Retry-After: <retry_after_seconds>`
And the counter `vectorizer_upsert_rejected_total{reason="queue_full"}` MUST increment

#### Scenario: Hard limit on gRPC path
Given a collection's in-flight depth is at hard limit
When a new gRPC upsert arrives
Then the system MUST respond with `RESOURCE_EXHAUSTED`
And the gRPC status detail MUST carry a `RetryInfo` with the configured delay

#### Scenario: Hard limit on MCP path
Given a collection's in-flight depth is at hard limit
When a new MCP `upsert` tool call arrives
Then the tool MUST return a structured error `{ code: "queue_full", retryAfterSeconds: N }`

### Requirement: Backpressure Metrics
The system SHALL expose Prometheus metrics covering queue and permit state.

#### Scenario: Metrics surface
Given the server is running with backpressure enabled
When a client scrapes `/metrics`
Then the response MUST include:
- gauge `vectorizer_upsert_queue_depth{collection}`
- gauge `vectorizer_upsert_in_flight{collection}`
- gauge `vectorizer_vocab_build_permits_available`
- counter `vectorizer_upsert_rejected_total{reason}`
- counter `vectorizer_bm25_empty_vocab_fallback_total{collection}`

### Requirement: Read-Path Isolation
The system SHALL keep read endpoints responsive while write endpoints are
saturated.

#### Scenario: Health probe under write saturation
Given the write side is saturated (vocab-build permits all held, queues at high-water)
When a probe issues `GET /health`
Then the response MUST be returned within 500 ms (p99)

#### Scenario: Collection listing under write saturation
Given the write side is saturated
When a client issues `GET /collections`
Then the response MUST be returned within 500 ms (p99)
And the response MUST reflect the live state of the store (no stale snapshot)

#### Scenario: Auth login under write saturation
Given the write side is saturated
When a user issues `POST /auth/login`
Then authentication MUST complete within 500 ms (p99)

### Requirement: Log Rate-Limiting for Empty-Vocabulary Fallback
The system SHALL rate-limit the "BM25 vocabulary is empty" warning so that
log volume cannot itself become a CPU/disk bottleneck.

#### Scenario: One warn per collection per window
Given `backpressure.log_rate_limit_per_5s = 1`
And 10 000 upserts to collection `repo-x:src` trigger the empty-vocab fallback within 5 s
When the logs are inspected
Then exactly 1 `WARN BM25 vocabulary is empty …` line MUST appear for that collection in that 5 s window
And the counter `vectorizer_bm25_empty_vocab_fallback_total{collection="repo-x:src"}` MUST increment by 10 000

### Requirement: Configurable Backpressure
The system SHALL expose backpressure tuning through configuration and env vars.

#### Scenario: Disabled by config
Given `backpressure.enabled = false`
When the server starts
Then no permit limits, queue limits, or 429 rejections MUST be applied
And metrics MUST still be registered (reporting zero) for dashboard stability

#### Scenario: Env-var override
Given `CORTEX_VECTORIZER_MAX_CONCURRENT_BUILDS=8` is set
And `backpressure.max_concurrent_vocab_builds = 4` is set in `config.yml`
When the server starts
Then the effective permit count MUST be 8 (env wins)

#### Scenario: Validation rejects inverted limits
Given `backpressure.upsert_queue_high_water = 1024`
And `backpressure.upsert_queue_hard_limit = 256`
When the server starts
Then startup MUST fail with a clear error "high_water must be < hard_limit"
