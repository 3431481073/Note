import type { Mode, Stats, View } from "../types";

interface BottomBarProps {
  view: View;
  stats: Stats;
  mode: Mode;
  onNew: () => void;
  onToggleTrash: () => void;
}

export default function BottomBar({
  view,
  stats,
  mode,
  onNew,
  onToggleTrash,
}: BottomBarProps) {
  const isTrash = view === "trash";
  const name = mode === "card" ? "Note" : "视图";
  const trashCount = mode === "card" ? stats.trash : stats.trash_outline;
  return (
    <div className="bottom-bar">
      {!isTrash && (
        <button
          className="bb-btn bb-btn-new"
          title={`新建${name}`}
          tabIndex={-1}
          onClick={(e) => {
            e.currentTarget.blur();
            onNew();
          }}
        >
          ＋
        </button>
      )}
      <button
        className="bb-btn bb-btn-trash"
        title={isTrash ? `返回${name}` : "回收站"}
        tabIndex={-1}
        onClick={(e) => {
          e.currentTarget.blur();
          onToggleTrash();
        }}
      >
        {isTrash ? name : "🗑"}
        {!isTrash && trashCount > 0 && (
          <span className="bb-badge">{trashCount}</span>
        )}
      </button>
    </div>
  );
}
