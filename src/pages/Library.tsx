import { useEffect, useMemo, useState } from "react";
import { api, Instance, loaderLabel } from "../api";
import CreateInstance from "../components/CreateInstance";
import { ConfirmDialog, RenameDialog } from "../components/Dialogs";
import InstancePage from "./InstancePage";

export default function Library({ onDiscover }: { onDiscover: () => void }) {
  const [items, setItems] = useState<Instance[]>([]);
  const [q, setQ] = useState("");
  const [view, setView] = useState<"grid" | "list">("grid");
  const [creating, setCreating] = useState(false);
  const [open, setOpen] = useState<Instance | null>(null);
  const [error, setError] = useState("");
  const [deleting, setDeleting] = useState<Instance | null>(null);
  const [renaming, setRenaming] = useState<Instance | null>(null);

  const load = () => api.list().then(setItems).catch((e) => setError(String(e)));
  useEffect(() => { load(); }, []);
  const shown = useMemo(() => items.filter((i) => i.name.toLowerCase().includes(q.toLowerCase())), [items, q]);

  async function remove(i: Instance) {
    setDeleting(null);
    try { await api.remove(i.id); await load(); } catch (e) { setError(String(e)); }
  }
  async function duplicate(i: Instance) {
    setError("");
    try { await api.duplicate(i.id); await load(); } catch (e) { setError(String(e)); }
  }

  if (open) return <InstancePage instance={open} onBack={() => setOpen(null)} />;

  return (
    <>
      <div className="head">
        <h1>Library</h1>
        <input className="field" style={{ width: 220 }} placeholder="Search instances" value={q} onChange={(e) => setQ(e.target.value)} />
        <div className="seg" role="group" aria-label="View">
          <button aria-pressed={view === "grid"} onClick={() => setView("grid")}>Grid</button>
          <button aria-pressed={view === "list"} onClick={() => setView("list")}>List</button>
        </div>
        <button className="btn primary" onClick={() => setCreating(true)}>New instance</button>
      </div>
      {error && <p className="err">{error}</p>}
      {items.length === 0 ? (
        <div className="empty">
          <h2>No instances yet</h2>
          <p>An instance is an isolated set of mods, resource packs and shaders for one Minecraft version.</p>
          <div className="row" style={{ justifyContent: "center" }}>
            <button className="btn primary" onClick={() => setCreating(true)}>Create an instance</button>
            <button className="btn" onClick={onDiscover}>Browse modpacks</button>
          </div>
        </div>
      ) : (
        <div className={view === "grid" ? "grid" : "list"}>
          {shown.map((i) => (
            <div className="card inst" key={i.id}>
              <div className="icon">{i.name.charAt(0).toUpperCase()}</div>
              <div className="grow"><strong>{i.name}</strong><div className="tags"><span className="tag">{i.mc_version}</span><span className="tag">{loaderLabel(i.loader)}</span></div></div>
              <div className="row">
                <button className="btn primary" onClick={() => setOpen(i)}>Open</button>
                <button className="btn" onClick={() => setRenaming(i)}>Rename</button>
                <button className="btn" onClick={() => duplicate(i)}>Duplicate</button>
                <button className="btn danger" onClick={() => setDeleting(i)}>Delete</button>
              </div>
            </div>
          ))}
          {shown.length === 0 && <div className="meta">No instances match "{q}".</div>}
        </div>
      )}
      {deleting && <ConfirmDialog title={`Delete "${deleting.name}"?`} message="This permanently deletes the instance with all of its mods, worlds and settings. This cannot be undone." confirmLabel="Delete instance" danger
        onConfirm={() => remove(deleting)} onCancel={() => setDeleting(null)} />}
      {renaming && <RenameDialog initial={renaming.name} onCancel={() => setRenaming(null)}
        onSave={async (n) => { await api.rename(renaming.id, n); setRenaming(null); await load(); }} />}
      {creating && <CreateInstance onClose={() => setCreating(false)} onCreated={() => { setCreating(false); load(); }} />}
    </>
  );
}
