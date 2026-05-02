// Inline SVG icons (Lucide-style stroke icons)
const Icon = ({ d, size = 16, fill = "none", strokeWidth = 1.6, ...rest }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill={fill} stroke="currentColor" strokeWidth={strokeWidth} strokeLinecap="round" strokeLinejoin="round" {...rest}>
    {d}
  </svg>
);

const Icons = {
  dashboard: (p) => <Icon {...p} d={<><rect x="3" y="3" width="7" height="9"/><rect x="14" y="3" width="7" height="5"/><rect x="14" y="12" width="7" height="9"/><rect x="3" y="16" width="7" height="5"/></>}/>,
  collections: (p) => <Icon {...p} d={<><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v6c0 1.66 4 3 9 3s9-1.34 9-3V5"/><path d="M3 11v6c0 1.66 4 3 9 3s9-1.34 9-3v-6"/></>}/>,
  search: (p) => <Icon {...p} d={<><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></>}/>,
  vectors: (p) => <Icon {...p} d={<><circle cx="6" cy="6" r="2"/><circle cx="18" cy="6" r="2"/><circle cx="6" cy="18" r="2"/><circle cx="18" cy="18" r="2"/><path d="M8 6h8M6 8v8M18 8v8M8 18h8"/></>}/>,
  monitor: (p) => <Icon {...p} d={<><rect x="3" y="4" width="18" height="12" rx="1"/><path d="M8 20h8M12 16v4"/></>}/>,
  keys: (p) => <Icon {...p} d={<><circle cx="7" cy="14" r="4"/><path d="m10.5 11 8-8 2 2-2 2 2 2-3 3-2-2-3 3"/></>}/>,
  mcp: (p) => <Icon {...p} d={<><polygon points="12,2 22,8 22,16 12,22 2,16 2,8"/><polyline points="12,8 16,10 16,14 12,16 8,14 8,10 12,8"/></>}/>,
  settings: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></>}/>,
  plus: (p) => <Icon {...p} d={<><path d="M12 5v14M5 12h14"/></>}/>,
  zap: (p) => <Icon {...p} d={<polygon points="13,2 3,14 12,14 11,22 21,10 12,10"/>}/>,
  cpu: (p) => <Icon {...p} d={<><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"/></>}/>,
  database: (p) => <Icon {...p} d={<><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></>}/>,
  bolt: (p) => <Icon {...p} d={<polygon points="13,2 3,14 12,14 11,22 21,10 12,10"/>}/>,
  layers: (p) => <Icon {...p} d={<><polygon points="12,2 2,7 12,12 22,7 12,2"/><polyline points="2,17 12,22 22,17"/><polyline points="2,12 12,17 22,12"/></>}/>,
  activity: (p) => <Icon {...p} d={<polyline points="22,12 18,12 15,21 9,3 6,12 2,12"/>}/>,
  chevron: (p) => <Icon {...p} d={<polyline points="9,18 15,12 9,6"/>}/>,
  copy: (p) => <Icon {...p} d={<><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></>}/>,
  trash: (p) => <Icon {...p} d={<><polyline points="3,6 5,6 21,6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></>}/>,
  bell: (p) => <Icon {...p} d={<><path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/></>}/>,
  filter: (p) => <Icon {...p} d={<polygon points="22,3 2,3 10,12.46 10,19 14,21 14,12.46"/>}/>,
  sparkles: (p) => <Icon {...p} d={<><path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M5.6 18.4l2.1-2.1M16.3 7.7l2.1-2.1"/></>}/>,
  globe: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 0 1 0 20 15 15 0 0 1 0-20"/></>}/>,
  shield: (p) => <Icon {...p} d={<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>}/>,
  flame: (p) => <Icon {...p} d={<path d="M8.5 14.5A2.5 2.5 0 0 0 11 17c1.7 0 3-1.3 3-3 0-1.4-.5-2.2-1.5-3.2C11.3 9.5 11 8 11 7c0-1.7-2-3-2-3s-1 3-3 5c-1.5 1.5-2 3-2 4 0 2.7 2.2 5 5 5"/>}/>,
  panel: (p) => <Icon {...p} d={<><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18"/></>}/>,
  panel2: (p) => <Icon {...p} d={<><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M15 3v18"/></>}/>,
  arrowDown: (p) => <Icon {...p} d={<><path d="M12 5v14"/><polyline points="19,12 12,19 5,12"/></>}/>,
  arrowUp: (p) => <Icon {...p} d={<><path d="M12 19V5"/><polyline points="5,12 12,5 19,12"/></>}/>,
  check: (p) => <Icon {...p} d={<polyline points="20,6 9,17 4,12"/>}/>,
  x: (p) => <Icon {...p} d={<><path d="M18 6 6 18M6 6l12 12"/></>}/>,
  refresh: (p) => <Icon {...p} d={<><polyline points="23,4 23,10 17,10"/><polyline points="1,20 1,14 7,14"/><path d="M3.5 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.65 4.36A9 9 0 0 0 20.5 15"/></>}/>,
  hex: (p) => <Icon {...p} d={<polygon points="12,2 22,7 22,17 12,22 2,17 2,7"/>}/>,
};

window.Icons = Icons;
