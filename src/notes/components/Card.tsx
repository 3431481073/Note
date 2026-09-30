import type { PageMeta, View } from "../types";
import { formatDate } from "../format";

interface CardProps {
  meta: PageMeta;
  active: boolean;
  view: View;
  onSelect: (id: string) => void;
  onDoubleSelect: (id: string) => void;
  onDelete: (id: string) => void;
  onRestore: (id: string) => void;
  onPurge: (id: string) => void;
}

export default function Card({
  meta,
  active,
  view,
  onSelect,
  onDoubleSelect,
  onDelete,
  onRestore,
  onPurge,
}: CardProps) {
  const isTrash = view === "trash";
  return (
    <div
      className={`card${active ? " active" : ""}${isTrash ? " trash" : ""}${meta.title ? " has-title" : ""}`}
      onClick={() => onSelect(meta.id)}
      onDoubleClick={() => onDoubleSelect(meta.id)}
    >
      <div className="card-head">
        {formatDate(isTrash ? (meta.deleted_at ?? meta.created_at) : meta.created_at)}
      </div>
      {meta.title && <div className="card-title">{meta.title}</div>}
      <div className="card-body">{meta.summary || "（空白便签）"}</div>
      {isTrash ? (
        <div className="card-actions">
          <button
            className="card-btn"
            title="恢复"
            tabIndex={-1}
            onClick={(e) => {
              e.stopPropagation();
              onRestore(meta.id);
            }}
          >
            ↩
          </button>
          <button
            className="card-btn card-btn-danger"
            title="彻底删除"
            tabIndex={-1}
            onClick={(e) => {
              e.stopPropagation();
              onPurge(meta.id);
            }}
          >
            🗑
          </button>
        </div>
      ) : (
        <button
          className="card-btn card-delete"
          title="删除"
          tabIndex={-1}
          onClick={(e) => {
            e.stopPropagation();
            onDelete(meta.id);
          }}
        >
          ×
        </button>
      )}
    </div>
  );
}
