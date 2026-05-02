// Command Palette
const { Icons } = window;

const CMD_ITEMS = [
  { section: "Navigation", items: [
    { label: "Go to Overview", icon: "dashboard", id: "nav:overview", hint: "G O" },
    { label: "Go to Collections", icon: "collections", id: "nav:collections", hint: "G C" },
    { label: "Go to Search Playground", icon: "search", id: "nav:search", hint: "G S" },
    { label: "Go to Vector Browser", icon: "vectors", id: "nav:vectors", hint: "G V" },
    { label: "Go to Monitoring", icon: "activity", id: "nav:monitoring", hint: "G M" },
    { label: "Go to API Keys", icon: "keys", id: "nav:keys", hint: "G K" },
  ]},
  { section: "Actions", items: [
    { label: "Create new collection…", icon: "plus", id: "act:create", hint: "N C" },
    { label: "Generate API key…", icon: "shield", id: "act:key", hint: "N K" },
    { label: "Force WAL checkpoint", icon: "zap", id: "act:checkpoint" },
    { label: "Reindex all collections", icon: "refresh", id: "act:reindex" },
  ]},
  { section: "Collections", items: window.MOCK.collections.slice(0,4).map(c => ({
    label: c.name, icon: "database", id: `coll:${c.name}`,
  }))},
];

const CommandPalette = ({ open, onClose, onNavigate }) => {
  const [q, setQ] = useState("");
  const [active, setActive] = useState(0);

  useEffect(() => { if (open) setQ(""); }, [open]);

  if (!open) return null;
  const flat = CMD_ITEMS.flatMap(s => s.items.map(it => ({...it, section: s.section})))
    .filter(it => it.label.toLowerCase().includes(q.toLowerCase()));

  const go = (it) => {
    if (it.id.startsWith("nav:")) onNavigate(it.id.slice(4));
    onClose();
  };

  return (
    <div className="cmd-overlay" onClick={onClose}>
      <div className="cmd-panel" onClick={e=>e.stopPropagation()}>
        <input className="cmd-input" placeholder="Search or type a command…" value={q} onChange={e=>{setQ(e.target.value); setActive(0);}} autoFocus
          onKeyDown={e => {
            if (e.key === "ArrowDown") { setActive(a => Math.min(a+1, flat.length-1)); e.preventDefault(); }
            if (e.key === "ArrowUp") { setActive(a => Math.max(a-1, 0)); e.preventDefault(); }
            if (e.key === "Enter" && flat[active]) go(flat[active]);
            if (e.key === "Escape") onClose();
          }}/>
        <div className="cmd-list">
          {CMD_ITEMS.map(s => {
            const items = s.items.filter(it => it.label.toLowerCase().includes(q.toLowerCase()));
            if (!items.length) return null;
            return (
              <div key={s.section}>
                <div className="cmd-section">{s.section}</div>
                {items.map((it, i) => {
                  const Ic = Icons[it.icon];
                  const idx = flat.indexOf(flat.find(x => x.id === it.id));
                  return (
                    <div key={it.id} className={`cmd-row ${idx===active?"active":""}`} onClick={()=>go(it)} onMouseEnter={()=>setActive(idx)}>
                      <Ic className="icon"/>
                      <span>{it.label}</span>
                      {it.hint && <span className="hint">{it.hint}</span>}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

window.CommandPalette = CommandPalette;
