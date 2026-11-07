import { useEffect, useState } from "react";
import Library from "./pages/Library";
import Soon from "./pages/Soon";

type Page = "library" | "discover" | "accounts" | "settings";
const NAV: { id: Page; label: string }[] = [
  { id: "library", label: "Library" }, { id: "discover", label: "Discover" },
  { id: "accounts", label: "Accounts" }, { id: "settings", label: "Settings" },
];

export default function App() {
  const [page, setPage] = useState<Page>("library");
  const [theme, setTheme] = useState<"dark" | "light">(
    () => (localStorage.getItem("theme") as "dark" | "light") ?? "dark");
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
  }, [theme]);

  return (
    <div className="shell">
      <nav className="side" aria-label="Main">
        <div className="brand">BlockMC</div>
        {NAV.map((n) => (
          <button key={n.id} className="nav" aria-current={page === n.id ? "page" : undefined} onClick={() => setPage(n.id)}>{n.label}</button>
        ))}
        <div className="spacer" />
        <button className="nav" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
          {theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
        </button>
      </nav>
      <main className="main">
        {page === "library" && <Library />}
        {page === "discover" && <Soon title="Discover" phase="Phase 4" text="Browse Modrinth mods and modpacks here." />}
        {page === "accounts" && <Soon title="Accounts" phase="Phase 3" text="Sign in with Microsoft to launch the game." />}
        {page === "settings" && <Soon title="Settings" phase="Phase 3" text="Java, memory and launch arguments will live here." />}
      </main>
    </div>
  );
}
