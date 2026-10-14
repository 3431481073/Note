import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { notesStore, useNotesStore } from "./store/notesStore";
import { notesApi } from "./store/notesApi";
import type { HitRect, Mode } from "./types";
import { applySeason, setupNotesEvents } from "./events/notesEvents";
import { registerShortcuts } from "./shortcuts/keys";
import Paper from "./components/Paper";
import CardList, { CardListHandle } from "./components/CardList";
import SearchBar from "./components/SearchBar";
import BottomBar from "./components/BottomBar";
import CloseButton from "./components/CloseButton";
import TrashConfirm from "./components/TrashConfirm";
import EmptyState from "./components/EmptyState";
import Seg from "./components/Seg";
import OutlineTree from "./components/OutlineTree";
import NodeContent from "./components/NodeContent";
import TagManager from "./components/TagManager";
import { findNode } from "./store/tree";

// —— 3D 折叠动画参数（对齐 111.txt）——
const EASE1 = "cubic-bezier(.55,.06,.32,1)";
const EASE_GROW = "cubic-bezier(.34,.08,.45,1)";
const EASE_FALL = "cubic-bezier(.55,.10,.85,.55)";
const EASE_HIT = "cubic-bezier(.12,.62,.35,1)";
const EASE_BACK = "cubic-bezier(.30,.06,.70,.5)";
// —— 布局常量（像素），全部来自配置 ——
const PAPER_W = 382; // 卡片态左侧（纸张）
const TREE_W = 550; // 视图态左侧（结构树）
const CARD_W = 226; // 中间列
const CONTENT_W = 400; // 视图态右侧（节点内容）
const GAP = 10; // 列间间隙

// 窗口固定 1300 宽、透明、居中、不 resize（见 hittest.rs），两种模式只是窗口内布局不同
const CARD_WIN_W = 740; // 卡片态「虚拟」布局宽，内容在 1300 窗口内视觉居中
const OUTLINE_WIN_W = 1300; // 视图态布局宽 = 窗口固定宽
const COL_TOP = 40; // 中间列在窗口内的固定顶部偏移（原 top: 40px）

// 卡片态内容在 1300 窗口内视觉居中，两侧各留 CARD_INSET
const CARD_INSET = (OUTLINE_WIN_W - CARD_WIN_W) / 2; // 280

// 中间列在窗口内的固定像素位置（卡片态）= 内边距 + 左宽 + 间隙 + 居中偏移
const CARD_COL_LEFT = CARD_INSET + PAPER_W + GAP + (CARD_WIN_W - (PAPER_W + GAP + CARD_W)) / 2; // 733
// 中间列在视图态的窗口位置
const OUTLINE_COL_LEFT =
  TREE_W + GAP + (OUTLINE_WIN_W - (TREE_W + GAP + CARD_W + GAP + CONTENT_W)) / 2; // 612

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

function reducedMotion(): boolean {
  return typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function animate(
  el: HTMLElement | null,
  keyframes: Keyframe[],
  options: KeyframeAnimationOptions,
): Promise<void> {
  return new Promise((resolve) => {
    if (!el) return resolve();
    const a = el.animate(keyframes, options);
    a.onfinish = () => resolve();
    a.oncancel = () => resolve();
  });
}

function nextFrame(): Promise<void> {
  return new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => r())));
}

// 收集所有可见面板的客户区矩形（物理像素），上报给 Rust 做 WM_NCHITTEST 点击穿透判断。
// 折起来的列是 transform: scaleX(0)，宽高为 0，靠下面的守卫天然被排除。
//
// 刻意保持「同步直报」这个最朴素的形态：曾经把它改成 rAF 节流 + 去重 + 监听
// transition/animation + ResizeObserver + MutationObserver，结果引入了难以复现的
// 不稳定（窗口像是卡住）。这套逻辑每帧都在跑，越复杂越危险——宁可多几次 invoke。
function reportHitRects() {
  const dpr = window.devicePixelRatio || 1;
  const rects: HitRect[] = [];
  document.querySelectorAll<HTMLElement>("[data-hit]").forEach((el) => {
    const r = el.getBoundingClientRect();
    if (r.width <= 0 || r.height <= 0) return;
    rects.push({
      x: Math.round(r.left * dpr),
      y: Math.round(r.top * dpr),
      w: Math.round(r.width * dpr),
      h: Math.round(r.height * dpr),
    });
  });
  notesApi.setHitRects(rects).catch(() => {});
}

// —— 关键帧 ——
const FOLD_PAPER: Keyframe[] = [
  { transform: "rotate(0deg) scale(1)", opacity: 1, offset: 0 },
  { transform: "rotate(52.8deg) scale(0.55)", opacity: 0.9, offset: 0.6 },
  { transform: "rotate(88deg) scale(0)", opacity: 0, offset: 1 },
];

const UNFOLD_PAPER: Keyframe[] = [
  { transform: "rotate(88deg) scale(0)", opacity: 0, offset: 0 },
  { transform: "rotate(52.8deg) scale(0.55)", opacity: 0.9, offset: 0.4 },
  { transform: "rotate(0deg) scale(1)", opacity: 1, offset: 1 },
];

const UNFOLD_LEFT: Keyframe[] = [
  { transform: "rotateY(86deg) scaleX(0)", offset: 0, easing: EASE_GROW },
  { transform: "rotateY(86deg) scaleX(1)", offset: 0.18, easing: EASE_FALL },
  { transform: "rotateY(0deg) scaleX(1)", offset: 0.64, easing: EASE_HIT },
  { transform: "rotateY(-11deg) scaleX(1)", offset: 0.75, easing: EASE_BACK },
  { transform: "rotateY(3.5deg) scaleX(1)", offset: 0.86, easing: "ease-out" },
  { transform: "rotateY(-1.1deg) scaleX(1)", offset: 0.93, easing: "ease-out" },
  { transform: "rotateY(0.35deg) scaleX(1)", offset: 0.97, easing: "ease-out" },
  { transform: "rotateY(0deg) scaleX(1)", offset: 1 },
];

const UNFOLD_RIGHT: Keyframe[] = [
  { transform: "rotateY(-86deg) scaleX(0)", offset: 0, easing: EASE_GROW },
  { transform: "rotateY(-86deg) scaleX(1)", offset: 0.18, easing: EASE_FALL },
  { transform: "rotateY(0deg) scaleX(1)", offset: 0.64, easing: EASE_HIT },
  { transform: "rotateY(11deg) scaleX(1)", offset: 0.75, easing: EASE_BACK },
  { transform: "rotateY(-3.5deg) scaleX(1)", offset: 0.86, easing: "ease-out" },
  { transform: "rotateY(1.1deg) scaleX(1)", offset: 0.93, easing: "ease-out" },
  { transform: "rotateY(-0.35deg) scaleX(1)", offset: 0.97, easing: "ease-out" },
  { transform: "rotateY(0deg) scaleX(1)", offset: 1 },
];

const FOLD_LEFT: Keyframe[] = [
  { transform: "rotateY(0deg) scaleX(1)", offset: 0, easing: EASE1 },
  { transform: "rotateY(86deg) scaleX(0)", offset: 1 },
];

const FOLD_RIGHT: Keyframe[] = [
  { transform: "rotateY(0deg) scaleX(1)", offset: 0, easing: EASE1 },
  { transform: "rotateY(-86deg) scaleX(0)", offset: 1 },
];

const FALL_LEFT: Keyframe[] = [
  { transform: "translateY(0) rotateY(86deg) scaleX(0)", offset: 0, easing: EASE1 },
  { transform: "translateY(150px) rotateY(86deg) scaleX(0)", offset: 1 },
];

const FALL_RIGHT: Keyframe[] = [
  { transform: "translateY(0) rotateY(-86deg) scaleX(0)", offset: 0, easing: EASE1 },
  { transform: "translateY(150px) rotateY(-86deg) scaleX(0)", offset: 1 },
];

export default function App() {
  const state = useNotesStore();
  const paperRef = useRef<HTMLDivElement>(null);
  const outlineRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const cardColumnRef = useRef<HTMLDivElement>(null);
  const cardListRef = useRef<CardListHandle>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const switchingRef = useRef(false);
  const [confirmId, setConfirmId] = useState<string | null>(null);
  const [shown, setShown] = useState(false);
  const [tagManagerOpen, setTagManagerOpen] = useState(false);

  useEffect(() => {
    notesStore.init();
    applySeason(new Date().getMonth() + 1);
    setupNotesEvents();
    getCurrentWindow()
      .show()
      .then(() => {
        getCurrentWindow().setFocus().catch(() => {});
        requestAnimationFrame(() => requestAnimationFrame(() => setShown(true)));
      })
      .catch(() => setShown(true));
    return registerShortcuts({ focusSearch: () => searchRef.current?.focus() });
  }, []);

  useEffect(() => {
    if (state.searchQuery.length > 0) {
      textareaRef.current?.blur();
    }
  }, [state.searchQuery]);

  // 首次显示与确认框开合时，重新上报命中矩形（透明区点击穿透）
  useEffect(() => {
    if (shown) reportHitRects();
  }, [shown]);

  useEffect(() => {
    reportHitRects();
  }, [confirmId]);

  const filtering =
    state.searchQuery.length > 0 ||
    state.activeTags.length > 0 ||
    state.activeProjects.length > 0;
  const isTrash = state.view === "trash";
  const listItems = state.searchResults ?? (isTrash ? state.trash : state.active);
  const displayPage = filtering ? state.previewPage : state.currentPage;
  const selectedNode = state.currentOutline
    ? findNode(state.currentOutline.nodes ?? [], state.activeNodeId ?? "")
    : null;

  async function foldPaper() {
    await animate(paperRef.current, FOLD_PAPER, { duration: 800, easing: EASE1, fill: "forwards" });
  }
  async function unfoldPaper() {
    await animate(paperRef.current, UNFOLD_PAPER, { duration: 800, easing: EASE1, fill: "forwards" });
  }
  // 中间列滑动：窗口固定不 resize。transform 只由 CSS 类控制（唯一来源），
  // slideCard 只切 data-mode，靠 CSS transition 平移。幂等，来回切多少次都不会有空窗/跳变。
  async function slideCard(to: Mode) {
    const card = cardColumnRef.current;
    if (!card) return;
    card.dataset.mode = to;
    await sleep(400);
  }
  async function unfoldPanels() {
    await Promise.all([
      animate(outlineRef.current, UNFOLD_LEFT, { duration: 1500, fill: "both" }),
      animate(contentRef.current, UNFOLD_RIGHT, { duration: 1500, fill: "both" }),
    ]);
  }
  async function foldPanels() {
    await Promise.all([
      animate(outlineRef.current, FOLD_LEFT, { duration: 400, fill: "forwards" }),
      animate(contentRef.current, FOLD_RIGHT, { duration: 400, fill: "forwards" }),
    ]);
  }
  async function fallPanels() {
    await Promise.all([
      animate(outlineRef.current, FALL_LEFT, { duration: 300, fill: "forwards" }),
      animate(contentRef.current, FALL_RIGHT, { duration: 300, fill: "forwards" }),
    ]);
  }

  async function transitionTo(to: Mode) {
    if (switchingRef.current || to === state.mode) return;
    switchingRef.current = true;
    try {
      await notesStore.prepareMode(to);
      if (reducedMotion()) {
        await notesStore.commitMode(to);
        return;
      }
      if (to === "outline") {
        // ① 左侧收回（纸张折起）→ ② 中间列移动 → ③ 两侧同时打开（树 + 内容）
        await foldPaper();
        await slideCard("outline");
        await sleep(150);
        await notesStore.commitMode("outline");
        await unfoldPanels();
        reportHitRects();
      } else {
        // ① 两侧并行收回（树 + 内容折起、下坠）→ ② 中间列移动 → ③ 左侧打开（纸张展开）
        await foldPanels();
        await fallPanels();
        await slideCard("card");
        await notesStore.commitMode("card");
        await unfoldPaper();
        reportHitRects();
      }
    } finally {
      switchingRef.current = false;
    }
  }

  const seg = <Seg mode={state.mode} onChange={(m) => transitionTo(m)} />;

  // 单击搜索结果：只更新左侧预览，不退出搜索。
  function handleSearchResultPreview(id: string) {
    notesStore.previewFromSearch(id);
  }

  // 双击搜索结果：退出搜索，滚轴跳到对应位置并选中。
  async function handleSearchResultOpen(id: string) {
    const index = await notesStore.openFromSearch(id);
    requestAnimationFrame(() => {
      if (index >= 0) cardListRef.current?.scrollToIndex(index, "center");
      textareaRef.current?.focus();
    });
  }

  async function handleConfirmPurge() {
    if (!confirmId) return;
    const id = confirmId;
    setConfirmId(null);
    await notesStore.purge(id);
  }

  return (
    <div className="window-frame" data-mode={state.mode}>
      <CloseButton onClose={() => notesStore.closeNotes()} />

      <div className={`panel${shown ? " shown" : ""}`}>
        <div className="paper-column" ref={paperRef} data-hit>
          <Paper
            ref={textareaRef}
            page={displayPage}
            readOnly={filtering || isTrash}
            metaIndex={state.metaIndex}
            onChange={(v) => notesStore.onContentInput(v)}
            onTitleChange={(t) => notesStore.onTitleInput(t)}
            onSetTags={(tags) => notesStore.setPageTags(tags)}
            onCreateTag={(name) => notesStore.createTag(name)}
            onDeleteTag={(id) => notesStore.deleteTag(id)}
            onManageTags={() => setTagManagerOpen(true)}
            onPickProject={() => notesStore.pickProject()}
            onSelectProject={(path) => notesStore.setPageProject(path)}
          />
        </div>

        <div className="outline-column" ref={outlineRef} data-hit>
          <OutlineTree
            nodes={state.currentOutline?.nodes ?? []}
            title={state.currentOutline?.title ?? ""}
            activeNodeId={state.activeNodeId}
            onSelect={(id) => notesStore.setActiveNode(id)}
            onAddRoot={(x, y) => notesStore.addRootNode(x, y)}
            onAddChild={(pid, x, y) => notesStore.addChildNode(pid, x, y)}
            onDelete={(id) => notesStore.deleteNode(id)}
            onRename={(id, t) => notesStore.renameNode(id, t)}
            onMove={(id, x, y) => notesStore.moveNode(id, x, y)}
            onRenameTitle={(t) => notesStore.renameOutline(t)}
            onExport={() => notesStore.exportOutline()}
          />
        </div>

        <div className="card-column" ref={cardColumnRef} data-hit>
          {seg}
          {state.mode === "card" ? (
            <>
              <SearchBar
                ref={searchRef}
                value={state.searchQuery}
                metaIndex={state.metaIndex}
                activeTags={state.activeTags}
                activeProjects={state.activeProjects}
                onChange={(v) => notesStore.onSearchChange(v)}
                onSetTags={(tags) => notesStore.setActiveTags(tags)}
                onSetProjects={(projects) => notesStore.setActiveProjects(projects)}
              />
              {listItems.length === 0 ? (
                <EmptyState view={state.view} />
              ) : (
                <CardList
                  ref={cardListRef}
                  items={listItems}
                  view={state.view}
                  activeId={state.currentPageId}
                  defaultProject={state.metaIndex.default_project}
                  hasMore={
                    filtering ? false : isTrash ? state.hasMoreTrash : state.hasMoreActive
                  }
                  onLoadMore={() => notesStore.loadMore()}
                  onSelect={(id) => {
                    if (filtering) handleSearchResultPreview(id);
                    else notesStore.selectPage(id);
                  }}
                  onDoubleSelect={(id) => {
                    if (filtering) handleSearchResultOpen(id);
                  }}
                  onDelete={(id) => notesStore.softDelete(id)}
                  onRestore={(id) => notesStore.restore(id)}
                  onPurge={(id) => setConfirmId(id)}
                />
              )}
              <BottomBar
                view={state.view}
                stats={state.stats}
                mode="card"
                onNew={() => {
                  notesStore.createNewPage().then(() => textareaRef.current?.focus());
                }}
                onToggleTrash={() => notesStore.setView(isTrash ? "active" : "trash")}
              />
            </>
          ) : (
            <>
              <div className="outline-list">
                {state.outlines.length === 0 ? (
                  <div className="empty-state">{isTrash ? "回收站是空的" : "还没有视图，点 ＋ 新建"}</div>
                ) : (
                  state.outlines.map((o) => (
                    <div
                      key={o.id}
                      className={`outline-item${state.currentOutline?.id === o.id ? " on" : ""}`}
                      onClick={() => notesStore.selectOutline(o.id)}
                    >
                      <span className="outline-item-title">{o.title || "（未命名视图）"}</span>
                      {isTrash ? (
                        <div className="outline-item-actions">
                          <button
                            className="outline-item-del restore"
                            title="恢复"
                            tabIndex={-1}
                            onClick={(e) => {
                              e.stopPropagation();
                              notesStore.restoreOutline(o.id);
                            }}
                          >
                            ↩
                          </button>
                          <button
                            className="outline-item-del purge"
                            title="彻底删除"
                            tabIndex={-1}
                            onClick={(e) => {
                              e.stopPropagation();
                              notesStore.purgeOutline(o.id);
                            }}
                          >
                            🗑
                          </button>
                        </div>
                      ) : (
                        <button
                          className="outline-item-del"
                          title="删除"
                          tabIndex={-1}
                          onClick={(e) => {
                            e.stopPropagation();
                            notesStore.deleteOutline(o.id);
                          }}
                        >
                          ×
                        </button>
                      )}
                    </div>
                  ))
                )}
              </div>
              <BottomBar
                view={state.view}
                stats={state.stats}
                mode="outline"
                onNew={() => notesStore.createOutline()}
                onToggleTrash={() => notesStore.setView(isTrash ? "active" : "trash")}
              />
            </>
          )}
        </div>

        <div className="content-column" ref={contentRef} data-hit>
          <NodeContent
            node={selectedNode}
            metaIndex={state.metaIndex}
            onTextChange={(id, t) => notesStore.updateNodeText(id, t)}
            onCreateTag={(name) => notesStore.createTag(name)}
            onDeleteTag={(id) => notesStore.deleteTag(id)}
            onManageTags={() => setTagManagerOpen(true)}
            onPickProject={() => notesStore.pickProject()}
            onImport={(node, title, tags, projects) =>
              notesStore.importNodeToNote(node, title, tags, projects)
            }
          />
        </div>
      </div>
      {confirmId && (
        <TrashConfirm onConfirm={handleConfirmPurge} onCancel={() => setConfirmId(null)} />
      )}
      {state.toast && <div className="toast">{state.toast}</div>}
      {tagManagerOpen && (
        <TagManager
          tags={state.metaIndex.tags}
          onClose={() => setTagManagerOpen(false)}
          onRename={(id, name) => notesStore.renameTag(id, name)}
          onSetDescription={(id, desc) => notesStore.setTagDescription(id, desc)}
          onDelete={(id) => notesStore.deleteTag(id)}
          onRestore={(id) => notesStore.restoreTag(id)}
          onMerge={(ids, name, desc) => notesStore.mergeTags(ids, name, desc)}
        />
      )}
    </div>
  );
}
