# Note（便签）

**一个接入「工作台」的便签应用 —— 卡片记事，结构树理思路。**

技术栈：**Tauri 2 + React 18 + TypeScript + Vite + SQLite（rusqlite bundled）**。

> **整理状态**：2026-10-03 按 `project_manager/docs/PLAYBOOK.md` 整理。
> 整理记录见 [log.md](log.md)。**整理过程代码零改动。**

---

## 一、它是什么

随手记下来的东西会散。记得越多，越找不回、越理不清。

**这个便签给同一份数据两种看法：**

| 模式 | 用途 |
| --- | --- |
| **卡片模式** | 快速记和快速找（虚拟列表滚动、按时间倒序、游标分页） |
| **结构树模式** | 把一堆零散便签整理成一棵可折叠的树 |

**随时切换，不用搬数据。**

**它不是独立软件** —— 作为插件挂进「工作台」。工作台是一个 **56×64 的无边框竖条**，贴在屏幕边缘；
点一下浮出便签窗口，不占桌面、不进任务栏。

## 二、⚠️ 它是一个移植项目

**这是理解这个仓库的关键。** 它不是从零写的，而是**把一份叫「便签 v12」的单文件 HTML 应用移植进一个已有的 Tauri 应用**。

移植链完整保留在 `docs/` 里：

```
specs/123.txt              ← 源头：便签 UI 界面与交互规格 v12（导出文档）
                             基准代码是「单文件 index.html（HTML+CSS+原生 JS，无构建依赖）」
                             用途写明：「在别处 1:1 复现这套界面的外观+交互+数据呈现」
        ↓
history/PORTING_REPORT.md  ← 移植进现有 Tauri 应用的可行性 / 冲突 / 落地报告
        ↓
specs/456.txt              ← 用户对该报告的回复（口径）
                             总原则：「用户能看见的 UI 与交互由我定死；
                                     看不见的后台/存储/性能/兼容/动画细节由你自行拍板；
                                     新旧数据冲突统一做一次迁移」
        ↓
history/IMPLEMENTATION_PLAN.md ← 落地执行方案（含结论、计划与疑问）
        ↓
history/DEVELOPMENT_LOG.md ← 实施记录（踩过的坑、验证结果、沉淀的认知）42,756 B，最大的文档
        ↓
history/UI_SPEC.md         ← 成型后的界面与交互规格
```

> **所以「v12」不是本项目的版本号，是它移植的那份源规格的版本号。**

## 三、形态一览

| 项 | 值 |
| --- | --- |
| **形态** | **Windows 桌面应用**（Tauri 2），**常驻托盘/边缘竖条 + 按需浮出的便签窗口** |
| **常驻服务** | 🔴 **有** —— **只读对外接口**（loopback HTTP），`127.0.0.1:17321`。见 [service.md](service.md) |
| **单实例** | ✅ `tauri-plugin-single-instance`（重复启动只聚焦已有窗口） |
| **数据** | **SQLite**（`rusqlite` bundled），库在**项目外**（见 [data.md](data.md)） |
| **交付物** | `npm run tauri build` 的产物。**仓库根没有任何 exe / dll** |
| **平台** | **仅 Windows** —— 依赖 `windows-sys`（窗口定位、点击热区） |
| **许可证** | MIT |
| **状态** | **保留项目，用户说"非常重要"** |

## 四、⚠️ 项目名是分裂的 —— 同一样东西五个名字

| 出现的地方 | 叫什么 |
| --- | --- |
| 目录名 / GitHub 仓库 | **`Note`**（`github.com/3431481073/Note`） |
| `package.json` 的 `name` | **`my-workbench-notes`** |
| `Cargo.toml` 的 `package.name` | **`my-workbench-notes`** |
| `tauri.conf.json` 的 `productName` | **`MyWorkbench`** ← ⚠️ **不叫 Note，也不叫便签** |
| `tauri.conf.json` 的 `identifier` | **`com.myworkbench.notes`** ← 决定数据目录 |
| `README.md` 的标题 | **`便签`** |

**盘点时最容易把它当成好几个项目。** 记住：**它们指的是同一个东西。**

> `productName` 是 `MyWorkbench` 而不是 `便签`，是因为**这个 Tauri 应用同时是"工作台壳"** ——
> `index.html` 那个 56×64 竖条就是工作台本身，`notes.html` 才是便签窗口。

## 五、架构

```
┌─ 前端（两个 HTML 入口 —— vite 的 rollupOptions.input 显式声明）
│    index.html  → src/workbench/   ← 工作台壳：56×64 竖条，显示便签数角标
│    notes.html  → src/notes/       ← 便签本体（主）
│        App.tsx · store/notesStore.ts(26 KB) · components/(14 个) ·
│        events/notesEvents.ts · shortcuts/keys.ts · styles/notes.css(37 KB)
│
└─ Rust  src-tauri/src/
     main.rs            注册 27 个命令 + single-instance 插件 + setup
     notes/
       db.rs        51,678 B   ← SQLite 全部数据层（最大的 Rust 文件）
       commands.rs   8,848 B   ← 暴露给前端的命令
       import.rs     🆕        ← 外部文本源 → 卡片（后台线程；见 10.2）
       http.rs       7,008 B   ← 只读 HTTP 接口（17321）
       config.rs     4,103 B   ← 配置目录 + 热重载 watcher（notify）
       summary.rs    3,556 B   ← 摘要/ID/时间戳 + 默认项目根（编译期路径）
       hittest.rs    2,091 B   ← Windows 点击穿透（WM_NCHITTEST 子类化）
       models.rs · window.rs · events.rs · trace.rs · mod.rs
     workbench/
       action.rs · mod.rs      ← toggle_notes / open_notes（给工作台调）
```

> 🔴 **2026-10-14**：上图与下面 8.3 的**命令数已更新为 27**（+`catcher_import_status`）。
> **`db.rs` / `commands.rs` / `config.rs` 的字节数与行号是 2026-10-03 的快照，本轮之后会漂** ——
> 按第九节的口径**只声明，不回填**。

**几个关键设计**：

| 设计 | 说明 |
| --- | --- |
| **窗口点击穿透** | 便签窗口固定 1300×648、透明、无边框。`hittest.rs` 装 `WM_NCHITTEST` 子类化：**透明区返回 `HTTRANSPARENT`，可见面板矩形内正常响应**。矩形由前端 `getBoundingClientRect() * devicePixelRatio` 回传 |
| **窗口居中到工作区** | `window.rs` 用 `primary_monitor().work_area()` 算，**排除任务栏**，与工作台竖条同一套逻辑 |
| **前端首屏后再 `show()`** | Rust 只创建隐藏窗口（`visible: false`），**前端渲染完一帧才自己 show** —— 避免先露出空白 webview 的闪烁 |
| **配置热重载** | `config.rs` 用 `notify` 监听 `seasons.json`，变化时广播 `notes://config-changed` |
| **墨色主题** | 墨色**按月切换 12 色**（`config/seasons.json`）+ 当季**节气水印 SVG**（`config/assets/watermark/`，12 个） |
| **时间戳单调递增** | `summary.rs::now_ms()` 用 CAS 保证同进程内严格递增 —— 否则毫秒并列会让"新建置顶"排序与游标分页退化成随机 ID 排序 |

## 六、目录

```
Note/
├── docs/                     ← 本目录：六件套 + OUTPUT-SLOT.md + 两个归档子目录
│   ├── README.md · run.md · data.md · deps.md · service.md · log.md
│   ├── OUTPUT-SLOT.md        ← 输出槽规格（127.0.0.1:17321）
│   ├── specs/                ← 移植当时的往来记录：123.txt · 456.txt
│   └── history/              ← 成文文档：DEVELOPMENT_LOG.md ·
│                                IMPLEMENTATION_PLAN.md ·
│                                PORTING_REPORT.md · UI_SPEC.md
├── data/                     ← 五区全空（数据在 %APPDATA%，见 data.md）
├── misc/                     ← 空
├── src/                      ← 前端源码（notes/ + workbench/）
├── src-tauri/                ← Tauri 约定目录，不可搬
│   ├── src/  capabilities/  icons/  gen/  target/
│   └── Cargo.toml · Cargo.lock · tauri.conf.json · build.rs
├── config/                   ← 🔴 **源码 + 运行时默认值的来源**（见下）
│   ├── seasons.json · tools.json
│   └── assets/watermark/     （12 个月份的节气 SVG）
├── test/                     ← opSeq.test.ts（留根）
├── node_modules/ · dist/
├── index.html · notes.html   ← vite 双入口（工具约定位置）
├── package.json · package-lock.json · tsconfig.json · vite.config.ts
├── README.md · LICENSE · .gitignore
└── services.yaml
```

> **为什么 `src-tauri/` `config/` `index.html` `notes.html` `dist/` `node_modules/` 和那几个配置文件不按四区搬？**
> 它们是**构建工具的约定位置**，搬了要改配置 = 重构。
> 见 `project_manager/docs/PLAYBOOK.md` 第二节「例外四」。
>
> **`config/` 是个特例**：它**既在源码里，又被运行时复制出去当默认值**（debug 从 `current_dir()/config`
> 复制到 `appConfigDir()/config`；release 从打包 resource 复制）。
> **它按四区判定表会被判进 `src/`，但那样 debug 分支就找不到它了。**

---

## 七、文档索引

### 本次整理新建

| 文档 | 内容 |
| --- | --- |
| [README.md](README.md) | 本文件：定位、移植背景、形态、项目名分裂、架构、目录 · **第十节：输入槽与输出槽** |
| [run.md](run.md) | 安装 / 开发 / 构建 / 测试 / 端口 / **重编前的 `custom-protocol` 警告** / 已知限制 |
| [data.md](data.md) | **项目内零数据**；`%APPDATA%` 里的便签库（实测规模）；外部持久状态 |
| [deps.md](deps.md) | npm + cargo 依赖；**`custom-protocol` 的完整来龙去脉**；对外契约表 |
| [service.md](service.md) | **只读接口 17321** 的完整说明 |
| [OUTPUT-SLOT.md](OUTPUT-SLOT.md) | **输出槽规格**：两条路由的 SQL / 逐字段表 / 边界三条 / 缺口两条 / 三条硬约束对照 |
| [LESSONS.md](LESSONS.md) | **学到什么** —— 项目开发史（`DEVELOPMENT_LOG.md`）与整理站各提炼若干条，**每条附来源**；并列出**没学到的空白** |
| [log.md](log.md) | 2026-10-03 整理记录（第九站 + 第二十站 + 补充段） |

### 搬进来的移植档案（原文一字未改）

| 文档 | 是什么 |
| --- | --- |
| [history/README.md](history/README.md) | 🆕 **`history/` 的索引** —— 每份是什么、**按什么顺序读**、时间线、**以及覆盖缺口** |
| [history/DEVELOPMENT_LOG.md](history/DEVELOPMENT_LOG.md) | **便签插件 · 开发日志**（42,756 B）—— 最大的文档。含踩过的坑与**多条"与常见说法不符"的实测认知** |
| [history/IMPLEMENTATION_PLAN.md](history/IMPLEMENTATION_PLAN.md) | **落地执行方案**（含结论、计划与疑问） |
| [history/PORTING_REPORT.md](history/PORTING_REPORT.md) | **可行性 / 冲突 / 落地报告** |
| [history/UI_SPEC.md](history/UI_SPEC.md) | **界面与交互规格**（成型版） |
| [specs/123.txt](specs/123.txt) | **源头**：便签 UI 规格 **v12** 导出文档（35,671 B，原文保留原名） |
| [specs/456.txt](specs/456.txt) | **用户对移植报告的口径回复**（9,693 B，原文保留原名） |

> **`123.txt` / `456.txt` 是刻意保留的原名** —— 它们不是临时文件，
> 是 README 文档表里正式登记的条目，也是整条移植链的始末两端。
>
> **归档位置**（2026-10-03，第二十站 C 段）：**往来记录 `123.txt` / `456.txt` 在 `specs/`；
> 成文文档 `DEVELOPMENT_LOG.md` / `IMPLEMENTATION_PLAN.md` / `PORTING_REPORT.md` / `UI_SPEC.md` 在 `history/`。**

### 仍在原位的

| 文档 | 位置 | 为什么不动 |
| --- | --- | --- |
| `README.md` | 仓库根 | GitHub 从仓库根读它 |
| `LICENSE` | 仓库根 | 许可证惯例 |

### 2026-10-09 新增（第二十一站）

| 文档 | 内容 |
| --- | --- |
| [INPUT-SLOT.md](INPUT-SLOT.md) | 🆕 **输入槽：无** —— 按 §7 五项格式正式化；补登两条"非槽但能改变行为"的通道（配置目录监听、**编译期硬编码的默认项目根**）。**它填的正是 [OUTPUT-SLOT.md](OUTPUT-SLOT.md) `:355-356` 自己挂着的那个待议项。** |
| [runtime-log.md](runtime-log.md) | 🆕 **运行日志**（六件套里此前缺失的一份）—— `trace.log` 的路径/格式/写入点/怎么读/怎么关；并说明为什么 `eprintln!` 在 GUI 程序里等于写进黑洞 |

---

## 八、证据索引

> **本节是本文档全部事实性断言的证据**（格式 `文件:行号`，路径相对 `Note\`）。
> **门槛**：行号是 2026-10-03 第二十站实测的行号，**源码若改动，行号会漂**。
> **本文档写不下的、更细的证据见 [data.md](data.md) / [deps.md](deps.md) / [run.md](run.md) / [service.md](service.md) / [OUTPUT-SLOT.md](OUTPUT-SLOT.md) 各自的「证据索引」。**

### 8.1 技术栈与形态（第一、三节）

| 断言 | 证据 |
| --- | --- |
| 技术栈 Tauri 2 + React 18 + TS + Vite + SQLite | `package.json:13-27`（`react ^18.3.1` / `vite ^5.4.11` / `typescript ^5.6.3`）· `src-tauri/Cargo.toml:11`（`tauri = "2"`）· `Cargo.toml:16`（`rusqlite 0.31 features=["bundled"]`） |
| 工作台竖条 56×64 | `src-tauri/tauri.conf.json:18-19`（`"width": 56, "height": 64`） |
| 工作台窗口：无边框 / 透明 / 置顶 / 不 resize | `tauri.conf.json:20-23`（`decorations:false` · `transparent:true` · `alwaysOnTop:true` · `resizable:false`） |
| **便签**窗口：无边框 / 透明 / 置顶 / **不进任务栏** | `src-tauri/src/notes/window.rs:8-13`（`.decorations(false)` `.transparent(true)` `.always_on_top(true)` **`.skip_taskbar(true)`** `.resizable(false)` `.shadow(false)`） |
| ⚠️ `tauri.conf.json:24` 是 `"skipTaskbar": false` | **那是【工作台】窗口**（`label: "main"`，`tauri.conf.json:15`），**不是便签窗口**。**两者不同，别搞混** |
| 单实例 | `src-tauri/src/main.rs:9-17`（`tauri_plugin_single_instance::init`）· `Cargo.toml:12` |
| 数据是 SQLite，库在项目外 | `src-tauri/src/notes/mod.rs:15`（`app.path().app_data_dir()?.join("notes.db")`） |
| 仓库根没有任何 exe / dll | 实测：根下 20 项全为目录或文本文件（`Get-ChildItem -Force`） |
| 仅 Windows | `Cargo.toml:21-22`（`[target.'cfg(windows)'.dependencies] windows-sys`）· `notes/hittest.rs:5-8` |
| 许可证 MIT | `LICENSE`（1,060 B） |

### 8.2 项目名分裂（第四节）

| 断言 | 证据 |
| --- | --- |
| `Note` | 目录名 · `git remote -v` → `…/3431481073/Note.git` |
| `my-workbench-notes`（npm） | `package.json:2`（`"name": "my-workbench-notes"`） |
| `my-workbench-notes`（cargo） | `src-tauri/Cargo.toml:2`（`name = "my-workbench-notes"`） |
| `MyWorkbench` | `tauri.conf.json:3`（`"productName": "MyWorkbench"`） |
| `com.myworkbench.notes` | `tauri.conf.json:5`（`"identifier": "com.myworkbench.notes"`） |
| `便签` | 仓库根 `README.md:3`（`# 便签`） |

### 8.3 架构与关键设计（第五节）

| 断言 | 证据 |
| --- | --- |
| 两个 HTML 入口由 vite 显式声明 | `vite.config.ts:23-28`（`rollupOptions.input.main = "index.html"` / `.notes = "notes.html"`） |
| `index.html` → 工作台壳 | `index.html:10`（`src="/src/workbench/main.tsx"`） |
| `notes.html` → 便签本体 | `notes.html:10`（`src="/src/notes/main.tsx"`） |
| **命令数 = 26** | `src-tauri/src/main.rs:24-51` 逐行计数 = **26**（24 个 `notes::commands::*` + 2 个 `workbench::action::*`）。**原文写 26，正确** ✅<br>🔴 **2026-10-14 更新：27**（25 + 2，新增 `catcher_import_status`）。这一行的 `:24-51` 行号同时作废 |
| **`components/` = 14 个** | 实测 `src/notes/components/` 目录项 = **14**：BottomBar · Card · CardList · CloseButton · EmptyState · NodeContent · OutlineTree · Paper · ProjectPicker · SearchBar · Seg · TagManager · TagSelect · TrashConfirm。<br>⚠️ **原文曾写 16 —— E 段已改正为 14**（见第九节 9.1） |
| **各文件字节数（原文全部正确）** | 实测：`db.rs` **51,678 B** / 1,331 行 · `commands.rs` **8,848 B** / 310 行 · `http.rs` **7,008 B** / 203 行 · `config.rs` **4,103 B** / 135 行 · `summary.rs` **3,556 B** / 106 行 · `hittest.rs` **2,091 B** / 60 行 —— **与原文逐项一致** ✅ |
| `notesStore.ts` "26 KB" | 实测 **26,437 B / 862 行** ✅ |
| `notes.css` "37 KB" | 实测 **37,579 B / 1,953 行** ✅ |
| 点击穿透：透明区 `HTTRANSPARENT` | `notes/hittest.rs:31-45`（`WM_NCHITTEST` → `ScreenToClient` → 命中矩形比较 → `HTTRANSPARENT`） |
| 子类化安装 | `notes/hittest.rs:50-54`（`SetWindowSubclass(hwnd, Some(hit_test_proc), SUBCLASS_ID, 0)`） |
| 矩形由前端回传 | `src/notes/App.tsx:72-86`（`reportHitRects`，`devicePixelRatio` 换算）→ `notes/commands.rs:103-106`（`set_hit_rects` 命令） |
| 窗口固定 1300×648 | `notes/window.rs:5,7`（`WebviewUrl::App("notes.html")` 在 :5 · `inner_size(1300.0, 648.0)` 在 :7） |
| 居中到工作区、排除任务栏 | `notes/window.rs:18-26`（`primary_monitor().work_area()`） |
| 前端首屏后才 `show()` | `notes/window.rs:14`（`visible(false)`）· `src/notes/App.tsx:161-167`（`getCurrentWindow().show()`） |
| 配置热重载 | `notes/config.rs:97-125`（`notify::recommended_watcher` → `app.emit("notes://config-changed", ())`） |
| 12 色按月 + 12 个水印 SVG | 实测 `config/assets/watermark/` = **12 个 `.svg`** · `config/seasons.json`（922 B） |
| `now_ms()` CAS 单调递增 | `notes/summary.rs:43-61`（`AtomicI64` + `compare_exchange`） |

### 8.4 目录（第六节）

| 断言 | 证据 |
| --- | --- |
| `src-tauri/` 是 Tauri 约定目录 | `src-tauri/tauri.conf.json` · `Cargo.toml` · `build.rs` 都在该目录下（Tauri 2 约定） |
| `config/` 既在源码里又被复制出去 | `notes/config.rs:39-48`（debug：`current_dir()/config` → `appConfigDir()/config`）· `config.rs:50-57`（release：`Resource/config` → `appConfigDir()/config`） |
| `test/` 留根 | 实测只有一个文件 `test/opSeq.test.ts`（147 行） |
| 双入口在工具约定位置 | 同 8.3 前两条 |
| **`docs\specs\` = 2 文件** | 实测（C2 搬迁，SHA256 6/6 一致）：`specs\123.txt`（35,671 B / 496 行）· `specs\456.txt`（9,693 B / 259 行） |
| **`docs\history\` = 4 文件** | 实测：`history\DEVELOPMENT_LOG.md`（42,756 B / 591 行）· `IMPLEMENTATION_PLAN.md`（14,502 B / 187 行）· `PORTING_REPORT.md`（16,402 B / 230 行）· `UI_SPEC.md`（18,694 B / 294 行） |
| **`docs\` 根 = 8 文件** | 实测（2026-10-03 补充段后）：`README` · `run` · `data` · `deps` · `service` · `log` · `OUTPUT-SLOT.md` · *补充段新增* `LESSONS.md` |
| **`docs\history\` = 5 文件** | 实测：原 4 份 + *补充段新增* `README.md`（索引） |

> 📌 **这三行是「收尾段自检」补的** —— E 段改了第六节的目录树，**但没同步 8.4**；
> 本段自检发现后补齐。**（这正是"改了一处、忘了它的证据索引"的实例。）**

### 8.5 整理状态（文首）

| 断言 | 证据 |
| --- | --- |
| "代码零改动" | 第二十站 A–D 段：**未打开、未写入任何 `.rs` / `.ts` / `.tsx` / `.json`**（`git status` 里 Note 的源代码改动只有**上游本来就有的** `db.rs`/`mod.rs`/`http.rs`，与本站无关） |

### 8.6 产出清单（第十一节）

| 断言 | 证据 |
| --- | --- |
| `dist\` **7 文件 / 0.25 MB** | **实测**（`Get-ChildItem -Recurse`）：`index.html` 471 B · `notes.html` 470 B · `assets\event-uN3fBrrf.js` 144,604 B · `assets\main-BAzd9ioo.css` 784 B · `assets\main-BtsBY_Pk.js` 627 B · `assets\notes-D9mOxkb4.js` 85,526 B · `assets\notes-DAaNLt_e.css` 27,897 B |
| exe **10,526,720 B**（2026-10-01 19:20:26） | **实测**（`Get-Item`） |
| `.bak-20261001` **10,493,952 B** | **实测**（同上） |
| 库三件套 **417,792 / 4,144,752 / 32,768 B** | **实测**（`Get-Item`，`%APPDATA%\com.myworkbench.notes\`） |
| **152 条便签**（活跃 134 / 回收站 18） | **实测**（只读查 `notes.db`：`SELECT COUNT(*) FROM pages` 等） |
| `.gitignore` 挡住 `dist` / `src-tauri/target` | `.gitignore:2-3` |
| `.bak-20261001` 的用途**不明** | `Note\docs\log.md` 第九站「未解决 7」（原文） |

---

## 九、未验证与订正

> 本节记录**本文档里没有独立证据的断言**，以及**本轮实测发现的错误**。
> **订正过的错误保留在此，不删除** —— 否则下次还会有人踩。

### 9.1 🔴 实测发现的错（1 处）—— **已于 E 段修正**

| 位置 | 原文 | 实测 | 处置 |
| --- | --- | --- | --- |
| 第五节架构图 | `components/(16 个)` | **14 个**（逐个列出见 8.3） | ✅ **E 段已改为 `components/(14 个)`**<br>**依据**：实测 `src/notes/components/` 目录项 = 14（BottomBar · Card · CardList · CloseButton · EmptyState · NodeContent · OutlineTree · Paper · ProjectPicker · SearchBar · Seg · TagManager · TagSelect · TrashConfirm） |

> **E 段拍板（2026-10-03）**：「**已知错留在文档里，比没写更坏**」——
> 所以 D 段"只登记不改"的口径在 E 段被**推翻**，本轮直接改掉。

### 9.2 ⚠️ 本文档里「原文如此、本轮未独立验证」的断言

| 断言 | 出处 | 状态 |
| --- | --- | --- |
| 「首次 `tauri dev` 会编译约 **462 个 crate**（约 2 分钟）」 | 仓库根 `README.md:40` | ⚠️ **未验证** —— 本轮没跑过构建 |
| 「透明窗口叠加 transform 动画在 Windows 上**仍有合成器伪影，未闭环**」 | 仓库根 `README.md:73` → `history/DEVELOPMENT_LOG.md` §19.8 | ⚠️ **未验证** —— 是作者的主观观察 |
| 「状态：**保留项目，用户说"非常重要"**」 | **用户口述** | ⚠️ **无文件证据** —— 本轮简报里也没有再确认 |
| 「`123.txt` 基准代码是『单文件 index.html（HTML+CSS+原生 JS，无构建依赖）』」 | `specs/123.txt` 原文 | ⚠️ **未逐字核** —— 未读完 35,671 B 全文 |
| 「`456.txt` 的总原则（三句）」 | `specs/456.txt` 原文 | ⚠️ **未逐字核** —— 未读完 9,693 B 全文 |
| 「它在别处 1:1 复现这套界面的外观+交互+数据呈现」 | `specs/123.txt` 原文引用 | ⚠️ **同上** |
| 「**MIT** 许可证」 | `LICENSE`（1,060 B） | ✅ **已核**（文件存在；未逐字读条款） |

### 9.3 ⚠️ C2 搬迁后本文档里过期了的位置描述 —— **已于 E 段修正**

| 位置 | 原状 | 处置 |
| --- | --- | --- |
| 第六节目录树 | **未列出 `docs/specs/` 与 `docs/history/`** | ✅ **E 段已补上两个子目录及其内容** |
| 第六节第一行 `├── docs/  ← 本目录（6 份移植档案 + 六件套）` | 那 6 份**已不在 `docs/` 根** | ✅ **E 段已改成 `六件套 + OUTPUT-SLOT.md + 两个归档子目录`** |

**依据**：C2 段实测搬迁结果（6 份逐个 `[System.IO.File]::Move`，SHA256 6/6 一致）——
`123.txt` / `456.txt` 在 `specs/`，其余 4 份在 `history/`。

---

## 十、输入槽与输出槽

> **本节是 2026-10-03 补充段新增。** 放在文末是**刻意的** ——
> 插在中间会把「第八节 / 第九节」的编号往后推，
> 而第八节里的证据索引小节（`8.1`–`8.5`）是按「第八节」命名的，**改号会把它们全部孤立**。
> **这是位置上的妥协，不是内容上的次要。**

### 10.1 输出槽 ✅ 有，且已声明

| 项 | 值 |
| --- | --- |
| **位置** | `http://127.0.0.1:17321`（只绑回环） |
| **完整规格** | **[OUTPUT-SLOT.md](OUTPUT-SLOT.md)** |
| **消费者** | **`github_operater` 的 `src\watch_notes.py`**（每 3 秒轮询，两条路由都用） |

### 10.2 输入槽 —— 🔴 **2026-10-14 起：有一条**（`_data` 的 `texts`）

> 🔴 **本节已改。** 原文（2026-10-03）写的是「**无对外输入槽**」——
> **在它自己的口径下那句话仍然对**：**没有任何程序【往】Note 里推东西**，写入口**只有 UI 一个**。
> **但"这个项目没有任何输入"已经不成立** —— 2026-10-14 起它**会去读别人的池子**。
> 完整声明见 **[INPUT-SLOT.md](INPUT-SLOT.md) §0.D**（那张表已从 1 行变 2 行）。

| 项 | 值 |
| --- | --- |
| **读什么** | `D:\MyProject\_data\index.db` 的 `texts` 表，`WHERE source='catcher' AND deleted=0` |
| **拿来干嘛** | 落成**普通卡片**（`kind='note'`）；按上游 `src_created_at` 排；标签镜像原名 + 自动挂「手机」 |
| **怎么读** | **只读**打开（`mode=ro`）；**后台线程**拉一次，不在启动路径上；认 `text_id` 去重（账本表 `imported_texts`） |
| **读不到会怎样** | 只写 `catcher_import_status` + `trace.log`，**Note 照常起** |
| **路径在哪** | `config\sources.json` 的 `catcher.index_db` —— **代码里没有 `_data` 这个路径** |
| **取证** | 命令 `catcher_import_status` · `%APPDATA%\com.myworkbench.notes\trace.log` 的 `catcher_import:` 行 |

**下面 10.3 的逐条清点不受影响** —— 它清点的是「**看起来像输入的东西**」，那些**确实都不是输入**；
**真正新增的那一条（`_data` 的库）本来就不在那张表里**，所以它不需要改。

---

> ⚠️ **以下为 2026-10-03 原文**，口径是「**有没有人主动投递数据给我**」。
> 在这个口径下它**仍然正确**（写入口确实只有 UI 一个），故原样保留。

**这个项目不接受任何来自程序或外部系统的数据输入。**
**它所有的内容都由人在界面上敲。** 按 `INTERFACES.md` 第二章的定义，**它没有输入槽。**

**为什么没有**：**它的业务模型就是"人 → 便签"。**
唯一的写入方是它自己的 UI；**17321 明确只读**（`http.rs:5`：「没有任何写路由」）。

### 10.3 逐条清点"看起来像输入"的东西 —— 都不是槽

| 声称的"输入" | 实际是什么 | 是槽吗 |
| --- | --- | --- |
| **便签正文 / 标题** | **人在 `Paper.tsx` 的 textarea 里敲** | ❌ **不是** —— 不经程序，是人的操作 |
| **标签 / 项目绑定** | **人在 UI 上选**；项目目录由 `pick_project_dir` 弹系统对话框选 | ❌ 不是 |
| **`config\seasons.json`**（922 B） | **项目自带的资源**，随包发布；运行时被复制到 `%APPDATA%\...\config\` | ❌ **不是** —— 它属于项目自己，不是外面喂进来的 |
| **`config\tools.json`**（171 B） | 同上 —— 是本项目向工作台的**自声明** | ❌ 不是 |
| **`config\assets\watermark\*.svg`**（12 个） | 同上 —— 项目自带素材 | ❌ 不是 |
| **搜索词 / 筛选条件** | 人在 UI 上敲 | ❌ 不是 |

### 10.4 ⚠️ 但有一个**事实上的、未声明的**外部输入口

| 项 | 内容 |
| --- | --- |
| **路径** | **`%APPDATA%\com.myworkbench.notes\config\seasons.json`** |
| **为什么算** | release 版启动时，`config.rs:132-135` 让配置监听器**盯着这个目录**，文件一变就广播 `notes://config-changed`（`config.rs:118-121`） |
| **谁能写** | **同机任何进程。** 写进去 = **改主题色与当季水印** |
| **严重性** | 🟡 **低** —— 只能改颜色和一张 SVG，**改不了便签数据**（数据口是只读的） |
| **为什么记它** | 🔴 **因为它是"程序外能改变这个应用行为"的唯一一条路**，而**它一个字都没有被声明过** |
| 🔴 **它是不是槽** | **按定义不是**（不是"项目接受输入的入口"，是一条**副作用通道**）。**但它应当被登记** —— 就像 `data.md` 登记了"工作台 `tools.json` 写死本 exe 路径"一样 |

**证据**：

```rust
// config.rs:118-121
if relevant {
    let _ = app.emit("notes://config-changed", ());
}

// config.rs:132-135（release 分支）
#[cfg(not(debug_assertions))]
fn watch_source_dir(app: &AppHandle) -> Option<PathBuf> {
    config_dir(app).ok()
}
```

> ⚠️ **debug 分支盯的是项目内的 `config\`**（`config.rs:127-130`，`current_dir()/config`）——
> **只有 release 才盯 `%APPDATA%`。** 运行中那个 exe 是 release（`target\release\`，PID 29608）。

### 10.5 一句话

> 🔴 **2026-10-14 更正**：原文写「**只有输出、没有输入**」。
> **"没有输出"不变。"没有输入"从 10.2 起不成立** —— 它现在会**只读**拉一条外部文本流（`_data` 的 `texts`）。
>
> **仍然成立的那半句（而且更要紧）**：**没有任何程序【往】Note 里写。**
> 它所有由人产生的内容**都来自人的手指**；🆕 唯一的例外是**导入进来的那些卡片**——
> 它们的正文来自手机 catcher，**不是人在这台机器上敲的**。

> ~~**`Note` 是一个【只有输出、没有输入】的项目。**~~
> ~~**它的每一字节数据都来自人的手指 —— 没有程序能喂它东西，也没有程序能改它。**~~
> ⚠️ **上面两行是 2026-10-03 原文，已不成立，划掉保留**（按 [LESSONS.md](LESSONS.md) #23 不改历史记录）。

---

## 十一、产出清单

> **本节是 2026-10-03 补充段新增**（服务 `PARADIGM.md` 里「七样说明」的**第 4 样：产生哪些结果**）。
> **此前这四样产出分别写在三份文档里，谁也不汇总。**

### 11.1 四样产出

| # | 产出 | 落在哪 | 实测 | 详细讲在 |
| --- | --- | --- | --- | --- |
| **1** | **构建产物**（前端资产） | `Note\dist\` | **7 文件 / 0.25 MB** | [run.md](run.md) 第三节 |
| **2** | **交付物**（exe） | `Note\src-tauri\target\release\my-workbench-notes.exe` | **10,526,720 B**（构建于 2026-10-01 19:20:26） | [run.md](run.md) 第三节 · [data.md](data.md) 第四节 |
| **3** | 🔴 **数据**（便签库） | **`%APPDATA%\com.myworkbench.notes\`** —— **不在项目里** | `notes.db` **417,792 B** + `-wal` **4,144,752 B** + `-shm` **32,768 B** | **[data.md](data.md)**（整份） |
| **4** | 🔴 **对外结果**（输出槽） | **`127.0.0.1:17321`** —— **两条只读路由** | **152 条便签**（活跃 134 / 回收站 18） | **[OUTPUT-SLOT.md](OUTPUT-SLOT.md)**（整份） |

### 11.2 三句必须一起说的话

1. **第 1、2 项在项目里**，但**都被 `.gitignore` 挡住了**（`.gitignore:2-3`：`dist` / `src-tauri/target`）——
   **它们不进版本控制，但它们是产出。**
2. 🔴 **第 3 项完全不在项目里** —— 它是这个项目**唯一的持久数据**，
   **而项目自己不持有它的任何备份**（见 [data.md](data.md) 第二节）。
3. 🔴 **第 4 项不是一个文件，是一个【活的窗口】** —— 它没有对应任何磁盘上的东西，
   **它是进程内嵌的、实时读同一个 SQLite 的只读 HTTP**（见 [OUTPUT-SLOT.md](OUTPUT-SLOT.md) 第一章）。

### 11.3 还有一样**不算产出**但容易被误认

| 东西 | 为什么不算 |
| --- | --- |
| **`src-tauri\target\release\my-workbench-notes.exe.bak-20261001`**（10,493,952 B） | **它是手动留档**，不是构建产物。**用途不明**（第九站的未解决 7 至今没答案） |
| 🆕 **`%APPDATA%\com.myworkbench.notes\trace.log`** | **它是运行日志，不是产出**（删了会重建、无用户内容）。**但它写在项目外**，所以列在这里 —— 🔴 **"删项目前必须知道"的清单现在多这一项。** 见 [runtime-log.md](runtime-log.md) |

### 11.4 一句话

> **这个项目产出四样东西：一个前端资产包、一个 exe、一个不在项目里的数据库、一个活的只读接口。**
>
> 🔴 **而它们四个，第一次被并列写在同一张表上，就是这一节。**
