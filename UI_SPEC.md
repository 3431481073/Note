# 便签 UI 界面与交互规格（导出文档）

> 用途：在别处 **1:1 复现**这套「便签」界面的**外观 + 交互 + 数据呈现**，再在其上叠加新功能、测试新 UI。
> 本文只讲「界面与交互」，不涉及后端具体实现（SQLite、命令、事件广播的内部逻辑只在驱动 UI 的范围内点到为止）。
> 基准代码：`src/notes/**`（React + TS + Vite），样式 `src/notes/styles/notes.css`，窗口 `src-tauri/src/notes/window.rs`。

---

## 1. 窗口壳（Shell）

| 属性 | 值 | 说明 |
| --- | --- | --- |
| 尺寸 | **740 × 648** | 逻辑像素，`inner_size(740.0, 648.0)` |
| 边框 | 无（`decorations(false)`） | 无系统标题栏/边框 |
| 透明 | 是（`transparent(true)`） | 圆角、阴影由前端 CSS 自绘 |
| 置顶 | 是（`always_on_top(true)`） | 盖在其他窗口上 |
| 任务栏 | 不显示（`skip_taskbar(true)`） | |
| 缩放 | 禁止（`resizable(false)`） | 固定大小，无拖拽边框 |
| 系统阴影 | 关（`shadow(false)`） | 阴影由面板 `box-shadow` 自绘 |
| 位置 | 屏幕居中（`center()`） | |
| 初始可见 | **否**（`visible(false)`） | 避免露出空白 webview 闪烁 |
| 标题 | 便签 | |

**显示时机**：前端首帧渲染完成后调用 `getCurrentWindow().show()` + `setFocus()`，再触发面板浮现动画（见 §8）。**关闭 = 隐藏窗口**（`hide()`，进程不退出）；单实例插件保证重复启动只把已有窗口置顶。

---

## 2. 布局总览

```
┌──────────────────────────────────────────────┐  ← 窗口 740×648（透明无边框）
│                                              │
│   ┌────────────────────────────────────────┐ │
│   │  面板 .panel  660×560                   │ │
│   │  left:40 top:24  padding:16 18          │ │
│   │                                        │ │
│   │  ┌───────────────────┐ ┌──────────────┐│ │
│   │  │ 纸张 .paper        │ │ 卡片列        ││ │
│   │  │ 382 × 520         │ │ 226 宽        ││ │
│   │  │ (rotate -0.6°)    │ │              ││ │
│   │  │  ┌─────────────┐  │ │ ┌──────────┐ ││ │
│   │  │  │ .paper-date │  │ │ │ 搜索框    │ ││ │
│   │  │  │ 右下角日期  │  │ │ │ 226×32   │ ││ │
│   │  │  └─────────────┘  │ │ ├──────────┤ ││ │
│   │  │  ┌─────────────┐  │ │ │ 卡片列表  │ ││ │
│   │  │  │ .paper-     │  │ │ │ 226×432  │ ││ │
│   │  │  │ content     │  │ │ │ (虚拟滚动)│ ││ │
│   │  │  │ textarea    │  │ │ │          │ ││ │
│   │  │  │ 28px 横线   │  │ │ │ [卡片]   │ ││ │
│   │  │  └─────────────┘  │ │ │ [卡片]   │ ││ │
│   │  │  (水印叠加层)    │  │ │ │  …       │ ││ │
│   │  └───────────────────┘ │ ├──────────┤ ││ │
│   │                        │ │ 底部按钮  │ ││ │
│   │                        │ │ 226×40   │ ││ │
│   │                        │ │ ＋ / 🗑  │ ││ │
│   │                        │ └──────────┘ ││ │
│   │                                             [×] 关闭
│   └────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

**关键尺寸表**：

| 元素 | 宽 × 高 | 位置/备注 |
| --- | --- | --- |
| 窗口 | 740 × 648 | |
| 面板 `.panel` | 660 × 560 | left 40 / top 24；padding 16(上下) 18(左右)；两栏 `gap:10` |
| 左栏 `.paper-column` | 382 | 固定宽 |
| 右栏 `.card-column` | 226 | 固定宽，纵向 `flex` + `gap:10` |
| 纸张 `.paper` | 382 × 520 | 圆角 14，`rotate(-0.6deg)` |
| 搜索框 `.search-input` | 226 × 32 | |
| 卡片列表 `.card-list-outer` | 226 × 432 | |
| 底部 `.bottom-bar` | 226 × 40 | 两按钮 `gap:8` |
| 单张卡片 `.card` | 100% × 84 | 列表槽 `.card-slot` 高 84 |

---

## 3. 视觉主题（设计 Tokens，CSS 变量）

复现时直接照搬这组值即可，全部定义在 `:root`：

```css
--ink-current: #64748b;   /* 主墨色（蓝灰），运行时会被月份色覆盖 */
--text:        #263238;   /* 正文近黑 */
--paper-bg:    #ffffff;   /* 纸张纯白 */
--paper-bg2:   #f6f9fc;   /* 纸张底部极浅冷色 */
--paper-line:  #eef2f7;   /* 纸面横线 */
--card-bg:     #ffffff;   /* 卡片纯白 */
--card-border: #e6ebf2;   /* 卡片描边冷灰 */
--card-shadow: 0 1px 3px rgba(15,23,42,0.06);
--trash-card-bg:#f2f5f9;  /* 回收站卡片浅灰 */
--trash-text:  #94a3b8;   /* 回收站文字灰蓝 */
--accent:      #2dd4bf;   /* 清爽薄荷青（强调/流光/角标/高亮） */
--danger:      #ef4444;   /* 删除红 */
```

- 全局字体：`-apple-system, "Segoe UI", "Microsoft YaHei", "PingFang SC", sans-serif`，抗锯齿。
- `html/body/#root` 背景 `transparent`、`overflow:hidden`、满屏。
- 主题是「清爽冷色」路线：纸/卡纯白、描边冷灰、强调薄荷青，**没有泛黄**。

---

## 4. 组件清单

### 4.1 纸张 `Paper`（左栏主体）
- 结构：`.paper` 内三层
  1. `.paper-watermark`：绝对定位铺满、`opacity:0.1`、居中、`pointer-events:none`，装季节水印 SVG（当前配置目录为空 → 不显示）。
  2. `.paper-date`：右上角日期，11px，`color: var(--ink-current)`，内容 `formatDate(updated_at || created_at)`（格式 `YYYY.MM.DD HH:mm`）。
  3. `textarea.paper-content`：正文，`flex:1`，15px / 行高 28px，无边框无 outline 不可 resize，透明背景 + **每 28px 一条横线**（`repeating-linear-gradient`），占位符「写点什么…」。
- 外观：`linear-gradient(180deg, #fff, #f6f9fc)` 底色，圆角 14，整体 `rotate(-0.6deg)`，大阴影 `0 18px 40px rgba(0,0,0,.12)`。
- 状态：
  - 有页：显示该页内容，可编辑。
  - 无页（`page=null`）：日期空、正文空、只显示占位符。
  - **搜索中**：`readOnly`，内容 = 搜索结果预览（不可编辑）。

### 4.2 搜索框 `SearchBar`（右栏顶部）
- 结构：`.search-input` 包裹 `<input>`，占位符「搜索便签…」，13px，`color: var(--ink-current)`。
- 外观：226×32，圆角 10，白底，`outline:1px solid var(--card-border)`。
- **光带状态**（与卡片高亮同款，见 §8）：
  - `:focus-within` 或 `.active`（有字）时，外圈出现旋转的薄荷青流光（3.2s 一圈）+ 光晕。
  - 此时**卡片高亮流光被关闭**（`:has(.search-input.active)` / `:has(:focus-within)` 让卡片 `::before/::after` `display:none`）——搜索框抢走光带焦点。

### 4.3 卡片列表 `CardList` + 卡片 `Card`（右栏中部）
- 虚拟滚动：`@tanstack/react-virtual`，单卡估高 92、槽高 84、`overscan:3`。
- 滚动容器 `.card-list-scroll`：226×432，`overflow-y:auto`，**滚动条三端全隐藏**。
- 卡片 `.card`（84 高）结构：
  - `.card-head`：日期，12px，`var(--ink-current)`。
  - `.card-body`：摘要（`meta.summary`，空则「（空白便签）」），14px，**最多 2 行**（`-webkit-line-clamp:2`）。
  - 动作按钮（右上角，24×24，圆角 6，半透明底）：
    - **便签页**：`×` 删除（`card-delete`）。
    - **回收站**：`↩` 恢复 + `🗑` 彻底删除（`card-actions`，`🗑` 为 `--danger` 红）。
- **卡片状态**：
  - **普通**：白底、`outline:1px` 灰边、浅阴影。
  - **活跃（当前选中）**：`outline` 透明、薄荷青辉光 `box-shadow:0 0 12px rgba(45,212,191,.22)`、`z-index` 提升、外圈**旋转流光**（`::before/::after` 双 `conic-gradient`，见 §8）。
  - **回收站卡片**：浅灰底 `#f2f5f9`、灰蓝文字。
- 分页：列表底部有「加载更多」按钮（仅 `hasMore` 时显示），12px，半透明，hover 变实。

### 4.4 底部按钮 `BottomBar`（右栏底部）
- 两个按钮，`height:32`，圆角 10，白底描边，`gap:8`：
  - **`＋` 新建**（`.bb-btn-new`，`flex:4`，约占 80%）：**只在便签页显示**，回收站视图不渲染。
  - **`🗑` 回收站 / `便签` 返回**（`.bb-btn-trash`，`flex:1`，约占 20%）：
    - 便签页：显示「🗑」，若 `stats.trash > 0` 右上角有薄荷青圆形角标（数字）。
    - 回收站：显示「便签」（返回）。
- hover：底 `#eef4f8`、描边变薄荷青。

### 4.5 空状态 `EmptyState`（卡片列表区占位）
- 226×432 居中文字，13px，`opacity:0.7`：
  - 便签页空：「还没有便签，点 ＋ 新建一条」
  - 回收站空：「回收站是空的」

### 4.6 关闭按钮 `CloseButton`（面板右上角外）
- 28×28 圆形，`×`，位于面板 `top:-16 right:-16`（浮出面板右上角），`z-index:30`，hover 底 `#f0e9da`。

### 4.7 确认框 `TrashConfirm`（模态）
- 全窗遮罩 `.confirm-mask`（`rgba(0,0,0,.18)`），居中白卡 300px：
  - 标题「彻底删除？」（15px 加粗）
  - 描述「删除后不可恢复。」（13px）
  - 按钮：`取消`（透明，hover 浅灰）/ `彻底删除`（红底白字）。
- 点遮罩或「取消」关闭；点「彻底删除」执行并关闭。

---

## 5. 界面状态机

| 状态 | 触发 | 纸面 | 卡片列表 | 底部 |
| --- | --- | --- | --- | --- |
| **启动浮现** | 打开窗口 | 加载中 | 加载中 | — |
| **便签页·有内容** | 默认 | 当前便签可编辑 | active 列表，选中项流光 | `＋` + `🗑(角标)` |
| **便签页·空** | 删光 | 空纸（占位符） | 空态文案 | `＋` + `🗑` |
| **回收站·有内容** | 点 🗑 | 当前回收项只读（灰） | trash 列表，灰卡 | `便签(返回)` |
| **回收站·空** | 清空回收站 | 空纸 | 空态「回收站是空的」 | `便签(返回)` |
| **搜索中** | 搜索框有字 | 预览结果（只读） | 搜索结果列表 | 不变 |
| **确认删除** | 点 🗑(回收站) | 不变 | 不变 | 遮罩+弹窗 |

**视图切换规则**：`setView(active↔trash)` 会清空搜索、重载列表、选中第一条（空则清空预览）。

---

## 6. 交互地图（动作 → 视觉反馈 → 状态/数据变化）

| # | 区域 | 交互 | 立即视觉反馈 | 状态/数据变化 |
| --- | --- | --- | --- | --- |
| 1 | 窗口 | 打开/被工作台启动 | 面板从中间浮现（opacity+scale） | `init()`：拉 stats → 加载 active 第一页 → 选中第一条（空则新建一条空白） |
| 2 | 纸面 | 输入文字 | 纸面实时显示、右上角摘要同步 | 乐观更新 `currentPage.content` 与列表摘要；300ms（>5000 字则 1000ms）防抖落盘 `updatePage` |
| 3 | 纸面 | 切走前 | — | `flushNow()` 立即落盘未保存内容 |
| 4 | 卡片 | 单击 | 该卡亮起流光、纸面切到该便签 | `selectPage(id)`：flush → `getPage` → 设 `currentPageId/currentPage` |
| 5 | 底部 `＋` | 单击 | 新建卡片置顶、纸面聚焦 | `createNewPage()`：flush → `createPage` → 幂等插入 active → `view:active` → 焦点回纸面；`creating` 防抖避免连建 |
| 6 | 卡片 `×` | 单击 | 卡片从列表消失 | `softDelete(id)`：后端软删 → 从 active 移入 trash → stats 更新；若删的是当前项，选下一条 active，空则清空纸面（空态） |
| 7 | 回收站卡 `↩` | 单击 | 卡片从回收站消失 | `restore(id)`：后端恢复 → 从 trash 移回 active；**留在回收站视图**；若恢复的是当前项，选下一条 trash，空则清空 |
| 8 | 回收站卡 `🗑` | 单击 | 弹出确认框 | 仅 `setConfirmId(id)`（打开模态） |
| 8a | 确认框 `彻底删除` | 单击 | 弹窗关闭、卡片消失 | `purge(id)`：后端物理删 → 从 trash 移除 → stats；若删的是当前项，选下一条 trash，空则清空（留在回收站） |
| 8b | 确认框 `取消`/遮罩 | 单击 | 弹窗关闭 | 无变化 |
| 9 | 底部 `🗑`/`便签` | 单击 | 视图切换、角标变化 | `setView(active↔trash)`：flush → 清空搜索 → 重载对应列表 → 选第一条或清空 |
| 10 | 搜索框 | 输入 | 输入即显示；聚焦时有流光 | `onSearchChange`：200ms 防抖 `search`；结果替换列表、`previewPage` 作只读预览；清空则退出搜索 |
| 11 | 搜索结果卡片 | 单击 | 退出搜索、纸面打开该便签 | `openFromSearch`：清空搜索 → `selectPage` → 滚动定位到该卡 |
| 12 | 卡片列表 | 滚轮 | 滚动；滚动中流光暂停 | 虚拟滚动；触底「加载更多」按钮拉下一页 |
| 13 | 关闭 `×` | 单击 | 窗口隐藏 | `closeNotes()`：flush → `hide()` |
| 14 | 全局 | `Ctrl/Cmd+F` | 聚焦搜索框 | `registerShortcuts` 监听 keydown |
| 15 | 卡片动作按钮 | Tab/键盘 | 不可聚焦（`tabIndex=-1`） | 只留鼠标，防止键盘误触删/恢复 |
| 16 | 各按钮 | hover | 底色/描边变化 | 见 §4 各组件 hover |

---

## 7. 数据模型（驱动 UI 的数据）

```ts
type View = "active" | "trash";

interface PageMeta {           // 列表里的一条卡片
  id: string;
  summary: string;             // 卡片正文摘要（空则「（空白便签）」）
  created_at: number;          // 毫秒时间戳
  deleted_at: number | null;   // null=未删（active），非 null=在回收站
  version: number;
}
interface Page extends PageMeta {
  content: string;             // 纸面全文
  updated_at: number;          // 日期用 updated_at || created_at
}
interface PageList { items: PageMeta[]; has_more: boolean; next_cursor: Cursor | null }
interface Stats { active: number; trash: number }   // 底部角标数字来源
```

**Store 单一数据源**（`notesStore`，`useSyncExternalStore` 订阅），核心状态：

```ts
view, active[], trash[], searchResults[], previewPage, searchQuery,
currentPageId, currentPage, stats, hasMoreActive, hasMoreTrash,
nextActiveCursor, nextTrashCursor, initialized, loading
```

**派生（App 里）**：
- `listItems = searchResults ?? (isTrash ? trash : active)` —— 决定卡片列表渲染谁。
- `displayPage = searching ? previewPage : currentPage` —— 决定纸面显示谁。
- `isTrash = view === "trash"`、`searching = searchQuery.length > 0`。

**后端事件回填**：写操作后端 `emit` 三个事件，前端幂等合并，避免乐观更新与回声重复插入：
- `notes://changed` → `handleChanged`（增量更新 active/trash 列表）
- `notes://stats` → `setStats`（角标）
- `notes://config-changed` → 重新 `applySeason`

**排序**：active 按 `created_at` 降序（新建置顶）；trash 按 `deleted_at` 降序（最新删的置顶）。时间戳在进程内单调递增保证并列不随机。

---

## 8. 动画 / 动效清单

| 动效 | 触发 | 实现要点 |
| --- | --- | --- |
| 面板浮现 | 窗口显示后 | `.panel` `opacity:0 + scale(.92) translateY(12px)` → `.shown` 过渡到正常（260ms，回弹贝塞尔） |
| 活跃卡片流光 | 选中卡片 | `@property --angle` 自定义属性动画 + 双 `conic-gradient`（`::before`/`::after`，`from` 差 135°）+ `mask-composite:exclude` 只留描边环；4s 一圈 |
| 搜索框流光 | 聚焦/有字 | 同款 conic-gradient，3.2s 一圈，附加 `drop-shadow` 光晕 |
| 流光抢占 | 搜索框活跃时 | 卡片流光 `display:none`（`:has()` 选择器） |
| 滚动暂停流光 | 列表滚动 | 滚动容器打 `data-scrolling="true"`，`animation-play-state:paused`，停止 120ms 后恢复 |
| 淡入淡出 | 卡片增删（预留） | `@keyframes page-turn / card-fly-out-* / card-shrink-out / new-page-bounce / panel-close` 已定义 |
| 减少动效 | 系统开启减少动效 | `@media (prefers-reduced-motion)` 关流光动画，改静态描边 |

---

## 9. 复现时的关键细节 / 坑

1. **滚动条三端全隐藏**：`scrollbar-width:none` + `-ms-overflow-style:none` + `::-webkit-scrollbar{display:none}`。
2. **流光必须做「滚动时暂停」**，否则 conic-gradient 跑在主线程与滚动抢帧会卡。
3. **卡片动作按钮 `tabIndex={-1}`**：防止键盘 Tab/回车误触删除/恢复，选择只靠鼠标点击。
4. **新建/删除/清空都走「幂等合并」**：乐观更新后后端回声可能先到，重复插入会导致同 id 多张卡、多焦点、卡顿——插入前 `some(id)` 判重。
5. **空态要兜底**：删光/清空后 `currentPageId=null`、显示 `EmptyState`，**不要再自动补空白便签**（否则会累积两个空白）。只有冷启动 0 条时才新建一条。
6. **纸面横线**：不是图片，是 `repeating-linear-gradient(transparent 0 27px, line 27px 28px)` + `background-attachment:local`，行高必须正好 28px 才对齐。
7. **日期格式**：`YYYY.MM.DD HH:mm`。
8. **右下角纸面日期**用 `updated_at || created_at`；**卡片头日期**用 `deleted_at || created_at`（回收站显示删除时间）。
9. **活跃卡片流光 `overflow:visible`**：卡片和列表外层都不能裁剪光带溢出（`.card{overflow:visible}`）。
10. **主题色单一来源**：`--ink-current` 会被月份配置覆盖，改配色只改 CSS 变量即可。

---

## 10. 文件对应关系（复现时对照）

| 界面部分 | 文件 |
| --- | --- |
| 布局/装配 | `src/notes/App.tsx` |
| 纸张 | `components/Paper.tsx` |
| 搜索框 | `components/SearchBar.tsx` |
| 卡片列表（虚拟滚动） | `components/CardList.tsx` |
| 单张卡片 | `components/Card.tsx` |
| 底部按钮 | `components/BottomBar.tsx` |
| 空态 | `components/EmptyState.tsx` |
| 关闭按钮 | `components/CloseButton.tsx` |
| 确认框 | `components/TrashConfirm.tsx` |
| 全部样式 | `styles/notes.css` |
| 状态/交互逻辑 | `store/notesStore.ts` |
| 数据模型 | `types.ts` |
| 后端命令封装 | `store/notesApi.ts` |
| 事件/季节主题 | `events/notesEvents.ts` |
| 快捷键 | `shortcuts/keys.ts` |
| 窗口壳 | `src-tauri/src/notes/window.rs` |
