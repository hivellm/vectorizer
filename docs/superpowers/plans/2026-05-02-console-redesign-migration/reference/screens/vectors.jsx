// Vector Browser
const { Icons } = window;

const VectorsScreen = () => {
  const [collection, setCollection] = useState(window.MOCK.collections[0].name);
  const [selected, setSelected] = useState(window.MOCK.vectors[0]);
  const [query, setQuery] = useState("");

  // fake values for embedding visualization
  const dims = useMemo(() => Array.from({length: 96}, (_, i) => Math.sin(i*0.4 + (selected ? selected.id.length : 0)) * 0.6 + Math.cos(i*0.7) * 0.3), [selected]);

  const filtered = window.MOCK.vectors.filter(v => v.text.toLowerCase().includes(query.toLowerCase()));

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Vector Browser</h1>
          <p className="page-sub">Inspect raw embeddings, payloads and norms across collections</p>
        </div>
        <div className="row" style={{gap:8}}>
          <select className="input" style={{width:220, fontSize:12}} value={collection} onChange={e=>setCollection(e.target.value)}>
            {window.MOCK.collections.map(c => <option key={c.name} value={c.name}>{c.name}</option>)}
          </select>
          <button className="btn"><Icons.plus size={13}/>Insert vector</button>
        </div>
      </div>

      <div className="grid" style={{gridTemplateColumns:"1.4fr 1fr", gap:14}}>
        <div className="card">
          <div className="card-head">
            <input className="input" placeholder="Filter by text or id…" value={query} onChange={e=>setQuery(e.target.value)} style={{height:30, padding:"4px 10px", fontSize:12}}/>
            <span className="pill muted mono right">{filtered.length} / {window.MOCK.vectors.length}</span>
          </div>
          <div className="card-body tight">
            <table className="tbl">
              <thead><tr>
                <th style={{width:120}}>ID</th><th>Text</th><th>Norm</th><th>Dim</th>
              </tr></thead>
              <tbody>
                {filtered.map(v => (
                  <tr key={v.id} className={selected.id===v.id?"active":""} onClick={()=>setSelected(v)}>
                    <td className="id">{v.id}</td>
                    <td style={{maxWidth:380, overflow:"hidden", textOverflow:"ellipsis", whiteSpace:"nowrap"}}>{v.text}</td>
                    <td className="num">{v.norm.toFixed(3)}</td>
                    <td className="num muted">{v.dim}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        <div className="col" style={{gap:14}}>
          <div className="card">
            <div className="card-head">
              <div className="row" style={{gap:8}}>
                <Icons.vectors size={14} className="muted"/>
                <span className="mono" style={{fontSize:12}}>{selected.id}</span>
              </div>
              <div className="row" style={{gap:6}}>
                <button className="btn sm"><Icons.copy size={11}/>Copy</button>
                <button className="btn sm magenta"><Icons.trash size={11}/>Delete</button>
              </div>
            </div>
            <div className="card-body">
              <div className="muted" style={{fontSize:10, textTransform:"uppercase", letterSpacing:"0.06em", marginBottom:6}}>Source text</div>
              <div style={{fontSize:13, lineHeight:1.6, marginBottom:14}}>{selected.text}</div>
              <dl className="kv">
                <dt>Collection</dt><dd>{collection}</dd>
                <dt>Dimension</dt><dd>{selected.dim}</dd>
                <dt>L2 norm</dt><dd>{selected.norm.toFixed(6)}</dd>
                <dt>Embedding</dt><dd>BM25 · pre-normalised</dd>
                <dt>Inserted</dt><dd>2026-04-22 09:14:22 UTC</dd>
                <dt>Payload</dt><dd>{`{ "source": "docs", "section": "architecture" }`}</dd>
              </dl>
            </div>
          </div>

          <div className="card">
            <div className="card-head">
              <div className="title">Embedding · first 96 of {selected.dim} dims</div>
              <span className="pill muted mono">f32</span>
            </div>
            <div className="card-body">
              <svg viewBox="0 0 480 100" width="100%" height="100" preserveAspectRatio="none">
                {dims.map((v, i) => {
                  const w = 480 / dims.length;
                  const h = Math.abs(v) * 46;
                  const y = v >= 0 ? 50 - h : 50;
                  const color = v >= 0 ? "var(--teal)" : "var(--magenta)";
                  return <rect key={i} x={i*w + 0.5} y={y} width={w-1} height={h} fill={color} opacity="0.85"/>;
                })}
                <line x1="0" y1="50" x2="480" y2="50" stroke="var(--border)" strokeWidth="1"/>
              </svg>
              <div className="row mono" style={{fontSize:10, color:"var(--text-3)", justifyContent:"space-between", marginTop:6}}>
                <span>dim 0</span><span>dim 48</span><span>dim 96</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

window.VectorsScreen = VectorsScreen;
