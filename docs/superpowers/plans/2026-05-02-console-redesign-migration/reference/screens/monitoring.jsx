// Monitoring screen — SIMD, WAL, Cache, Live metrics
const { Sparkline, useTick } = window.UI;
const { Icons } = window;

const MonitoringScreen = () => {
  const tick = useTick(1500);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Monitoring</h1>
          <p className="page-sub">Real-time metrics across SIMD dispatch, WAL, query cache and HTTP throughput</p>
        </div>
        <div className="row" style={{gap:8}}>
          <span className="pill green live"><span className="dot green"/>live · 1.5s refresh</span>
          <button className="btn"><Icons.copy size={13}/>Export Prometheus</button>
        </div>
      </div>

      {/* Throughput strip */}
      <div className="card" style={{marginBottom:14}}>
        <div className="card-head">
          <div className="title">HTTP / MCP throughput · last 60s</div>
          <span className="sub mono">requests/sec</span>
        </div>
        <div className="card-body">
          <div className="row" style={{gap:24, marginBottom:12}}>
            <div><div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>Total</div><div className="tnum" style={{fontSize:24, fontWeight:600}}>{(2480 + Math.round(Math.sin(tick/2)*120)).toLocaleString()}</div></div>
            <div><div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>REST</div><div className="tnum" style={{fontSize:24, fontWeight:600, color:"var(--teal-hi)"}}>1,841</div></div>
            <div><div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>MCP</div><div className="tnum" style={{fontSize:24, fontWeight:600, color:"var(--magenta-hi)"}}>639</div></div>
            <div><div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>p99</div><div className="tnum" style={{fontSize:24, fontWeight:600}}>2.8ms</div></div>
            <div><div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>5xx rate</div><div className="tnum" style={{fontSize:24, fontWeight:600, color:"var(--green)"}}>0.00%</div></div>
          </div>
          <Sparkline data={window.MOCK.spark(60, 2400, 220)} width={1100} height={70} color="var(--teal)"/>
        </div>
      </div>

      <div className="grid grid-2" style={{gap:14, marginBottom:14}}>
        <div className="card">
          <div className="card-head">
            <div className="title">SIMD Backend</div>
            <span className="pill teal mono">avx2</span>
          </div>
          <div className="card-body">
            <dl className="kv">
              <dt>Active backend</dt><dd>Avx2Backend · 8 f32 lanes</dd>
              <dt>Selection</dt><dd className="muted">VNNI → AVX-512F → <span style={{color:"var(--teal-hi)"}}>AVX2+FMA</span> → AVX2 → SSE2 → scalar</dd>
              <dt>Override</dt><dd>VECTORIZER_SIMD_BACKEND=auto</dd>
              <dt>FMA fusion</dt><dd><span className="pill green">enabled</span></dd>
              <dt>Architecture</dt><dd>x86_64 · Intel Xeon 6248 · 24 cores</dd>
            </dl>
            <div className="divider"/>
            <div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em", marginBottom:8}}>Primitive throughput</div>
            <div className="col" style={{gap:8}}>
              {[
                ["dot_product", 0.92, "12.4 Gflop/s"],
                ["cosine_similarity", 0.88, "11.8 Gflop/s"],
                ["euclidean_dist²", 0.86, "11.2 Gflop/s"],
                ["l2_norm", 0.79, "9.8 Gflop/s"],
              ].map(([n,p,v]) => (
                <div key={n}>
                  <div className="row" style={{fontSize:11, marginBottom:3}}>
                    <span className="mono">{n}</span><span className="right mono muted">{v}</span>
                  </div>
                  <div className="bar"><span style={{width:`${p*100}%`}}/></div>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="card">
          <div className="card-head">
            <div className="title">Write-Ahead Log</div>
            <span className="pill green">healthy</span>
          </div>
          <div className="card-body">
            <div className="grid grid-3" style={{gap:14, marginBottom:14}}>
              <div><div className="muted" style={{fontSize:10, textTransform:"uppercase"}}>Sequence</div><div className="tnum" style={{fontSize:18, fontWeight:600}}>8,811,998</div></div>
              <div><div className="muted" style={{fontSize:10, textTransform:"uppercase"}}>Size on disk</div><div className="tnum" style={{fontSize:18, fontWeight:600}}>284 MB</div></div>
              <div><div className="muted" style={{fontSize:10, textTransform:"uppercase"}}>Last checkpoint</div><div className="tnum" style={{fontSize:18, fontWeight:600}}>2m ago</div></div>
            </div>
            <Sparkline data={window.MOCK.spark(40, 280, 40)} width={460} height={70} color="var(--magenta)"/>
            <div className="divider"/>
            <dl className="kv">
              <dt>Path</dt><dd>/var/lib/vectorizer/vectorizer.wal</dd>
              <dt>Format</dt><dd>JSON-Lines · global atomic seq</dd>
              <dt>Recovery</dt><dd>strict-monotonic · per-collection filter</dd>
              <dt>Replay rate</dt><dd>~42k ops/sec</dd>
            </dl>
            <div className="row" style={{gap:6, marginTop:12}}>
              <button className="btn sm"><Icons.zap size={11}/>Force checkpoint</button>
              <button className="btn sm">Tail entries</button>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-2" style={{gap:14}}>
        <div className="card">
          <div className="card-head">
            <div className="title">Query Cache</div>
            <span className="pill green mono">94.2% hit rate</span>
          </div>
          <div className="card-body">
            <div className="grid grid-4" style={{gap:14, marginBottom:14}}>
              <div><div className="muted" style={{fontSize:10, textTransform:"uppercase"}}>Hits</div><div className="tnum" style={{fontSize:18, fontWeight:600, color:"var(--green)"}}>4.21M</div></div>
              <div><div className="muted" style={{fontSize:10, textTransform:"uppercase"}}>Misses</div><div className="tnum" style={{fontSize:18, fontWeight:600, color:"var(--text-2)"}}>258K</div></div>
              <div><div className="muted" style={{fontSize:10, textTransform:"uppercase"}}>Evictions</div><div className="tnum" style={{fontSize:18, fontWeight:600, color:"var(--amber)"}}>1,204</div></div>
              <div><div className="muted" style={{fontSize:10, textTransform:"uppercase"}}>TTL</div><div className="tnum" style={{fontSize:18, fontWeight:600}}>5min</div></div>
            </div>
            <Sparkline data={window.MOCK.spark(40, 94, 3)} width={460} height={70} color="var(--green)"/>
            <div className="divider"/>
            <dl className="kv">
              <dt>Layer</dt><dd>QueryCache (LRU + TTL) · response-level</dd>
              <dt>Key</dt><dd>(collection, query, limit, threshold)</dd>
              <dt>Capacity</dt><dd>10,000 entries · 184 MB</dd>
              <dt>Invalidation</dt><dd>collection-scoped on writes</dd>
            </dl>
          </div>
        </div>

        <div className="card">
          <div className="card-head">
            <div className="title">File-ops Cache</div>
            <span className="pill teal mono">3-tier LRU</span>
          </div>
          <div className="card-body">
            <div className="col" style={{gap:14}}>
              {[
                ["File content cache", 78, 100, "files"],
                ["Summary cache", 432, 500, "summaries"],
                ["File list cache (TTL 60s)", 18, 50, "collections"],
              ].map(([n,used,cap,unit]) => (
                <div key={n}>
                  <div className="row" style={{fontSize:12, marginBottom:5}}>
                    <span>{n}</span>
                    <span className="right mono muted">{used}/{cap} {unit}</span>
                  </div>
                  <div className="bar"><span className="amber" style={{width:`${used/cap*100}%`}}/></div>
                </div>
              ))}
            </div>
            <div className="divider"/>
            <dl className="kv">
              <dt>Owner</dt><dd>FileOperationsManager</dd>
              <dt>Eviction</dt><dd>LRU · per-entry TTL</dd>
              <dt>Triggers</dt><dd>file watcher events · explicit clear_*</dd>
            </dl>
          </div>
        </div>
      </div>
    </div>
  );
};

window.MonitoringScreen = MonitoringScreen;
