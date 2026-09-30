import { useEffect, useRef, useState } from "react";
import type { Tag } from "../types";

type Option = Tag | string;

function norm(o: Option): Tag {
  if (typeof o === "string") {
    return { id: o, name: o, description: "", extinguished: false, deleted_at: null, card_count: 0 };
  }
  return o;
}

interface TagSelectProps {
  label: string;
  options: Option[];
  selected: string[];
  color: "mint" | "proj";
  allowAdd?: boolean;
  onToggle: (id: string) => void;
  onAdd: (name: string) => void;
  onDelete?: (id: string) => void;
  onManage?: () => void;
}

export default function TagSelect({
  label,
  options,
  selected,
  color,
  allowAdd = true,
  onToggle,
  onAdd,
  onDelete,
  onManage,
}: TagSelectProps) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState("");
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDoc = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onDoc);
    return () => document.removeEventListener("mousedown", onDoc);
  }, [open]);

  const normOptions = options.map(norm);
  const activeOptions = normOptions.filter((t) => !t.extinguished);
  // 卡片显示 = 所有激活标签中指向它的那些；熄灭的不显示。
  const selectedTags = selected
    .map((id) => normOptions.find((t) => t.id === id))
    .filter((t): t is Tag => !!t && !t.extinguished);

  function submit() {
    const v = draft.trim();
    if (v) onAdd(v);
    setDraft("");
  }

  return (
    <div className="tag-select" ref={rootRef}>
      <button
        type="button"
        className={`drop-trigger drop-trigger-${color}`}
        onClick={() => setOpen((o) => !o)}
      >
        {label}
        {selectedTags.length > 0 && <span className="drop-count">{selectedTags.length}</span>}
      </button>
      {selectedTags.map((t) => (
        <span key={t.id} className={`chip chip-${color}`}>
          {t.name}
          {t.description ? <span className="chip-tooltip">{t.description}</span> : null}
          <button type="button" className="chip-x" aria-label="移除" onClick={() => onToggle(t.id)}>
            ×
          </button>
        </span>
      ))}
      {open && (
        <div className="drop-panel">
          <div className="drop-list">
            {activeOptions.length === 0 ? (
              <div className="drop-empty">暂无</div>
            ) : (
              activeOptions.map((t) => (
                <div key={t.id} className={`drop-opt${selected.includes(t.id) ? " on" : ""}`}>
                  <button type="button" className="drop-opt-main" onClick={() => onToggle(t.id)}>
                    {selected.includes(t.id) ? "✓ " : ""}
                    {t.name}
                  </button>
                  {onDelete && (
                    <button
                      type="button"
                      className="drop-opt-del"
                      aria-label="删除"
                      onClick={() => onDelete(t.id)}
                    >
                      ×
                    </button>
                  )}
                </div>
              ))
            )}
          </div>
          {allowAdd && (
            <div className="drop-new">
              <input
                value={draft}
                placeholder="新建…"
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") submit();
                  if (e.key === "Escape") setOpen(false);
                }}
              />
              <button type="button" onClick={submit}>
                添加
              </button>
            </div>
          )}
          {onManage && (
            <button type="button" className="drop-manage" onClick={onManage}>
              标签管理…
            </button>
          )}
        </div>
      )}
    </div>
  );
}
