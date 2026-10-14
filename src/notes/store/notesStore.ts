// 便签页状态仓库：单一数据源，外部存储 + 订阅模式（useSyncExternalStore）。
// 只做数据与竞态协调；UI 滚动、动画、焦点由组件负责。
import { useSyncExternalStore } from "react";
import type {
  ChangedPayload,
  Cursor,
  MetaIndex,
  Mode,
  OutlineNode,
  Page,
  PageMeta,
  Stats,
  UpdateResult,
  View,
} from "../types";
import { notesApi } from "./notesApi";
import {
  applyChangedPayload,
  cmpActive,
  cmpTrash,
  insertSorted,
  makeSummary,
} from "./opSeq";
import { addChild, addRoot, newNode, outlineToText, removeNode, updateNode } from "./tree";

export interface NotesState {
  view: View;
  active: PageMeta[];
  trash: PageMeta[];
  searchResults: PageMeta[] | null;
  previewPage: Page | null;
  searchQuery: string;
  currentPageId: string | null;
  currentPage: Page | null;
  stats: Stats;
  hasMoreActive: boolean;
  hasMoreTrash: boolean;
  nextActiveCursor: Cursor | null;
  nextTrashCursor: Cursor | null;
  activeTags: string[];
  activeProjects: string[];
  metaIndex: MetaIndex;
  mode: Mode;
  outlines: PageMeta[];
  currentOutline: Page | null;
  activeNodeId: string | null;
  initialized: boolean;
  loading: boolean;
  toast: string | null;
}

const initial: NotesState = {
  view: "active",
  active: [],
  trash: [],
  searchResults: null,
  previewPage: null,
  searchQuery: "",
  currentPageId: null,
  currentPage: null,
  stats: { active: 0, trash: 0, outline: 0, trash_outline: 0 },
  activeTags: [],
  activeProjects: [],
  metaIndex: { tags: [], projects: [], default_project: null },
  mode: "card",
  outlines: [],
  currentOutline: null,
  activeNodeId: null,
  hasMoreActive: false,
  hasMoreTrash: false,
  nextActiveCursor: null,
  nextTrashCursor: null,
  initialized: false,
  loading: false,
  toast: null,
};

// —— 本地编辑竞态跟踪（模块级，不进入响应式 state）——
let localDirty = false;
let localVersion = 0;
let flushTimer: number | null = null;
let flushSeq = 0;
let searchSeq = 0;
let searchTimer: number | null = null;
let outlinesSeq = 0;
let previewSeq = 0;

function metaOf(p: Page): PageMeta {
  return {
    id: p.id,
    summary: p.summary,
    created_at: p.created_at,
    deleted_at: p.deleted_at,
    version: p.version,
    title: p.title,
    kind: p.kind,
    tags: p.tags,
    projects: p.projects,
  };
}

function mergeUnique(base: PageMeta[], incoming: PageMeta[]): PageMeta[] {
  const seen = new Set(base.map((m) => m.id));
  const extra = incoming.filter((m) => !seen.has(m.id));
  return base.concat(extra);
}

class NotesStore {
  private state: NotesState = initial;
  private listeners = new Set<() => void>();
  private creating = false;
  private initializing = false;

  subscribe = (fn: () => void): (() => void) => {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  };

  getState = (): NotesState => this.state;

  private setState(patch: Partial<NotesState>) {
    this.state = { ...this.state, ...patch };
    this.listeners.forEach((fn) => fn());
  }

  // ---------- 初始化 ----------
  async init() {
    // 幂等 + 防重入闸门：init 在 App 挂载时调用，重复挂载/页面重载都会再进来。
    if (this.initializing || this.state.initialized) return;
    this.initializing = true;
    try {
      await notesApi.trace("init: begin").catch(() => {});
      let stats = this.state.stats;
      try {
        stats = await notesApi.getStats();
        this.setState({ stats });
      } catch {
        /* 忽略，稍后由 stats 事件补齐 */
      }
      try {
        const metaIndex = await notesApi.getMetaIndex();
        this.setState({ metaIndex });
      } catch {
        /* 忽略 */
      }

      const loaded = await this.loadFirstPage("active");
      if (!loaded) {
        // 读列表失败时**绝不能**因为「看起来是空的」就新建便签——那会凭空造出空白便签。
        // 曾经的事故就是：这条路径静默吞错 + 每次启动都判定为空 → 每次启动多一张空白便签。
        await notesApi.trace("init: list_pages failed, skip auto-create").catch(() => {});
        this.setState({ initialized: true });
        return;
      }

      if (this.state.active.length > 0) {
        await this.selectPage(this.state.active[0].id);
      } else if (
        stats.active + stats.trash + stats.outline + stats.trash_outline === 0
      ) {
        // 只有「整个库真的没有任何页面」（全新安装）才自动建一张，
        // 这样便签全在回收站、或用户清空列表时，不会每次启动都冒出一张空白便签。
        await notesApi.trace("init: empty library, auto-create").catch(() => {});
        await this.createNewPage();
      } else {
        this.setState({ currentPageId: null, currentPage: null });
      }
      await notesApi.trace("init: done").catch(() => {});
      this.setState({ initialized: true });
      // 后台导入（catcher 文本 → 卡片）可能在前端挂上监听【之前】就结束了，
      // 那次 notes://catcher-imported 没人听见 —— 这里对一次账。
      await this.reconcileImport();
    } finally {
      this.initializing = false;
    }
  }

  /** 拉取某一视图的第一页。返回是否成功——调用方据此决定能否把「空列表」当真。 */
  async loadFirstPage(view: View): Promise<boolean> {
    try {
      const res = await notesApi.listPages(view, null);
      if (view === "active") {
        this.setState({
          active: res.items,
          hasMoreActive: res.has_more,
          nextActiveCursor: res.next_cursor,
        });
      } else {
        this.setState({
          trash: res.items,
          hasMoreTrash: res.has_more,
          nextTrashCursor: res.next_cursor,
        });
      }
      return true;
    } catch (e) {
      console.error("loadFirstPage failed", e);
      void notesApi.trace(`loadFirstPage(${view}) failed`).catch(() => {});
      return false;
    }
  }

  async loadMore() {
    const { view } = this.state;
    if (view === "active") {
      if (!this.state.hasMoreActive) return;
      const res = await notesApi.listPages("active", this.state.nextActiveCursor);
      this.setState({
        active: mergeUnique(this.state.active, res.items),
        hasMoreActive: res.has_more,
        nextActiveCursor: res.next_cursor,
      });
    } else {
      if (!this.state.hasMoreTrash) return;
      const res = await notesApi.listPages("trash", this.state.nextTrashCursor);
      this.setState({
        trash: mergeUnique(this.state.trash, res.items),
        hasMoreTrash: res.has_more,
        nextTrashCursor: res.next_cursor,
      });
    }
  }

  // ---------- 页面选择 ----------
  async selectPage(id: string) {
    if (id === this.state.currentPageId) return;
    await this.flushNow();
    try {
      const page = await notesApi.getPage(id);
      localDirty = false;
      localVersion = page.version;
      this.setState({ currentPageId: id, currentPage: page, view: this.viewOf(page) });
    } catch (e) {
      console.error("getPage failed", e);
    }
  }

  private viewOf(p: Page): View {
    return p.deleted_at === null ? "active" : "trash";
  }

  async createNewPage() {
    // 防抖：连点/按键重复触发时只新建一次，避免“疯狂创造”多个便签。
    if (this.creating) return;
    this.creating = true;
    try {
      await this.flushNow();
      const page = await notesApi.createPage();
      localDirty = false;
      localVersion = page.version;
      const meta = metaOf(page);
      this.setState({
        view: "active",
        currentPageId: page.id,
        currentPage: page,
        // 幂等：后端 create 事件可能已先行回填，避免同一 id 重复插入（导致多卡片聚焦/卡顿）。
        active: this.state.active.some((m) => m.id === page.id)
          ? this.state.active
          : insertSorted(this.state.active, meta, cmpActive),
      });
    } finally {
      this.creating = false;
    }
  }

  // ---------- 视图切换 ----------
  async setView(view: View) {
    if (view === this.state.view) return;
    await this.flushNow();
    this.setState({ view, searchQuery: "", searchResults: null, previewPage: null, activeTags: [], activeProjects: [] });
    if (this.state.mode === "outline") {
      await this.loadOutlines(view);
      if (this.state.outlines.length > 0) {
        await this.selectOutline(this.state.outlines[0].id);
      } else {
        this.setState({ currentOutline: null, activeNodeId: null });
      }
    } else {
      await this.loadFirstPage(view);
      const list = view === "active" ? this.state.active : this.state.trash;
      if (list.length > 0) {
        await this.selectPage(list[0].id);
      } else {
        this.setState({ currentPageId: null, currentPage: null });
      }
    }
  }

  // ---------- 编辑 ----------
  onContentInput(content: string) {
    const cur = this.state.currentPage;
    if (!cur || this.state.currentPageId === null) return;
    localDirty = true;
    const summary = makeSummary(content);
    const id = this.state.currentPageId;
    this.setState({
      currentPage: { ...cur, content, summary },
      active: this.patchMeta(this.state.active, id, { summary }),
      trash: this.patchMeta(this.state.trash, id, { summary }),
    });

    const delay = content.length > 5000 ? 1000 : 300;
    this.scheduleFlush(id, delay);
  }

  onTitleInput(title: string) {
    const cur = this.state.currentPage;
    if (!cur || this.state.currentPageId === null) return;
    localDirty = true;
    const id = this.state.currentPageId;
    this.setState({
      currentPage: { ...cur, title },
      active: this.patchMeta(this.state.active, id, { title }),
      trash: this.patchMeta(this.state.trash, id, { title }),
    });
    this.scheduleFlush(id, 300);
  }

  async setPageTags(tags: string[]) {
    const cur = this.state.currentPage;
    if (!cur || this.state.currentPageId === null) return;
    localDirty = true;
    const id = this.state.currentPageId;
    this.setState({
      currentPage: { ...cur, tags },
      active: this.patchMeta(this.state.active, id, { tags }),
      trash: this.patchMeta(this.state.trash, id, { tags }),
    });
    this.scheduleFlush(id, 300);
  }

  setPageProjects(projects: string[]) {
    const cur = this.state.currentPage;
    if (!cur || this.state.currentPageId === null) return;
    localDirty = true;
    const id = this.state.currentPageId;
    this.setState({
      currentPage: { ...cur, projects },
      active: this.patchMeta(this.state.active, id, { projects }),
      trash: this.patchMeta(this.state.trash, id, { projects }),
      metaIndex: {
        ...this.state.metaIndex,
        projects: Array.from(new Set([...this.state.metaIndex.projects, ...projects])),
      },
    });
    this.scheduleFlush(id, 300);
  }

  setPageProject(project: string) {
    const cur = this.state.currentPage;
    if (!cur || this.state.currentPageId === null) return;
    localDirty = true;
    const id = this.state.currentPageId;
    const projects = [project];
    this.setState({
      currentPage: { ...cur, projects },
      active: this.patchMeta(this.state.active, id, { projects }),
      trash: this.patchMeta(this.state.trash, id, { projects }),
      metaIndex: {
        ...this.state.metaIndex,
        projects: Array.from(new Set([...this.state.metaIndex.projects, project])),
      },
    });
    this.scheduleFlush(id, 300);
  }

  async pickProject() {
    try {
      const p = await notesApi.pickProjectDir();
      if (p) this.setPageProject(p);
    } catch (e) {
      console.error("pick project failed", e);
    }
  }

  async deleteTag(id: string) {
    await notesApi.deleteTag(id);
    await this.refreshTags();
  }

  async refreshTags() {
    const metaIndex = await notesApi.getMetaIndex().catch(() => this.state.metaIndex);
    this.setState({ metaIndex });
  }

  // ---------- 后台导入（catcher 文本 → 卡片）----------

  /**
   * 后台把 catcher 文本导成卡片之后，把列表 / 标签 / 计数对上。
   *
   * 两条路都走这里：`notes://catcher-imported` 事件到达时一次；`init` 收尾时对一次账
   * （导入可能跑在监听挂上之前，那次事件就此丢掉）。重复调用是幂等的。
   *
   * **只刷新，不提示** —— 用户明确不要「本轮新导入 N 条」这类提示 / 角标。
   */
  async refreshAfterImport() {
    if (!this.state.initialized) return; // init 还没收尾，交给它那一次
    try {
      this.setState({ stats: await notesApi.getStats() });
    } catch {
      /* 忽略，只是个计数 */
    }
    try {
      this.setState({ metaIndex: await notesApi.getMetaIndex() });
    } catch {
      /* 忽略 */
    }
    // 正在编辑时不重排列表，免得覆盖还没落盘的正文。
    if (!localDirty) {
      await this.loadFirstPage(this.state.view);
    }
  }

  private async reconcileImport() {
    try {
      const st = await notesApi.catcherImportStatus();
      if (st.imported > 0) await this.refreshAfterImport();
    } catch {
      /* 导入是后台的事，读不到就算了 —— 绝不能因为它影响启动 */
    }
  }

  async createTag(name: string, description?: string) {
    const tag = await notesApi.createTag(name, description);
    await this.refreshTags();
    return tag;
  }

  async renameTag(id: string, name: string) {
    await notesApi.renameTag(id, name);
    await this.refreshTags();
  }

  async setTagDescription(id: string, description: string) {
    await notesApi.setTagDescription(id, description);
    await this.refreshTags();
  }

  async restoreTag(id: string) {
    await notesApi.restoreTag(id);
    await this.refreshTags();
  }

  async mergeTags(fromIds: string[], toName: string, toDescription?: string) {
    const tag = await notesApi.mergeTags(fromIds, toName, toDescription);
    await this.refreshTags();
    // 合并会给卡片加新标签，刷新当前页的标签列表。
    const id = this.state.currentPageId;
    if (id) {
      const page = await notesApi.getPage(id).catch(() => this.state.currentPage);
      if (page) {
        this.setState({
          currentPage: page,
          active: this.patchMeta(this.state.active, id, { tags: page.tags }),
          trash: this.patchMeta(this.state.trash, id, { tags: page.tags }),
        });
      }
    }
    return tag;
  }

  private patchMeta(list: PageMeta[], id: string, patch: Partial<PageMeta>): PageMeta[] {
    const idx = list.findIndex((m) => m.id === id);
    if (idx < 0) return list;
    const next = list.slice();
    next[idx] = { ...next[idx], ...patch };
    return next;
  }

  private scheduleFlush(id: string, delay: number) {
    if (flushTimer !== null) clearTimeout(flushTimer);
    flushTimer = window.setTimeout(() => this.doFlush(id), delay);
  }

  /** 落盘当前页。返回是否「确实已落盘」（没有待写内容也算成功）。 */
  private async doFlush(id: string): Promise<boolean> {
    const seq = ++flushSeq;
    const cur = this.state.currentPage;
    if (!cur) return true;
    try {
      const res: UpdateResult = await notesApi.updatePage(
        id,
        cur.content,
        cur.title,
        cur.tags,
        cur.projects,
      );
      if (seq !== flushSeq) return true;
      localVersion = res.version;
      localDirty = false;
      // 标签关联可能已变，刷新标签列表（card_count）。
      void this.refreshTags();
      return true;
    } catch (e) {
      // 写库失败：保持 dirty，等待下次输入重试
      localDirty = true;
      console.error("flush failed", e);
      return false;
    }
  }

  /** 立即落盘（切换页面 / 关闭前调用）。 */
  /** 立即落盘（切换页面 / 关闭前调用）。返回是否落盘成功——关闭流程据此决定要不要退出。 */
  async flushNow(): Promise<boolean> {
    if (flushTimer !== null) {
      clearTimeout(flushTimer);
      flushTimer = null;
    }
    if (!localDirty) return true;
    const id = this.state.currentPageId;
    if (!id) return true;
    return await this.doFlush(id);
  }

  private closing = false;

  /**
   * 点 × 或 Alt+F4：**先落盘，成功才真的退出进程**。
   *
   * 关掉就是真的关掉：进程结束、不后台常驻（以前只是 hide()，进程会带着冻结的页面
   * 常驻好几天，重开时又要恢复合成，那是黑块/僵住那一类怪现象的温床）。
   * 代价是重新打开为一次冷启动，实测约 1.8 秒（其中约 1.4 秒是 WebView2 自身引导）。
   *
   * 落盘失败就**不退出**并提示——「关了但没存上」比「关不掉」糟糕得多。
   */
  async closeNotes() {
    if (this.closing) return;
    this.closing = true;
    let saved: boolean;
    try {
      saved = await this.flushNow();
    } catch (e) {
      console.error("flush on close failed", e);
      saved = false;
    }
    if (!saved) {
      this.closing = false; // 允许再试
      this.showToast("保存失败，已取消关闭");
      return;
    }
    try {
      await notesApi.trace("quit: saved, exiting").catch(() => {});
      await notesApi.quitNotes();
    } catch (e) {
      this.closing = false;
      console.error("quit failed", e);
      this.showToast("退出失败");
    }
  }

  // ---------- 搜索 ----------
  onSearchChange(q: string) {
    this.setState({ searchQuery: q });
    this.scheduleSearch();
  }

  setActiveTags(tags: string[]) {
    this.setState({ activeTags: tags });
    this.scheduleSearch();
  }

  setActiveProjects(projects: string[]) {
    this.setState({ activeProjects: projects });
    this.scheduleSearch();
  }

  private scheduleSearch() {
    if (searchTimer !== null) clearTimeout(searchTimer);
    searchTimer = window.setTimeout(() => this.doSearch(), 200);
  }

  private async doSearch() {
    const seq = ++searchSeq;
    const q = this.state.searchQuery;
    const tags = this.state.activeTags;
    const projects = this.state.activeProjects;
    if (q.length === 0 && tags.length === 0 && projects.length === 0) {
      this.setState({ searchResults: null, previewPage: null });
      return;
    }
    try {
      const res = await notesApi.search(q, this.state.view, null, tags, projects);
      if (seq !== searchSeq) return;
      const items = res.items;
      this.setState({ searchResults: items });
      if (items.length > 0) {
        const first = items[0];
        const prev = this.state.previewPage;
        if (!prev || prev.id !== first.id) {
          const page = await notesApi.getPage(first.id);
          if (seq !== searchSeq) return;
          this.setState({ previewPage: page });
        }
      } else {
        this.setState({ previewPage: null });
      }
    } catch (e) {
      console.error("search failed", e);
    }
  }

  /** 单击搜索结果：只更新左侧预览内容，不退出搜索。 */
  async previewFromSearch(id: string) {
    if (this.state.previewPage?.id === id) return;
    const seq = ++previewSeq;
    try {
      const page = await notesApi.getPage(id);
      if (seq !== previewSeq) return;
      this.setState({ previewPage: page });
    } catch (e) {
      console.error("preview search result failed", e);
    }
  }

  /** 双击搜索结果：退出搜索并打开该文章。返回其在当前视图列表中的下标，供滚动。 */
  async openFromSearch(id: string): Promise<number> {
    this.setState({ searchQuery: "", searchResults: null, previewPage: null, activeTags: [], activeProjects: [] });
    await this.selectPage(id);
    const list = this.state.view === "active" ? this.state.active : this.state.trash;
    return list.findIndex((m) => m.id === id);
  }

  // ---------- 删除 / 恢复 / 彻底删除 ----------
  async softDelete(id: string) {
    const stats = await notesApi.softDelete(id);
    const meta = this.state.active.find((m) => m.id === id);
    const active = this.state.active.filter((m) => m.id !== id);
    const trash = meta
      ? insertSorted(this.state.trash, { ...meta, deleted_at: Date.now() }, cmpTrash)
      : this.state.trash;
    this.setState({ active, trash, stats });
    if (this.state.currentPageId === id) {
      localDirty = false;
      const next = active[0];
      if (next) await this.selectPage(next.id);
      else this.setState({ currentPageId: null, currentPage: null });
    }
  }

  async restore(id: string) {
    await notesApi.restorePage(id);
    const meta = this.state.trash.find((m) => m.id === id);
    const trash = this.state.trash.filter((m) => m.id !== id);
    const active = meta
      ? insertSorted(this.state.active, { ...meta, deleted_at: null }, cmpActive)
      : this.state.active;
    this.setState({ active, trash });
    // 恢复后留在回收站视图，不跳回便签页；若恢复的是当前预览项，切到下一个回收站项。
    if (this.state.currentPageId === id) {
      localDirty = false;
      const next = trash[0];
      if (next) await this.selectPage(next.id);
      else this.setState({ currentPageId: null, currentPage: null });
    }
  }

  async purge(id: string) {
    const stats = await notesApi.purgePage(id);
    this.setState({
      trash: this.state.trash.filter((m) => m.id !== id),
      stats,
    });
    if (this.state.currentPageId === id) {
      localDirty = false;
      const next = this.state.trash[0];
      if (next) await this.selectPage(next.id);
      else this.setState({ currentPageId: null, currentPage: null });
    }
  }

  // ---------- 视图（outline）模式 ----------
  async prepareMode(mode: Mode) {
    await this.flushNow();
    if (mode === "outline") {
      await this.loadOutlines(this.state.view);
    } else {
      await this.loadFirstPage("active");
    }
  }

  async commitMode(mode: Mode) {
    this.setState({
      mode,
      view: "active",
      searchQuery: "",
      searchResults: null,
      previewPage: null,
      activeTags: [],
      activeProjects: [],
    });
    if (mode === "outline") {
      if (this.state.outlines.length > 0) {
        await this.selectOutline(this.state.outlines[0].id);
      } else {
        this.setState({ currentOutline: null, activeNodeId: null });
      }
    } else {
      if (this.state.active.length > 0) {
        await this.selectPage(this.state.active[0].id);
      } else {
        await this.createNewPage();
      }
    }
  }

  async loadOutlines(view: View) {
    const seq = ++outlinesSeq;
    try {
      const outlines = await notesApi.listOutlines(view);
      if (seq !== outlinesSeq) return; // 过期结果丢弃，避免旧视图数据覆盖新视图数据
      this.setState({ outlines });
    } catch (e) {
      if (seq !== outlinesSeq) return;
      console.error("list outlines failed", e);
    }
  }

  async selectOutline(id: string) {
    if (id === this.state.currentOutline?.id) return;
    try {
      const page = await notesApi.getPage(id);
      this.setState({ currentOutline: page, activeNodeId: null });
    } catch (e) {
      console.error("get outline failed", e);
    }
  }

  async createOutline() {
    await this.flushNow();
    const page = await notesApi.createPage("outline");
    const meta = metaOf(page);
    this.setState({
      mode: "outline",
      currentOutline: page,
      activeNodeId: null,
      outlines: this.state.outlines.some((m) => m.id === page.id)
        ? this.state.outlines
        : [meta, ...this.state.outlines],
    });
  }

  async saveOutline() {
    const o = this.state.currentOutline;
    if (!o) return;
    await notesApi.saveOutline(o.id, JSON.stringify(o.nodes ?? []));
  }

  setActiveNode(id: string) {
    this.setState({ activeNodeId: id });
  }

  addRootNode(x?: number, y?: number) {
    const o = this.state.currentOutline;
    if (!o) return;
    const nodes = addRoot(o.nodes ?? [], newNode("新节点", x, y));
    this.setState({ currentOutline: { ...o, nodes } });
    void this.saveOutline();
  }

  addChildNode(parentId: string, x?: number, y?: number) {
    const o = this.state.currentOutline;
    if (!o) return;
    const nodes = addChild(o.nodes ?? [], parentId, newNode("新节点", x, y));
    this.setState({ currentOutline: { ...o, nodes } });
    void this.saveOutline();
  }

  async renameOutline(title: string) {
    const o = this.state.currentOutline;
    if (!o) return;
    this.setState({ currentOutline: { ...o, title } });
    const meta = this.state.outlines.map((m) => (m.id === o.id ? { ...m, title } : m));
    this.setState({ outlines: meta });
    await notesApi.updatePage(o.id, "", title).catch(() => {});
  }

  async exportOutline() {
    const o = this.state.currentOutline;
    if (!o) return;
    const text = outlineToText(o.nodes ?? []);
    try {
      await navigator.clipboard.writeText(text);
      this.showToast("已导出，内容已复制到剪贴板");
    } catch {
      this.showToast("导出失败：无法访问剪贴板");
    }
  }

  showToast(msg: string) {
    this.setState({ toast: msg });
    window.setTimeout(() => this.setState({ toast: null }), 1800);
  }

  async deleteOutline(id?: string) {
    const targetId = id ?? this.state.currentOutline?.id;
    if (!targetId) return;
    await notesApi.softDelete(targetId);
    const wasCurrent = this.state.currentOutline?.id === targetId;
    await this.loadOutlines(this.state.view);
    if (wasCurrent) {
      this.setState({ currentOutline: null, activeNodeId: null });
      if (this.state.outlines.length > 0) {
        await this.selectOutline(this.state.outlines[0].id);
      }
    }
  }

  async restoreOutline(id: string) {
    await notesApi.restorePage(id);
    const wasCurrent = this.state.currentOutline?.id === id;
    await this.loadOutlines(this.state.view);
    if (wasCurrent) {
      this.setState({ currentOutline: null, activeNodeId: null });
      if (this.state.outlines.length > 0) {
        await this.selectOutline(this.state.outlines[0].id);
      }
    }
  }

  async purgeOutline(id: string) {
    await notesApi.purgePage(id);
    const wasCurrent = this.state.currentOutline?.id === id;
    await this.loadOutlines(this.state.view);
    if (wasCurrent) {
      this.setState({ currentOutline: null, activeNodeId: null });
      if (this.state.outlines.length > 0) {
        await this.selectOutline(this.state.outlines[0].id);
      }
    }
  }

  deleteNode(id: string) {
    const o = this.state.currentOutline;
    if (!o) return;
    const nodes = removeNode(o.nodes ?? [], id);
    this.setState({
      currentOutline: { ...o, nodes },
      activeNodeId: this.state.activeNodeId === id ? null : this.state.activeNodeId,
    });
    void this.saveOutline();
  }

  renameNode(id: string, title: string) {
    const o = this.state.currentOutline;
    if (!o) return;
    const nodes = updateNode(o.nodes ?? [], id, { title });
    this.setState({ currentOutline: { ...o, nodes } });
    void this.saveOutline();
  }

  moveNode(id: string, x: number, y: number) {
    const o = this.state.currentOutline;
    if (!o) return;
    // 写库前 clamp，防拖出画布太远造成脏数据。
    const cx = Math.round(Math.max(-2000, Math.min(2000, x)));
    const cy = Math.round(Math.max(-2000, Math.min(2000, y)));
    const nodes = updateNode(o.nodes ?? [], id, { x: cx, y: cy });
    this.setState({ currentOutline: { ...o, nodes } });
    void this.saveOutline();
  }

  setNodeTags(id: string, tags: string[]) {
    const o = this.state.currentOutline;
    if (!o) return;
    const nodes = updateNode(o.nodes ?? [], id, { tags });
    this.setState({ currentOutline: { ...o, nodes } });
    void this.saveOutline();
  }

  setNodeProjects(id: string, projects: string[]) {
    const o = this.state.currentOutline;
    if (!o) return;
    const nodes = updateNode(o.nodes ?? [], id, { projects });
    this.setState({ currentOutline: { ...o, nodes } });
    void this.saveOutline();
  }

  updateNodeText(id: string, text: string) {
    const o = this.state.currentOutline;
    if (!o) return;
    const nodes = updateNode(o.nodes ?? [], id, { text });
    this.setState({ currentOutline: { ...o, nodes } });
    void this.saveOutline();
  }

  async importNodeToNote(node: OutlineNode, titleOverride?: string, tagIds?: string[], projects?: string[]) {
    const o = this.state.currentOutline;
    if (!o) return;
    if (node.noteId) {
      // 已导入：撤销（软删对应便签）
      await notesApi.softDelete(node.noteId);
      const nodes = updateNode(o.nodes ?? [], node.id, { addedToNotes: false, noteId: null });
      this.setState({ currentOutline: { ...o, nodes } });
      void this.saveOutline();
      return;
    }
    const page = await notesApi.createPage("note");
    const viewName = o.title || "视图";
    const content = node.text || "";
    const title = titleOverride ?? "";
    const finalTags = tagIds ?? [];
    const finalProjects = projects ?? [];
    const res = await notesApi.updatePage(page.id, content, title, finalTags, finalProjects, viewName);
    const meta: PageMeta = {
      id: page.id,
      summary: res.summary,
      created_at: page.created_at,
      deleted_at: null,
      version: res.version,
      title,
      kind: "note",
      tags: finalTags,
      projects: finalProjects,
      from_view: viewName,
    };
    this.setState({
      active: this.state.active.some((m) => m.id === meta.id)
        ? this.state.active
        : insertSorted(this.state.active, meta, cmpActive),
      currentOutline: {
        ...o,
        nodes: updateNode(o.nodes ?? [], node.id, { addedToNotes: true, noteId: page.id }),
      },
    });
    void this.saveOutline();
  }

  // ---------- 事件回填 ----------
  handleChanged(p: ChangedPayload) {
    // 单窗口：事件都是本窗口写入的回声。编辑中跳过当前页的摘要回填，避免覆盖未保存内容。
    if (localDirty && p.id === this.state.currentPageId) {
      if (p.version > localVersion) localVersion = p.version;
      return;
    }
    const lists = applyChangedPayload(
      { active: this.state.active, trash: this.state.trash },
      p,
    );
    this.setState({ active: lists.active, trash: lists.trash });

    if (this.state.currentPageId === p.id && this.state.currentPage) {
      this.setState({
        currentPage: {
          ...this.state.currentPage,
          summary: p.summary ?? this.state.currentPage.summary,
          version: p.version,
          deleted_at: p.deleted_at ?? this.state.currentPage.deleted_at,
        },
      });
    }
  }

  setStats(stats: Stats) {
    this.setState({ stats });
  }
}

export const notesStore = new NotesStore();

/** 订阅整个状态对象（引用稳定，Object.is 判定变更）。 */
export function useNotesStore(): NotesState {
  return useSyncExternalStore(notesStore.subscribe, notesStore.getState);
}
