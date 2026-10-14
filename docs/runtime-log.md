# 运行日志（runtime-log）

> 按 `D:\MyProject\CLAUDE.md` §1 的六件套之一：**运行时的行为日志，可关。**
>
> 🔴 **本文件是 2026-10-09 新增。** 在此之前，这个项目**没有任何运行日志** ——
> 排查 2026-10-09 那次"窗口像是卡住"的事故时，**事后重启，现场证据全没了，只能靠猜**
> （静态排查连续推翻九个假设）。`trace.log` 就是为这件事加的。

---

## 一、结论先行

| 项 | 值 |
| --- | --- |
| **有没有运行日志** | ✅ **有，一个**（`trace.log`） |
| **在哪** | 🔴 **不在项目里** —— `%APPDATA%\com.myworkbench.notes\trace.log`（与 `notes.db` 同目录） |
| **形态** | **纯文本、逐行追加**，一行一条事件 |
| **格式** | `<epoch_ms>\t<消息>`，**毫秒时间戳 + 制表符 + 消息** |
| **谁写** | 应用自己（Rust 侧直接写；前端经 `trace` 命令写） |
| **可关吗** | ✅ **可关** —— 见第五节（目前是**常开**） |
| **会自动清理吗** | ❌ **不会。无轮转、无上限** —— 见第四节 |

---

## 二、为什么需要它（一句话）

**因为 17321 是只读的，而"窗口卡住"这类现象只在屏幕上看得到 —— 没有日志就只能靠猜。**
（这正好接上 [LESSONS.md](LESSONS.md) #12：「合成器层的视觉伪影测不到…**必须人眼看**」）

---

## 三、写些什么（逐条）

**时间戳是 `epoch_ms`**（毫秒）。读取换算见第六节。

| 前缀 | 什么时候写 | 出处 |
| --- | --- | --- |
| `setup: app started` | 应用启动、`setup()` 一进来 | `src-tauri/src/notes/mod.rs`（`trace::log("setup: app started")`） |
| `page_load: Started` | 页面**开始**加载 | `src-tauri/src/notes/window.rs`（`on_page_load`） |
| `page_load: Finished url=…` | 页面加载结束（**含加载失败** —— 见第七节） | 同上 |
| `ui\tinit: begin` / `ui\tinit: done` | 前端初始化起止 —— **两者之差 = 前端可用耗时** | 前端 `notesStore.init()` 经 `trace` 命令 |
| `ui\tinit: empty library, auto-create` | 判定"空库"并自动建了一张空白便签 | 同上 |
| `ui\tinit: list_pages failed, skip auto-create` | 读列表失败、**跳过了**自动新建 | 同上 |
| `ui\tloadFirstPage(<view>) failed` | 拉某一视图第一页失败 | 同上 |
| `ui\tquit: saved, exiting` | 已落盘、准备退出 | 前端 `notesStore.closeNotes()` |
| `db: create_page id=… kind=…` | **每建一张便签**（唯一权威记录） | `src-tauri/src/notes/commands.rs`（`create_page`） |
| `db: quit_notes (frontend confirmed saved)` | 真正退出进程前 | `src-tauri/src/notes/commands.rs`（`quit_notes`） |

> ⚠️ **前端写的那几条消息里带一个制表符**（前缀 `ui` + TAB）——
> 所以一行实际是 `<epoch_ms>\tui\t<消息>`。**按第一个制表符切两段**来读（第六节给的就是这个写法）。

### 3.1 🔴 它**不**记什么（隐私边界，刻意如此）

| 不记的东西 | 为什么 |
| --- | --- |
| **便签正文** | 日志落在 `%APPDATA%`，**不复制用户内容** |
| **便签标题** | 同上 |
| **标签名 / 项目路径** | 同上（`create_page` 只记 `id` 与 `kind`） |

**→ 日志里没有用户数据。** 只有"谁在什么时候被创建/退出/加载"这类**事件**。

---

## 四、⚠️ 两条已知限制

| # | 限制 | 说明 |
| --- | --- | --- |
| 1 | 🔴 **无轮转、无上限** | 只 `append`，**永远不删**。增长**极慢**（启动几行 + 每建一张便签 1 行 + 每次关闭 2 行），但**长期不清会一直涨** |
| 2 | ⚠️ **写失败是静默的** | 目录不可写 / 磁盘满 → **什么都不发生**（诊断设施绝不反过来成为故障源，这是刻意的） |

---

## 五、怎么关（本文件标题里的"可关"）

**目前是常开的。** 要关，两条路（都**不改行为**，只停止记录）：

| 方式 | 怎么做 | 影响 |
| --- | --- | --- |
| **最小改动** | 让 `trace::init()` 不设路径（`LOG_PATH` 保持 `None`）→ `log()` 直接 `return` | 全部日志停写，调用点一行不用改 |
| **彻底移除** | 删掉 `src-tauri/src/notes/trace.rs` 及所有 `trace::log(...)` 调用点 + 前端 `notesApi.trace` 调用 | 需要改代码 |

> 📌 也可以只删文件：`Remove-Item "$env:APPDATA\com.myworkbench.notes\trace.log"` ——
> 应用下次启动会**重建**它。

---

## 六、怎么读（时间戳是 epoch 毫秒）

```powershell
Get-Content "$env:APPDATA\com.myworkbench.notes\trace.log" | ForEach-Object {
  $p = $_ -split "`t", 2
  "{0}  {1}" -f ([DateTimeOffset]::FromUnixTimeMilliseconds([int64]$p[0]).ToLocalTime().ToString("MM-dd HH:mm:ss.fff")), $p[1]
}
```

**⚠️ 不要用 `Get-Content` 直接看中文** —— PowerShell 5.1 默认按系统 ANSI（本机 GBK）解码，
UTF-8 文件会**显示成乱码**（文件本身没问题）。上面这条只解析 ASCII 数字与 `ui`/`db` 前缀，**不受影响**。

---

## 七、🔴 另一个"运行日志"：stderr（与 `trace.log` 不是一回事）

`http.rs` 用 `eprintln!` 打两行：

| 行 | 时机 | 出处 |
| --- | --- | --- |
| `[notes-http] 只读接口已就绪 http://127.0.0.1:17321/notes/changes` | 绑端口成功 | `src-tauri/src/notes/http.rs` |
| `[notes-http] 无法绑定 …（端口可能被占用，只读接口未启用）` | 绑端口失败 | 同上 |

**⚠️ 这两行去哪了？** release 版按 `windows_subsystem = "windows"` 编译（`src-tauri/src/main.rs:2`），
**没有控制台** → **双击启动时这两行谁也看不到。**

> **这正是 `trace.log` 存在的另一半理由**：`eprintln!` 在 GUI 程序里等于**写进黑洞**。

---

## 八、未验证与纪律声明

| 项 | 状态 |
| --- | --- |
| 本文件**未新增任何实测** | ⚠️ 表格内容来自**本轮读源码** + **实读 `trace.log` 内容**。**未跑构建、未碰端口。** |
| "增长极慢" | ⚠️ **定性判断**（依据：实测看到的写入点密度），**未做长期统计** |
| `page_load: Finished` 在**加载失败**时是否也触发 | 🔴 **未实测** —— [OUTPUT-SLOT.md](OUTPUT-SLOT.md) 与 [LESSONS.md](LESSONS.md) 都记着同一类未闭环项 |
| 删除 `trace.log` 后重启会重建 | **推断**（`create(true)` + `append(true)`），**未实测** |

---

## 九、证据索引

> 格式 `文件:行号`，路径相对 `Note\`。行号是 **2026-10-09** 实测值（与其它文档的 10-03 快照不同 —— 见 [log.md](log.md) 的「行号漂移」条目）。

| 断言 | 证据 |
| --- | --- |
| 日志实现就在一个文件里 | `src-tauri/src/notes/trace.rs`（**本轮新增**） |
| 路径 = `app_data_dir()/trace.log` | `src-tauri/src/notes/trace.rs`（`init()`：`app.path().app_data_dir()` + `join("trace.log")`） |
| `notes.db` 同目录 | `src-tauri/src/notes/mod.rs`（`setup()`：`app_data_dir()?.join("notes.db")`） |
| 前端能写 | `src-tauri/src/notes/commands.rs:130`（`pub fn trace(msg: String)`，前缀 `ui\t`） |
| 建便签记一行 | `src-tauri/src/notes/commands.rs`（`create_page` 里的 `trace::log`） |
| 退出前记一行 | `src-tauri/src/notes/commands.rs:139`（`pub fn quit_notes`） |
| 页面加载记两行 | `src-tauri/src/notes/window.rs`（`on_page_load`） |
| release 无控制台 | `src-tauri/src/main.rs:2`（`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`） |
| 端口常量 | `src-tauri/src/notes/http.rs:30`（`pub const PORT: u16 = 17321;`） |
