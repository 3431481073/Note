// 后端命令的薄封装，统一类型与错误处理
import { invoke } from "@tauri-apps/api/core";
import type {
  CatcherImportStatus,
  Cursor,
  HitRect,
  MetaIndex,
  Page,
  PageList,
  PageMeta,
  Stats,
  Tag,
  UpdateResult,
  View,
} from "../types";

export const notesApi = {
  getConfigDir(): Promise<string> {
    return invoke<string>("get_config_dir");
  },

  createPage(kind?: string): Promise<Page> {
    return invoke<Page>("create_page", { kind });
  },

  listPages(view: View, cursor: Cursor | null): Promise<PageList> {
    return invoke<PageList>("list_pages", { view, cursor });
  },

  getPage(id: string): Promise<Page> {
    return invoke<Page>("get_page", { id });
  },

  updatePage(
    id: string,
    content: string,
    title?: string,
    tags?: string[],
    projects?: string[],
    fromView?: string | null,
  ): Promise<UpdateResult> {
    return invoke<UpdateResult>("update_page", { id, content, title, tags, projects, fromView });
  },

  search(
    query: string,
    view: View,
    cursor: Cursor | null,
    tags?: string[],
    projects?: string[],
  ): Promise<PageList> {
    return invoke<PageList>("search", { query, view, cursor, tags, projects });
  },

  softDelete(id: string): Promise<Stats> {
    return invoke<Stats>("soft_delete", { id });
  },

  restorePage(id: string): Promise<Page> {
    return invoke<Page>("restore_page", { id });
  },

  purgePage(id: string): Promise<Stats> {
    return invoke<Stats>("purge_page", { id });
  },

  getStats(): Promise<Stats> {
    return invoke<Stats>("get_stats");
  },

  getMetaIndex(): Promise<MetaIndex> {
    return invoke<MetaIndex>("get_meta_index");
  },

  deleteTag(id: string): Promise<void> {
    return invoke<void>("delete_tag", { id });
  },

  listTags(): Promise<Tag[]> {
    return invoke<Tag[]>("list_tags");
  },

  createTag(name: string, description?: string): Promise<Tag> {
    return invoke<Tag>("create_tag", { name, description });
  },

  renameTag(id: string, name: string): Promise<void> {
    return invoke<void>("rename_tag", { id, name });
  },

  setTagDescription(id: string, description: string): Promise<void> {
    return invoke<void>("set_tag_description", { id, description });
  },

  restoreTag(id: string): Promise<void> {
    return invoke<void>("restore_tag", { id });
  },

  mergeTags(fromIds: string[], toName: string, toDescription?: string): Promise<Tag> {
    return invoke<Tag>("merge_tags", { fromIds, toName, toDescription });
  },

  pickProjectDir(): Promise<string | null> {
    return invoke<string | null>("pick_project_dir");
  },

  listOutlines(view: View): Promise<PageMeta[]> {
    return invoke<PageMeta[]>("list_outlines", { view });
  },

  saveOutline(id: string, nodesJson: string): Promise<void> {
    return invoke<void>("save_outline", { id, nodesJson });
  },

  setHitRects(rects: HitRect[]): Promise<void> {
    return invoke<void>("set_hit_rects", { rects });
  },

  /** 诊断日志：往 app_data_dir/trace.log 追加一行。只用于事后取证，失败无声。 */
  trace(msg: string): Promise<void> {
    return invoke<void>("trace", { msg });
  },

  /** 后台导入（catcher 文本 → 卡片）的状态。前台只读，绝不触发导入。 */
  catcherImportStatus(): Promise<CatcherImportStatus> {
    return invoke<CatcherImportStatus>("catcher_import_status");
  },

  /** 结束便签进程。**调用前必须已经落盘成功**（见 notesStore.closeNotes）。 */
  quitNotes(): Promise<void> {
    return invoke<void>("quit_notes");
  },
};
