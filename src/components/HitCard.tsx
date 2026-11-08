import { ReactNode } from "react";
import { Hit } from "../api";

export const fmt = (n: number) => n >= 1e6 ? `${(n / 1e6).toFixed(1)}M` : n >= 1e3 ? `${(n / 1e3).toFixed(n >= 1e4 ? 0 : 1)}k` : String(n);

export default function HitCard({ hit, action }: { hit: Hit; action: ReactNode }) {
  return (
    <article className="hit">
      {hit.icon_url ? <img className="icon big" src={hit.icon_url} alt="" loading="lazy" /> : <div className="icon big">{hit.title.charAt(0).toUpperCase()}</div>}
      <div className="grow">
        <div><strong>{hit.title}</strong> <span className="meta">by {hit.author}</span></div>
        <p className="desc">{hit.description}</p>
        <div className="tags">{hit.categories.slice(0, 5).map((c) => <span className="tag" key={c}>{c}</span>)}</div>
      </div>
      <div className="side-col">
        <div className="stats"><span title="Downloads">⬇ {fmt(hit.downloads)}</span><span title="Followers">♥ {fmt(hit.follows)}</span></div>
        {action}
      </div>
    </article>
  );
}
