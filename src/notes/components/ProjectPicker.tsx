import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ProjectPickerProps {
  project: string;
  onSelect: (path: string) => void;
  onBrowse: () => void;
}

export default function ProjectPicker({ project, onSelect, onBrowse }: ProjectPickerProps) {
  const [open, setOpen] = useState(false);
  const [folders, setFolders] = useState<string[]>([]);
  const rootRef = useRef<HTMLDivElement>(null);

  const name = project ? project.split(/[\\/]/).pop() || project : "绑定项目";

  async function loadFolders() {
    try {
      setFolders(await invoke<string[]>("list_project_folders"));
    } catch {
      setFolders([]);
    }
  }

  useEffect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [open]);

  return (
    <div className="proj-wrap" ref={rootRef}>
      <button
        type="button"
        className="proj-btn"
        title={project || "左键选择项目 · 右键绑定文件夹"}
        onClick={() => {
          const next = !open;
          setOpen(next);
          if (next) loadFolders();
        }}
        onContextMenu={(e) => {
          e.preventDefault();
          onBrowse();
        }}
      >
        📁 {name}
      </button>
      {open && (
        <div className="proj-panel">
          {folders.length === 0 ? (
            <div className="proj-empty">无</div>
          ) : (
            folders.map((f) => (
              <button
                type="button"
                key={f}
                className={`proj-opt${f === project ? " on" : ""}`}
                onClick={() => {
                  onSelect(f);
                  setOpen(false);
                }}
              >
                {f.split(/[\\/]/).pop()}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}
