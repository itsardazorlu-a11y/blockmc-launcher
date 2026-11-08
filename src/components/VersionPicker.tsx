import { useMemo, useState } from "react";

export default function VersionPicker({ versions, value, onChange }: { versions: string[]; value: string; onChange: (v: string) => void }) {
  const [filter, setFilter] = useState("");
  const shown = useMemo(() => versions.filter((v) => v.includes(filter.trim())), [versions, filter]);
  return (
    <div className="vpick">
      <input className="field" placeholder={`Filter ${versions.length} versions`} value={filter} onChange={(e) => setFilter(e.target.value)} aria-label="Filter versions" />
      <div className="vlist" role="listbox" aria-label="Minecraft version">
        {shown.map((v) => (
          <button type="button" role="option" aria-selected={v === value} key={v} className="vopt" onClick={() => onChange(v)}>
            {v}{v === versions[0] && <span className="tag">latest</span>}
          </button>
        ))}
        {shown.length === 0 && <div className="meta" style={{ padding: 8 }}>No matching version</div>}
      </div>
    </div>
  );
}
