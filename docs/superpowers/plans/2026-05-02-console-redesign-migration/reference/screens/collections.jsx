// Collections screen
const { StatusPill, Sparkline } = window.UI;
const { Icons } = window;

const CollectionsScreen = () => {
  const [filter, setFilter] = useState("");
  const [selected, setSelected] = useState(window.MOCK.collections[0]);

  const list = window.MOCK.collections.filter(c => c.name.toLowerCase().includes(filter.toLowerCase()));

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Collections</h1>
          <p className="page-sub">{window.MOCK.collections.length} collections · 587,963 total vectors · 6.7 GB on disk</p>
        </div>
        <div className="row" style={{gap:8}}>
          <button className="btn"><Icons.filter size={13}/>Filter</button>
          <button className="btn"><Icons.refresh size={13}/>Reindex all</button>
          <button className="btn primary"><Icons.plus size={13}/>Create collection</button>
        </div>
      </div>

      <div className="grid grid-1-2" style={{gap:14}}>
        {/* List */}
        <div className="card">
          <div className="card-head">
            <input className="input" placeholder="Filter collections…" value={filter} onChange={e=>setFilter(e.target.value)}
              style={{height:30, padding:"4px 10px", fontSize:12}}/>
          </div>
          <div className="card-body tight">
            <div style={{maxHeight:560, overflowY:"auto"}}>
              {list.map(c => (
                <div key={c.name}
                  onClick={()=>setSelected(c)}
                  style={{
                    padding:"12px 14px",
                    borderBottom:"1px solid var(--border)",
                    cursor:"pointer",
                    background: selected.name===c.name ? "var(--panel-hi)" : "transparent",
                    borderLeft: selected.name===c.name ? "2px solid var(--teal)" : "2px solid transparent",
                  }}>
                  <div className="row" style={{marginBottom:4}}>
                    <Icons.database size={13} className="muted"/>
                    <span style={{fontSize:13, fontWeight:500}}>{c.name}</span>
                    <span className="right"><StatusPill status={c.status}/></span>
                  </div>
                  <div className="row mono" style={{fontSize:11, color:"var(--text-2)", gap:14, marginLeft:21}}>
                    <span>{c.vectors.toLocaleString()} vec</span>
                    <span>{c.dim}d</span>
                    <span>{c.size}</span>
                    <span>{c.indexType}</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Detail */}
        <div className="col" style={{gap:14}}>
          <div className="card">
            <div className="card-head">
              <div className="row" style={{gap:10}}>
                <Icons.database size={16} className="muted"/>
                <div>
                  <div style={{fontSize:15, fontWeight:600}}>{selected.name}</div>
                  <div className="mono muted-2" style={{fontSize:11}}>collection · last indexed {selected.lastIndex}</div>
                </div>
              </div>
              <div className="row" style={{gap:6}}>
                <button className="btn sm"><Icons.refresh size={11}/>Reindex</button>
                <button className="btn sm"><Icons.copy size={11}/>Copy ID</button>
                <button className="btn sm magenta"><Icons.trash size={11}/>Delete</button>
              </div>
            </div>
            <div className="card-body">
              <div className="grid grid-4" style={{gap:14, marginBottom: 14}}>
                <div>
                  <div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>Vectors</div>
                  <div className="tnum" style={{fontSize:22, fontWeight:600}}>{selected.vectors.toLocaleString()}</div>
                </div>
                <div>
                  <div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>Dimension</div>
                  <div className="tnum" style={{fontSize:22, fontWeight:600}}>{selected.dim}</div>
                </div>
                <div>
                  <div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>Storage</div>
                  <div className="tnum" style={{fontSize:22, fontWeight:600}}>{selected.size}</div>
                </div>
                <div>
                  <div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em"}}>p99 latency</div>
                  <div className="tnum" style={{fontSize:22, fontWeight:600}}>{selected.p99}<span style={{fontSize:13, color:"var(--text-2)"}}>ms</span></div>
                </div>
              </div>
              <dl className="kv">
                <dt>Index type</dt><dd>{selected.indexType} · M=16, ef=200</dd>
                <dt>Distance</dt><dd>cosine (pre-normalised)</dd>
                <dt>Quantization</dt><dd><span className="pill teal">{selected.quantization}</span> · 4× compression</dd>
                <dt>Embedding</dt><dd>{selected.provider} <span className="muted">· dim {selected.dim}</span></dd>
                <dt>Created</dt><dd>2025-11-04 14:22:08 UTC</dd>
                <dt>Owner</dt><dd>admin@hivellm</dd>
                <dt>WAL offset</dt><dd>8,811,998</dd>
              </dl>
            </div>
          </div>

          <div className="grid grid-2" style={{gap:14}}>
            <div className="card">
              <div className="card-head"><div className="title">Query throughput · 24h</div><span className="sub">qpm</span></div>
              <div className="card-body" style={{padding:"18px 16px"}}>
                <Sparkline data={window.MOCK.spark(40, selected.queriesPerMin, selected.queriesPerMin*0.2)} width={420} height={100} color="var(--teal)"/>
                <div className="row mono" style={{fontSize:11, color:"var(--text-2)", justifyContent:"space-between", marginTop:6}}>
                  <span>−24h</span><span>−12h</span><span>now</span>
                </div>
              </div>
            </div>
            <div className="card">
              <div className="card-head"><div className="title">Vector growth · 7d</div></div>
              <div className="card-body" style={{padding:"18px 16px"}}>
                <Sparkline data={window.MOCK.spark(40, selected.vectors/1000, 8)} width={420} height={100} color="var(--magenta)"/>
                <div className="row mono" style={{fontSize:11, color:"var(--text-2)", justifyContent:"space-between", marginTop:6}}>
                  <span>−7d</span><span>−3d</span><span>now</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

window.CollectionsScreen = CollectionsScreen;
