// Shared small UI bits used across screens
const { useState, useEffect, useRef, useMemo } = React;

const Sparkline = ({ data, color = "var(--teal)", width = 80, height = 28, fill = true }) => {
  if (!data || !data.length) return null;
  const min = Math.min(...data), max = Math.max(...data);
  const range = max - min || 1;
  const points = data.map((d, i) => `${(i/(data.length-1))*width},${height - ((d-min)/range)*(height-4) - 2}`).join(" ");
  const area = `0,${height} ${points} ${width},${height}`;
  return (
    <svg width={width} height={height} style={{ display: "block" }}>
      {fill && <polygon points={area} fill={color} opacity="0.12" />}
      <polyline points={points} fill="none" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
};

const StatusPill = ({ status }) => {
  const map = {
    healthy: { cls: "green", label: "healthy" },
    "in-sync": { cls: "green", label: "in-sync" },
    ok: { cls: "green", label: "ok" },
    indexing: { cls: "amber", label: "indexing" },
    "catching-up": { cls: "amber", label: "catching up" },
    warning: { cls: "amber", label: "warning" },
    error: { cls: "red", label: "error" },
  };
  const m = map[status] || { cls: "muted", label: status };
  return <span className={`pill ${m.cls}`}><span className={`dot ${m.cls}`}/>{m.label}</span>;
};

const Ring = ({ value, max = 100, size = 110, label, sub, color = "var(--teal)" }) => {
  const r = size/2 - 8;
  const c = 2 * Math.PI * r;
  const pct = Math.min(value/max, 1);
  return (
    <div style={{ position: "relative", width: size, height: size }}>
      <svg width={size} height={size}>
        <circle cx={size/2} cy={size/2} r={r} fill="none" stroke="var(--bg-3)" strokeWidth="6"/>
        <circle cx={size/2} cy={size/2} r={r} fill="none" stroke={color} strokeWidth="6"
          strokeDasharray={c} strokeDashoffset={c*(1-pct)} strokeLinecap="round"
          transform={`rotate(-90 ${size/2} ${size/2})`} style={{transition: "stroke-dashoffset 0.6s"}}/>
      </svg>
      <div style={{ position: "absolute", inset: 0, display: "grid", placeItems: "center", textAlign: "center" }}>
        <div>
          <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: "-0.02em", fontFeatureSettings: '"tnum" 1' }}>{label}</div>
          <div style={{ fontSize: 10, color: "var(--text-2)", textTransform: "uppercase", letterSpacing: "0.06em", marginTop: 2 }}>{sub}</div>
        </div>
      </div>
    </div>
  );
};

const HexLogo = ({ size = 28 }) => (
  <img src="assets/logo.png" alt="Vectorizer" width={size} height={size} style={{ display: "block" }}/>
);

// Live tick — updates every second, returns an integer
const useTick = (ms = 1500) => {
  const [t, setT] = useState(0);
  useEffect(() => { const id = setInterval(() => setT(x => x+1), ms); return () => clearInterval(id); }, [ms]);
  return t;
};

window.UI = { Sparkline, StatusPill, Ring, HexLogo, useTick };
