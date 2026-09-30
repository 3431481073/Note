# 便签 v12 移植进现有 Tauri 应用的可行性 / 冲突 / 落地报告

> 目标：把 `123.txt`（v12 改进版 HTML 原型）里的新能力，合入现有的 Tauri2 + React + SQLite 便签应用，**保住原版既有交互**，再上新功能。
> 对比基线：v1 = 现有 `src/notes/**`（React）+ `src-tauri/**`（Rust/SQLite）；v12 = `123.txt`（单文件 HTML + localStorage）。
> 日期：2026-09-17

---

## 0. 一句话结论

- **UI 层（外观/组件）**：v12 的卡片模式与原版高度对齐，可复用率约 70%；但**新增的「结构树 + 节点内容面板 + 分段控件 + 标签/项目」是全新模块**，需从零接进 React。
- **数据层（最大冲突）**：v1 是「扁平 page（无标题/无标签/无树）」，v12 是「Page.kind + 递归 nodes + tags/projects」。**SQLite 必须做一次 schema 迁移**，localStorage 只是原型临时方案，不能当生产存储。
- **两个必须让用户拍板的点**：① 窗口尺寸（v1 固定 740×648，v12 需要约 1300 宽的三栏）② 回收站恢复是否跳回便签页（v1「留在回收站」，v12 改成「跳回并选中」——与你早先的诉求相反）。

---

## 1. 新旧差异全景

### 1.1 已确定的新功能（v12 相对 v1 新增）

| 功能 | 说明 | 归属 |
| --- | --- | --- |
| 卡片/视图 分段切换 | 便签页与「结构视图」两模式，带 3D 折叠过渡 | 新 |
| 结构树（outline） | 可拖动节点、加/删子节点、双击改名、贝塞尔连线、点阵画布 | 新 |
| 节点内容面板 | 编辑节点正文 + 「＋卡片」把节点导入为带「视图：」标签的便签 | 新 |
| 标签 / 项目 | 下拉多选、chips 展示、搜索过滤（标签 AND / 项目 OR） | 新 |
| 纸张标题栏 | 便签可选标题 `title` | 新 |
| 卡片标签行 | 卡片显示最多 3 个 chip（视图：琥珀 / 项目紫 / 普通薄荷） | 新 |
| Toast / Launcher / Esc 级联 / Delete 删节点 | 辅助交互 | 新 |

### 1.2 行为差异（已明确改变的点）

| 维度 | v1（现状） | v12（目标） |
| --- | --- | --- |
| 便签排序 | `created_at` 降序（新建置顶） | `updated_at` 降序（最近编辑置顶） |
| 摘要来源 | 正文前 28 字（无标题） | 正文前 90 字 + 显式 `title` |
| 恢复行为 | 留在回收站，不跳页 | **切回便签页并选中** |
| 空态兜底 | 冷启动 0 条才建 1 条空白 | 便签 0 条 `ensureCurrent` 补空；视图空态另立文案 |
| 持久化 | SQLite | localStorage（原型） |
| 窗口 | 740×648 固定 | 1300×560 响应式缩放 |

---

## 2. 移植可行性分级

- ✅ **直接复用（改配色即可）**：卡片、搜索框、底部按钮、空态、关闭按钮、确认框、纸张横线、流光 token——v12 与原版对齐，样式几乎照搬。
- 🔶 **需改造**：纸张（加标题/标签/项目行）、卡片（加 chip 行 + 标题）、搜索（加过滤）、底部按钮（视图语义）、数据模型/命令/事件。
- 🔴 **需新建**：结构树渲染与拖拽、节点内容面板、分段控件 + 模式切换动画、标签/项目下拉、Toast、树布局引擎。
- ⚠️ **需重构**：`.panel` 的 flex 双栏布局 → 绝对定位 + `perspective`（支撑 3D 折叠）；`active` 排序/游标从 `created_at` 切到 `updated_at`。

---

## 3. 核心兼容性冲突（逐条）

### 3.1 数据模型：扁平 page → kind + 递归树（最大改造）

- v1 表 `pages`：`id, content, summary, created_at, updated_at, deleted_at, version`。**没有 title / tags / projects / kind / nodes**。
- v12 `Page`：`kind: "note"|"outline"`、`title`、`text`、`tags[]`、`projects[]`、递归 `nodes[]`、`fromView`。

**迁移建议**（加列，尽量向后兼容）：
```sql
ALTER TABLE pages ADD COLUMN title      TEXT NOT NULL DEFAULT '';
ALTER TABLE pages ADD COLUMN kind       TEXT NOT NULL DEFAULT 'note';
ALTER TABLE pages ADD COLUMN tags       TEXT NOT NULL DEFAULT '[]';  -- JSON 数组
ALTER TABLE pages ADD COLUMN projects   TEXT NOT NULL DEFAULT '[]';  -- JSON 数组
ALTER TABLE pages ADD COLUMN from_view  TEXT;
ALTER TABLE pages ADD COLUMN nodes_json TEXT;                        -- outline 树整体 JSON
```
- 标签/项目用 JSON 列即可（个人便签量级小，`LIKE '%"x"%'` 过滤够用；真要多端查询再上 join 表 `page_tags/page_projects`）。
- 树存 `nodes_json`（整棵树一次读改写，只在视图模式编辑），不必拆 `nodes` 表——避免父子排序/坐标/连线都分散到 SQL。
- **旧数据无 title**：迁移时 `title=''`、`kind='note'`，摘要仍从 `content` 取，保证旧库无损打开。

### 3.2 排序 / 游标 / 索引从 created_at 切到 updated_at

- v1 依赖 `created_at` 做「新建置顶」+ 游标分页 `(created_at, id)` + 索引 `idx_active`。
- v12 按 `updated_at` 降序——**编辑一条旧便签它要跳到最顶**。
- 牵连面：`db.list` 的 ORDER BY 与游标、`idx_active` 索引、前端 `cmpActive`、`insertSorted`、`now_ms` 单调性（`update()` 现在也写 `updated_at=now_ms()`，单调仍成立）。
- 这是一个**全局行为变更**，单测 `list_active_sorted_by_created_desc`、`cursor_pagination_no_overlap` 需同步重写为 `updated_at`。

### 3.3 持久化：localStorage 不能当生产存储

- v12 用 `localStorage`（key `notes-ui-v12-aligned`）+ `seed()` 演示数据。
- 移植必须**回落到 SQLite**（v1 已有 WAL/busy_timeout/游标/version），localStorage 只作原型。`seed()` 的演示数据**不应进入生产**。
- 结果：后端命令 `update_page` 要扩展为「标题/正文/标签/项目」一次更新；新增节点树读写命令。

### 3.4 事件 / 多端一致性

- v1 事件只有 `notes://changed`（`ChangedPayload{op,id,version,view,created_at,deleted_at,summary,ts}`）+ `notes://stats` + `notes://config-changed`。
- 新增实体（标题/标签/项目/树）需要：**扩展 `ChangedPayload` 带 title/tags/projects/kind**，或新增 `notes://tree-changed` / `notes://tags-changed`。否则乐观更新 + 事件回显会对不上（当前 `applyChangedPayload` 只懂 create/update/delete/restore/purge）。
- 工作台角标读 `notes://stats`（active/trash 计数）。v12 有「视图（outline）」这个第三类，需决定：outline 计入 active 计数？还是单独计数？**要和 `stats` 语义对齐**。

### 3.5 窗口几何：740 宽装不下 1300 的三栏

- v1：窗口 740×648，`.panel` 660×560，双栏（纸 382 + 卡 226）。
- v12：Stage 1300×560，视图模式三栏（树 550 + 卡 226 + 内容 400，另有间隙与 `left` 绝对定位）≈ 1226 内容宽。
- **冲突**：现有固定 740 窗口放不下结构树三栏。三条路（需拍板）：
  1. 窗口改大（如 1300×640）——最贴合 v12，但失去「小工具」定位、与工作台右缘工具条的视觉比例变化。
  2. 视图模式**换列**而非加列（卡片列与内容面板二选一 / 折叠卡片列）——塞进 740，但偏离 v12 布局。
  3. 窗口随模式动态 resize（Tauri `set_size`）——体验尚可，但要处理置顶/居中/缩放抖动。
- 同时 v12 的响应式 `scale(.82/.68)` 依赖浏览器视口，Tauri 固定窗口下基本用不上。

### 3.6 布局与 3D 折叠动画

- v1 `.panel` 是 `display:flex` 双栏。v12 各列**绝对定位 + `left` 坐标 + `perspective:2400px`**，靠 `rotateY/rotate/scaleX` 做 6 段折叠。
- 移植：`.panel` 要么改成绝对定位（重构），要么在 React 里用「动画互斥锁 + 时间轴」驱动 `transform`。v12 的 `state.running` 互斥、`T1~T4` 时间常量、5 条缓动曲线都要照搬。
- **务实建议**：第一版先做「瞬时切换」或简单淡入淡出，折叠动画作为第二阶段独立上（动画是纯前端、不影响数据，可后置）。

### 3.7 标题 / 摘要语义

- v1 无标题，`summary = 前 28 字`（Rust `make_summary`）。
- v12：显式 `title` + `text`，摘要前 90 字，且 `titleOf` 回落到节点标题。
- 需改 `make_summary`（28→90）或引入独立 `title` 字段 + 新的标题显示逻辑。**注意**：改摘要长度会影响现有卡片展示与单测。

### 3.8 搜索语义

- v1：`content LIKE` + `ESCAPE '\'`，只能搜正文。
- v12：搜 `title+text`，并叠加**标签多选 AND / 项目多选 OR**过滤。
- 后端 `list/search` 需增加 tag/project 过滤参数；前端 `notesStore` 需加 `activeTags/activeProjects` 状态与派生过滤。标签/项目过滤放在后端（SQL）还是前端（内存）要定——量小可前端过滤，量大走后端。

### 3.9 恢复行为与底部按钮语义冲突

- **恢复**：v1「留在回收站」（你之前明确要求过），v12 改「跳回便签页并选中」。**这是与既有诉求的冲突，需确认最终以哪个为准**。
- **底部 `＋`**：v1 固定「新建便签」；v12 在视图模式变成「新建视图」。`BottomBar` 的语义由 `mode` 决定，需要改 `onNew` 的分支。

### 3.10 原型特有、无需进生产的点

- `seed()` 演示数据、`localStorage` key、`launcher` 页面内唤出按钮（Tauri 里已有单实例插件 + 工作台工具按钮承担「再点置顶」）、响应式缩放——这些是 HTML 原型的环境产物，移植时**不要照搬**。

---

## 4. 需要新建的模块 / 引擎

| 模块 | 前端 | 后端 | 说明 |
| --- | --- | --- | --- |
| 结构树（outline） | `OutlineTree` 组件 + SVG/Canvas 连线 | `nodes_json` 读写命令 | 拖拽、布局、贝塞尔连线 |
| 树布局引擎 | `layoutTree()` / 测量 `_w/_h` | — | 节点绝对坐标计算，React 版需用 ref 测量 |
| 节点内容面板 | `NodeContent` 组件 | 复用 update 命令 | 编辑节点 + 「＋卡片」导入 |
| 分段控件 + 模式切换 | `Seg` 组件 + `toCard/toOutline` | — | `mode` 状态 + 3D 折叠动画（可后置） |
| 标签/项目下拉 | `DropPanel` + `SearchFilters` | tag/project 过滤参数 | 多选、新建标签、chips |
| Toast | `Toast` 组件 | — | 轻量提示 |
| 数据结构类型 | `types.ts` 扩 `Page/Node` | `models.rs` 扩字段 | 两侧类型对齐 |
| 事件扩展 | `applyChangedPayload` 支持新字段 | `ChangedPayload` 扩字段 | 多端一致性 |

---

## 5. 现有后台需要提升 / 改造的点

1. **`models.rs` / `db.rs`**：`Page/PageMeta` 加 `title/kind/tags/projects`；`PageList` 返回带这些字段的 meta（卡片要显示标题+chip）。
2. **`update_page`**：从「只改 content」扩为「title + content + tags + projects」一次落库（一个事务）。
3. **`create_page`**：支持 `kind`（note/outline），建 outline 时初始化空树。
4. **`search`/`list`**：加 tag/project 过滤；排序列切 `updated_at`（若采纳）。
5. **`summary`**：`make_summary` 长度 28→90，或新增 `make_title`。
6. **`events.rs`**：`ChangedPayload` 扩 `title/tags/projects/kind`（或新增树事件）。
7. **`stats`**：决定 outline 是否计入 `active`，避免工作台角标语义混乱。
8. **迁移**：`migrate()` 增加 `schema_migrations` 版本号驱动的 `ALTER TABLE` 增量迁移，保证旧库无损升级（当前只有 `CREATE TABLE IF NOT EXISTS`，无版本升级机制——**这一步是接入新字段的前提**）。
9. **`window.rs`**：按 §3.5 决策调整窗口尺寸/可缩放/动态 resize。
10. **单测**：排序、游标、summary、迁移相关用例同步重写。

---

## 6. 现有前端需要改造的点

1. **`types.ts`**：`View` 之外加 `Mode`；`PageMeta` 加 `title/tags/projects/kind`；新增 `Node`。
2. **`notesStore.ts`**：
   - 加 `mode`、`activeTags`、`activeProjects`、`activeNodeId`、`running`（动画锁）状态。
   - `selectPage`/`setView`/`openFromSearch` 适配 `kind`。
   - `onContentInput` 拆分「标题输入」与「正文输入」；新增标签/项目更新。
   - 恢复/新建的分支按最终行为调整（§3.9）。
3. **`App.tsx`**：装配分段控件、结构树、内容面板；模式切换逻辑。
4. **`Card.tsx`**：加标题行 + chip 行。
5. **`Paper.tsx`**：加 `.paper-meta`（标签/项目）+ `.paper-title`。
6. **`notes.css`**：`.panel` flex → 绝对定位 + `perspective`；新增 tree/panel/drop/toast 样式；`@property --halo` 双写 `mask-composite`。
7. **`opSeq.ts`**：`cmpActive` 切 `updated_at`；`applyChangedPayload` 支持新字段。

---

## 7. 下一版落地路线（分阶段，每阶段可独立验证）

- **阶段 0 · 基线锁定**：把当前 v1 打成 tag/分支，跑通全量测试（17/17 + 12/12）作为回归基线。
- **阶段 1 · 数据迁移（纯后端）**：schema 迁移（加列/版本号）→ 扩展 `models/db/commands/events` → 重写单测。**此阶段不碰 UI，验证旧库能无损打开、旧功能不回归。**
- **阶段 2 · 排序与搜索（后端+前端小改）**：切 `updated_at` 排序、tag/project 过滤。回归验证列表/搜索。
- **阶段 3 · 卡片模式增强（前端）**：纸张标题 + 标签/项目 + 卡片 chip 行 + 搜索过滤器。此时仍是 740 双栏，可先交付。
- **阶段 4 · 结构视图（新模块）**：树渲染 + 拖拽 + 内容面板 + 「＋卡片」导入 + 分段控件。**先做瞬时切换，后上折叠动画。**
- **阶段 5 · 窗口/动效收尾**：窗口尺寸决策、3D 折叠、Toast、Esc/Delete 快捷键、减少动效降级。
- **阶段 6 · 回归与发布**：全量测试、手测清单（见 §8）、更新 `DEVELOPMENT_LOG.md`。

---

## 8. 中途需要检查的清单（checklist）

**数据**
- [ ] 旧 `notes.db` 用新版启动：无报错、旧便签标题为空但不丢内容。
- [ ] 新建/编辑/软删/恢复/彻底删除在迁移后仍正常，version 递增正确。
- [ ] 排序切 `updated_at` 后：编辑旧便签置顶、新建置顶、游标分页无重复无遗漏。
- [ ] 标签/项目过滤：多选 AND/OR 逻辑、空结果、特殊字符（`% _ \`）不误匹配。

**UI**
- [ ] 卡片模式与原版视觉一致（流光、滚动暂停、角标、80/20 按钮）。
- [ ] 视图模式：拖拽节点、连线跟随、双击改名、加/删节点、坐标持久化、`justDragged` 防误点。
- [ ] 「＋卡片」导入：生成带「视图：」标签便签，重复导入撤销。
- [ ] 模式切换动画互斥（`running` 期间忽略重复点击）。
- [ ] 空态：便签 0 条 / 回收站空 / 搜索无结果 / 视图空节点 四种文案正确。

**壳 / 全局**
- [ ] 单实例：重复启动仍只置顶已有窗口。
- [ ] 关闭 = 隐藏、重新唤出数据不丢、未保存内容先 flush。
- [ ] `Ctrl+F` 聚焦搜索、`Esc` 级联关闭、`Delete` 仅视图模式删节点。
- [ ] `prefers-reduced-motion` 下动画全部降级为静态描边。
- [ ] 工作台角标计数语义（outline 是否计入）正确。

**回归**
- [ ] `npm run build`、`npm test`、`cargo test --release` 全绿。
- [ ] release exe 重编（确认 `custom-protocol` 仍在，避免退回 dev 连 `localhost:1420`）。

---

## 9. 风险与建议（按优先级）

1. **先做数据迁移、后做 UI**——数据是硬约束，UI 可渐进。不要在 localStorage 原型上继续长功能，尽早回到 SQLite。
2. **折叠动画后置**——它是纯视觉、最易出 bug 且不影响数据正确性；先用瞬时切换保证功能闭环。
3. **排序切 `updated_at` 要单独立项**——它会改变「新建置顶」的老习惯，且牵连游标/索引/测试，别混在其他改动里。
4. **窗口尺寸与恢复行为是两处「需求冲突」**——务必先和用户确认，避免返工。
5. **保留 version 乐观更新机制**——新增标题/标签/树后，事件回显的幂等合并要继续成立，否则会重现「同 id 重复插入→多聚焦卡顿」。
6. **树坐标持久化要防脏数据**——节点 `x/y/_w/_h` 写入库前做 clamp，避免越界拖出画布。

---

## 10. 一句话行动建议

> 建议按「**先迁数据 → 再上排序/搜索 → 卡片模式增强（可先交付）→ 结构视图（瞬时切换）→ 折叠动画 + 窗口收尾**」的顺序推进；每个阶段都以「旧功能不回归 + 全量测试通过」为出口，窗口尺寸与恢复行为两处冲突先和用户确认再动工。
