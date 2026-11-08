export default function Pager({ page, pages, onChange }: { page: number; pages: number; onChange: (p: number) => void }) {
  if (pages <= 1) return null;
  const nums = new Set<number>([1, pages, page - 1, page, page + 1]);
  const shown = [...nums].filter((n) => n >= 1 && n <= pages).sort((a, b) => a - b);
  const items: (number | "gap")[] = [];
  shown.forEach((n, i) => { if (i > 0 && n - shown[i - 1] > 1) items.push("gap"); items.push(n); });
  return (
    <nav className="pager" aria-label="Pages">
      <button className="pg" disabled={page <= 1} onClick={() => onChange(page - 1)} aria-label="Previous page">‹</button>
      {items.map((n, i) => n === "gap"
        ? <span key={`g${i}`} className="gap">…</span>
        : <button key={n} className="pg" aria-current={n === page ? "page" : undefined} onClick={() => onChange(n)}>{n}</button>)}
      <button className="pg" disabled={page >= pages} onClick={() => onChange(page + 1)} aria-label="Next page">›</button>
    </nav>
  );
}
