import type { PageMeta, View } from "../types";
import { formatDate, projectName } from "../format";

interface CardProps {
  meta: PageMeta;
  active: boolean;
  view: View;
  /** 默认项目根（如 `D:\MyProject`）；指向它的卡片不显示项目标记。 */
  defaultProject?: string | null;
  onSelect: (id: string) => void;
  onDoubleSelect: (id: string) => void;
  onDelete: (id: string) => void;
  onRestore: (id: string) => void;
  onPurge: (id: string) => void;
}

/** Windows 路径比较：忽略大小写与结尾分隔符。 */
function samePath(a: string, b: string | null | undefined): boolean {
  if (!b) return false;
  const norm = (s: string) => s.replace(/[\\/]+$/, "").toLowerCase();
  return norm(a) === norm(b);
}

export default function Card({
  meta,
  active,
  view,
  defaultProject,
  onSelect,
  onDoubleSelect,
  onDelete,
  onRestore,
  onPurge,
}: CardProps) {
  const isTrash = view === "trash";
  const rawProject = meta.projects?.[0] ?? "";
  // 只显示「更具体」的项目：指向默认项目根（如 D:\MyProject）的卡片不显示——根目录对
  // 每张卡片都一样，标出来没有信息量；只有子目录（如 D:\MyProject\Note）才显示。
  const showProject =
    rawProject && !samePath(rawProject, defaultProject) ? projectName(rawProject) : "";
  return (
    <div
      className={`card${active ? " active" : ""}${isTrash ? " trash" : ""}${meta.title ? " has-title" : ""}${showProject ? " has-project" : ""}`}
      onClick={() => onSelect(meta.id)}
      onDoubleClick={() => onDoubleSelect(meta.id)}
    >
      <div className="card-head">
        {formatDate(isTrash ? (meta.deleted_at ?? meta.created_at) : meta.created_at)}
      </div>
      {meta.title && <div className="card-title">{meta.title}</div>}
      <div className="card-body">{meta.summary || "（空白便签）"}</div>
      {/* 卡片内部最底一行；title 给完整路径，因为这里只显示末段目录名会截断 */}
      {showProject && (
        <div className="card-project" title={rawProject}>
          📁 {showProject}
        </div>
      )}
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
