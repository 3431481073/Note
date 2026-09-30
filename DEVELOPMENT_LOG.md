# 便签插件 · 开发日志

> 本文档在开发过程中自主维护，记录项目背景、开发过程、踩过的坑、解决方式与沉淀下来的认知。
> 持续更新，不等提醒。

---

## 1. 项目概述

把「便签」作为一个小插件接入「工作台」。工作台是一个 Tauri 2 桌面壳，主窗口是一个竖长条工具条（56×64、无边框、透明、置顶、贴右缘垂直居中）；便签是第二个独立窗口（740×648、无边框、透明、置顶、不进任务栏）。

技术栈：

| 层 | 技术 |
| --- | --- |
| 桌面壳 | Tauri 2（Rust 后端 + Webview 前端） |
| 前端 | React 18 + TypeScript + Vite 5（多页：`index.html` 工作台 / `notes.html` 便签） |
| 虚拟列表 | `@tanstack/react-virtual` |
| 状态 | `useSyncExternalStore`（零依赖自研 store） |
| 存储 | SQLite（`rusqlite` bundled，WAL，游标分页 `LIMIT 51`） |
| 事件 | Tauri 2 `emit` / `listen` |
| 测试 | 前端 vitest；后端 Rust 单元测试 |

---

## 2. 环境准备与坑

### 2.1 rustc 缺失（工具链损坏）

**现象**：`cargo` 能跑但 `rustc` 报
```
missing manifest in toolchain 'stable-x86_64-pc-windows-msvc'
```
工具链 bin 目录里只有 `cargo.exe`，缺 `rustc.exe`。

**解决**：`rustup default stable` 再 `rustup update stable`，修复后 rustc / cargo 均回到 1.98.1。

**认知**：`rustup` 维护的工具链目录可能因为部分文件被删/中断而「半残」，此时 `cargo` 可用但真正编译时才暴露。先修工具链，再谈编译。

### 2.2 依赖必须装在项目内，不污染全局环境

用户明确要求：项目依赖全部装在项目目录下，不要污染全局。落实为：

- npm 依赖只用**本地安装**（`npm install`，不用 `-g`），产物落在项目 `node_modules/`。
- cargo 依赖/构建产物落在 `src-tauri/target/`（标准项目内 target，无全局污染）。

### 2.3 首次 cargo 编译时间极长

Tauri 2 依赖树很大（wry / webview2-com / tao / ico / sqlite bundled 等），首次 `cargo check` 要下载 462 个 crate 并编译约 2 分钟。属正常，二次编译快。

---

## 3. 编译期踩坑与修复

### 3.1 TS6310：`tsconfig.node.json` 不允许 `noEmit`

**现象**：`tsc --noEmit` 报
```
Referenced project 'tsconfig.node.json' may not disable emit.
```
**根因**：`tsconfig.json` 里 `references` 引了 `tsconfig.node.json`，而后者同时开了 `composite: true` 与 `noEmit: true`——composite 项目不允许 noEmit。

**解决**：`vite.config.ts` 本就由 Vite 自己用 esbuild 处理，无需 tsc 检查。直接删掉 `tsconfig.node.json`、去掉 `references`，让 `tsc --noEmit` 只检查 `src`。**认知**：不要盲目照搬脚手架的多 tsconfig 结构；先想清楚「谁负责类型检查谁」。

### 3.2 Tauri `assetProtocol` 需要 `protocol-asset` feature

**现象**：`cargo check` 失败
```
The `tauri` dependency features ... does not match the allowlist defined under `tauri.conf.json`.
... add the `protocol-asset` feature.
```
**根因**：`tauri.conf.json` 里开了 `app.security.assetProtocol`（用于加载季节水印 SVG），但 `Cargo.toml` 的 tauri 依赖 `features = []`。

**解决**：`tauri = { version = "2", features = ["protocol-asset"] }`。**认知**：Tauri 2 把不少能力做成 feature 开关，配置里用了但 feature 没开会在 build script 阶段被拦下，报错信息直接点名了要加的 feature。

### 3.3 Rust 类型错误三连

首轮 `cargo check` 报了 6 个错误，集中在窗口定位/句柄传递：

1. `app.handle()` 返回 `&AppHandle`，而函数要 `AppHandle` —— `.clone()`。
2. `app.get_webview_window("main")` 找不到方法 —— `get_webview_window` 来自 `tauri::Manager` trait，需 `use tauri::Manager;`。
3. `area.size.width`（`u32`）与 `i32` 混算，`saturating_sub` 参数类型不匹配 —— 统一先做 u32 的 `saturating_sub`，再 `as i32` 后与 `position.x`（i32）相加。

**认知**：Tauri 的几何类型里 `PhysicalSize`/`LogicalSize` 是 `u32`/`f64`，而坐标 `position` 是 `i32`，混算很容易踩；先把同类型算完再强转，别在表达式里到处 `as`。

---

## 4. 测试期踩坑与修复（最有价值的坑）

### 4.1 时间戳毫秒分辨率导致「新建置顶」与排序不确定

**现象**：Rust 单测 `list_active_sorted_by_created_desc` 失败——连续 `create()` 的三条记录 `created_at` 相同（同一毫秒内），`ORDER BY created_at DESC, id DESC` 退化成按随机 nanoid 排序，新笔记不一定置顶。

**根因**：`now_ms()` 直接读系统时钟，毫秒分辨率下快速连续调用会拿到相同值。

**解决**：把 `now_ms()` 改成**进程内严格单调递增**——用一个 `AtomicI64` 存上次返回值，CAS 保证每次调用 `max(now, last + 1)`，使 `created_at` / `deleted_at` 在同一进程内天然唯一且递增。

**认知**：凡是「按时间排序 + 时间做游标分页」的地方，时间戳必须单调且（尽量）唯一，否则并列时刻的排序与分页边界会出随机错。这里顺带治好了游标分页在并列时间戳下的潜在越界/重复风险。

### 4.2 单测断言本身写错（version 计数）

**现象**：`soft_delete_restore_purge_cycle` 断言 `purge` 返回 `restored.version + 2` 失败（实际 `+1`）。

**追查**：version 全程是「每次写操作 +1」：create=1 → update=2 → soft_delete=3 → restore=4 → 再 soft_delete=5，`purge` 返回当前 version=5，即 `restored.version + 1`。**是测试断言错，不是实现错。**

**认知**：测试失败先分清是「实现错」还是「断言错」——用笔把 version/状态机从头推一遍，别急着改实现。

### 4.3 游标分页测试用不足一页的数据，误报失败

**现象**：`cursor_pagination_no_overlap` 断言 `ids.len()==3` 却得到 6。

**根因**：`PAGE_LIMIT = 51`，只插 3 条时 `next_cursor` 为 `None`，第二页等于又把第一页全量返回一遍（重复 3 条）。

**解决**：改插 60 条（> 51），用循环沿 `next_cursor` 走到头，再断言 60 条无重复。

**认知**：分页测试必须用**超过一页**的数据才能真正触发分页路径；数据量不足时 `next_cursor == None`，测的其实是「重复查询第一页」而不是分页。

---

## 5. 架构决策与设计要点

- **事件名走 Tauri 事件总线**：后端写操作后 `emit` `notes://changed` / `notes://stats` / `notes://config-changed`，前端 `listen` 做增量更新；工作台通过 `notes://stats` 同步角标。单窗口场景下用「乐观更新 + 事件回放 + 幂等合并」收敛状态。
- **幂等合并（§8.4 规则表）**：`mergeOps` 把 create/update/delete/restore/purge 的折叠规则落到纯函数里（create+delete=净零、delete+restore=净零、create+update=create…），前端 `applyChangedPayload` 幂等，避免「乐观更新 + 后端事件回显」造成重复插入。
- **游标分页**：`(created_at, id)` 双列游标，`LIMIT 51` 多取一条判断 `has_more`，避免 `OFFSET` 的深分页性能问题；LIKE 用 `ESCAPE '\'` 且前端 `escapeLike` 转义 `\ % _`。
- **主题**：`--ink-current` 按月切换 12 种墨色，来自 `config/seasons.json`；季节水印 SVG 通过 `assetProtocol` + `convertFileSrc` 注入，`?t=Date.now()` 防缓存。
- **光带特效**：CSS `@property --angle` + `conic-gradient` + `mask-composite: exclude` 实现旋转流光，`prefers-reduced-motion` 降级。

---

## 6. 验证结果（最终）

| 命令 | 结果 |
| --- | --- |
| `npm run build`（tsc --noEmit + vite build） | ✅ 通过，产物 `dist/` |
| `npm test`（vitest） | ✅ 17/17 |
| `cargo check` | ✅ 0 warning |
| `cargo test`（Rust） | ✅ 12/12 |
| `cargo build`（debug，验证 MSVC 链接） | ✅ 链接成功 |

---

## 7. 沉淀的通用认知

1. **先修工具链再谈编译**——rustup 半残时 `cargo` 可用但编译才炸，环境问题要前置排查。
2. **脚手架配置别照抄**——`tsconfig` 引用、`vite.config` 谁来检查要想清楚，删掉不需要的反而更稳。
3. **能力开关要在 feature 层对上**——Tauri 2 配置里开的 `assetProtocol` 等能力，`Cargo.toml` 不配 feature 会 build script 拦截。
4. **时间戳排序必须单调唯一**——并列时间戳会让「置顶」和游标分页随机出错，单调时钟是这类系统的基础件。
5. **测试失败先分责任**——实现错 vs 断言错 vs 测试数据不足，用纸面推导状态机，别盲改实现。

---

## 8. 注册进真实工作台（2026-09-10 追加）

### 8.1 关键认知：真实工作台是另一个独立项目

`D:\MyProject\Note` 的上层目录里存在一个**真实的、已存在的** `D:\MyProject\Workbench` 项目，架构与我早期在 Note 内部搭的「工作台桩」完全不同：

- 真实工作台是**配置驱动的命令启动器**：所有工具都写在 `config/tools.json` 里（JSON 数组），每个工具一个 `{id, emoji, svg_file, label, action}` 对象。
- 点击工具 = 用 `cmd /C` 执行 `action`（Windows shell 命令），`Command::spawn` 非阻塞。
- 图标放 `config/icons/`，`svg_file` 指向相对路径，加载时后端读文件内容注入前端渲染。

**结论**：把便签「注册进工作台」= 往 `tools.json` 加一条工具 + 提供图标 + 让 `action` 能启动便签 exe。工作台没有插件 IPC 机制，注册的本质就是「命令启动」。

### 8.2 注册做了什么

1. `Workbench/config/icons/notes.svg` —— 便签图标（`currentColor` 单色线条，Lucide「file-text」风格，与工作台白色线条一致）。
2. `Workbench/config/tools.json` —— 追加 `notes` 条目：`label: 便签`、`emoji: 📝`、`svg_file: icons/notes.svg`、`action: start "" "D:\MyProject\Note\src-tauri\target\release\my-workbench-notes.exe"`。
3. Note 侧改两处，让「被工作台启动」=「直接弹便签窗口」：
   - `tauri.conf.json`：主按钮窗口 `visible: false`（否则会和工作台自己的右缘工具条重叠，出现两个悬浮条）。
   - `notes/mod.rs`：`setup` 里创建便签窗口后直接 `show()` + `set_focus()`。

### 8.3 踩坑 / 注意点

- **`action` 里不能用嵌套双引号**（本次「黑框报找不到文件夹」的根因）：工作台用 Rust `Command::args(["/C", action])` 执行，Rust 会把内部 `"` 转义成 `\"`，而 `cmd.exe` 不认这种转义（cmd 用的是 `""` 规则），于是 `start` 参数被拆乱、把 exe 路径误当成文件夹报「找不到文件夹」。**路径无空格时直接写 `start D:\xx\exe`（不加任何引号）**；含空格才需要 `start "" "路径"`，但那样又踩回嵌套引号的坑，最稳是把工具放到无空格目录。
- **`cmd /C exe`（不加 `start`）会留一个等待子进程的 cmd**：`start` 让目标程序独立进程运行、cmd 立即退出，更干净。
- **release exe 才是自包含的**：`cargo build`（debug）带 `cfg(dev)`，会去连 vite dev server（`http://localhost:1420`）；只有 `cargo build --release`（或 `tauri build`）才把 `dist` 资产内嵌进二进制，能被工作台直接启动。

- **⚠️ 决定 dev/prod 的其实是 `custom-protocol` feature，不是 debug/release**（本次「localhost 拒绝访问」的根因）：Tauri 2 的 build.rs 里写死 `let dev = !custom_protocol;`，而 `custom-protocol` **不在默认 feature 里**。我只开了 `protocol-asset`（为水印 asset 协议），没开 `custom-protocol`，于是**即使是 `--release` 也走 dev 模式**、去连 `http://localhost:1420`。**修复**：`tauri = { version = "2", features = ["protocol-asset", "custom-protocol"] }`。验证方法：exe 里能搜到 vite 的 hash chunk 文件名（如 `notes-CtBn0kik.js`）= 已内嵌资产；而 `localhost:1420` 字符串只是配置里的 `devUrl` 残留、不再被使用。
- **多实例隐患（待处理）**：工作台每次点击都 `spawn` 一个新进程，而便签窗口的关闭按钮是「隐藏」而非退出，重复点击会起第二个便签进程、争用同一个 `notes.db`。后续若要根治，需引入单实例插件或让关闭按钮真正退出。

### 8.4 待办 / 可优化

- [ ] 单实例：工作台重复点击只聚焦已有便签，不再新起进程。
- [ ] 便签 exe 绝对路径写死在 `tools.json` 里，文件夹移动后需同步改。

---

## 9. 界面与逻辑打磨（2026-09-10 追加）

用户提出一批界面/逻辑问题，逐一修复并沉淀根因：

### 9.1 窗口边框 / 可调节大小 / 居中浮现

- **浅灰边框 + 可调节大小的框**：`create_notes_window` 之前没关 `resizable`，Windows 给无边框窗口画了可拖拽的 resize 边框，就是用户看到的「浅灰框」。加 `.resizable(false)` 关闭；`.shadow(false)` 去掉无边框透明窗口自带的阴影边缘。
- **居中**：加 `.center()`，窗口从屏幕中央出现。
- **「先弹一个莫名其妙的框子」**：两层原因——(a) 工作台 `cmd /C start` 会闪一个 cmd 黑框，在 `runner.rs` 加 `CREATE_NO_WINDOW` 隐藏；(b) Rust `setup` 里直接 `show()`，webview 未加载完就露出空白页面。改为**前端首屏渲染完成后自行 `show()`**，Rust 只创建隐藏窗口。
- **「从中间浮现」**：`.panel` 初始 `opacity:0` + `scale(0.92) translateY(12px)`，窗口显示后加 `.shown` 过渡到正常，配合 `transition` 实现干净的浮现动画。

### 9.2 恢复跳回便签页

`restore()` 里原本在恢复「当前预览项」时 `setState({ currentPage: page, view: "active" })`，把视图强制切回便签页。改为：恢复后**留在回收站视图**，若恢复的是当前预览项则切到下一个回收站项，否则清空预览。

### 9.3 「疯狂创造」多个便签 + 多聚焦

- **连点/按键重复触发**：`＋` 按钮点击后保留焦点，按住回车会反复触发 `click` → 连建。给 `createNewPage()` 加 `creating` 重入守卫，并在按钮 `onClick` 里 `e.currentTarget.blur()`；新建后把焦点还给纸面 `textarea`。
- **重复插入导致多卡片聚焦/卡顿（关键根因）**：`createNewPage()` 乐观 `insertSorted` 时没做幂等，若后端 `notes://changed`(create) 事件**先于**本地 setState 到达，同一条 meta 会被插入两次 → 列表出现同 id 两条 → 两张卡片同时高亮/流光 → 更卡。加 `active.some(m => m.id === page.id)` 幂等判断。**这也解释了「每次打开少几条」：重复条目在重开时随真实数据消失，看着像被删了。**

### 9.4 滚动卡顿

- 流光特效用 `@property --angle` + `conic-gradient`，跑在**主线程**上，滚动时和滚动抢帧。三招：
  1. 去掉伪元素上的 `filter: drop-shadow(...)` 与 `will-change: --angle`（filter 每帧重算最贵；自定义属性动画无法被 GPU 加速，will-change 无益），静态辉光改用 `.card.active` 的 `box-shadow`。
  2. 滚动时给滚动容器打 `data-scrolling`，CSS `animation-play-state: paused` 暂停流光。
  3. 根治重复插入（见 9.3）避免多张卡片同时流光。

### 9.5 底部按钮区

- 去掉 `bb-spacer`（`flex:1` 把 `＋` 顶到最左、垃圾桶顶到最右），改为 `justify-content: center` + `gap`，两个按钮居中。
- `.bb-btn` 加底色/描边/阴影（与搜索框一致），去掉原来透明背景。

### 9.6 其他

- `db.rs` 打开连接时加 `busy_timeout(5s)`，缓解多实例争用 `notes.db` 时的 `database is locked`。

---

## 10. 界面再打磨 + 单实例（2026-09-10 追加）

### 10.1 工作台「无法访问此页面」回归（重要教训）

上一轮用裸 `cargo build --release` 重编工作台，而它 `Cargo.toml` 里 `tauri` 是 `features = []`。Tauri 2 用 `custom-protocol` feature 决定 dev/prod，没开就按 dev 连 `localhost:1420`。修复：工作台也补上 `features = ["custom-protocol"]`，重编后验证 exe 内嵌了 `index-*.js`。

**认知**：重编任何 Tauri 2 项目前，先确认 tauri 依赖带了 `custom-protocol`；否则哪怕 `--release` 也走 dev 模式。工作台当初能用是 `tauri build` 编的，裸 `cargo build --release` 会把它打回 dev。

### 10.2 隐藏滚动条

`.card-list-scroll` 加 `scrollbar-width: none` + `-ms-overflow-style: none` + `::-webkit-scrollbar { display: none }`，三端都藏。

### 10.3 底部按钮区 80/20 + 回收站无加号

- `.bottom-bar` 去掉居中，`＋` 占 `flex:4`、垃圾桶 `flex:1`（约 80%/20%），与搜索框同宽。
- 回收站视图不渲染 `＋`（`{!isTrash && ...}`），只剩「返回便签」按钮。

### 10.4 单实例：只开一个窗口

接入 `tauri-plugin-single-instance`，`main.rs`：

```rust
.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
    if let Some(win) = app.get_webview_window("notes") {
        let _ = win.show(); let _ = win.unminimize(); let _ = win.set_focus();
    }
}))
```

第二次启动时插件检测到已有实例，回调里把已有便签窗口显示到最前、本进程随后退出——实现「重复点击只置顶已有窗口」。

### 10.5 主题配色清爽化

暖黄纸面换成冷白配色：纸/卡纯白、描边冷灰、强调色换薄荷青 `#2dd4bf`，流光 rgba 全部同步改色；`--ink-current` 默认改 `#64748b`；`seasons.json` 12 个月墨色换成冷色系（去掉 8/9/10/11 月的橄榄/金/棕/驼色）。

**注**：`config_dir` 走 `appConfigDir()/config`，release 下首次启动从 resource 复制；当前 `%APPDATA%\com.myworkbench.notes\config` 是空目录（无 seasons.json），`applySeason` 静默失败、回落到 CSS 默认 `--ink-current`、水印不显示——正好符合「简约干净」，配色由 CSS 决定、改 CSS 即生效。

---

## 11. 空白便签累积 + Tab 键改选择（2026-09-10 追加）

### 11.1 空白便签删不掉、清空回收站后冒两个空白（根因：三处自动新建）

**现象**：主界面删到只剩一个空白便签时它「怎么都删不掉」；回到回收站把垃圾清空后，自动跳回便签页并**多出两个空白便签**。

**根因**：三处 `createNewPage()` 自动新建空白便签：

1. `softDelete()`——删掉「当前预览且 active 已空」的便签时 `else await this.createNewPage()`，于是「删掉最后一个又自动补一个空白」→ 表现为「删不掉」。
2. `purge()`——清空回收站最后一项时 `else await this.createNewPage()`，而 `createNewPage()` 内部会 `view: "active"`，所以**既跳回便签页又新建空白**。
3. `init()`——启动时 active 为空则自动新建一个。

前两处叠加：删掉最后一个便签补一个空白(A)，进回收站把 A 清掉时又补一个空白(B)，回到便签页就剩 A+B 两个空白。

**修复**：删掉 `softDelete` 与 `purge` 里的自动新建，改为清空预览即可（`else this.setState({ currentPageId: null, currentPage: null })`）；`init()` 保留（首次启动没便签时留一个空白供直接输入，符合用户「留一个也没关系」）。这样：

- 使用中可以把便签**删到 0 条**，空态由 `EmptyState`（「还没有便签，点 ＋ 新建一条」）承接，不再冒出空白；
- 清空回收站后**留在回收站空态、不跳回便签页、不新建**；
- 只有冷启动时若一条都没有，才由 `init()` 新建一个空白，不会累积成两个。

### 11.2 Tab 键会「切换选择」

**现象**：用户强调「选择变化应该只能靠鼠标点」，但按 Tab 键会把选择/焦点切到卡片上的删除/恢复/彻底删除按钮，再按回车会误删。

**根因**：卡片里的动作按钮（`×`/`↩`/`🗑`）默认可被 Tab 聚焦，聚焦后回车触发按钮 `click`——虽带 `stopPropagation` 不触发选卡，但会触发删除/恢复/彻底删除，连带 `softDelete` 切到下一个便签，观感就是「按 Tab 把选择改了」。

**修复**：给三个卡片动作按钮加 `tabIndex={-1}`，从键盘焦点链里移除，只保留鼠标点击。卡片本体（`div`）本不可聚焦，其余可 Tab 的只剩搜索框、底部 `＋`/🗑、关闭按钮这些动作项，不再落到「删/恢复」这类会改选择的按钮上。

---

## 12. v12 落地 · 阶段 1 数据迁移（2026-09-17 追加）

> 依据 `123.txt`（v12 改进版原型）+ `456.txt`（最终口径）+ `IMPLEMENTATION_PLAN.md`（执行方案）推进。本阶段只动后端、零前端改动。

### 12.1 数据模型：扁平 page → 支持 title/kind/tags/projects/树

`pages` 表新增 6 列（`schema_migrations` 版本化迁移，v2）：

```sql
ALTER TABLE pages ADD COLUMN title      text not null default '';
ALTER TABLE pages ADD COLUMN kind       text not null default 'note';
ALTER TABLE pages ADD COLUMN tags       text not null default '[]';  -- JSON 数组
ALTER TABLE pages ADD COLUMN projects   text not null default '[]';  -- JSON 数组
ALTER TABLE pages ADD COLUMN from_view  text;
ALTER TABLE pages ADD COLUMN nodes_json text;                        -- outline 树整体 JSON
```

- **迁移机制**：`migrate()` 改为「基础 v1 结构（`IF NOT EXISTS` 幂等）→ 查 `schema_migrations` 当前版本 → `<2` 时在一个事务里 `ALTER` 加列 + 写入版本 2」。老库无损、全新库同一条路径、可复跑。
- **技术选型**（我拍板）：标签/项目用 **JSON 数组列**、树用 **nodes_json 单列整存**（个人量级最优、迁移成本最低；量大再拆 join 表 / nodes 表）。

### 12.2 摘要 28 → 20 字

`make_summary` 取前 20 个 Unicode 字符（用户定「20～40 字折中、20 够用」）。同步改 3 条摘要单测。

### 12.3 统计语义：outline 单独计数

`Stats` 增 `outline` 字段；`stats()` 按 `kind` 分开统计——`active` 只数 `kind='note'`、`outline` 数 `kind='outline'`、`trash` 数所有已删。工作台角标只读 `active`，视图数另算。

### 12.4 便签列表只出 note

`list` 的 active 分支加 `AND kind='note'`，保证 outline 不出现在便签卡片列；回收站仍返回全部（含视图）。排序保持 `created_at` 降序（用户定死：新建置顶、编辑不置顶）。

### 12.5 命令 / 事件扩展（向后兼容）

- `create_page(kind: Option<String>)`：默认 `note`，可建 `outline`。
- `update_page(id, content, title, tags, projects)`：`title/tags/projects` 为 `Option`，**未传则保留库中原值**——旧前端只传 `{id, content}` 也照常工作，不破坏正文编辑。
- `ChangedPayload` 增 `title/kind/tags/projects`（均 `Option`）：create/restore 时带上，update/delete/purge 时置 `None`（单窗口下元数据本地乐观态已一致，事件只回显摘要/version）。

### 12.6 验证

| 项 | 结果 |
| --- | --- |
| `cargo test --release` | ✅ 15/15（原 12 + 新增 3：元数据写读与保留 / outline 单独计数 / 旧库迁移） |
| `cargo build --release` | ✅ release exe 重编成功 |
| `npm run build` / `npm test` | ✅ 前端零改动，仍 17/17 通过 |

**阶段 1 出口达成**：旧库无损升级、命令签名向后兼容、旧功能零回归，为阶段 3/4 的新 UI 打好数据地基。

---

## 13. v12 落地 · 阶段 2 搜索过滤 + 摘要（2026-09-17 追加）

### 13.1 搜索改搜 title + content

`db.list` 的搜索条件从 `content LIKE` 扩为 `(content LIKE ... OR title LIKE ...)`，标题与正文都参与匹配。

### 13.2 LIKE 转义收到后端（修掉一个旧隐患）

原来前端 `escapeLike` 定义了却没接进搜索链路，`db.list` 直接对原始串做 `LIKE`，输入 `%`/`_`/`\` 会被当通配符。本轮在 Rust 侧新增 `escape_like()`，`db.list` 内部先转义再匹配——**单一数据源**，前端无需再记转义。前端 `escapeLike` 保留（仍被单测覆盖），后续若做前端过滤可复用。

### 13.3 标签 AND / 项目 OR 过滤（后端，保证分页正确）

`db.list` 增 `tags: &[String]`（AND，全部命中）、`projects: &[String]`（OR，任一命中）参数，用 SQLite `json_each` 匹配 JSON 列：

```sql
-- 标签 AND：每个标签一个 EXISTS
EXISTS (SELECT 1 FROM json_each(pages.tags) WHERE json_each.value = ?)
-- 项目 OR：多个 EXISTS 用 OR 连接
```

**放后端而不是前端**：前端过滤只过滤「已加载页」，会漏掉未加载分页；后端过滤才能与游标分页一起保证不漏。`json_each` 在 bundled SQLite 里可用（测试验证通过）。

### 13.4 动态 SQL + 参数

`db.list` 改为「按条件拼 WHERE + `Vec<rusqlite::types::Value>` 参数 + `params_from_iter`」，避免 4 种 view×cursor 组合 × 过滤器组合的笛卡尔爆炸。

### 13.5 前端摘要对齐 20 字

前端 `opSeq.ts` 的 `makeSummary` 28→20，与后端 `make_summary` 一致（乐观显示与后端落库不再长度不一致）。

### 13.6 验证

| 项 | 结果 |
| --- | --- |
| `cargo test --release` | ✅ **18/18**（新增 escape_like / search_matches_title / filter_tags_and_projects） |
| `npm test` | ✅ 17/17（makeSummary 20） |
| `npm run build` | ✅ 通过，产物 hash 更新为 `notes-DUJ5C2qI.js` |
| `cargo build --release` | ✅ exe 重编成功 |

**阶段 2 出口达成**：搜索/过滤后端就绪（前端尚未暴露标签/项目过滤 UI，属阶段 3），摘要前后端一致。

---

## 14. v12 落地 · 阶段 3 卡片模式增强（2026-09-17 追加）

### 14.1 后端：标签/项目候选索引 `get_meta_index`

- `models.rs` 加 `MetaIndex { tags, projects }`；`db.rs` 加 `meta_index()`，用 `json_each` 从 JSON 列聚合去重取值；`commands.rs` 加 `get_meta_index` 命令并在 `main.rs` 注册。
- 前端下拉的候选来自这里（**派生自页面数据**，没有独立的标签表；新建标签 = 挂到某条便签上，下次自动出现在候选里）。

### 14.2 数据模型前端对齐

- `types.ts`：`PageMeta/Page` 加 `title/kind/tags/projects`（`Page.nodes` 留 `unknown` 给阶段 4）；`Stats` 加 `outline`；新增 `MetaIndex`；`ChangedPayload` 加可选 `title/kind/tags/projects`。
- `opSeq.ts` 的 `payloadToMeta` 补齐新字段；测试 `meta()` 工厂同步。

### 14.3 纸张编辑：标题 + 标签/项目

- 新组件 `TagSelect`：触发按钮 + chips（薄荷/紫）+ 下拉多选 + 新建输入（`allowAdd` 控制是否显示新建）。
- `Paper.tsx`：顶部 `.paper-meta`（标签 + 项目两个 `TagSelect`）+ `.paper-title` 标题输入（有字加粗 16px、占位「标题（可选）」）；**readOnly 时隐藏 meta、标题只读**。
- `notesStore`：`onTitleInput / setPageTags / setPageProjects` 乐观更新 + 300ms 防抖落盘；`doFlush` 改为一次事务发 `title+content+tags+projects`；`setPageTags/Projects` 顺带把新标签合入 `metaIndex`。

### 14.4 搜索过滤 UI

- `SearchBar.tsx`：聚焦或有字时展开 `.search-filters`（标签 AND / 项目 OR 两个 `TagSelect`，`allowAdd=false`）。
- `notesStore`：`activeTags/activeProjects` 状态 + `setActiveTags/setActiveProjects`（变更后重搜）；清空搜索/切视图/打开结果时一并清空过滤。

### 14.5 顺带修正：回收站纸面只读

原 `readOnly={searching}` 漏了回收站——回收站条目在纸面可被编辑。按 v12 规格改为 `readOnly={searching || isTrash}`，回收站条目只读。

### 14.6 工作台角标对齐

`Stats` 加了 `outline`，工作台 `useState<Stats>` 初始值补 `outline: 0`；角标仍只显示 `stats.active`（= 便签数，符合「视图单独计数、工作台只数便签」）。

### 14.7 验证

| 项 | 结果 |
| --- | --- |
| `cargo test --release` | ✅ **19/19**（新增 meta_index 聚合测试） |
| `npm test` | ✅ 17/17 |
| `npm run build` | ✅ 通过（60 模块，notes 产物 CSS/JS 体积随新组件增长） |
| `cargo build --release` | ✅ exe 重编成功 |

**阶段 3 出口达成**：标题/标签/项目编辑、搜索过滤可用（卡片列随后按反馈补加了标题，见 §15）。

---

## 15. 界面打磨一轮（2026-09-17 追加）

用户反馈一批搜索栏/标题/项目/下拉的体验问题，逐一调整：

1. **搜索过滤器让位**：`.search-filters` 从「绝对定位覆盖卡片列表」改为「inline 占位下推」，`.card-list-outer`/`.empty-state` 由固定 432px 改 `flex:1`；过滤器展开时卡片列表往下顺、能看到下方内容。
2. **标签/项目图标更明显**：触发按钮改有色底（标签薄荷青、项目紫）+ 描边 + 加粗，`🏷 标签`/`📁 项目` 一眼可辨。
3. **选中标签实时过滤**：`doSearch` 改为无参、`onSearchChange/setActiveTags/setActiveProjects` 统一走 `scheduleSearch`；**即使无搜索词，选中标签/项目也立即按过滤出结果**。App 侧 `searching` → `filtering`（含标签/项目维度），`readOnly/hasMore/onSelect` 同步。
4. **标题空时自适应收窄**：`.paper-title:placeholder-shown:not(:focus)` 缩小字号/透明度/间距；无标题不占满整行，有标题或聚焦时正常加粗。
5. **卡片列显示标题（有标题时）**：`.card` 加 `has-title` 态，标题加粗 1 行截断，此时正文缩略降为 1 行（回收站标题灰字）。
6. **项目不允许手动新建**：项目 `TagSelect` 加 `allowAdd={false}`（项目将来自动绑定视图节点的父文件夹，属阶段 4）。
7. **点击外部关闭下拉**：`TagSelect` 加 `document` mousedown 监听，点击容器外（卡片/正文/其他）自动关闭下拉。

验证：`npm run build` ✅、`npm test` ✅ 17/17、`cargo build --release` ✅（本轮纯前端改动，后端无变化）。

---

## 16. 界面打磨二轮：滚动条/标签删除/项目绑定（2026-09-17 追加）

1. **纸张隐藏滚动条 + 内容居中**：`.paper-content` 加 `scrollbar-width:none`/`-ms-overflow-style:none`/`::-webkit-scrollbar{display:none}`，去掉右侧滚动条占位；`text-align:center` 让正文在纸面内居中。
2. **标签可删除**：后端 `db.delete_tag`（从所有页面移除该标签，`json_each` 不依赖，直接读改 JSON）+ `delete_tag` 命令；前端 `TagSelect` 加 `onDelete`，下拉选项 hover 时右侧出现红色 `×` 删除按钮（仅纸张编辑页，搜索过滤不显示）。删除后刷新 `metaIndex`、同步列表/当前页 tags。
3. **项目改为路径绑定**：项目不再手动多选/新建，改为「📁 文件夹」按钮——右键弹出系统文件夹选择器（`tauri-plugin-dialog`，新增 `pick_project_dir` 命令）选择绑定路径；新建便签默认项目 = 本 Note 项目上一层文件夹（`CARGO_MANIFEST_DIR` 的祖父目录，即 `D:\MyProject`）。按钮显示文件夹名、`title` 显示完整路径。
4. 新增依赖 `tauri-plugin-dialog = "2"`（含 `tauri-plugin-fs`/`rfd` 传递依赖）。

验证：`cargo test --release` ✅ **20/20**（新增 delete_tag 测试）、`npm run build` ✅、`npm test` ✅ 17/17、`cargo build --release` ✅。

---

## 17. 修复项目选择器卡死 + 项目列表（2026-09-17 追加）

### 17.1 根因：阻塞式文件选择器跑在主线程

`pick_project_dir` 之前是**同步命令**，直接调 `blocking_pick_folder()`。Tauri 同步命令跑在主线程，一打开文件选择器就把整个界面卡死（进入选择页就「无响应」，选完仍卡）。

**修复**：命令改 `async`，用 `tauri::async_runtime::spawn_blocking` 把 `blocking_pick_folder()` 挪到后台线程跑，主线程保持响应。

```rust
#[tauri::command]
pub async fn pick_project_dir(app: AppHandle) -> Result<Option<String>, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        app.dialog().file().blocking_pick_folder()
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(result.and_then(|f| f.as_path().map(|p| p.to_string_lossy().to_string())))
}
```

**认知**：Tauri 同步命令在主线程执行；任何阻塞式 UI 调用（文件/目录选择器、长 IO）必须 `async` + `spawn_blocking`，否则界面卡死。

### 17.2 项目按钮左键列出可选项目

- 后端 `list_project_folders`：列出默认项目根（`D:\MyProject`，`summary::default_project_dir()` 即 `CARGO_MANIFEST_DIR` 祖父目录）下的**子文件夹**作为候选项目。
- 前端新组件 `ProjectPicker`：**左键**打开下拉列出候选项目文件夹（空则显示「无」），点击绑定；**右键**弹系统文件选择器任意选一个文件夹绑定。`default_project_dir` 从 `db.rs` 提取到 `summary.rs` 共用。

验证：`cargo test --release` ✅ 20/20、`npm run build` ✅（61 模块）、`npm test` ✅ 17/17、`cargo build --release` ✅。

---

## 18. v12 落地 · 阶段 4 结构视图（2026-09-17 追加）

### 18.1 后端：outline 增删改查 + 窗口 resize

- `db.rs`：`create(kind)` 对 outline 初始化 `nodes_json='[]'`；新增 `list_outlines()`（列出未删除的视图）与 `save_outline(id, nodes_json)`（整树覆盖保存）。
- `commands.rs`：`list_outlines` / `save_outline` / `resize_notes_window(width,height)`（用 `LogicalSize::set_size` + `center()` 动态加宽/复原窗口）。
- `summary.rs`：`default_project_dir()` 从 db 提取共用。

### 18.2 前端：结构视图

- `types.ts`：`Mode`（card/outline）、`OutlineNode`（递归树）；`Page.nodes` 从 `unknown` 改为 `OutlineNode[]`。
- 新 `tree.ts`：不可变树操作（`addRoot/addChild/removeNode/updateNode/findNode/newNode/newId`）。
- `notesStore`：`mode/outlines/currentOutline/activeNodeId` 状态 + `setMode/loadOutlines/selectOutline/createOutline/saveOutline/addRootNode/addChildNode/deleteNode/renameNode/updateNodeText/importNodeToNote`。`setMode` 切换时 `resizeNotesWindow`（卡片 740×648 / 视图 1300×648）。
- 新组件 `Seg`（卡片/视图分段）、`OutlineTree`（缩进树 + 双击改名 + 加/删节点）、`NodeContent`（节点正文 + 「＋卡片」导入/撤销）。
- `App.tsx`：按 `mode` 条件渲染两套布局——卡片模式（纸 + 卡片列）/ 视图模式（树 550 + 视图列表 226 + 内容 400，`.panel.outline` 加宽到 1236）。

### 18.3 「＋卡片」导入语义

节点导入为便签：新建 note，标题=节点标题、正文=节点文本、标签带 `视图：<视图名前6字>`、项目继承视图的 projects；节点标记 `addedToNotes + noteId`，再点则软删该便签撤销。乐观插入 active 列表（幂等），保证切换回卡片模式立即看到。

### 18.4 验证

| 项 | 结果 |
| --- | --- |
| `cargo test --release` | ✅ **21/21**（新增 outline_create_list_save） |
| `npm run build` | ✅（65 模块） |
| `npm test` | ✅ 17/17 |
| `cargo build --release` | ✅ exe 重编成功 |

**已知简化（待补）**：① 树目前是「缩进竖向树」，未实现 v12 的**自由拖拽 + 贝塞尔连线 + 点阵画布**；② 卡片↔视图切换目前是**瞬时切换**，未实现 **3D 折叠动画**。两项均为纯前端视觉/交互，不影响数据，可后续单独补。

---

## 19. 卡片↔视图 3D 折叠动画 · 13 小时踩坑复盘（2026-09-18 追加）

> 这是本项目到目前为止最大的一次教训：一个「中间列平移闪一下」的小动画，从中午改到晚上，来回绕了十几个小时。核心原因不是动画本身难，而是**在错误的前提上反复改代码**。下面把整条弯路、根因、教训一条条写清楚。

### 19.1 一句话结论

「闪」从头到尾都不是动画逻辑的问题——**是我（AI）在 debug 构建上测、用户（人）在 release 构建上跑，两边根本不是同一份代码**。我修了十几个小时的「闪」，其实在我自己的构建里早就修好了；用户一直看到的，是 9/17 23:12 那个还带「窗口 resize + 补偿」的旧 release 二进制。

### 19.2 完整时间线（弯路在哪）

| 阶段 | 做了什么 | 结果 |
| --- | --- | --- |
| 起点 | 卡片↔视图用 `resize_notes_window` 把窗口在 740↔1300 之间缩放，中间列用 `WINDOW_RECENTER` 补偿位移 | 弹、闪、跳（±280px 瞬移，因为窗口 `SetWindowPos` 和 CSS transform 两个 compositor 永远不同帧） |
| 第 1 次改 | 窗口固定 1300 不 resize，透明区用 `WM_NCHITTEST` 点击穿透，中间列纯 transform 平移 | 我测「完美」；用户说「还在闪」 |
| 第 2 次改 | 猜是「`fill:forwards` 残留」，加 `cancel + 内联 style` | 没解决，还引入了「连切多次后消失一会」的新 bug |
| 第 3 次改 | 猜是「光带抢主线程」，加切换时暂停光带 | 还是没解决 |
| 转折 | 用户点破：「你测的不闪、我开的闪」 | 发现**环境不一致** |
| 查环境 | 查运行进程 PID 35124 的路径 = `target\release\my-workbench-notes.exe`；release 时间戳 23:12:28（早于我所有 9/18 的修复） | **实锤：用户在跑旧 release，我在测新 debug** |
| 实验 E0–E9 | 写探针（落盘日志 + rAF 采 transform 时间线 + 动画状态 + 祖先 transform + longtask） | 证伪「fill 残留」「transform 跳变」：computed transform 全程单调平滑，无跳变 |
| 最终方案 | ① 中间列 transform 只由 CSS 类控制（`data-mode` + `transition`），slideCard 只切类；② 窗口手动 `work_area()` 居中；③ 叉叉按 `data-mode` 动态定位 | 平移干净、不消失、不偏下 |

### 19.3 教训一（最严重）：先对齐环境，再定位任何「我这边正常你那边不行」的 bug

这是浪费十几小时的头号原因。

- **现象**：我反复「修好了」，用户反复「还闪」。不是我没修对，是修的和跑的不是同一个二进制。
- **根因**：项目里同时存在 `target/debug/*.exe`（我 `cargo build` 出来的）和 `target/release/*.exe`（用户 23:12 编的旧代码）。我测 debug、用户开 release。
- **实锤方式**：直接查**运行中进程的完整路径**（`Get-Process | Select Path`）+ **二进制时间戳**（release 23:12 < 我所有源码 9/18 凌晨的 mtime）。
- **认知**：凡是「你测不闪、他开闪」「works on my machine」这类问题，**第一步永远是确认两边跑的是不是同一个文件**：进程路径、exe 时间戳、加载的是 dev server 还是打包 dist、有没有 StrictMode 导致 dev 下 double render。这一步没做就往下改，等于在错误的前提上叠错误。

### 19.4 教训二：视觉 bug 不要靠猜，靠探针测

- 我连着猜错两次（`fill:forwards` 残留、光带抢主线程），每次都花时间实现，最后都被实验证伪。
- **正确做法**（E0–E9）：给 `.card-column` 加探针，rAF 采样 `getComputedStyle().transform`、`getAnimations()` 的 playState/currentTime、render 计数、祖先 transform、longtask，落盘成时间线。数据一摆出来，「computed transform 全程单调、无跳变」一句话就把所有「transform 跳变」类假设全排除了。
- **认知**：你看不见的渲染效果，就别在脑子里模拟——用探针把 `computed style` / 动画状态 / 长任务这些**能测的**量全部抓到，逐个排除。能测的先测，测不到的（真正的合成器层视觉伪影）才需要人眼看。

### 19.5 教训三：transform 只保留一个来源

中间列平移先后经历了三种写法：

1. `el.animate()` + `fill:forwards`（动画会累积，`n` 一直涨）；
2. `cancel()` + 内联 `style.transform` + `animate()` + 再 `cancel()`（多来源抢同一个 `transform`，脆，还带来「cancel 后消失」的新 bug）；
3. **CSS 类 + `transition`，slideCard 只切 `data-mode`**（最终方案）。

- **认知**：同一个 CSS 属性被「内联 style + WAAPI + fill + 类」多处同时写，就是 bug 温床。**让属性只有单一来源、且操作幂等**（切类设两遍结果一样），天然免疫空窗/跳变/残留。最简方案往往就是最稳方案。

### 19.6 教训四：动画期间绝不要让原生窗口 resize/重定位

- 最原始的 ±280 跳，本质是 `SetWindowPos`（OS 窗口位置）和 CSS `transform`（webview 合成器）是**两个不同的 compositor**，永远无法帧同步；无论怎么「串行、等动画结束、算补偿」都补不回来。
- **正解**：窗口固定 1300 宽不动，卡片态/视图态只是窗口内 CSS 布局差异；透明区用 `WM_NCHITTEST` 返回 `HTTRANSPARENT` 做点击穿透（内存里比较矩形，无原生重绘）。这样中间列平移就是一条纯 CSS transform，参考系永远不变。

### 19.7 教训五：Windows 下 `.center()` 不一定按工作区居中，要用 `work_area()`

- `tao 0.35.3` 的 `platform_impl/windows` 里**没有 `center` 的实现**（源码 grep 不到），所以 notes 窗口 `.center()` 会偏下。
- 项目里主窗口（workbench）一直是手动 `monitor.work_area()` 居中的——正确做法早就在旁边，却没对齐。
- **认知**：同仓库里已有的正确范式（workbench 的 `work_area` 居中、`saturating_sub` 防下溢）要主动去对齐，别自己另起炉灶。

### 19.8 教训六（仍未闭环）：透明窗口 + transform 的合成器伪影，测不到、只能人眼确认

- E0–E9 证明了「transform 不跳变」，但用户描述的「闪」里可能**还有**一部分是合成器层的东西（`transparent: true` + `alwaysOnTop` + `position: fixed` + `perspective: 1600px` + `dpr=1.5` 半像素），它**不体现在 computed style**，探针抓不到。
- 这是「我只能从 CLI 测什么」的边界：DOM/CSS/JS 层能全测，合成器层的视觉伪影测不到，必须人眼看。
- 候选方向（待用户视觉确认后逐个试）：① 透明窗口合成（关 `transparent` 试）；② 把 `.card-column` 挪出 `.window-frame` 的 `perspective` 上下文；③ `dpr=1.5` 下半像素对齐（733×1.5=1099.5）。

### 19.9 元教训：一个「小动画」其实是一堆独立 bug 缠在一起

这十几小时不是只修一个 bug，而是依次拆出了 6 个独立问题：窗口 resize 跳变、transform 多来源、环境不一致（debug/release）、垂直居中、叉叉定位、可能的透明合成。它们互相遮蔽——修好一个才暴露下一个。

**认知**：遇到「一个 bug 反复修不好」，先停下来把「这其实是不是好几个 bug？」问一遍，然后**每个子问题单独隔离、单独验证**，而不是在一个大函数里反复打补丁。一次只动一个变量，改完用数据/日志确认，再进下一个。
