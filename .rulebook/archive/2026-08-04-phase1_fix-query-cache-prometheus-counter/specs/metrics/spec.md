# MetricsSink wiring

## ADDED Requirements

### Requirement: A registered metric must be reachable from production

Every producer that emits through a `MetricsSink` SHALL be constructed with a
real sink at its production wiring site. A component built with the default
constructor injects `NoopMetricsSink`, which makes the emission call and drops
it, so the metric can never appear on a scrape.

#### Scenario: Query cache reads reach Prometheus
Given a server started from bootstrap
When two identical text searches run against the same collection
Then `vectorizer_cache_requests_total{cache_type="query",result="miss"}` has increased
And `vectorizer_cache_requests_total{cache_type="query",result="hit"}` has increased

#### Scenario: HiveHub quota checks reach Prometheus
Given a deployment with HiveHub integration enabled
When a quota check runs
Then the `hub_quota_*` metric families report on a scrape

#### Scenario: The test harness does not write to the global registry
Given the in-process server test harness
When it builds its query cache
Then it uses the Noop sink, so one test's cache traffic cannot move a counter another test asserts on

### Requirement: A metrics test must exercise the shipping path

A test that asserts a Prometheus counter SHALL build its subject the way the
production wiring site builds it. Constructing the subject with the default
(Noop) constructor and asserting the global registry proves nothing about what
ships.

#### Scenario: The cache metrics test uses the production sink
Given the query cache Prometheus test
When it constructs the cache
Then it injects `PrometheusMetricsSink`, as bootstrap does
