import { useMetrics } from '@/hooks/useMetrics';
import { useStats } from '@/hooks/useStats';
import { useRuntimeMetrics } from '@/hooks/useRuntimeMetrics';
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
} from '@/components/console';
import { formatBytes, formatNumber, formatRelativeTime } from '@/utils/formatters';

// System health (CPU, memory, uptime, connections) and WAL stats are read
// live from GET /metrics/runtime (phase25), streamed over the dashboard WS
// (phase29). Throughput/per-route panels populate once the server sees
// sustained traffic; they read 0/empty at idle, which is real, not mocked.

/** Format an uptime in seconds as a compact "1d 2h 3m" / "4h 5m" / "6m 7s". */
function formatUptime(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${sec}s`;
  return `${sec}s`;
}

function MonitoringPage() {
  const { metrics } = useMetrics();
  const { stats } = useStats();
  const { metrics: runtimeMetrics, qpsHistory, loading: runtimeLoading, error: runtimeError } = useRuntimeMetrics();

  const cache = stats.cache;
  const hitPct = Math.max(0, Math.min(100, cache.hitRate * 100));

  // Total throughput: prefer /metrics/runtime qpsWindow60s; fall back to
  // useMetrics qps (from /stats) if runtime is not yet available.
  const runtimeUnavailable = runtimeLoading || runtimeError !== null;
  const totalQps = runtimeUnavailable ? metrics.qps : runtimeMetrics.qpsWindow60s;

  // Routes sorted descending by qps for display.
  const routes = [...runtimeMetrics.throughputByRoute].sort((a, b) => b.qps - a.qps);

  // Maximum qps across all routes — used to scale per-route Bar percents.
  const maxRouteQps = routes.length > 0 ? routes[0].qps : 1;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Monitoring</h1>
          <p className="page-sub">
            Real-time metrics across query cache, HTTP throughput and per-route latency
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <Pill tone="green" live>
            <span className="dot green" />
            live · 2s refresh
          </Pill>
          <button className="btn">
            <Icons.copy size={13} />
            Export Prometheus
          </button>
        </div>
      </div>

      {/* Throughput strip — wired to /metrics/runtime */}
      <Card>
        <CardHead title="HTTP throughput · last 60s" sub="requests/sec" />
        <CardBody>
          <div className="row" style={{ gap: 24, marginBottom: 12, flexWrap: 'wrap' }}>
            <div>
              <div
                className="muted"
                style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
              >
                Total QPS
              </div>
              <div className="tnum" style={{ fontSize: 24, fontWeight: 600 }}>
                {runtimeUnavailable ? '--' : totalQps.toLocaleString(undefined, { maximumFractionDigits: 1 })}
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
                {runtimeUnavailable
                  ? '--'
                  : `${(runtimeMetrics.errorRate5xx60s * 100).toFixed(2)}%`}
              </div>
            </div>
          </div>
          {/* Client-side ring buffer of qpsWindow60s — one sample per 2s tick. */}
          {qpsHistory.length > 1 ? (
            <Sparkline
              data={qpsHistory}
              width={1100}
              height={70}
              color="var(--teal)"
              ariaLabel="HTTP requests per second over the last 60 samples"
            />
          ) : (
            <div
              className="muted"
              style={{ height: 70, display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 12 }}
            >
              Collecting samples…
            </div>
          )}
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      {/* System health — live from /metrics/runtime (cpu/mem/uptime/connections) */}
      <Card>
        <CardHead title="System" sub="process resource usage · live" />
        <CardBody>
          <div className="grid grid-4" style={{ gap: 14 }}>
            <div>
              <div className="muted" style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                CPU
              </div>
              <div className="tnum" style={{ fontSize: 22, fontWeight: 600 }}>
                {runtimeUnavailable ? '--' : `${runtimeMetrics.cpuPercent.toFixed(1)}%`}
              </div>
              <Bar percent={Math.max(0, Math.min(100, runtimeMetrics.cpuPercent))} ariaLabel="CPU utilization" />
            </div>
            <div>
              <div className="muted" style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                Memory (RSS)
              </div>
              <div className="tnum" style={{ fontSize: 22, fontWeight: 600 }}>
                {runtimeUnavailable ? '--' : formatBytes(runtimeMetrics.memoryRssBytes, 0)}
              </div>
              <Bar
                percent={Math.max(0, Math.min(100, runtimeMetrics.memoryPercent))}
                tone="amber"
                ariaLabel="Memory utilization"
              />
            </div>
            <div>
              <div className="muted" style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                Uptime
              </div>
              <div className="tnum" style={{ fontSize: 22, fontWeight: 600 }}>
                {runtimeUnavailable ? '--' : formatUptime(runtimeMetrics.uptimeSeconds)}
              </div>
            </div>
            <div>
              <div className="muted" style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                Active connections
              </div>
              <div className="tnum" style={{ fontSize: 22, fontWeight: 600 }}>
                {runtimeUnavailable ? '--' : formatNumber(runtimeMetrics.activeConnections)}
              </div>
            </div>
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <div className="grid grid-2" style={{ gap: 14, marginBottom: 14 }}>
        {/* Per-route throughput — wired to /metrics/runtime.throughput_by_route */}
        <Card>
          <CardHead title="Per-route throughput · last 60s" sub="qps + p99" />
          <CardBody>
            {runtimeUnavailable ? (
              <div className="muted" style={{ fontSize: 12 }}>
                Waiting for /metrics/runtime…
              </div>
            ) : routes.length === 0 ? (
              <div className="muted" style={{ fontSize: 12 }}>
                No route data yet.
              </div>
            ) : (
              <div className="col" style={{ gap: 8 }}>
                {routes.map((r) => (
                  <div key={r.route}>
                    <div className="row" style={{ fontSize: 11, marginBottom: 3 }}>
                      <span className="mono">{r.route}</span>
                      <span className="right mono muted">
                        {r.qps.toFixed(1)} qps · p99 {r.p99Ms.toFixed(1)}ms
                      </span>
                    </div>
                    <Bar
                      percent={maxRouteQps > 0 ? (r.qps / maxRouteQps) * 100 : 0}
                      ariaLabel={`${r.route} throughput: ${r.qps.toFixed(1)} qps, p99 ${r.p99Ms.toFixed(1)}ms`}
                    />
                  </div>
                ))}
              </div>
            )}
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
                  {/* Live from /metrics/runtime.wal (phase25 §3). */}
                  {runtimeUnavailable ? '--' : formatNumber(runtimeMetrics.wal.currentSeq)}
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
                  {runtimeUnavailable ? '--' : formatBytes(runtimeMetrics.wal.sizeBytes, 0)}
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
                  {!runtimeUnavailable && runtimeMetrics.wal.lastCheckpointAt > 0
                    ? formatRelativeTime(runtimeMetrics.wal.lastCheckpointAt * 1000)
                    : '--'}
                </div>
              </div>
            </div>
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Path">/var/lib/vectorizer/vectorizer.wal</KeyValueRow>
              <KeyValueRow term="Format">JSON-Lines · global atomic seq</KeyValueRow>
              <KeyValueRow term="Recovery">strict-monotonic · per-collection filter</KeyValueRow>
            </KeyValue>
            <div className="row" style={{ gap: 6, marginTop: 12 }}>
              <button className="btn sm">
                <Icons.zap size={11} />
                Force checkpoint
              </button>
              <button className="btn sm">Tail entries</button>
            </div>
          </CardBody>
        </Card>
      </div>

      {/* Query Cache — live from GET /health via useStats */}
      <Card>
          <CardHead
            title="Query Cache"
            right={<Pill tone="green" className="mono">{`${hitPct.toFixed(1)}% hit rate`}</Pill>}
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
                  {formatNumber(cache.hits)}
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
                  {formatNumber(cache.misses)}
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
                  {formatNumber(cache.evictions)}
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
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Layer">QueryCache (LRU + TTL) · response-level</KeyValueRow>
              <KeyValueRow term="Key">(collection, query, limit, threshold)</KeyValueRow>
              <KeyValueRow term="Capacity">10,000 entries · 184 MB</KeyValueRow>
              <KeyValueRow term="Invalidation">collection-scoped on writes</KeyValueRow>
            </KeyValue>
          </CardBody>
      </Card>
    </div>
  );
}

export default MonitoringPage;
