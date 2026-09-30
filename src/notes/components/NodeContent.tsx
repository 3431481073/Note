import { useState } from "react";
import type { MetaIndex, OutlineNode, Tag } from "../types";
import TagSelect from "./TagSelect";
import ProjectPicker from "./ProjectPicker";

interface NodeContentProps {
  node: OutlineNode | null;
  metaIndex: MetaIndex;
  onTextChange: (id: string, text: string) => void;
  onCreateTag: (name: string) => Promise<Tag>;
  onDeleteTag: (id: string) => void;
  onManageTags: () => void;
  onPickProject: () => void;
  onImport: (node: OutlineNode, title?: string, tags?: string[], projects?: string[]) => void;
}

export default function NodeContent({
  node,
  metaIndex,
  onTextChange,
  onCreateTag,
  onDeleteTag,
  onManageTags,
  onPickProject,
  onImport,
}: NodeContentProps) {
  const [importing, setImporting] = useState(false);
  const [title, setTitle] = useState("");
  const [tags, setTags] = useState<string[]>([]);
  const [project, setProject] = useState("");

  if (!node) {
    return (
      <div className="content-inner">
        <div className="content-empty">← 在左侧选择一个节点，内容会显示在这里</div>
      </div>
    );
  }

  return (
    <div className="content-inner">
      <div className="paper-watermark" aria-hidden="true"></div>
      <div className="node-top">
        <span className="node-top-title">{node.title || "（未命名）"}</span>
        <button
          className={`import-btn${node.addedToNotes ? " done" : ""}`}
          onClick={() => {
            if (node.addedToNotes) {
              onImport(node);
            } else {
              setTitle("");
              setTags([]);
              setProject("");
              setImporting(true);
            }
          }}
        >
          {node.addedToNotes ? "✓ 已添加" : "＋ 卡片"}
        </button>
      </div>
      <textarea
        className="node-body"
        value={node.text}
        placeholder="节点内容…"
        onChange={(e) => onTextChange(node.id, e.target.value)}
        spellCheck={false}
      />

      {importing && (
        <div className="confirm-mask" onClick={() => setImporting(false)}>
          <div className="confirm-box" onClick={(e) => e.stopPropagation()}>
            <div className="confirm-title">添加到卡片列表</div>
            <input
              className="tag-inline"
              autoFocus
              value={title}
              placeholder="标题（可留空）"
              onChange={(e) => setTitle(e.target.value)}
            />
            <TagSelect
              label="🏷 标签"
              options={metaIndex.tags}
              selected={tags}
              color="mint"
              onToggle={(id) => setTags(tags.includes(id) ? tags.filter((t) => t !== id) : [...tags, id])}
              onAdd={async (name) => {
                const t = await onCreateTag(name);
                if (!tags.includes(t.id)) setTags([...tags, t.id]);
              }}
              onDelete={onDeleteTag}
              onManage={onManageTags}
            />
            <ProjectPicker project={project} onSelect={setProject} onBrowse={onPickProject} />
            <div className="confirm-actions">
              <button className="confirm-cancel" onClick={() => setImporting(false)}>
                取消
              </button>
              <button
                className="confirm-ok"
                onClick={() => {
                  onImport(node, title.trim(), tags, project ? [project] : []);
                  setImporting(false);
                }}
              >
                确定
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
