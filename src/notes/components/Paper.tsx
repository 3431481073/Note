import { forwardRef } from "react";
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
    return (
      <div className="paper">
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
