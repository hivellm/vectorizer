// Search Playground — 4 search types as tabs
const { Icons } = window;

const SEARCH_TYPES = [
  { id: "intelligent", label: "Intelligent", desc: "Multi-query generation + domain expansion + MMR diversification", icon: "sparkles" },
  { id: "semantic", label: "Semantic", desc: "High-precision search with semantic reranking and similarity threshold", icon: "zap" },
  { id: "contextual", label: "Contextual", desc: "Context-aware search with metadata filtering and context reranking", icon: "layers" },
  { id: "multi", label: "Multi-collection", desc: "Cross-collection search with intelligent reranking and deduplication", icon: "globe" },
];

const MOCK_RESULTS = [
  { score: 0.942, title: "Capability Registry — REST/MCP Parity", snippet: "REST and MCP must have identical functionality. The registry currently covers 31 MCP tools + the canonical <mark>REST counterparts</mark> for 29 of them.", coll: "vectorizer-docs", id: "vec_8h2k3l9d", path: "docs/architecture/capabilities.md" },
  { score: 0.918, title: "Multi-collection Search Implementation", snippet: "<mark>Cross-collection search</mark> with intelligent reranking and deduplication. Performance metrics show 3-4× better coverage than traditional search.", coll: "vectorizer-docs", id: "vec_9q2k1p8r", path: "docs/intelligent-search.md" },
  { score: 0.886, title: "Phase 9 — Advanced Integrations", snippet: "Advanced Embedding Models: ONNX and Real Models (MiniLM, E5, MPNet, GTE) with GPU acceleration enabled across all collections.", coll: "hivellm-codebase", id: "vec_4m7y0n3t", path: "ROADMAP.md:540" },
  { score: 0.871, title: "MCP Tool Schema Parity Test", snippet: "registry_and_legacy_agree_on_overlapping_input_schemas enforces byte-for-byte schema parity between the two sources of truth.", coll: "hivellm-codebase", id: "vec_x4n9q2k0", path: "src/server/mcp/tools.rs:142" },
  { score: 0.842, title: "QueryCache: cache the response not the SearchResult", snippet: "A naive design would cache Vec<SearchResult> and let each handler JSON-serialize on every hit. Caching the already-formatted Value trades a few bytes…", coll: "vectorizer-docs", id: "vec_l1z6v4b2", path: "docs/architecture/caching.md" },
  { score: 0.811, title: "Batch insert with text-based input", snippet: "batch_insert_texts endpoint with automatic embedding generation. Server-side embedding generation for consistency across all interfaces.", coll: "vectorizer-docs", id: "vec_t9c5h8e3", path: "docs/api/batch.md" },
];

const SearchScreen = () => {
  const [type, setType] = useState("intelligent");
  const [query, setQuery] = useState("How does REST/MCP parity work in the capability registry?");
  const [collection, setCollection] = useState("all");
  const [limit, setLimit] = useState(10);
  const [threshold, setThreshold] = useState(0.7);
  const [running, setRunning] = useState(false);
  const [hasResults, setHasResults] = useState(true);

  const run = () => {
    setRunning(true);
    setTimeout(() => { setRunning(false); setHasResults(true); }, 600);
  };

  const cur = SEARCH_TYPES.find(s => s.id === type);
  const CurIc = Icons[cur.icon];

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">Search Playground</h1>
          <p className="page-sub">Test the four intelligent search modes against any collection · results streamed from <span className="mono" style={{color:"var(--text)"}}>POST /{type}_search</span></p>
        </div>
      </div>

      {/* Tabs */}
      <div className="card" style={{marginBottom:14}}>
        <div className="tabs">
          {SEARCH_TYPES.map(s => {
            const Ic = Icons[s.icon];
            return (
              <div key={s.id} className={`tab ${type===s.id?"active":""}`} onClick={()=>setType(s.id)}>
                <Ic size={13}/>{s.label}
              </div>
            );
          })}
        </div>
        <div style={{padding:"12px 18px", borderBottom:"1px solid var(--border)", display:"flex", alignItems:"center", gap:10}}>
          <CurIc size={14} style={{color:"var(--teal-hi)"}}/>
          <span style={{fontSize:12, color:"var(--text-1)"}}>{cur.desc}</span>
        </div>

        <div className="card-body" style={{padding:"16px 18px"}}>
          <div className="grid" style={{gridTemplateColumns:"1fr 200px 110px 110px auto", gap:10, alignItems:"end"}}>
            <div className="field">
              <label className="field-label">Query</label>
              <input className="input" value={query} onChange={e=>setQuery(e.target.value)} placeholder="Type a natural-language query…"/>
            </div>
            <div className="field">
              <label className="field-label">Collection</label>
              <select className="input" value={collection} onChange={e=>setCollection(e.target.value)}>
                <option value="all">all collections</option>
                {window.MOCK.collections.map(c=> <option key={c.name} value={c.name}>{c.name}</option>)}
              </select>
            </div>
            <div className="field">
              <label className="field-label">Limit</label>
              <input className="input mono" type="number" value={limit} onChange={e=>setLimit(+e.target.value)}/>
            </div>
            <div className="field">
              <label className="field-label">Threshold</label>
              <input className="input mono" type="number" step="0.05" value={threshold} onChange={e=>setThreshold(+e.target.value)}/>
            </div>
            <button className="btn primary" onClick={run} disabled={running}>
              {running ? <><Icons.refresh size={13}/>Running…</> : <><Icons.search size={13}/>Run search</>}
            </button>
          </div>
        </div>
      </div>

      {/* Results + Request */}
      <div className="grid" style={{gridTemplateColumns:"1.7fr 1fr", gap:14}}>
        <div className="card">
          <div className="card-head">
            <div className="title">Results</div>
            <div className="row" style={{gap:8}}>
              <span className="pill muted mono">{hasResults ? `${MOCK_RESULTS.length} hits` : "—"}</span>
              <span className="pill teal mono">38 ms · MMR=0.5</span>
            </div>
          </div>
          <div className="card-body tight">
            {hasResults ? MOCK_RESULTS.map((r, i) => (
              <div key={i} className="result">
                <div className="rank">#{i+1}</div>
                <div className="body">
                  <div className="title">{r.title}</div>
                  <div className="snippet" dangerouslySetInnerHTML={{__html: r.snippet}}/>
                  <div className="meta">
                    <span><Icons.database size={10}/> {r.coll}</span>
                    <span>{r.id}</span>
                    <span>{r.path}</span>
                  </div>
                </div>
                <div className="score">
                  <div className="v">{r.score.toFixed(3)}</div>
                  <div className="l">cosine</div>
                </div>
              </div>
            )) : (
              <div style={{padding:40, textAlign:"center", color:"var(--text-2)"}}>Run a query to see results</div>
            )}
          </div>
        </div>

        <div className="col" style={{gap:14}}>
          <div className="card">
            <div className="card-head"><div className="title">Request</div><span className="pill muted mono">POST</span></div>
            <div className="card-body">
              <pre className="code"><span className="c"># curl</span>{"\n"}
curl -X <span className="k">POST</span> http://localhost:15002/<span className="s">{type}_search</span> \{"\n"}
  -H <span className="s">"Authorization: Bearer $VK"</span> \{"\n"}
  -d <span className="s">'{`{`}{"\n"}
    "query": "{query.length > 38 ? query.slice(0,38)+"…" : query}",{"\n"}
    "collection": "{collection}",{"\n"}
    "limit": {limit},{"\n"}
    "threshold": {threshold},{"\n"}
    "mmr_lambda": 0.5{"\n"}
  {`}`}'</span></pre>
            </div>
          </div>
          <div className="card">
            <div className="card-head"><div className="title">Pipeline</div></div>
            <div className="card-body">
              <ol style={{margin:0, padding:0, listStyle:"none"}}>
                {[
                  ["embed", "BM25 · 384-d", "1.2ms"],
                  ["expand", "+3 query variants", "2.1ms"],
                  ["search", "HNSW · ef=200", "28.4ms"],
                  ["rerank", "MMR λ=0.5", "5.8ms"],
                  ["dedup", "score Δ < 0.02", "0.5ms"],
                ].map((s, i) => (
                  <li key={i} style={{display:"grid", gridTemplateColumns:"24px 1fr auto", padding:"7px 0", borderBottom: i<4 ? "1px solid var(--border)" : "none", alignItems:"center"}}>
                    <span className="mono muted-2" style={{fontSize:11}}>{i+1}.</span>
                    <div>
                      <div style={{fontSize:12, fontWeight:500}}>{s[0]}</div>
                      <div className="muted mono" style={{fontSize:10}}>{s[1]}</div>
                    </div>
                    <span className="mono" style={{fontSize:11, color:"var(--teal-hi)"}}>{s[2]}</span>
                  </li>
                ))}
              </ol>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

window.SearchScreen = SearchScreen;
