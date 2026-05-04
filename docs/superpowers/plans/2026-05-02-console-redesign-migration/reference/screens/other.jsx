// Replication, API Keys, MCP Tools, Settings
const { StatusPill } = window.UI;
const { Icons } = window;

const ReplicationScreen = () => (
  <div className="page">
    <div className="page-head">
      <div>
        <h1 className="page-title">Replication</h1>
        <p className="page-sub">Master → replica state · 4 connected replicas across 3 regions</p>
      </div>
      <div className="row" style={{gap:8}}>
        <span className="pill green live"><span className="dot green"/>master · accepting writes</span>
        <button className="btn"><Icons.plus size={13}/>Add replica</button>
      </div>
    </div>

    <div className="grid grid-4" style={{marginBottom:14}}>
      <div className="kpi accent-teal"><div className="label">Master offset</div><div className="value tnum" style={{fontSize:20}}>8,812,004</div><div className="delta neutral mono">+184/s</div></div>
      <div className="kpi"><div className="label">Connected replicas</div><div className="value tnum">4 / 4</div><div className="delta up">all in-sync</div></div>
      <div className="kpi accent-amber"><div className="label">Max lag</div><div className="value tnum">164<span className="unit">ms</span></div><div className="delta down">us-east-02</div></div>
      <div className="kpi"><div className="label">Write concern</div><div className="value" style={{fontSize:18}}>Majority(3)</div><div className="delta neutral mono">wait ≤ 200ms</div></div>
    </div>

    <div className="card">
      <div className="card-head"><div className="title">Replicas</div></div>
      <div className="card-body tight">
        <table className="tbl">
          <thead><tr><th>ID</th><th>Region</th><th>Offset</th><th>Lag</th><th>Last ACK</th><th>Status</th><th></th></tr></thead>
          <tbody>
            {window.MOCK.replicas.map(r => (
              <tr key={r.id}>
                <td className="mono" style={{fontWeight:500}}>{r.id}</td>
                <td><span className="pill muted mono">{r.region}</span></td>
                <td className="num">{r.offset.toLocaleString()}</td>
                <td className="num"><span style={{color: r.lag > 100 ? "var(--amber)" : r.lag > 0 ? "var(--text-1)" : "var(--green)"}}>{r.lag}ms</span></td>
                <td className="num muted">{r.lag === 0 ? "just now" : "1s ago"}</td>
                <td><StatusPill status={r.status}/></td>
                <td><button className="btn sm">Resync</button></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  </div>
);

const ApiKeysScreen = () => (
  <div className="page">
    <div className="page-head">
      <div>
        <h1 className="page-title">API Keys</h1>
        <p className="page-sub">Localhost-only management · 5 active keys · 7.0M calls in last 30d</p>
      </div>
      <div className="row" style={{gap:8}}>
        <button className="btn"><Icons.refresh size={13}/>Audit log</button>
        <button className="btn primary"><Icons.plus size={13}/>Generate key</button>
      </div>
    </div>

    <div className="card">
      <div className="card-head"><div className="title">Keys</div><span className="sub">{window.MOCK.apiKeys.length} active</span></div>
      <div className="card-body tight">
        <table className="tbl">
          <thead><tr><th>Name</th><th>Key</th><th>Role</th><th>Calls (30d)</th><th>Last used</th><th>Created</th><th></th></tr></thead>
          <tbody>
            {window.MOCK.apiKeys.map(k => (
              <tr key={k.id}>
                <td><div className="row" style={{gap:8}}><Icons.shield size={13} className="muted"/><span style={{fontWeight:500}}>{k.name}</span></div></td>
                <td className="mono" style={{fontSize:11, color:"var(--text-2)"}}>{k.masked}</td>
                <td>
                  <span className={`pill ${k.role === "Admin" ? "magenta" : k.role === "ReadOnly" ? "muted" : "teal"} mono`}>{k.role}</span>
                </td>
                <td className="num">{k.calls.toLocaleString()}</td>
                <td className="num muted">{k.lastUsed}</td>
                <td className="num muted">{k.created}</td>
                <td><div className="row" style={{gap:4, justifyContent:"flex-end"}}>
                  <button className="btn sm"><Icons.copy size={11}/></button>
                  <button className="btn sm"><Icons.trash size={11}/></button>
                </div></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>

    <div className="card" style={{marginTop:14}}>
      <div className="card-head"><div className="title">Permission matrix</div></div>
      <div className="card-body tight">
        <table className="tbl">
          <thead><tr><th>Role</th><th>List/Search</th><th>Insert/Update</th><th>Delete</th><th>Reindex</th><th>Admin endpoints</th></tr></thead>
          <tbody>
            {[
              ["Admin", true, true, true, true, true],
              ["ReadWrite", true, true, true, false, false],
              ["Mcp", true, true, false, false, false],
              ["ReadOnly", true, false, false, false, false],
            ].map((row, i) => (
              <tr key={i}>
                <td><span className={`pill ${i===0?"magenta":i===1?"teal":i===2?"amber":"muted"} mono`}>{row[0]}</span></td>
                {row.slice(1).map((v, j) => (
                  <td key={j}>{v ? <Icons.check size={14} style={{color:"var(--green)"}}/> : <Icons.x size={14} style={{color:"var(--text-3)"}}/>}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  </div>
);

const McpScreen = () => (
  <div className="page">
    <div className="page-head">
      <div>
        <h1 className="page-title">MCP Tools</h1>
        <p className="page-sub">31 tools registered · StreamableHTTP at <span className="mono" style={{color:"var(--text)"}}>http://localhost:15002/mcp</span> · capability registry parity ✓</p>
      </div>
      <div className="row" style={{gap:8}}>
        <span className="pill green"><span className="dot green"/>schema parity OK</span>
        <button className="btn"><Icons.copy size={13}/>Cursor config</button>
      </div>
    </div>

    <div className="grid grid-3" style={{marginBottom:14}}>
      <div className="kpi accent-teal"><div className="label">Active connections</div><div className="value tnum">12</div><div className="delta neutral">Cursor · 8 · others · 4</div></div>
      <div className="kpi"><div className="label">Tool calls today</div><div className="value tnum">373K</div><div className="delta up">+8.1%</div></div>
      <div className="kpi accent-magenta"><div className="label">Errors</div><div className="value tnum">0</div><div className="delta up">100% success</div></div>
    </div>

    <div className="card">
      <div className="card-head"><div className="title">Tools</div><span className="sub">sorted by usage</span></div>
      <div className="card-body tight">
        <table className="tbl">
          <thead><tr><th>Tool</th><th>Calls (24h)</th><th>p99</th><th>Schema</th><th>Status</th></tr></thead>
          <tbody>
            {window.MOCK.mcpTools.map(t => (
              <tr key={t.name}>
                <td className="mono" style={{fontWeight:500}}>{t.name}</td>
                <td className="num">{t.calls.toLocaleString()}</td>
                <td className="num">{t.p99}ms</td>
                <td><span className="pill teal mono">parity ✓</span></td>
                <td><StatusPill status={t.status}/></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  </div>
);

const SettingsScreen = () => (
  <div className="page">
    <div className="page-head">
      <div><h1 className="page-title">Settings</h1><p className="page-sub">Server configuration · /etc/vectorizer/config.yaml</p></div>
    </div>
    <div className="grid grid-2" style={{gap:14}}>
      <div className="card">
        <div className="card-head"><div className="title">General</div></div>
        <div className="card-body"><dl className="kv">
          <dt>Server name</dt><dd>hivellm-prod-01</dd>
          <dt>Bind address</dt><dd>127.0.0.1:15002</dd>
          <dt>Workspace</dt><dd>/var/lib/vectorizer</dd>
          <dt>Log level</dt><dd>info</dd>
          <dt>Telemetry</dt><dd>Prometheus + tracing</dd>
        </dl></div>
      </div>
      <div className="card">
        <div className="card-head"><div className="title">Defaults</div></div>
        <div className="card-body"><dl className="kv">
          <dt>Embedding</dt><dd>BM25</dd>
          <dt>Index</dt><dd>HNSW (M=16, ef=200)</dd>
          <dt>Quantization</dt><dd>SQ-8bit (auto)</dd>
          <dt>Distance</dt><dd>cosine</dd>
          <dt>Cache TTL</dt><dd>300s</dd>
        </dl></div>
      </div>
    </div>
  </div>
);

window.ReplicationScreen = ReplicationScreen;
window.ApiKeysScreen = ApiKeysScreen;
window.McpScreen = McpScreen;
window.SettingsScreen = SettingsScreen;
