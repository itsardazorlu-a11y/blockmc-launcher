import { ReactNode, useEffect, useRef, useState } from "react";
import { Fetcher, Hit, SearchPage } from "../api";
import HitCard from "./HitCard";
import Pager from "./Pager";

const SORTS = [
  { id: "relevance", label: "Relevance" }, { id: "downloads", label: "Download count" },
  { id: "follows", label: "Follow count" }, { id: "newest", label: "Recently published" }, { id: "updated", label: "Recently updated" },
];
const SIZES = [10, 20, 30, 50];

/** Modrinth-style search page: query, sort, page size, filters slot, results and numbered pages. */
export default function Browse({ fetchPage, action, extraKey = "", filters, hint }: {
  fetchPage: Fetcher; action: (h: Hit) => ReactNode; extraKey?: string; filters?: ReactNode; hint?: string;
}) {
  const [input, setInput] = useState("");
  const [p, setP] = useState({ q: "", sort: "relevance", size: 20, page: 1 });
  const [data, setData] = useState<SearchPage>({ hits: [], total_hits: 0 });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const seq = useRef(0);
  const top = useRef<HTMLDivElement>(null);
  const first = useRef(true);

  useEffect(() => { if (first.current) { first.current = false; return; } setP((x) => ({ ...x, page: 1 })); }, [extraKey]);

  useEffect(() => {
    const id = ++seq.current;
    setLoading(true); setError("");
    fetchPage(p.q, p.sort, (p.page - 1) * p.size, p.size)
      .then((d) => { if (id === seq.current) setData(d); })
      .catch((e) => { if (id === seq.current) { setData({ hits: [], total_hits: 0 }); setError(String(e)); } })
      .finally(() => { if (id === seq.current) setLoading(false); });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [p, extraKey]);

  const pages = Math.max(1, Math.ceil(data.total_hits / p.size));
  const go = (page: number) => { setP((x) => ({ ...x, page })); top.current?.scrollIntoView({ behavior: "smooth", block: "start" }); };

  return (
    <div ref={top}>
      <form className="toolbar" onSubmit={(e) => { e.preventDefault(); setP((x) => ({ ...x, q: input.trim(), page: 1 })); }}>
        <input className="field grow" placeholder={hint ?? "Search…"} value={input} onChange={(e) => setInput(e.target.value)} />
        <label className="lab">Sort by
          <select className="field" value={p.sort} onChange={(e) => setP((x) => ({ ...x, sort: e.target.value, page: 1 }))}>
            {SORTS.map((s) => <option key={s.id} value={s.id}>{s.label}</option>)}
          </select></label>
        <label className="lab">Show
          <select className="field" value={p.size} onChange={(e) => setP((x) => ({ ...x, size: Number(e.target.value), page: 1 }))}>
            {SIZES.map((n) => <option key={n} value={n}>{n}</option>)}
          </select></label>
        <button className="btn primary" type="submit">Search</button>
      </form>
      {filters}
      <div className="meta" style={{ margin: "4px 0 12px" }}>{loading ? "Searching…" : `${new Intl.NumberFormat().format(data.total_hits)} results`}</div>
      {error && <p className="err">{error}</p>}
      {!loading && !error && data.hits.length === 0 && <div className="empty"><h2>No results</h2><p>Try a different search or change the filters.</p></div>}
      <div className={`results${loading ? " dim" : ""}`}>{data.hits.map((h) => <HitCard key={h.project_id} hit={h} action={action(h)} />)}</div>
      <Pager page={p.page} pages={pages} onChange={go} />
    </div>
  );
}
