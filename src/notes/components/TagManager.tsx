import { useEffect, useMemo, useState } from "react";
import type { Tag } from "../types";
import { formatDate } from "../format";

interface TagManagerProps {
  tags: Tag[];
  onClose: () => void;
  onRename: (id: string, name: string) => Promise<void>;
  onSetDescription: (id: string, description: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onRestore: (id: string) => Promise<void>;
  onMerge: (fromIds: string[], toName: string, toDescription?: string) => Promise<Tag>;
}

/** 直接可编辑的输入框：失焦或回车才提交。 */
function EditableInput({
  value,
  onCommit,
  placeholder,
}: {
  value: string;
  onCommit: (v: string) => void;
  placeholder?: string;
}) {
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  return (
    <input
      className="tag-inline"
      value={draft}
      placeholder={placeholder}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => {
        if (draft !== value) onCommit(draft);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
      }}
    />
  );
}

export default function TagManager({
  tags,
  onClose,
  onRename,
  onSetDescription,
  onDelete,
  onRestore,
  onMerge,
}: TagManagerProps) {
  const active = tags.filter((t) => !t.extinguished);
  const extinguished = useMemo(
    () =>
      tags
        .filter((t) => t.extinguished)
        .sort((a, b) => (b.deleted_at ?? 0) - (a.deleted_at ?? 0)),
    [tags],
  );

  const [err, setErr] = useState<string | null>(null);

  // 合并状态
  const [merging, setMerging] = useState(false);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [mergeName, setMergeName] = useState("");
  const [mergeDesc, setMergeDesc] = useState("");

  const nameConflict = useMemo(() => {
    if (!merging || !mergeName.trim()) return false;
    return active.some((t) => t.name === mergeName.trim() && !selected.has(t.id));
  }, [merging, mergeName, active, selected]);

  const affected = useMemo(() => {
    let n = 0;
    for (const t of active) if (selected.has(t.id)) n += t.card_count;
    return n;
  }, [active, selected]);

  function toggleSelect(id: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  async function doMerge() {
    const ids = [...selected];
    if (ids.length < 2) return;
    try {
      await onMerge(ids, mergeName.trim(), mergeDesc.trim());
      setMerging(false);
      setSelected(new Set());
      setMergeName("");
      setMergeDesc("");
      setErr(null);
    } catch (e) {
      setErr(String(e));
    }
  }

  return (
    <div className="tag-mask" data-hit onMouseDown={onClose}>
      <div className="tag-panel" onMouseDown={(e) => e.stopPropagation()}>
        <div className="tag-panel-head">
          <span>标签管理</span>
          <button className="tag-close" onClick={onClose}>×</button>
        </div>

        {err && <div className="tag-error">{err}</div>}

        {/* 激活标签：名字 + 说明 直接可改，右侧删除 */}
        <div className="tag-section-title">激活标签</div>
        <div className="tag-list">
          {active.length === 0 ? (
            <div className="tag-empty">暂无标签</div>
          ) : (
            active.map((t) => (
              <div key={t.id} className="tag-row">
                <EditableInput
                  value={t.name}
                  placeholder="标签名"
                  onCommit={(v) => onRename(t.id, v.trim()).catch((e) => setErr(String(e)))}
                />
                <EditableInput
                  value={t.description}
                  placeholder="说明"
                  onCommit={(v) => onSetDescription(t.id, v).catch((e) => setErr(String(e)))}
                />
                <span className="tag-count">{t.card_count} 张</span>
                <button
                  className="tag-action tag-action-danger"
                  onClick={() => onDelete(t.id).catch((e) => setErr(String(e)))}
                >
                  ×
                </button>
              </div>
            ))
          )}
        </div>

        {/* 合并 */}
        <div className="tag-section-title">合并标签</div>
        {!merging ? (
          <button className="tag-merge-btn" onClick={() => setMerging(true)}>
            合并多个标签…
          </button>
        ) : (
          <div className="tag-merge-box">
            <div className="tag-merge-list">
              {active.map((t) => (
                <label key={t.id} className="tag-merge-opt">
                  <input
                    type="checkbox"
                    checked={selected.has(t.id)}
                    onChange={() => toggleSelect(t.id)}
                  />
                  <span>{t.name}</span>
                  {t.description && <em>（{t.description}）</em>}
                </label>
              ))}
            </div>
            <div className="tag-merge-fields">
              <input
                placeholder="合并后的名字"
                value={mergeName}
                onChange={(e) => setMergeName(e.target.value)}
              />
              <input
                placeholder="合并后的说明（自己写）"
                value={mergeDesc}
                onChange={(e) => setMergeDesc(e.target.value)}
              />
            </div>
            {nameConflict && <div className="tag-error">已有同名标签「{mergeName.trim()}」</div>}
            <div className="tag-merge-meta">这会影响最多 {affected} 张卡片</div>
            <div className="tag-desc-actions">
              <button
                disabled={selected.size < 2 || !mergeName.trim() || nameConflict}
                onClick={doMerge}
              >
                合并
              </button>
              <button onClick={() => setMerging(false)}>取消</button>
            </div>
          </div>
        )}

        {/* 回收站 */}
        <div className="tag-section-title">回收站</div>
        <div className="tag-list">
          {extinguished.length === 0 ? (
            <div className="tag-empty">空</div>
          ) : (
            extinguished.map((t) => (
              <div key={t.id} className="tag-row tag-row-dead">
                <span className="tag-name">{t.name}</span>
                <span className="tag-count">{t.card_count} 张 · {formatDate(t.deleted_at ?? 0)}</span>
                <button className="tag-action" onClick={() => onRestore(t.id).catch((e) => setErr(String(e)))}>
                  恢复
                </button>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
