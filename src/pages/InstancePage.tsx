import { useCallback, useEffect, useState } from "react";
import { api, Hit, Instance, InstalledMod, Kind, UpdateInfo } from "../api";

type Tab = "installed" | Kind;
const TABS: { id: Tab; label: string }[] = [
  { id: "installed", label: "Installed" }, { id: "mod", label: "Mods" },
  { id: "resourcepack", label: "Resource packs" }, { id: "shader", label: "Shader packs" },
];

export default function InstancePage({ instance, onBack }: { instance: Instance; onBack: () => void }) {
  const [tab, setTab] = useState<Tab>("installed");
  const [installed, setInstalled] = useState<InstalledMod[]>([]);
  const [updates, setUpdates] = useState<UpdateInfo[]>([]);
  const [hits, setHits] = useState<Hit[]>([]);
  const [q, setQ] = useState("");
  const [busy, setBusy] = useState<Set<string>>(new Set());
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const reload = useCallback(() => api.installed(instance.id).then(setInstalled).catch((e) => setError(String(e))), [instance.id]);
  useEffect(() => { reload(); }, [reload]);

  const search = useCallback(async (kind: Kind, query: string) => {
    setLoading(true); setError("");
    try { setHits(await api.search(instance.id, kind, query)); } catch (e) { setHits([]); setError(String(e)); }
    setLoading(false);
  }, [instance.id]);

  useEffect(() => { if (tab !== "installed") search(tab, q); /* eslint-disable-next-line */ }, [tab]);

  async function work(key: string, fn: () => Promise<string | void>) {
    setBusy((b) => new Set(b).add(key)); setError(""); setNote("");
    try { const msg = await fn(); if (msg) setNote(msg); await reload(); } catch (e) { setError(String(e)); }
    setBusy((b) => { const n = new Set(b); n.delete(key); return n; });
  }

  const isInstalled = (id: string) => installed.some((i) => i.project_id === id);
  const kind = tab === "installed" ? "mod" : tab;

  return (
    <>
      <div className="head">
        <button className="btn" onClick={onBack}>Back</button>
        <h1>{instance.name}</h1>
        <span className="meta">{instance.mc_version} · {instance.loader}</span>
        <button className="btn" onClick={() => api.openFolder(instance.id).catch((e) => setError(String(e)))}>Open folder</button>
      </div>
      <div className="tabs" role="tablist">
        {TABS.map((t) => <button key={t.id} role="tab" aria-selected={tab === t.id} className="tab" onClick={() => setTab(t.id)}>{t.label}</button>)}
      </div>
      {error && <p className="err">{error}</p>}
      {note && <p className="ok">{note}</p>}

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
                  <div className="card" key={m.project_id}>
                    <div className="grow"><strong>{m.title}</strong><div className="meta">{m.version_number} · {m.kind} · {m.filename}</div></div>
                    {up && <button className="btn primary" disabled={busy.has(m.project_id)}
                      onClick={() => work(m.project_id, async () => { await api.update(instance.id, m.project_id); setUpdates((x) => x.filter((u) => u.project_id !== m.project_id)); return `${m.title} updated to ${up.latest}.`; })}>
                      Update to {up.latest}</button>}
                    <button className="btn danger" disabled={busy.has(m.project_id)}
                      onClick={() => work(m.project_id, async () => { await api.uninstall(instance.id, m.project_id); return `${m.title} removed.`; })}>Remove</button>
                  </div>
                );
              })}
            </div>
          )}
        </>
      ) : (
        <>
          <form className="row" style={{ marginBottom: 16 }} onSubmit={(e) => { e.preventDefault(); search(tab, q); }}>
            <input className="field" placeholder="Search Modrinth" value={q} onChange={(e) => setQ(e.target.value)} />
            <button className="btn primary" type="submit">Search</button>
          </form>
          {loading && <p className="meta">Searching…</p>}
          {!loading && hits.length === 0 && !error && <div className="empty"><h2>No results</h2><p>Try another search. Results are filtered to Minecraft {instance.mc_version}.</p></div>}
          <div className="list">
            {hits.map((h) => (
              <div className="card" key={h.project_id}>
                {h.icon_url ? <img className="icon" src={h.icon_url} alt="" /> : <div className="icon">{h.title.charAt(0)}</div>}
                <div className="grow"><strong>{h.title}</strong> <span className="meta">by {h.author}</span><div className="meta">{h.description}</div></div>
                {isInstalled(h.project_id)
                  ? <span className="meta">Installed</span>
                  : <button className="btn primary" disabled={busy.has(h.project_id)}
                      onClick={() => work(h.project_id, async () => { const r = await api.install(instance.id, h.project_id, kind); return r.length > 1 ? `Installed ${h.title} and ${r.length - 1} required dependency(ies).` : `Installed ${h.title}.`; })}>
                      {busy.has(h.project_id) ? "Installing…" : "Install"}</button>}
              </div>
            ))}
          </div>
        </>
      )}
    </>
  );
}
