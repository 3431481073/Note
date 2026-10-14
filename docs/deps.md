# 依赖

---

## 一、结论先行

| 问题 | 答案 |
| --- | --- |
| **有模型文件吗** | **没有** —— 无权重、无数据集、无 AI 调用 |
| **对 `_shared/models/` 的贡献** | **零** |
| **对 `_shared/routes.yaml` 的贡献** | **零**（未追加任何路由） |
| **外部可执行依赖** | **无**（不调用任何外部二进制） |
| **依赖其他项目吗** | **有** —— 工作台（见第五节，双向） |
| **有项目依赖它吗** | **有** —— 同一个：工作台 |

---

## 二、npm 依赖

`package.json`，**运行时 4 个 + 开发 7 个**（实测，2026-10-03）。

### 运行时（4）

| 包 | 版本 | 用途 |
| --- | --- | --- |
| `react` / `react-dom` | `^18.3.1` | UI（**注意：React 18，不是 19** —— 与 `markdown-reader` / `Lattice` 的 React 19 不同） |
| `@tauri-apps/api` | `^2.1.1` | 前端调 Rust 命令 / 事件 |
| `@tanstack/react-virtual` | `^3.10.8` | **卡片列表的虚拟滚动** |

### 开发（7）

| 包 | 版本 |
| --- | --- |
| `@tauri-apps/cli` | `^2.1.0` |
| `@types/react` / `@types/react-dom` | `^18.3.12` / `^18.3.1` |
| `@vitejs/plugin-react` | `^4.3.4` |
| `typescript` | `^5.6.3` |
| `vite` | `^5.4.11` |
| **`vitest`** | `^2.1.8` |

> **与本工程其他前端项目对比**：`markdown-reader` 用 React 19 + Vite 8；
> `Lattice` 用 React 19 + Vite 7 + Electron 44。**本项目是 React 18 + Vite 5 —— 最旧的一套。**
> 这是移植时间（2026-09）和源规格决定的，**不是落后的信号，是"能用就不动"的一致选择**。

---

## 三、🔴 `custom-protocol` 的来龙去脉 —— 这一节必读

`src-tauri/Cargo.toml`：

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset", "custom-protocol"] }
```

**没有 `[features]` 段。**

### 3.1 为什么写成这样（作者自己的记录）

**`history/DEVELOPMENT_LOG.md` §8.3 原文：**

> **⚠️ 决定 dev/prod 的其实是 `custom-protocol` feature，不是 debug/release**
> （本次「localhost 拒绝访问」的根因）：Tauri 2 的 build.rs 里写死 `let dev = !custom_protocol;`，
> 而 `custom-protocol` **不在默认 feature 里**。我只开了 `protocol-asset`（为水印 asset 协议），
> 没开 `custom-protocol`，于是**即使是 `--release` 也走 dev 模式**、去连 `http://localhost:1420`。
> **修复**：`tauri = { version = "2", features = ["protocol-asset", "custom-protocol"] }`。

**`history/DEVELOPMENT_LOG.md` §10.1 原文：**

> 上一轮用裸 `cargo build --release` 重编工作台，而它 `Cargo.toml` 里 `tauri` 是 `features = []`。
> Tauri 2 用 `custom-protocol` feature 决定 dev/prod，没开就按 dev 连 `localhost:1420`。
> 修复：工作台也补上 `features = ["custom-protocol"]`。
>
> **认知**：重编任何 Tauri 2 项目前，先确认 tauri 依赖带了 `custom-protocol`；
> 否则哪怕 `--release` 也走 dev 模式。

**`history/IMPLEMENTATION_PLAN.md` §6 checklist 与 `history/PORTING_REPORT.md` 也各有一条：**

> release exe 重编（**确认 `custom-protocol` 仍在**，避免退回 dev 连 `localhost:1420`）。

### 3.2 所以这是**有意选择**，不是踩坑

**同一条坑，两个项目给出了相反的结论，而且各自都对：**

| | `markdown-reader` | **`Note`** |
| --- | --- | --- |
| 痛点 | `tauri dev` 热重载 | **裸 `cargo build --release` 产出 dev 模式 exe** |
| 对策 | **不写** `custom-protocol`（`[dependencies]` 里只留 `protocol-asset`） | **写死** `custom-protocol` |
| 代价 | 裸 cargo 构建 → 白屏 | **`tauri dev` 可能不走热重载**（加载 `dist/` 而非 1420） |

> **出处**：`markdown-reader` 侧见 `project_manager/docs/PLATFORM-NOTES.md` 的 **P-1**
> 与 `markdown-reader/docs/PROGRESS.md` §1。

### 3.3 能同时满足两边的写法（本项目未采用）

```toml
[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

| 命令 | 结果 |
| --- | --- |
| `tauri dev`（CLI 传 `--no-default-features`） | custom-protocol 关 → 走 `devUrl` → **热重载正常** ✅ |
| 裸 `cargo build --release` | 默认特性开 → custom-protocol 开 → **自包含** ✅ |
| `tauri build`（CLI 自己注入） | 开 ✅ |

**这是 P-1 原文推荐的写法。**

**⚠️ 本项目本次整理没有改它** —— 理由见 [log.md](log.md) 未解决 1。
**在没跑过一次 `tauri dev` 之前，"改了会不会更好"是猜，不是判断。**

---

## 四、cargo 依赖

`src-tauri/Cargo.toml`：

```toml
[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["protocol-asset", "custom-protocol"] }
tauri-plugin-single-instance = "2"
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
rand = "0.8"
notify = "6"
thiserror = "1"

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.59", features = [
  "Win32_UI_WindowsAndMessaging", "Win32_UI_Shell", "Win32_Graphics_Gdi", "Win32_Foundation"
] }

[profile.release]
codegen-units = 1
lto = true
strip = true
```

**逐个说用途：**

| 依赖 | 干什么 |
| --- | --- |
| `rusqlite` **`bundled`** | SQLite 数据层。**`bundled` 意味着把 SQLite C 源码一起编译进去** —— 不依赖系统库，代价是首次编译慢 |
| `notify` | **配置热重载**：`config.rs` 监听 `seasons.json` 变化 |
| `tauri-plugin-single-instance` | 重复启动只聚焦已有窗口 |
| `tauri-plugin-dialog` | 选项目目录（`pick_project_dir` 命令） |
| `windows-sys` | **窗口定位 + 点击穿透**（`SetWindowSubclass` / `WM_NCHITTEST` / `ScreenToClient`） |
| `rand` | `nanoid()` 生成 21 字符 ID |
| `thiserror` | 🔴 **全项目无使用点**（声明了但没用到 —— 见第八节 8.2） |

**`[profile.release]` 开了 `lto` + `codegen-units = 1` + `strip`** ——
**这解释了为什么 `target/` 会有 8.45 GB**（release 的 2.26 GB 里有大量中间产物）。

> **本项目没有 `release/` 交付目录，也没有 `bundle.resources`** —— 见 [data.md](data.md) 第六节的存疑项。

---

## 五、对外契约表

**本项目在依赖图里有两条边，方向相反 —— 是双向的。**

### ① 被依赖方：`Workbench` → **Note**

| 列 | 内容 |
| --- | --- |
| **依赖方** | 工作台（`D:\MyProject\Workbench`） |
| **依赖什么** | **Note 的 exe 绝对路径**，写在工作台侧的 `tools.json` 里 |
| **断了会怎样** | **响亮失败** —— 从工作台点便签，启动不了 |

**本项目自己登记了这条待办**（`history/DEVELOPMENT_LOG.md` §8.4）：

> - [ ] 便签 exe 绝对路径写死在 `tools.json` 里，**文件夹移动后需同步改**。

**本项目在 `config/tools.json` 里的自声明**：

```json
{ "version": 1, "tools": [
    { "id": "notes", "name": "便签", "icon": "📝", "command": "toggle_notes", "enabled": true }
] }
```

**→ 工作台调用的入口是 Tauri 命令 `toggle_notes`**（`src-tauri/src/workbench/action.rs`），
**不是 HTTP**。这一点和 Lattice 一样：工作台只知道怎么"启动/切换"，不走数据接口。

### ② 依赖方：**Note** → `Workbench`

| 列 | 内容 |
| --- | --- |
| **依赖方** | Note |
| **依赖什么** | `src/workbench/`（**开发用的最小外壳**，README 明说）—— 用来在没有真工作台时也能起窗口 |
| **断了会怎样** | **不影响运行** —— 最小外壳是替代品，不是依赖 |

---

## 六、环境位置

**没有 `venv/` 之类**（非 Python 项目）。依赖环境在：

- **`node_modules\`**（83.28 MB，在仓库根）
- `dist\`（0.25 MB，构建产物）
- `src-tauri\target\`（**2026-10-03 删掉了 `debug/` 的 6,193 MB；`release/` 2,258.93 MB 保留**）
- Rust 工具链与 cargo 注册表缓存在用户目录，**不在本项目内**

---

## 七、证据索引

> 格式 `文件:行号`，路径相对 `Note\`。行号是 2026-10-03 第二十站实测值。

### 7.1 结论先行（第一节）

| 断言 | 证据 |
| --- | --- |
| **没有模型文件** | **实测**：项目内无 `*.safetensors` / `*.bin` / `*.gguf` / `*.onnx` / `models/` 内容 |
| **对 `_shared/models/` 贡献零** | 同上（无模型可贡献） |
| **对 `_shared/routes.yaml` 贡献零** | **实测**：源码里没有任何写 `routes.yaml` 的代码 |
| **无外部可执行依赖** | **实测**：`Cargo.toml:10-22` 无 `Command::new` 类依赖；源码里无 `std::process::Command` 调用 |
| **依赖工作台（双向）** | 见 7.5 |

### 7.2 npm 依赖（第二节）

| 断言 | 证据 |
| --- | --- |
| 运行时 4 个，版本如表中 | `package.json:13-18`（`@tanstack/react-virtual ^3.10.8` · `@tauri-apps/api ^2.1.1` · `react ^18.3.1` · `react-dom ^18.3.1`） |
| 开发 7 个，版本如表中 | `package.json:19-27`（`@tauri-apps/cli ^2.1.0` · `@types/react ^18.3.12` · `@types/react-dom ^18.3.1` · `@vitejs/plugin-react ^4.3.4` · `typescript ^5.6.3` · `vite ^5.4.11` · `vitest ^2.1.8`） |
| 「React 18，不是 19」 | `package.json:16-17`（`^18.3.1`）✅ |
| 「本项目是 React 18 + Vite 5 —— 最旧的一套」 | `package.json:16-17,25`。⚠️ **与 `markdown-reader` / `Lattice` 的对比本轮未复核**（禁止碰其他项目） |

### 7.3 `custom-protocol`（第三节）

| 断言 | 证据 |
| --- | --- |
| `Cargo.toml` 里那一行 | `src-tauri/Cargo.toml:11` |
| **没有 `[features]` 段** | `Cargo.toml` 全文（1-27 行）无 `[features]` ✅ |
| §3.1 引的 `DEVELOPMENT_LOG.md` §8.3 / §10.1 原文 | `docs/history/DEVELOPMENT_LOG.md` —— ⚠️ **引文本身未逐字复核**（42,756 B 全文），**只核了文件存在与大小** |
| §3.2 与 `markdown-reader` 的对比表 | `project_manager/docs/PLATFORM-NOTES.md` 的 **P-1** —— ⚠️ **未打开复核**（禁止碰其他项目） |
| §3.3 那段 `[features]` 写法**未被采用** | `Cargo.toml` 无 `[features]` ✅ |

### 7.4 cargo 依赖（第四节）

| 断言 | 逐条证据（全在 `src-tauri/Cargo.toml`） |
| --- | --- |
| `tauri-build` | `:8` |
| `tauri` + 两个 feature | `:11` |
| `tauri-plugin-single-instance` | `:12` |
| `tauri-plugin-dialog` | `:13` |
| `serde` / `serde_json` | `:14-15` |
| `rusqlite` bundled | `:16`（`features = ["bundled"]`） |
| `rand` | `:17` |
| `notify` | `:18` |
| `thiserror` | `:19` |
| `windows-sys` + 4 个 feature | `:21-22` |
| `[profile.release]`：`codegen-units=1` / `lto` / `strip` | `:24-27` |
| **逐条用途** | `rusqlite bundled` → `db.rs` 全文件 · `notify` → `config.rs:102-112` · `single-instance` → `main.rs:9` · `dialog` → `commands.rs:81-84` · `windows-sys` → `hittest.rs:5-8` + `window.rs:29-32` · `rand` → `summary.rs:29-37`（`nanoid`）· `thiserror` → **依赖声明了但全项目无 `thiserror` 使用点**（⚠️ **见第八节 8.2**） |
| `target/` 2,258.93 MB | **实测 2026-10-03**：`src-tauri\target\` = **2,258.9 MB / 4,943 文件** |
| 「没有 `release/` 交付目录，也没有 `bundle.resources`」 | **实测**：项目内无 `release/` 目录；`tauri.conf.json:36-45` 的 `bundle` 段无 `resources` ✅ |

### 7.5 对外契约表（第五节）

| 断言 | 证据 |
| --- | --- |
| **工作台依赖 Note** | `history/DEVELOPMENT_LOG.md` §8.4（待办原文）· ⚠️ **工作台那份 `tools.json` 未打开核实** |
| 本项目 `config/tools.json` 的内容 | **实测**：`Note\config\tools.json` **171 B**，内容 `{"version":1,"tools":[{"id":"notes","name":"便签","icon":"📝","command":"toggle_notes","enabled":true}]}` |
| 工作台调的是 Tauri 命令 `toggle_notes` | `src-tauri/src/workbench/action.rs:16-26`（`toggle_notes`）· `main.rs:49-50`（注册 `toggle_notes` / `open_notes`） |
| **Note 依赖工作台**（`src/workbench/`） | `src/workbench/` 3 文件（`App.tsx` 33 行 · `main.tsx` 7 行 · `workbench.css` 59 行）· `vite.config.ts:23-27`（`main: "index.html"`） |
| 「不影响运行 —— 最小外壳是替代品」 | **推断** —— `index.html` 是构建入口、`src/workbench/App.tsx` 只调 `get_stats` + `toggle_notes`（实测 :11,:26），**没有对外部工作台的硬依赖** |

### 7.6 环境位置（第六节）

| 断言 | 实测值 |
| --- | --- |
| `node_modules\` 83.28 MB | **实测 83.3 MB / 3,078 文件** ✅ |
| `dist\` 0.25 MB | **实测 0.2 MB / 7 文件**（四舍五入差异） |
| `target\` release 2,258.93 MB | **实测 2,258.9 MB / 4,943 文件** ✅ |
| Rust 工具链不在项目内 | **实测** `cargo` = `C:\Users\asus\.cargo\bin\cargo.exe` ✅ |

---

## 八、未验证与订正

### 8.1 ⚠️ 本轮未独立验证的断言

| 断言 | 状态 |
| --- | --- |
| §3.1 引用的 `DEVELOPMENT_LOG.md` §8.3 / §10.1 **原文逐字** | ⚠️ **未逐字核**（只核了文件存在 + 42,756 B 大小） |
| §3.2 与 `markdown-reader` 的对比表 | ⚠️ **未复核**（禁止碰其他项目） |
| §5 工作台那份 `tools.json` 里写的绝对路径 | ⚠️ **未打开**（在 `D:\MyProject\Workbench\`，禁止碰其他项目） |
| §二次的「与 `Lattice` / `markdown-reader` 的技术栈对比」 | ⚠️ **未复核** |

### 8.2 🔴 一处内部不一致 —— **已于 E 段修正**

| 位置 | 问题 |
| --- | --- |
| 第四节表格 | 列了 `thiserror = "1"` 并给了用途「错误类型」，但**全项目源码里搜不到 `thiserror` 的使用点** |

✅ **E 段已把该行用途改为「🔴 **全项目无使用点**（声明了但没用到 —— 见第八节 8.2）」。**

**依据**（三路交叉搜索，全部零命中）：
| 搜什么 | 结果 |
| --- | --- |
| `thiserror`（`src-tauri/src/**/*.rs`） | **零命中** |
| `derive(Error)` | **零命中** |
| `#[error(` | **零命中** |

**旁证**：`db.rs` 用的是 `rusqlite::Result`；`config.rs` / `commands.rs` 用的是 `Result<_, String>` —— **都不需要 `thiserror`**。
**⚠️ 但"依赖该不该删"本轮不判** —— 删依赖是代码改动，**本站代码改动 = 零**。已列进 [log.md](log.md) 第二十站未解决 5。
