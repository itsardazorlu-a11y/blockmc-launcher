import { useCallback, useEffect, useState } from "react";
import { api, Instance, InstalledMod, Kind, loaderLabel, UpdateInfo } from "../api";
import Browse from "../components/Browse";
import { ConfirmDialog } from "../components/Dialogs";

type Tab = "installed" | Kind;
const TABS: { id: Tab; label: string }[] = [
  { id: "installed", label: "Installed" }, { id: "mod", label: "Mods" },
  { id: "resourcepack", label: "Resource packs" }, { id: "shader", label: "Shader packs" },
];

export default function InstancePage({ instance, onBack }: { instance: Instance; onBack: () => void }) {
  const [tab, setTab] = useState<Tab>("installed");
  const [installed, setInstalled] = useState<InstalledMod[]>([]);
  const [updates, setUpdates] = useState<UpdateInfo[]>([]);
  const [busy, setBusy] = useState<Set<string>>(new Set());
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [confirm, setConfirm] = useState<{ m: InstalledMod; deps: string[] } | null>(null);

  const reload = useCallback(() => api.installed(instance.id).then(setInstalled).catch((e) => setError(String(e))), [instance.id]);
  useEffect(() => { reload(); }, [reload]);

  async function work(key: string, fn: () => Promise<string | void>) {
    setBusy((b) => new Set(b).add(key)); setError(""); setNote("");
    try { const msg = await fn(); if (msg) setNote(msg); await reload(); } catch (e) { setError(String(e)); }
    setBusy((b) => { const n = new Set(b); n.delete(key); return n; });
  }
  const isInstalled = (id: string) => installed.some((i) => i.project_id === id);

  return (
    <>
      <div className="head">
        <button className="btn" onClick={onBack}>← Back</button>
        <h1>{instance.name}</h1>
        <span className="tag">{instance.mc_version}</span><span className="tag">{loaderLabel(instance.loader)}</span>
        <button className="btn" onClick={() => api.openFolder(instance.id).catch((e) => setError(String(e)))}>Open folder</button>
      </div>
      <div className="tabs" role="tablist">
        {TABS.map((t) => <button key={t.id} role="tab" aria-selected={tab === t.id} className="tab" onClick={() => setTab(t.id)}>{t.label}</button>)}
      </div>
      {error && <p className="err">{error}</p>}
      {note && <p className="ok">{note}</p>}

      {confirm && <ConfirmDialog title={`Remove ${confirm.m.title}?`} confirmLabel="Remove anyway" danger
        message={`${confirm.deps.join(", ")} ${confirm.deps.length === 1 ? "needs" : "need"} this to work. Removing it may stop ${confirm.deps.length === 1 ? "that mod" : "those mods"} from loading.`}
        onCancel={() => setConfirm(null)}
        onConfirm={() => { const c = confirm; setConfirm(null); work(c.m.project_id, async () => { await api.uninstall(instance.id, c.m.project_id); return `${c.m.title} removed.`; }); }} />}
      {tab === "installed" ? (
        <>
          <div className="row" style={{ justifyContent: "flex-start", marginBottom: 16 }}>
            <button className="btn" disabled={busy.has("check") || installed.length === 0}
              onClick={() => work("check", async () => { const u = await api.checkUpdates(instance.id); setUpdates(u); return u.length ? `${u.length} update(s) available.` : "Everything is up to date."; })}>
              {busy.has("check") ? "Checking…" : "Check for updates"}
            </button>
          </div>
          {installed.length === 0 ? (
            <div className="empty"><h2>Nothing installed</h2><p>Open the Mods, Resource packs or Shader packs tab to add some.</p></div>
          ) : (
            <div className="list">
              {installed.map((m) => {
                const up = updates.find((u) => u.project_id === m.project_id);
                return (
                  <div className="card inst" key={m.project_id}>
                    <div className="grow"><strong>{m.title}</strong><div className="meta">{m.version_number} · {m.kind} · {m.filename}</div></div>
                    {up && <button className="btn primary" disabled={busy.has(m.project_id)}
                      onClick={() => work(m.project_id, async () => { await api.update(instance.id, m.project_id); setUpdates((x) => x.filter((u) => u.project_id !== m.project_id)); return `${m.title} updated to ${up.latest}.`; })}>
                      Update to {up.latest}</button>}
                    <button className="btn danger" disabled={busy.has(m.project_id)}
                      onClick={() => work(m.project_id, async () => {
                        const deps = await api.dependents(instance.id, m.project_id);
                        if (deps.length) { setConfirm({ m, deps }); return; }
                        await api.uninstall(instance.id, m.project_id);
                        return `${m.title} removed.`;
                      })}>Remove</button>
                  </div>
                );
              })}
            </div>
          )}
        </>
      ) : (
        <Browse
          key={tab}
          hint={`Search ${TABS.find((t) => t.id === tab)!.label.toLowerCase()} for Minecraft ${instance.mc_version}`}
          extraKey={`${instance.id}:${tab}`}
          fetchPage={(q, s, o, l) => api.search(instance.id, tab, q, s, o, l)}
          action={(h) => isInstalled(h.project_id)
            ? <span className="meta">✓ Installed</span>
            : <button className="btn primary" disabled={busy.has(h.project_id)}
                onClick={() => work(h.project_id, async () => { const r = await api.install(instance.id, h.project_id, tab); return r.length > 1 ? `Installed ${h.title} and ${r.length - 1} required dependency(ies).` : `Installed ${h.title}.`; })}>
                {busy.has(h.project_id) ? "Installing…" : "Install"}</button>}
        />
      )}
    </>
  );
}
