// Overview screen
const { Sparkline, StatusPill, Ring, useTick } = window.UI;
const { Icons } = window;

const OverviewScreen = () => {
  const tick = useTick(2000);
  const qps = 2480 + Math.round(Math.sin(tick/2) * 120);
  const mem = 62.4 + Math.sin(tick/3) * 1.2;
  const cpu = 38 + Math.sin(tick/2.5) * 6;
  const conns = 184 + Math.round(Math.cos(tick) * 14);

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Overview</h1>
          <p className="page-sub">Real-time health of the Vectorizer node · cluster <span className="mono" style={{color:"var(--text)"}}>hivellm-prod-01</span></p>
        </div>
        <div className="row" style={{gap:8}}>
          <button className="btn"><Icons.refresh size={13}/>Refresh</button>
          <button className="btn primary"><Icons.plus size={13}/>New Collection</button>
        </div>
      </div>

      {/* KPIs */}
      <div className="grid grid-4" style={{marginBottom: 14}}>
        <div className="kpi accent-teal">
          <div className="label"><Icons.zap size={12}/>Queries / sec</div>
          <div className="value tnum">{qps.toLocaleString()}<span className="unit">qps</span></div>
          <div className="delta up"><Icons.arrowUp size={11}/>+12.4% vs 24h</div>
          <div className="spark"><Sparkline data={window.MOCK.spark(20, 2400, 200)} color="var(--teal)"/></div>
        </div>
        <div className="kpi">
          <div className="label"><Icons.cpu size={12}/>Search latency p99</div>
          <div className="value tnum">2.8<span className="unit">ms</span></div>
          <div className="delta up"><Icons.arrowDown size={11}/>−0.4ms vs 24h</div>
          <div className="spark"><Sparkline data={window.MOCK.spark(20, 2.8, 0.4)} color="var(--text-2)"/></div>
        </div>
        <div className="kpi accent-magenta">
          <div className="label"><Icons.layers size={12}/>Total vectors</div>
          <div className="value tnum">587,963</div>
          <div className="delta neutral"><Icons.arrowUp size={11}/>+1,204 today</div>
          <div className="spark"><Sparkline data={window.MOCK.spark(20, 580, 8)} color="var(--magenta)"/></div>
        </div>
        <div className="kpi">
          <div className="label"><Icons.flame size={12}/>Cache hit rate</div>
          <div className="value tnum">94.2<span className="unit">%</span></div>
          <div className="delta up"><Icons.arrowUp size={11}/>+1.8% vs 24h</div>
          <div className="spark"><Sparkline data={window.MOCK.spark(20, 94, 2)} color="var(--green)"/></div>
        </div>
      </div>

      {/* Top row: System + Quantization */}
      <div className="grid grid-2-1" style={{marginBottom: 14}}>
        <div className="card">
          <div className="card-head">
            <div className="title">System Health</div>
            <span className="pill green live"><span className="dot green"/>healthy · uptime 14d 3h</span>
          </div>
          <div className="card-body">
            <div className="grid grid-3" style={{gap:18, alignItems:"center"}}>
              <div style={{display:"grid", placeItems:"center"}}>
                <Ring value={cpu} max={100} label={`${cpu.toFixed(0)}%`} sub="CPU" color="var(--teal)"/>
              </div>
              <div style={{display:"grid", placeItems:"center"}}>
                <Ring value={mem} max={100} label={`${mem.toFixed(1)}%`} sub="MEMORY" color="var(--magenta)"/>
              </div>
              <div style={{display:"grid", placeItems:"center"}}>
                <Ring value={conns} max={500} label={conns} sub="CONNECTIONS" color="var(--amber)"/>
              </div>
            </div>
            <div className="divider"/>
            <dl className="kv">
              <dt>SIMD backend</dt><dd><span className="pill teal">avx2</span> <span className="muted-2">· 8 lanes f32</span></dd>
              <dt>Server binary</dt><dd>vectorizer 3.0.0 · rustc 1.82</dd>
              <dt>Bind</dt><dd>127.0.0.1:15002 (REST) · /mcp (StreamableHTTP)</dd>
              <dt>Total memory</dt><dd>4.21 GB / 16 GB · saved 1.2 GB by quantization</dd>
              <dt>Workspace</dt><dd>/var/lib/vectorizer · 8 collections</dd>
            </dl>
          </div>
        </div>

        <div className="card">
          <div className="card-head">
            <div className="title">Quantization</div>
            <span className="sub">SQ-8bit · default</span>
          </div>
          <div className="card-body">
            <div style={{textAlign:"center", marginBottom:14}}>
              <div style={{fontSize:36, fontWeight:600, letterSpacing:"-0.02em"}}>4.0×</div>
              <div className="muted" style={{fontSize:11, textTransform:"uppercase", letterSpacing:"0.06em"}}>compression ratio</div>
            </div>
            <div className="col" style={{gap:10}}>
              <div>
                <div className="row" style={{fontSize:11, marginBottom:4}}>
                  <span className="muted">MAP score</span><span className="right mono">+8.9%</span>
                </div>
                <div className="bar"><span style={{width:"82%"}}/></div>
              </div>
              <div>
                <div className="row" style={{fontSize:11, marginBottom:4}}>
                  <span className="muted">Recall@10</span><span className="right mono">98.4%</span>
                </div>
                <div className="bar"><span className="magenta" style={{width:"98%"}}/></div>
              </div>
              <div>
                <div className="row" style={{fontSize:11, marginBottom:4}}>
                  <span className="muted">Memory saved</span><span className="right mono">1.2 GB</span>
                </div>
                <div className="bar"><span className="amber" style={{width:"68%"}}/></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Collections + events */}
      <div className="grid grid-2-1">
        <div className="card">
          <div className="card-head">
            <div className="title">Top Collections</div>
            <a className="btn ghost sm">View all <Icons.chevron size={11}/></a>
          </div>
          <div className="card-body tight">
            <table className="tbl">
              <thead><tr>
                <th>Name</th><th>Vectors</th><th>QPM</th><th>p99</th><th>Quant</th><th>Status</th>
              </tr></thead>
              <tbody>
                {window.MOCK.collections.slice(0,6).map(c => (
                  <tr key={c.name}>
                    <td><div style={{display:"flex",alignItems:"center",gap:8}}>
                      <Icons.database size={13} className="muted"/>
                      <span style={{fontWeight:500}}>{c.name}</span>
                      <span className="pill muted">{c.dim}d</span>
                    </div></td>
                    <td className="num">{c.vectors.toLocaleString()}</td>
                    <td className="num">{c.queriesPerMin.toLocaleString()}</td>
                    <td className="num">{c.p99}ms</td>
                    <td><span className="pill teal">{c.quantization}</span></td>
                    <td><StatusPill status={c.status}/></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        <div className="card">
          <div className="card-head">
            <div className="title">Recent Events</div>
            <span className="pill green live"><span className="dot green"/>live</span>
          </div>
          <div className="card-body tight">
            <div className="scroll-body">
              {window.MOCK.events.map((e, i) => (
                <div key={i} style={{padding:"10px 14px", borderBottom:"1px solid var(--border)", display:"flex", gap:10, alignItems:"flex-start"}}>
                  <span className="mono muted-2" style={{fontSize:10, paddingTop:2, minWidth:54}}>{e.t}</span>
                  <span className={`dot ${e.level==="ok"?"green":e.level==="warn"?"amber":"teal"}`} style={{marginTop:6}}/>
                  <span style={{fontSize:12, color:"var(--text-1)"}}>{e.msg}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

window.OverviewScreen = OverviewScreen;
