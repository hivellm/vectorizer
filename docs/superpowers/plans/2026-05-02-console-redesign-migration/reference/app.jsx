// Main App with Tweaks integration
const { useState, useEffect } = React;
const { Sidebar, Topbar } = window.Chrome;
const { CommandPalette } = window;

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "accentTeal": "#1fb6b6",
  "accentMagenta": "#e5337a",
  "bgDepth": "deep",
  "density": "high",
  "radius": 8,
  "showHexBg": true
}/*EDITMODE-END*/;

const SCREENS = {
  overview: () => <window.OverviewScreen/>,
  collections: () => <window.CollectionsScreen/>,
  search: () => <window.SearchScreen/>,
  vectors: () => <window.VectorsScreen/>,
  monitoring: () => <window.MonitoringScreen/>,
  replication: () => <window.ReplicationScreen/>,
  keys: () => <window.ApiKeysScreen/>,
  mcp: () => <window.McpScreen/>,
  settings: () => <window.SettingsScreen/>,
};

const CRUMBS = {
  overview: ["Vectorizer", "Overview"],
  collections: ["Vectorizer", "Collections"],
  search: ["Vectorizer", "Search Playground"],
  vectors: ["Vectorizer", "Vector Browser"],
  monitoring: ["Vectorizer", "Monitoring"],
  replication: ["Vectorizer", "Replication"],
  keys: ["Vectorizer", "API Keys"],
  mcp: ["Vectorizer", "MCP Tools"],
  settings: ["Vectorizer", "Settings"],
};

const App = () => {
  const [active, setActive] = useState("overview");
  const [collapsed, setCollapsed] = useState(false);
  const [cmdOpen, setCmdOpen] = useState(false);
  const [tweaks, setTweak] = window.useTweaks ? window.useTweaks(TWEAK_DEFAULTS) : [TWEAK_DEFAULTS, () => {}];

  // Apply tweaks as CSS vars
  useEffect(() => {
    const r = document.documentElement;
    r.style.setProperty("--teal", tweaks.accentTeal);
    r.style.setProperty("--magenta", tweaks.accentMagenta);
    r.style.setProperty("--radius", tweaks.radius + "px");
    if (tweaks.bgDepth === "deep") {
      r.style.setProperty("--bg", "#0b0e13");
      r.style.setProperty("--bg-1", "#11151c");
    } else if (tweaks.bgDepth === "soft") {
      r.style.setProperty("--bg", "#13171f");
      r.style.setProperty("--bg-1", "#181d27");
    } else {
      r.style.setProperty("--bg", "#0a0a0a");
      r.style.setProperty("--bg-1", "#111111");
    }
    document.body.style.fontSize = tweaks.density === "high" ? "13px" : tweaks.density === "med" ? "14px" : "15px";
  }, [tweaks]);

  // ⌘K
  useEffect(() => {
    const h = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") { e.preventDefault(); setCmdOpen(o => !o); }
      if (e.key === "Escape") setCmdOpen(false);
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, []);

  const Screen = SCREENS[active] || SCREENS.overview;

  return (
    <div className="app" style={{gridTemplateColumns: collapsed ? "60px 1fr" : "232px 1fr"}}>
      <Sidebar active={active} onNavigate={setActive} collapsed={collapsed} onToggleCollapsed={() => setCollapsed(c=>!c)}/>
      <div className="main">
        <Topbar crumbs={CRUMBS[active]} onOpenCmd={() => setCmdOpen(true)} onOpenTweaks={() => window.dispatchEvent(new CustomEvent("__open_tweaks"))}/>
        <Screen/>
      </div>
      <CommandPalette open={cmdOpen} onClose={() => setCmdOpen(false)} onNavigate={setActive}/>

      {/* Tweaks panel */}
      {window.TweaksPanel && window.TweakSection && (
        <window.TweaksPanel title="Tweaks">
          <window.TweakSection title="Accent colors">
            <window.TweakColor label="Primary (teal)" value={tweaks.accentTeal} onChange={v => setTweak("accentTeal", v)}/>
            <window.TweakColor label="Highlight (magenta)" value={tweaks.accentMagenta} onChange={v => setTweak("accentMagenta", v)}/>
          </window.TweakSection>
          <window.TweakSection title="Surface">
            <window.TweakRadio label="Background depth" value={tweaks.bgDepth} options={[
              {value:"deep", label:"Deep"},{value:"soft", label:"Soft"},{value:"true", label:"True black"}
            ]} onChange={v => setTweak("bgDepth", v)}/>
            <window.TweakSlider label="Corner radius" value={tweaks.radius} min={0} max={14} step={1} onChange={v => setTweak("radius", v)}/>
          </window.TweakSection>
          <window.TweakSection title="Density">
            <window.TweakRadio label="Information density" value={tweaks.density} options={[
              {value:"high", label:"High"},{value:"med", label:"Medium"},{value:"low", label:"Low"}
            ]} onChange={v => setTweak("density", v)}/>
          </window.TweakSection>
        </window.TweaksPanel>
      )}
    </div>
  );
};

ReactDOM.createRoot(document.getElementById("root")).render(<App/>);
