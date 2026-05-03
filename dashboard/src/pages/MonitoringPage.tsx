import {
  Icons,
  Sparkline,
  Pill,
  Card,
  CardHead,
  CardBody,
  Bar,
  KeyValue,
  KeyValueRow,
  useTick,
} from '@/components/console';

const SPARK = (n: number, base: number, amp: number): number[] =>
  Array.from({ length: n }, (_, i) => base + Math.sin(i / 2) * amp + Math.random() * amp * 0.3);

// TODO(metrics-endpoint): wire real values from /metrics in Task 4.1.
// TODO(stats-endpoint):   wire real values from /stats   in Task 4.3.

function MonitoringPage() {
  const tick = useTick(1500);
  const total = 2480 + Math.round(Math.sin(tick / 2) * 120);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Monitoring</h1>
          <p className="page-sub">
            Real-time metrics across SIMD dispatch, WAL, query cache and HTTP throughput
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <Pill tone="green" live>
            <span className="dot green" />
            live · 1.5s refresh
          </Pill>
          <button className="btn">
            <Icons.copy size={13} />
            Export Prometheus
          </button>
        </div>
      </div>

      {/* Throughput strip */}
      <Card>
        <CardHead title="HTTP / MCP throughput · last 60s" sub="requests/sec" />
        <CardBody>
          <div className="row" style={{ gap: 24, marginBottom: 12, flexWrap: 'wrap' }}>
            <div>
              <div
                className="muted"
                style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
              >
                Total
              </div>
              <div className="tnum" style={{ fontSize: 24, fontWeight: 600 }}>
                {total.toLocaleString()}
              </div>
            </div>
            <div>
              <div
                className="muted"
                style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
              >
                REST
              </div>
              <div
                className="tnum"
                style={{ fontSize: 24, fontWeight: 600, color: 'var(--teal-hi)' }}
              >
                1,841
              </div>
            </div>
            <div>
              <div
                className="muted"
                style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
              >
                MCP
              </div>
              <div
                className="tnum"
                style={{ fontSize: 24, fontWeight: 600, color: 'var(--magenta-hi)' }}
              >
                639
              </div>
            </div>
            <div>
              <div
                className="muted"
                style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
              >
                p99
              </div>
              <div className="tnum" style={{ fontSize: 24, fontWeight: 600 }}>
                2.8ms
              </div>
            </div>
            <div>
              <div
                className="muted"
                style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
              >
                5xx rate
              </div>
              <div
                className="tnum"
                style={{ fontSize: 24, fontWeight: 600, color: 'var(--green)' }}
              >
                0.00%
              </div>
            </div>
          </div>
          <Sparkline
            data={SPARK(60, 2400, 220)}
            width={1100}
            height={70}
            color="var(--teal)"
            ariaLabel="HTTP plus MCP requests per second over the last 60 seconds"
          />
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <div className="grid grid-2" style={{ gap: 14, marginBottom: 14 }}>
        {/* SIMD Backend */}
        <Card>
          <CardHead
            title="SIMD Backend"
            right={<Pill tone="teal" className="mono">avx2</Pill>}
          />
          <CardBody>
            <KeyValue>
              <KeyValueRow term="Active backend">Avx2Backend · 8 f32 lanes</KeyValueRow>
              <KeyValueRow term="Selection">
                <span className="muted">
                  VNNI → AVX-512F →{' '}
                  <span style={{ color: 'var(--teal-hi)' }}>AVX2+FMA</span> → AVX2 → SSE2 → scalar
                </span>
              </KeyValueRow>
              <KeyValueRow term="Override">VECTORIZER_SIMD_BACKEND=auto</KeyValueRow>
              <KeyValueRow term="FMA fusion">
                <Pill tone="green">enabled</Pill>
              </KeyValueRow>
              <KeyValueRow term="Architecture">x86_64 · Intel Xeon 6248 · 24 cores</KeyValueRow>
            </KeyValue>
            <div className="divider" />
            <div
              className="muted"
              style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: 8 }}
            >
              Primitive throughput
            </div>
            <div className="col" style={{ gap: 8 }}>
              {(
                [
                  ['dot_product', 92, '12.4 Gflop/s'],
                  ['cosine_similarity', 88, '11.8 Gflop/s'],
                  ['euclidean_dist²', 86, '11.2 Gflop/s'],
                  ['l2_norm', 79, '9.8 Gflop/s'],
                ] as const
              ).map(([name, percent, label]) => (
                <div key={name}>
                  <div className="row" style={{ fontSize: 11, marginBottom: 3 }}>
                    <span className="mono">{name}</span>
                    <span className="right mono muted">{label}</span>
                  </div>
                  <Bar percent={percent} ariaLabel={`${name} throughput at ${label}`} />
                </div>
              ))}
            </div>
          </CardBody>
        </Card>

        {/* Write-Ahead Log */}
        <Card>
          <CardHead
            title="Write-Ahead Log"
            right={<Pill tone="green">healthy</Pill>}
          />
          <CardBody>
            <div className="grid grid-3" style={{ gap: 14, marginBottom: 14 }}>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  Sequence
                </div>
                <div className="tnum" style={{ fontSize: 18, fontWeight: 600 }}>
                  8,811,998
                </div>
              </div>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  Size on disk
                </div>
                <div className="tnum" style={{ fontSize: 18, fontWeight: 600 }}>
                  284 MB
                </div>
              </div>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  Last checkpoint
                </div>
                <div className="tnum" style={{ fontSize: 18, fontWeight: 600 }}>
                  2m ago
                </div>
              </div>
            </div>
            <Sparkline
              data={SPARK(40, 280, 40)}
              width={460}
              height={70}
              color="var(--magenta)"
              ariaLabel="WAL size over time"
            />
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Path">/var/lib/vectorizer/vectorizer.wal</KeyValueRow>
              <KeyValueRow term="Format">JSON-Lines · global atomic seq</KeyValueRow>
              <KeyValueRow term="Recovery">strict-monotonic · per-collection filter</KeyValueRow>
              <KeyValueRow term="Replay rate">~42k ops/sec</KeyValueRow>
            </KeyValue>
            <div className="row" style={{ gap: 6, marginTop: 12 }}>
              {/* TODO(stats-endpoint): wire real WAL actions */}
              <button className="btn sm">
                <Icons.zap size={11} />
                Force checkpoint
              </button>
              <button className="btn sm">Tail entries</button>
            </div>
          </CardBody>
        </Card>
      </div>

      <div className="grid grid-2" style={{ gap: 14 }}>
        {/* Query Cache */}
        <Card>
          <CardHead
            title="Query Cache"
            right={<Pill tone="green" className="mono">94.2% hit rate</Pill>}
          />
          <CardBody>
            <div className="grid grid-4" style={{ gap: 14, marginBottom: 14 }}>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  Hits
                </div>
                <div
                  className="tnum"
                  style={{ fontSize: 18, fontWeight: 600, color: 'var(--green)' }}
                >
                  4.21M
                </div>
              </div>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  Misses
                </div>
                <div
                  className="tnum"
                  style={{ fontSize: 18, fontWeight: 600, color: 'var(--text-2)' }}
                >
                  258K
                </div>
              </div>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  Evictions
                </div>
                <div
                  className="tnum"
                  style={{ fontSize: 18, fontWeight: 600, color: 'var(--amber)' }}
                >
                  1,204
                </div>
              </div>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  TTL
                </div>
                <div className="tnum" style={{ fontSize: 18, fontWeight: 600 }}>
                  5min
                </div>
              </div>
            </div>
            <Sparkline
              data={SPARK(40, 94, 3)}
              width={460}
              height={70}
              color="var(--green)"
              ariaLabel="Query cache hit rate over time"
            />
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Layer">QueryCache (LRU + TTL) · response-level</KeyValueRow>
              <KeyValueRow term="Key">(collection, query, limit, threshold)</KeyValueRow>
              <KeyValueRow term="Capacity">10,000 entries · 184 MB</KeyValueRow>
              <KeyValueRow term="Invalidation">collection-scoped on writes</KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>

        {/* File-ops Cache */}
        <Card>
          <CardHead
            title="File-ops Cache"
            right={<Pill tone="teal" className="mono">3-tier LRU</Pill>}
          />
          <CardBody>
            <div className="col" style={{ gap: 14 }}>
              {(
                [
                  ['File content cache', 78, 100, 'files'],
                  ['Summary cache', 432, 500, 'summaries'],
                  ['File list cache (TTL 60s)', 18, 50, 'collections'],
                ] as const
              ).map(([name, used, cap, unit]) => (
                <div key={name}>
                  <div className="row" style={{ fontSize: 12, marginBottom: 5 }}>
                    <span>{name}</span>
                    <span className="right mono muted">
                      {used}/{cap} {unit}
                    </span>
                  </div>
                  <Bar
                    percent={(used / cap) * 100}
                    tone="amber"
                    ariaLabel={`${name} utilization`}
                  />
                </div>
              ))}
            </div>
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Owner">FileOperationsManager</KeyValueRow>
              <KeyValueRow term="Eviction">LRU · per-entry TTL</KeyValueRow>
              <KeyValueRow term="Triggers">file watcher events · explicit clear_*</KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>
      </div>
    </div>
  );
}

export default MonitoringPage;
