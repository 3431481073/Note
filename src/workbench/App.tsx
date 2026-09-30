import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Stats } from "../notes/types";
import "./workbench.css";

export default function App() {
  const [stats, setStats] = useState<Stats>({ active: 0, trash: 0, outline: 0, trash_outline: 0 });

  useEffect(() => {
    invoke<Stats>("get_stats")
      .then(setStats)
      .catch(() => {});
    let un: (() => void) | undefined;
    listen<Stats>("notes://stats", (e) => setStats(e.payload)).then(
      (u) => (un = u),
    );
    return () => un?.();
  }, []);

  return (
    <div className="wb-toolbar">
      <button
        className="wb-btn"
        title="便签"
        onClick={() => invoke("toggle_notes").catch(() => {})}
      >
        <span className="wb-icon">📝</span>
        {stats.active > 0 && <span className="wb-badge">{stats.active}</span>}
      </button>
    </div>
  );
}
