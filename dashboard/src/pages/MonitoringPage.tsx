import { useMetrics } from '@/hooks/useMetrics';
import { useStats } from '@/hooks/useStats';
import {
  Icons,
  Pill,
  Card,
  CardHead,
  CardBody,
  KeyValue,
  KeyValueRow,
} from '@/components/console';
import { formatBytes, formatNumber, formatRelativeTime } from '@/utils/formatters';

// Reusable "endpoint not yet exposed" placeholder. Used everywhere on this
// page where the backend doesn't currently emit data — keeps the layout
// stable while making it crystal clear that nothing is mocked.
function NotExposed({ subject }: { subject: string }) {
  return (
    <div
      className="muted"
      style={{
        padding: 24,
        textAlign: 'center',
        fontSize: 12,
        lineHeight: 1.6,
      }}
    >
      {subject} not yet exposed by the server. Coming soon.
    </div>
  );
}

function MonitoringPage() {
  const { metrics, loading: metricsLoading } = useMetrics();
  const { stats } = useStats();
  const total = metrics.qps;
  const cache = stats.cache;
  const hitPct = Math.max(0, Math.min(100, cache.hitRate * 100));
  const cacheHasData = cache.hits + cache.misses + cache.evictions > 0;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Monitoring</h1>
          <p className="page-sub">
            Live metrics from <span className="mono">/health</span>. Other surfaces light up
            once the server begins emitting them.
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

      {/* Throughput strip — only Total is real (from /stats). */}
      <Card>
        <CardHead title="HTTP / MCP throughput" sub="requests/sec" />
        <CardBody>
          <div className="row" style={{ gap: 24, marginBottom: 4, flexWrap: 'wrap' }}>
            <div>
              <div
                className="muted"
                style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.06em' }}
              >
                Total
              </div>
              <div className="tnum" style={{ fontSize: 24, fontWeight: 600 }}>
                {metricsLoading ? '…' : total ? total.toLocaleString() : '—'}
              </div>
            </div>
          </div>
          <div className="muted" style={{ fontSize: 11, marginTop: 8 }}>
            REST/MCP split, p99 latency and 5xx-rate are not yet exposed by the server.
          </div>
        </CardBody>
      </Card>

      <div style={{ height: 14 }} />

      <div className="grid grid-2" style={{ gap: 14, marginBottom: 14 }}>
        {/* SIMD Backend */}
        <Card>
          <CardHead title="SIMD Backend" />
          <CardBody>
            <NotExposed subject="SIMD backend introspection" />
          </CardBody>
        </Card>

        {/* Write-Ahead Log */}
        <Card>
          <CardHead title="Write-Ahead Log" />
          <CardBody>
            {stats.walSequence !== undefined ||
            stats.walSizeBytes !== undefined ||
            stats.walLastCheckpointAt ? (
              <div className="grid grid-3" style={{ gap: 14, marginBottom: 14 }}>
                <div>
                  <div
                    className="muted"
                    style={{ fontSize: 10, textTransform: 'uppercase' }}
                  >
                    Sequence
                  </div>
                  <div className="tnum" style={{ fontSize: 18, fontWeight: 600 }}>
                    {stats.walSequence !== undefined ? formatNumber(stats.walSequence) : '—'}
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
                    {stats.walSizeBytes !== undefined ? formatBytes(stats.walSizeBytes, 0) : '—'}
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
                    {stats.walLastCheckpointAt
                      ? formatRelativeTime(stats.walLastCheckpointAt)
                      : '—'}
                  </div>
                </div>
              </div>
            ) : (
              <NotExposed subject="WAL stats" />
            )}
          </CardBody>
        </Card>
      </div>

      <div className="grid grid-2" style={{ gap: 14 }}>
        {/* Query Cache (real /health data) */}
        <Card>
          <CardHead
            title="Query Cache"
            right={
              cacheHasData ? (
                <Pill tone="green" className="mono">{`${hitPct.toFixed(1)}% hit rate`}</Pill>
              ) : (
                <Pill tone="muted" className="mono">—</Pill>
              )
            }
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
                  style={{
                    fontSize: 18,
                    fontWeight: 600,
                    color: cache.hits ? 'var(--green)' : 'var(--text-2)',
                  }}
                >
                  {cache.hits ? formatNumber(cache.hits) : '—'}
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
                  {cache.misses ? formatNumber(cache.misses) : '—'}
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
                  style={{
                    fontSize: 18,
                    fontWeight: 600,
                    color: cache.evictions ? 'var(--amber)' : 'var(--text-2)',
                  }}
                >
                  {cache.evictions ? formatNumber(cache.evictions) : '—'}
                </div>
              </div>
              <div>
                <div
                  className="muted"
                  style={{ fontSize: 10, textTransform: 'uppercase' }}
                >
                  Capacity
                </div>
                <div className="tnum" style={{ fontSize: 18, fontWeight: 600 }}>
                  {cache.capacity ? formatNumber(cache.capacity) : '—'}
                </div>
              </div>
            </div>
            <div className="divider" />
            <KeyValue>
              <KeyValueRow term="Layer">QueryCache (LRU + TTL) · response-level</KeyValueRow>
              <KeyValueRow term="Key">(collection, query, limit, threshold)</KeyValueRow>
              <KeyValueRow term="Size">
                {cache.size ? formatNumber(cache.size) : '—'} entries
              </KeyValueRow>
              <KeyValueRow term="Invalidation">collection-scoped on writes</KeyValueRow>
            </KeyValue>
          </CardBody>
        </Card>

        {/* File-ops Cache */}
        <Card>
          <CardHead title="File-ops Cache" />
          <CardBody>
            <NotExposed subject="File-ops cache stats" />
          </CardBody>
        </Card>
      </div>
    </div>
  );
}

export default MonitoringPage;
