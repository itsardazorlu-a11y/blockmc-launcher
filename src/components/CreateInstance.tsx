import { useState } from "react";
import { api, Loader } from "../api";

export default function CreateInstance({ onClose, onCreated }: { onClose: () => void; onCreated: () => void }) {
  const [name, setName] = useState("");
  const [version, setVersion] = useState("1.20.1");
  const [loader, setLoader] = useState<Loader>("fabric");
  const [error, setError] = useState("");

  async function submit() {
    try { await api.create(name, version, loader); onCreated(); } catch (e) { setError(String(e)); }
  }

  return (
    <div className="scrim" onClick={onClose}>
      <div className="modal" role="dialog" aria-modal="true" aria-label="New instance" onClick={(e) => e.stopPropagation()}>
        <h2>New instance</h2>
        <input className="field" autoFocus placeholder="Name" value={name} onChange={(e) => setName(e.target.value)} />
        <input className="field" placeholder="Minecraft version" value={version} onChange={(e) => setVersion(e.target.value)} />
        <select className="field" value={loader} onChange={(e) => setLoader(e.target.value as Loader)}>
          <option value="vanilla">Vanilla</option><option value="fabric">Fabric</option><option value="forge">Forge</option>
          <option value="neoforge">NeoForge</option><option value="quilt">Quilt</option>
        </select>
        {error && <div className="err">{error}</div>}
        <div className="row">
          <button className="btn" onClick={onClose}>Cancel</button>
          <button className="btn primary" onClick={submit}>Create instance</button>
        </div>
      </div>
    </div>
  );
}
