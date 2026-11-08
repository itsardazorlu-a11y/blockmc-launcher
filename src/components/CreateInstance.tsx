import { useEffect, useState } from "react";
import { api, Loader, LOADERS } from "../api";
import VersionPicker from "./VersionPicker";

export default function CreateInstance({ onClose, onCreated }: { onClose: () => void; onCreated: () => void }) {
  const [name, setName] = useState("");
  const [versions, setVersions] = useState<string[] | null>(null);
  const [version, setVersion] = useState("");
  const [loader, setLoader] = useState<Loader>("fabric");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api.gameVersions()
      .then((v) => { setVersions(v); setVersion(v[0] ?? ""); })
      .catch((e) => { setError(`Could not load the version list (${e}). Type a version instead.`); setVersions([]); });
  }, []);

  async function submit() {
    setBusy(true); setError("");
    try { await api.create(name, version.trim(), loader); onCreated(); } catch (e) { setError(String(e)); setBusy(false); }
  }

  return (
    <div className="scrim" onClick={onClose}>
      <form className="modal" role="dialog" aria-modal="true" aria-label="New instance" onClick={(e) => e.stopPropagation()}
        onSubmit={(e) => { e.preventDefault(); if (name.trim() && version.trim() && !busy) submit(); }}>
        <h2>New instance</h2>
        <label className="lab">Name
          <input className="field" autoFocus placeholder="My survival world" value={name} onChange={(e) => setName(e.target.value)} /></label>
        <div className="lab">Mod loader
          <div className="seg" role="group" aria-label="Mod loader">
            {LOADERS.map((l) => <button type="button" key={l.id} aria-pressed={loader === l.id} onClick={() => setLoader(l.id)}>{l.label}</button>)}
          </div></div>
        <div className="lab">Minecraft version{version && <span className="meta"> · selected {version}</span>}
          {versions === null ? <div className="meta">Loading versions…</div>
            : versions.length > 0 ? <VersionPicker versions={versions} value={version} onChange={setVersion} />
            : <input className="field" placeholder="e.g. 1.21.1" value={version} onChange={(e) => setVersion(e.target.value)} />}
        </div>
        {error && <div className="err">{error}</div>}
        <div className="row">
          <button type="button" className="btn" onClick={onClose}>Cancel</button>
          <button type="submit" className="btn primary" disabled={busy || !name.trim() || !version.trim()}>{busy ? "Creating…" : "Create instance"}</button>
        </div>
      </form>
    </div>
  );
}
