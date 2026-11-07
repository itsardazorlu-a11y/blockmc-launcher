import { useEffect, useState } from "react";
import { api, Hit } from "../api";

export default function ModpackDialog({ onClose, onInstalled }: { onClose: () => void; onInstalled: () => void }) {
  const [q, setQ] = useState("");
  const [hits, setHits] = useState<Hit[]>([]);
  const [busy, setBusy] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function search() {
    setLoading(true); setError("");
    try { setHits(await api.searchModpacks(q, "downloads", 0)); } catch (e) { setError(String(e)); }
    setLoading(false);
  }
  useEffect(() => { search(); /* eslint-disable-next-line */ }, []);

  async function install(h: Hit) {
    if (busy) return;
    setBusy(h.project_id); setError("");
    try { await api.installModpack(h.project_id); onInstalled(); } catch (e) { setError(String(e)); setBusy(""); }
  }

  return (
    <div className="scrim" onClick={() => !busy && onClose()}>
      <div className="modal" style={{ width: "min(640px, 94vw)", maxHeight: "86vh" }} role="dialog" aria-modal="true" aria-label="Install modpack" onClick={(e) => e.stopPropagation()}>
        <h2>Install modpack</h2>
        <form className="row" onSubmit={(e) => { e.preventDefault(); search(); }}>
          <input className="field" autoFocus placeholder="Search Modrinth modpacks" value={q} onChange={(e) => setQ(e.target.value)} />
          <button className="btn primary" type="submit" disabled={!!busy}>Search</button>
        </form>
        {error && <div className="err">{error}</div>}
        {busy && <div className="meta">Installing. Large packs can take several minutes, please keep this window open.</div>}
        <div className="list" style={{ overflow: "auto" }}>
          {loading && <div className="meta">Searching…</div>}
          {hits.map((h) => (
            <div className="card" key={h.project_id}>
              {h.icon_url ? <img className="icon" src={h.icon_url} alt="" /> : <div className="icon">{h.title.charAt(0)}</div>}
              <div className="grow"><strong>{h.title}</strong> <span className="meta">by {h.author}</span><div className="meta">{h.description}</div></div>
              <button className="btn primary" disabled={!!busy} onClick={() => install(h)}>{busy === h.project_id ? "Installing…" : "Install"}</button>
            </div>
          ))}
        </div>
        <div className="row"><button className="btn" disabled={!!busy} onClick={onClose}>Close</button></div>
      </div>
    </div>
  );
}
