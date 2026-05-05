import { useEffect, useState } from 'react';
import { useApiClient } from '@/hooks/useApiClient';
import {
  Icons,
  StatusPill,
  Pill,
  Card,
  CardHead,
  CardBody,
  Kpi,
  Tbl,
  Th,
  Td,
} from '@/components/console';
import { formatNumber } from '@/utils/formatters';

interface Replica {
  id: string;
  region: string;
  offset: number;
  lag: number;
  status: string;
  last_ack?: string;
}

function ClusterPage() {
  const api = useApiClient();
  const [replicas, setReplicas] = useState<Replica[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setLoading(true);
      try {
        const resp = await api.get<{ replicas?: Replica[] } | Replica[]>('/replication');
        const payload = (resp as { data?: unknown }).data ?? resp;
        const arr = Array.isArray(payload)
          ? (payload as Replica[])
          : ((payload as { replicas?: Replica[] })?.replicas ?? []);
        if (cancelled) return;
        setReplicas(arr);
        setError(null);
      } catch {
        if (cancelled) return;
        setReplicas([]);
        setError('Replication endpoint not yet exposed by the server.');
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const inSync = replicas.filter((r) => r.status === 'in-sync').length;
  const maxLag = replicas.reduce((m, r) => Math.max(m, r.lag ?? 0), 0);
  const maxLagReplica = replicas.find((r) => (r.lag ?? 0) === maxLag);
  const regions = new Set(replicas.map((r) => r.region)).size;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Replication</h1>
          <p className="page-sub">
            {replicas.length
              ? `Master → replica state · ${replicas.length} replicas across ${regions} region${regions === 1 ? '' : 's'}`
              : 'Master → replica state'}
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <Pill tone="green" live>
            <span className="dot green" />
            master · accepting writes
          </Pill>
          <button className="btn">
            <Icons.plus size={13} />
            Add replica
          </button>
        </div>
      </div>

      <div className="grid grid-4" style={{ marginBottom: 14 }}>
        <Kpi
          accent="teal"
          label="Master offset"
          value="—"
        />
        <Kpi
          label="Connected replicas"
          value={replicas.length ? `${inSync} / ${replicas.length}` : '—'}
        />
        <Kpi
          accent="amber"
          label="Max lag"
          value={replicas.length ? String(maxLag) : '—'}
          unit={replicas.length ? 'ms' : undefined}
          delta={
            maxLagReplica
              ? { tone: maxLag > 100 ? 'down' : 'neutral', text: maxLagReplica.id.replace(/^replica-/, '') }
              : undefined
          }
        />
        <Kpi label="Write concern" value="—" />
      </div>

      <Card>
        <CardHead
          title="Replicas"
          right={error ? <Pill tone="amber">{error}</Pill> : null}
        />
        <CardBody tight>
          {loading && replicas.length === 0 && (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              Loading…
            </div>
          )}
          {!loading && replicas.length === 0 && (
            <div style={{ padding: 24, color: 'var(--text-2)', textAlign: 'center' }}>
              No replicas to display.
            </div>
          )}
          {replicas.length > 0 && (
            <Tbl>
              <thead>
                <tr>
                  <Th>ID</Th>
                  <Th>Region</Th>
                  <Th>Offset</Th>
                  <Th>Lag</Th>
                  <Th>Last ACK</Th>
                  <Th>Status</Th>
                  <Th />
                </tr>
              </thead>
              <tbody>
                {replicas.map((r) => (
                  <tr key={r.id}>
                    <Td className="mono" style={{ fontWeight: 500 }}>{r.id}</Td>
                    <Td>
                      <Pill tone="muted" className="mono">{r.region}</Pill>
                    </Td>
                    <Td className="num">{formatNumber(r.offset)}</Td>
                    <Td className="num">
                      <span
                        style={{
                          color:
                            r.lag > 100
                              ? 'var(--amber)'
                              : r.lag > 0
                                ? 'var(--text-1)'
                                : 'var(--green)',
                        }}
                      >
                        {r.lag}ms
                      </span>
                    </Td>
                    <Td className="num muted">{r.last_ack ?? (r.lag === 0 ? 'just now' : '—')}</Td>
                    <Td>
                      <StatusPill status={r.status} />
                    </Td>
                    <Td>
                      <button className="btn sm">Resync</button>
                    </Td>
                  </tr>
                ))}
              </tbody>
            </Tbl>
          )}
        </CardBody>
      </Card>
    </div>
  );
}

export default ClusterPage;
