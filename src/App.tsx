import { useEffect, useState } from "react";
import Library from "./pages/Library";
import Discover from "./pages/Discover";
import Settings from "./pages/Settings";

type Page = "library" | "discover" | "settings";
const NAV: { id: Page; label: string }[] = [
  { id: "library", label: "Library" }, { id: "discover", label: "Discover" }, { id: "settings", label: "Settings" },
];

export default function App() {
  const [page, setPage] = useState<Page>("library");
  const [theme, setTheme] = useState<"dark" | "light">(() => (localStorage.getItem("theme") as "dark" | "light") ?? "dark");
  useEffect(() => { document.documentElement.dataset.theme = theme; localStorage.setItem("theme", theme); }, [theme]);

  return (
    <div className="shell">
      <nav className="side" aria-label="Main">
        <div className="brand"><span className="logo" />BlockMC</div>
        {NAV.map((n) => (
          <button key={n.id} className="nav" aria-current={page === n.id ? "page" : undefined} onClick={() => setPage(n.id)}>{n.label}</button>
        ))}
      </nav>
      <main className="main"><div className="content">
        {page === "library" && <Library onDiscover={() => setPage("discover")} />}
        {page === "discover" && <Discover onInstalled={() => setPage("library")} />}
        {page === "settings" && <Settings theme={theme} setTheme={setTheme} />}
      </div></main>
    </div>
  );
}
