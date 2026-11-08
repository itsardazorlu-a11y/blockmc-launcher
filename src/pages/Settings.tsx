import { useEffect, useState } from "react";
import { api } from "../api";

export default function Settings({ theme, setTheme }: { theme: "dark" | "light"; setTheme: (t: "dark" | "light") => void }) {
  const [dir, setDir] = useState("");
  const [error, setError] = useState("");
  useEffect(() => { api.dataFolder().then(setDir).catch((e) => setError(String(e))); }, []);
  return (
    <>
      <div className="head"><h1>Settings</h1></div>
      <section className="panel">
        <h2>Appearance</h2>
        <div className="seg" role="group" aria-label="Theme">
          <button aria-pressed={theme === "dark"} onClick={() => setTheme("dark")}>Dark</button>
          <button aria-pressed={theme === "light"} onClick={() => setTheme("light")}>Light</button>
        </div>
      </section>
      <section className="panel">
        <h2>Your data</h2>
        <p className="meta">Instances, mods and worlds are stored here:</p>
        <code className="path">{dir || "…"}</code>
        <div style={{ marginTop: 12 }}><button className="btn" onClick={() => api.openDataFolder().catch((e) => setError(String(e)))}>Open folder</button></div>
        {error && <p className="err">{error}</p>}
      </section>
      <section className="panel"><h2>About</h2><p className="meta">BlockMC 1.0.0 · mod and modpack manager powered by Modrinth. BlockMC is not affiliated with Modrinth, Mojang or Microsoft.</p></section>
    </>
  );
}
