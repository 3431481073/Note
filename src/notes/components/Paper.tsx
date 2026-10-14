import { forwardRef, type ClipboardEvent } from "react";
import type { MetaIndex, Page, Tag } from "../types";
import { formatDate } from "../format";
import TagSelect from "./TagSelect";
import ProjectPicker from "./ProjectPicker";

interface PaperProps {
  page: Page | null;
  readOnly: boolean;
  metaIndex: MetaIndex;
  onChange: (value: string) => void;
  onTitleChange: (title: string) => void;
  onSetTags: (tags: string[]) => void;
  onCreateTag: (name: string) => Promise<Tag>;
  onDeleteTag: (id: string) => void;
  onManageTags: () => void;
  onPickProject: () => void;
  onSelectProject: (path: string) => void;
}

const Paper = forwardRef<HTMLTextAreaElement, PaperProps>(
  (
    {
      page,
      readOnly,
      metaIndex,
      onChange,
      onTitleChange,
      onSetTags,
      onCreateTag,
      onDeleteTag,
      onManageTags,
      onPickProject,
      onSelectProject,
    },
    ref,
  ) => {
    const tags = page?.tags ?? [];
    const project = page?.projects?.[0] ?? "";

    /**
     * 没选中任何文字时，Ctrl+C 复制整条便签（标题 + 正文）。
     * 标题是 <input>、正文是 <textarea>，两个独立元素，没法用鼠标跨元素选择，
     * 否则只能复制两次。
     */
    function handleCopy(e: ClipboardEvent<HTMLDivElement>) {
      // input/textarea 内部的选区不体现在 window.getSelection() 上，得看 selectionStart/End。
      const active = document.activeElement;
      if (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) {
        if (active.selectionStart !== active.selectionEnd) return; // 真的选中了 → 放行原生
      }
      const sel = window.getSelection();
      if (sel && !sel.isCollapsed) return; // 别处有选中 → 放行原生

      const text = [page?.title ?? "", page?.content ?? ""]
        .filter((s) => s.length > 0)
        .join("\n");
      if (!text) return; // 空便签，别拦，交回原生行为

      // 必须同步写入：这是 copy 事件，异步的 navigator.clipboard.writeText
      // 可能因用户手势窗口关闭而 reject。
      e.preventDefault();
      e.clipboardData.setData("text/plain", text);
    }

    return (
      <div className="paper" onCopy={handleCopy}>
        <div className="paper-watermark" aria-hidden="true"></div>
        <div className="paper-date">
          {page ? formatDate(page.updated_at || page.created_at) : ""}
        </div>
        {!readOnly && (
          <div className="paper-meta">
            {page?.from_view ? (
              <span className="chip chip-view">视图：{page.from_view.length > 6 ? page.from_view.slice(0, 6) + "…" : page.from_view}</span>
            ) : null}
            <TagSelect
              label="🏷 标签"
              options={metaIndex.tags}
              selected={tags}
              color="mint"
              onToggle={(id) =>
                onSetTags(tags.includes(id) ? tags.filter((t) => t !== id) : [...tags, id])
              }
              onAdd={async (name) => {
                const tag = await onCreateTag(name);
                if (!tags.includes(tag.id)) onSetTags([...tags, tag.id]);
              }}
              onDelete={onDeleteTag}
              onManage={onManageTags}
            />
            <ProjectPicker project={project} onSelect={onSelectProject} onBrowse={onPickProject} />
          </div>
        )}
        <input
          className="paper-title"
          value={page?.title ?? ""}
          placeholder="标题（可选）"
          readOnly={readOnly}
          onChange={(e) => onTitleChange(e.target.value)}
          spellCheck={false}
        />
        <textarea
          ref={ref}
          className="paper-content"
          readOnly={readOnly}
          value={page?.content ?? ""}
          placeholder="写点什么…"
          onChange={(e) => onChange(e.target.value)}
          spellCheck={false}
        />
      </div>
    );
  },
);
Paper.displayName = "Paper";
export default Paper;
