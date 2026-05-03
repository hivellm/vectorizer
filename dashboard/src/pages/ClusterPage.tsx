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

// TODO(replication-endpoint): swap for useReplication() once a /replication
// endpoint exists. The static fallback below mirrors reference/data.js
// MOCK_REPLICAS so the page renders the canonical visual until then.
const FALLBACK: Replica[] = [
  { id: 'replica-eu-west-01',  region: 'eu-west-1',  offset: 8_812_004, lag: 0,   status: 'in-sync',     last_ack: 'just now' },
  { id: 'replica-us-east-01',  region: 'us-east-1',  offset: 8_811_998, lag: 6,   status: 'in-sync',     last_ack: '1s ago' },
  { id: 'replica-us-east-02',  region: 'us-east-1',  offset: 8_811_840, lag: 164, status: 'catching-up', last_ack: '2s ago' },
  { id: 'replica-ap-south-01', region: 'ap-south-1', offset: 8_811_998, lag: 6,   status: 'in-sync',     last_ack: '1s ago' },
];

const MASTER_OFFSET = 8_812_004;

function ClusterPage() {
  const api = useApiClient();
  const [replicas, setReplicas] = useState<Replica[]>(FALLBACK);
  const [usingFallback, setUsingFallback] = useState(true);

  // Best-effort live fetch; static fallback shows immediately.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const resp = await api.get<{ replicas?: Replica[] } | Replica[]>('/replication');
        const payload = (resp as { data?: unknown }).data ?? resp;
        const arr = Array.isArray(payload)
          ? (payload as Replica[])
          : ((payload as { replicas?: Replica[] })?.replicas ?? []);
        if (cancelled) return;
        if (arr.length > 0) {
          setReplicas(arr);
          setUsingFallback(false);
        }
      } catch {
        // ignore — fallback already rendered
      }
    })();
    return () => { cancelled = true; };
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
            Master → replica state · {replicas.length} replicas across {regions} region{regions === 1 ? '' : 's'}
            {usingFallback && ' (static)'}
          </p>
        </div>
        <div className="row" style={{ gap: 8 }}>
          <Pill tone="green" live>
            <span className="dot green" />
            master · accepting writes
          </Pill>
          {/* TODO(actions): wire add-replica modal */}
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
          value={formatNumber(MASTER_OFFSET)}
          delta={{ tone: 'neutral', text: '+184/s' }}
        />
        <Kpi
          label="Connected replicas"
          value={`${inSync} / ${replicas.length}`}
          delta={{ tone: inSync === replicas.length ? 'up' : 'down', text: inSync === replicas.length ? 'all in-sync' : `${replicas.length - inSync} catching up` }}
        />
        <Kpi
          accent="amber"
          label="Max lag"
          value={String(maxLag)}
          unit="ms"
          delta={{ tone: maxLag > 100 ? 'down' : 'neutral', text: maxLagReplica ? maxLagReplica.id.replace(/^replica-/, '') : '—' }}
        />
        <Kpi
          label="Write concern"
          value="Majority(3)"
          delta={{ tone: 'neutral', text: 'wait ≤ 200ms' }}
        />
      </div>

      <Card>
        <CardHead title="Replicas" />
        <CardBody tight>
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
                    {/* TODO(actions): wire resync */}
                    <button className="btn sm">Resync</button>
                  </Td>
                </tr>
              ))}
            </tbody>
          </Tbl>
        </CardBody>
      </Card>
    </div>
  );
}

export default ClusterPage;
