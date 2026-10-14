// 便签页共享类型定义

export type View = "active" | "trash";
export type Mode = "card" | "outline";

export interface OutlineNode {
  id: string;
  title: string;
  text: string;
  tags: string[];
  projects: string[];
  addedToNotes: boolean;
  noteId: string | null;
  collapsed: boolean;
  children: OutlineNode[];
  x?: number;
  y?: number;
}

export interface Cursor {
  ts: number;
  id: string;
}

export interface PageMeta {
  id: string;
  summary: string;
  created_at: number;
  deleted_at: number | null;
  version: number;
  title: string;
  kind: string;
  tags: string[];
  projects: string[];
  from_view?: string | null;
}

export interface Page extends PageMeta {
  content: string;
  updated_at: number;
  nodes?: OutlineNode[];
}

export interface PageList {
  items: PageMeta[];
  has_more: boolean;
  next_cursor: Cursor | null;
}

export interface Stats {
  active: number;
  trash: number;
  outline: number;
  trash_outline: number;
}

export interface MetaIndex {
  tags: Tag[];
  projects: string[];
  /** 默认项目根（如 `D:\MyProject`）。指向它的卡片不显示项目标记。 */
  default_project: string | null;
}

export interface Tag {
  id: string;
  name: string;
  description: string;
  extinguished: boolean;
  deleted_at: number | null;
  card_count: number;
}

export interface UpdateResult {
  id: string;
  version: number;
  summary: string;
  updated_at: number;
}

export type Op = "create" | "update" | "delete" | "restore" | "purge";

export interface ChangedPayload {
  op: Op;
  id: string;
  version: number;
  view: View;
  created_at: number | null;
  deleted_at: number | null;
  summary: string | null;
  title?: string | null;
  kind?: string | null;
  tags?: string[] | null;
  projects?: string[] | null;
  ts: number;
}

export interface HitRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** 后台导入（catcher 文本 → 卡片）的状态。`state`：pending / running / done / failed / skipped。 */
export interface CatcherImportStatus {
  state: string;
  imported: number;
  skipped: number;
  message: string | null;
}
