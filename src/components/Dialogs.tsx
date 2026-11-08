import { useState } from "react";

export function ConfirmDialog({ title, message, confirmLabel, danger, onConfirm, onCancel }: {
  title: string; message: string; confirmLabel: string; danger?: boolean; onConfirm: () => void; onCancel: () => void;
}) {
  return (
    <div className="scrim" onClick={onCancel}>
      <div className="modal" role="alertdialog" aria-modal="true" aria-label={title} onClick={(e) => e.stopPropagation()}>
        <h2>{title}</h2>
        <p className="meta" style={{ fontSize: 15 }}>{message}</p>
        <div className="row">
          <button className="btn" autoFocus onClick={onCancel}>Cancel</button>
          <button className={`btn ${danger ? "danger" : "primary"}`} onClick={onConfirm}>{confirmLabel}</button>
        </div>
      </div>
    </div>
  );
}

export function RenameDialog({ initial, onSave, onCancel }: { initial: string; onSave: (name: string) => Promise<void>; onCancel: () => void }) {
  const [name, setName] = useState(initial);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  async function submit() {
    setBusy(true); setError("");
    try { await onSave(name); } catch (e) { setError(String(e)); setBusy(false); }
  }
  return (
    <div className="scrim" onClick={onCancel}>
      <form className="modal" role="dialog" aria-modal="true" aria-label="Rename instance" onClick={(e) => e.stopPropagation()}
        onSubmit={(e) => { e.preventDefault(); if (name.trim() && !busy) submit(); }}>
        <h2>Rename instance</h2>
        <input className="field" autoFocus value={name} onChange={(e) => setName(e.target.value)} aria-label="Instance name" />
        {error && <div className="err">{error}</div>}
        <div className="row">
          <button type="button" className="btn" onClick={onCancel}>Cancel</button>
          <button type="submit" className="btn primary" disabled={busy || !name.trim()}>Save</button>
        </div>
      </form>
    </div>
  );
}
