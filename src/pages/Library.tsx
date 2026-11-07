import { useEffect, useMemo, useState } from "react";
import { api, Instance, Loader } from "../api";
import CreateInstance from "../components/CreateInstance";
import InstancePage from "./InstancePage";
import ModpackDialog from "../components/ModpackDialog";

export default function Library() {
  const [items, setItems] = useState<Instance[]>([]);
  const [q, setQ] = useState("");
  const [view, setView] = useState<"grid" | "list">("grid");
  const [creating, setCreating] = useState(false);
  const [packs, setPacks] = useState(false);
  const [error, setError] = useState("");
  const [open, setOpen] = useState<Instance | null>(null);

  const load = () => api.list().then(setItems).catch((e) => setError(String(e)));
  useEffect(() => { load(); }, []);

  const shown = useMemo(() => items.filter((i) => i.name.toLowerCase().includes(q.toLowerCase())), [items, q]);

  async function remove(i: Instance) {
    if (!window.confirm(`Delete "${i.name}" and all of its files? This cannot be undone.`)) return;
    try { await api.remove(i.id); await load(); } catch (e) { setError(String(e)); }
  }

  if (open) return <InstancePage instance={open} onBack={() => setOpen(null)} />;

  return (
    <>
      <div className="head">
        <h1>Library</h1>
        <input className="field" style={{ width: 220 }} placeholder="Search instances" value={q} onChange={(e) => setQ(e.target.value)} />
        <button className="btn" onClick={() => setView(view === "grid" ? "list" : "grid")}>{view === "grid" ? "List view" : "Grid view"}</button>
        <button className="btn" onClick={() => setPacks(true)}>Install modpack</button>
        <button className="btn primary" onClick={() => setCreating(true)}>New instance</button>
      </div>
      {error && <p className="err">{error}</p>}
      {items.length === 0 ? (
        <div className="empty"><h2>No instances yet</h2><p>Create your first instance to get an isolated Minecraft setup.</p></div>
      ) : (
        <div className={view === "grid" ? "grid" : "list"}>
          {shown.map((i) => (
            <div className="card" key={i.id}>
              <div className="icon">{i.name.charAt(0).toUpperCase()}</div>
              <div className="grow"><strong>{i.name}</strong><div className="meta">{i.mc_version} · {label(i.loader)}</div></div>
              <button className="btn primary" onClick={() => setOpen(i)}>Open</button>
              <button className="btn danger" onClick={() => remove(i)}>Delete</button>
            </div>
          ))}
        </div>
      )}
      {packs && <ModpackDialog onClose={() => setPacks(false)} onInstalled={() => { setPacks(false); load(); }} />}
      {creating && <CreateInstance onClose={() => setCreating(false)} onCreated={() => { setCreating(false); load(); }} />}
    </>
  );
}

const label = (l: Loader) => ({ vanilla: "Vanilla", fabric: "Fabric", forge: "Forge", neoforge: "NeoForge", quilt: "Quilt" })[l];
