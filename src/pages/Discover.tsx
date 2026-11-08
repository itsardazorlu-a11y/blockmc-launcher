import { useEffect, useState } from "react";
import { api, Hit } from "../api";
import Browse from "../components/Browse";

export default function Discover({ onInstalled }: { onInstalled: () => void }) {
  const [versions, setVersions] = useState<string[]>([]);
  const [mc, setMc] = useState("");
  const [loader, setLoader] = useState("");
  const [busy, setBusy] = useState("");
  const [note, setNote] = useState("");
  const [error, setError] = useState("");

  useEffect(() => { api.gameVersions().then(setVersions).catch(() => setVersions([])); }, []);

  async function install(h: Hit) {
    if (busy) return;
    setBusy(h.project_id); setError(""); setNote("");
    try { const inst = await api.installModpack(h.project_id); setNote(`Installed "${inst.name}" (${inst.mc_version}, ${inst.loader}). Find it in your Library.`); }
    catch (e) { setError(String(e)); }
    setBusy("");
  }

  return (
    <>
      <div className="head"><h1>Discover modpacks</h1></div>
      {note && <div className="banner ok">{note} <button className="btn" onClick={onInstalled}>Open Library</button></div>}
      {error && <p className="err">{error}</p>}
      {busy && <div className="banner">Installing… large packs can take several minutes. Keep this window open.</div>}
      <Browse
        hint="Search modpacks"
        extraKey={`${mc}|${loader}`}
        fetchPage={(q, s, o, l) => api.searchModpacks(q, s, o, l, mc || null, loader || null)}
        filters={
          <div className="toolbar">
            <label className="lab">Minecraft version
              <select className="field" value={mc} onChange={(e) => setMc(e.target.value)}>
                <option value="">Any version</option>{versions.map((v) => <option key={v} value={v}>{v}</option>)}
              </select></label>
            <label className="lab">Loader
              <select className="field" value={loader} onChange={(e) => setLoader(e.target.value)}>
                <option value="">Any loader</option><option value="fabric">Fabric</option><option value="forge">Forge</option>
                <option value="neoforge">NeoForge</option><option value="quilt">Quilt</option>
              </select></label>
          </div>
        }
        action={(h) => <button className="btn primary" disabled={!!busy} onClick={() => install(h)}>{busy === h.project_id ? "Installing…" : "Install"}</button>}
      />
    </>
  );
}
