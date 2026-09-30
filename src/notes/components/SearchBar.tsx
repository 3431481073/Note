import { forwardRef, useState } from "react";
import type { MetaIndex } from "../types";
import TagSelect from "./TagSelect";

interface SearchBarProps {
  value: string;
  metaIndex: MetaIndex;
  activeTags: string[];
  activeProjects: string[];
  onChange: (v: string) => void;
  onSetTags: (tags: string[]) => void;
  onSetProjects: (projects: string[]) => void;
}

const SearchBar = forwardRef<HTMLInputElement, SearchBarProps>(
  (
    { value, metaIndex, activeTags, activeProjects, onChange, onSetTags, onSetProjects },
    ref,
  ) => {
    const [focused, setFocused] = useState(false);
    const showFilters =
      focused || value.length > 0 || activeTags.length > 0 || activeProjects.length > 0;
    return (
      <div className="search-wrap">
        <div className={`search-input${value ? " active" : ""}`}>
          <input
            ref={ref}
            className="search-input-field"
            type="text"
            placeholder="搜索便签…"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onFocus={() => setFocused(true)}
            onBlur={() => setFocused(false)}
            spellCheck={false}
          />
        </div>
        {showFilters && (
          <div className="search-filters" onMouseDown={(e) => e.preventDefault()}>
            <TagSelect
              label="🏷 标签"
              options={metaIndex.tags}
              selected={activeTags}
              color="mint"
              allowAdd={false}
              onToggle={(v) =>
                onSetTags(
                  activeTags.includes(v) ? activeTags.filter((t) => t !== v) : [...activeTags, v],
                )
              }
              onAdd={() => {}}
            />
            <TagSelect
              label="📁 项目"
              options={metaIndex.projects}
              selected={activeProjects}
              color="proj"
              allowAdd={false}
              onToggle={(v) =>
                onSetProjects(
                  activeProjects.includes(v)
                    ? activeProjects.filter((t) => t !== v)
                    : [...activeProjects, v],
                )
              }
              onAdd={() => {}}
            />
          </div>
        )}
      </div>
    );
  },
);
SearchBar.displayName = "SearchBar";
export default SearchBar;
