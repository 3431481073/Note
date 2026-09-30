// 键盘快捷键。返回清理函数。
export interface ShortcutHandlers {
  focusSearch: () => void;
}

export function registerShortcuts(h: ShortcutHandlers): () => void {
  const onKey = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      h.focusSearch();
    }
  };
  window.addEventListener("keydown", onKey);
  return () => window.removeEventListener("keydown", onKey);
}
