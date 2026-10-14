# 服务

## 一、结论：**有常驻服务**

| 项 | 值 |
| --- | --- |
| **服务名** | **只读对外接口** |
| **形态** | **loopback HTTP + JSON** |
| **绑定** | `127.0.0.1`，**固定端口 17321** |
| **生命周期** | **随应用启动，随应用退出** —— ⚠️ **不是独立进程**，是 `my-workbench-notes.exe` 主进程内嵌的 |
| **实现** | `src-tauri/src/notes/http.rs`（7,008 B） |
| **注册** | `src-tauri/src/notes/mod.rs:21` → `http::spawn(app.handle().clone())` |

**端口是写死的常量**（与 Lattice 的随机高位端口不同）：

```rust
// src-tauri/src/notes/http.rs:30
pub const PORT: u16 = 17321;
```

**启动时打印一行**：

```
[notes-http] 只读接口已就绪 http://127.0.0.1:17321/notes/changes
```

---

## 二、端口表

| 端口 | 谁 | 常驻 | 何时出现 |
| --- | --- | --- | --- |
| **17321** | **只读对外接口** | ✅ **应用运行时常驻** | 只要 Note 在跑（**实测：PID 29608 正在监听**） |
| **1420** | vite dev server | ❌ | 仅 `npm run tauri dev` / `npm run dev`。`strictPort: true` —— 被占用则**启动失败** |
| **1421** | vite HMR | ❌ | 仅 dev（`vite.config.ts` 写死） |

> ⚠️ **`vite.config.ts` 的 `host: "0.0.0.0"`** —— dev server **绑所有网卡**。
> 本机开发无所谓，但**若这台机器在不可信网络上，dev 模式会把源码暴露给局域网**。

---

## 三、端点（2 个）

| 端点 | 返回 |
| --- | --- |
| `GET /notes/changes` | **全量变更清单** —— 每条只有 **`id` / `updated_at` / `deleted_at`** 三个字段，**含已删除的** |
| `GET /notes/{id}` | **单条完整内容**（按 id） |

**设计意图**：这是一个**同步/镜像接口** —— 外部软件先拉 `/notes/changes` 拿到变更清单
（三个字段足够判断"哪些新增、哪些改了、哪些删了"），再按需拉 `/notes/{id}` 取正文。

---

## 四、三条硬约束（设计如此，不是缺失）

| 约束 | 说明 |
| --- | --- |
| **无写接口** | 只有 `GET`。**外部软件读得到，改不了。** |
| **无鉴权** | 不做认证 —— 守的是"只绑回环"这一条 |
| **只绑回环** | `127.0.0.1`，**不对外网开放** |

> README 原文：「**无写接口、无鉴权、只绑回环**」。

**这意味着**：想改便签，只能用便签自己的 UI。数据口是**单向的观测窗口**，不是 API。

---

## 五、状态从哪来

数据口读的是**同一个 SQLite 库**（`%APPDATA%\com.myworkbench.notes\notes.db`），
经由 `src-tauri/src/notes/db.rs`（51,678 B）的数据层。

**因此**：

- 它看到的是**已经落盘/已进 WAL 的数据**，不是内存里的 UI 状态；
- **`notes.db-wal` 里尚未 checkpoint 的事务影响可见性** —— 见 [data.md](data.md) 第一节。

---

## 六、验证方式

**本次整理未运行任何代码**，所以：
- ✅ **已实测**：端口 17321 确实在监听，持有者是 PID 29608（`my-workbench-notes.exe`）
- ❌ **未实测**：两个端点的实际返回内容与格式

需要验证时：

```powershell
curl.exe http://127.0.0.1:17321/notes/changes
```

（**不要用 `Invoke-WebRequest`** —— PowerShell 对 JSON 的包装会改变观感。）

---

## 七、与外部软件的关系

| 消费者 | 依赖什么 | 断了会怎样 |
| --- | --- | --- |
| **`github_operater`**（`src\watch_notes.py`） | **`http://127.0.0.1:17321/notes/changes` 与 `/notes/{id}` —— 两条都用** | **响亮失败**（连不上） |

### ✅ 实测证据（2026-10-03 收尾段，只读打开）

| 行 | 原文 |
| --- | --- |
| **`github_operater\src\watch_notes.py:50`** | `BASE = "http://127.0.0.1:17321"` |
| `:51` | `CHANGES_URL = f"{BASE}/notes/changes"` |
| `:77` | `return _get_json(f"{BASE}/notes/{note_id}")` |
| `:52` | `INTERVAL = 3.0`（**每 3 秒轮询一次**） |
| `:5`（docstring） | 「只对 Note 的只读接口发 **GET**，不发任何写请求。」 |

**⚠️ 文件在 `src\` 下不是一个根文件** —— 第十站整理 `github_operater` 时已按四区归位。

> **工作台不走这个接口** —— 它调的是 Tauri 命令 `toggle_notes`（见 [deps.md](deps.md) 第五节）。
>
> ⚠️ **本节原先写的是「数据口的实际消费者本次未查明」** ——
> **那句话在第九站写下时是对的，但 [log.md](log.md) 未解决 3 一直挂着它。**
> **2026-10-03 收尾段已查明并改正**：
> 证据是 `PLAYBOOK.md:2131`（工程级文档里早就记着）+ `watch_notes.py:50/51/77`（实例确认）。
>
> **完整规格见 [OUTPUT-SLOT.md](OUTPUT-SLOT.md) 第八节。**

---

## 八、🔴 本文件的接口细节已移到 [OUTPUT-SLOT.md](OUTPUT-SLOT.md)

**2026-10-03 第二十站 B 段**把这个 17321 接口**正式化为「输出槽」**，
**逐字段规格（两条路由的 SQL、10 个字段的类型/含义/可空性、边界三条、缺口两条、
证据等级、`v1` 落地状态、三条硬约束对照）全部写在 [OUTPUT-SLOT.md](OUTPUT-SLOT.md)。**

**本文件保留「服务视角」**（它是不是服务、端口表、生命周期、硬约束），
**接口视角请看 [OUTPUT-SLOT.md](OUTPUT-SLOT.md)。**

> ⚠️ **本节以下的第九节，是 B 段之后补的证据索引。**
> **第一～七节写于第九站整理时，当时「端点的实际返回内容未实测」；
> B 段仍然没有实测（禁止碰端口），但已经【读源码】把字段表写全了。**

---

## 九、证据索引

> 格式 `文件:行号`，路径相对 `Note\`。行号是 2026-10-03 第二十站实测值。

### 9.1 服务结论（第一节）

| 断言 | 证据 |
| --- | --- |
| 有常驻服务 | `src-tauri/src/notes/mod.rs:21`（`http::spawn(app.handle().clone());`） |
| 形态 = loopback HTTP + JSON | `src-tauri/src/notes/http.rs:19-21`（`std::net::TcpListener/TcpStream`，**无 HTTP 框架**）· `:193`（`Content-Type: application/json; charset=utf-8`） |
| 只绑 `127.0.0.1` | `http.rs:82`（`SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT)`）✅ |
| **固定端口 17321** | `http.rs:30`（`pub const PORT: u16 = 17321;`）✅ |
| 随应用启动/退出，**不是独立进程** | `mod.rs:14-25`（`setup()` 里 `http::spawn`，跑在一个 `std::thread::spawn` 里 —— `http.rs:80`） |
| **实测：PID 29608 正在监听** | `Get-NetTCPConnection -State Listen` 实测：`127.0.0.1:17321` → OwningProcess **29608** ✅ |
| 那个 PID 就是便签 exe | 实测 `Get-Process -Id 29608` 的 `Path` = `D:\MyProject\Note\src-tauri\target\release\my-workbench-notes.exe` ✅ |
| 实现文件 7,008 B | **实测** `http.rs` = **7,008 B / 203 行** ✅ |
| 启动时打印一行 | `http.rs:90`（`eprintln!("[notes-http] 只读接口已就绪 http://127.0.0.1:{PORT}/notes/changes");`） |

### 9.2 端口表（第二节）

| 端口 | 断言 | 证据 |
| --- | --- | --- |
| **17321** | 常驻 | 见 9.1 |
| **1420** | vite dev server，`strictPort: true` | `vite.config.ts:5`（`const port = 1420;`）· `:13`（`strictPort: true`） |
| **1421** | HMR | `vite.config.ts:18`（`port: 1421`）· `:17`（`host: "localhost"`） |
| ⚠️ `host: "0.0.0.0"` | dev server 绑所有网卡 | `vite.config.ts:14` ✅ |
| 1420/1421 **当前未监听** | —— | `Get-NetTCPConnection` 实测：两端口均不在监听表 ✅ |

### 9.3 端点与硬约束（第三、四节）

| 断言 | 证据 |
| --- | --- |
| `GET /notes/changes` | `http.rs:137-152`（路由）→ `db.rs:325-341`（`changes_all`）→ SQL 在 `db.rs:328` |
| `GET /notes/{id}` | `http.rs:154-163`（路由）→ `db.rs:316-319`（`get`）→ SQL 在 `db.rs:305-314` |
| 每条只有 3 个字段 | `http.rs:37-43`（`ChangeRow { id, updated_at, deleted_at }`） |
| 含已删除的 | `db.rs:328` 的 SQL **无 `WHERE`** ✅ |
| **无写接口** | `http.rs:5`（注释）· `http.rs:130-132`（非 GET → **405**） |
| **无鉴权** | `http.rs:7`（注释）· 全文件无 auth 代码 ✅ |
| **只绑回环** | `http.rs:82` ✅ |
| 「无写接口、无鉴权、只绑回环」原文 | 仓库根 `README.md:55` |

### 9.4 状态从哪来（第五节）

| 断言 | 证据 |
| --- | --- |
| 读的是同一个 SQLite 库 | `http.rs:135`（`let db = app.state::<Database>();`）—— 就是 `mod.rs:17` `app.manage(database)` 挂上去的那个 |
| 库路径 | `mod.rs:15` |
| 数据层是 `db.rs` | `db.rs`（**实测 51,678 B / 1,331 行**） |
| 「看到的是已落盘/已进 WAL 的数据」 | **推断** —— 依据：`db.rs:98` 设了 `journal_mode=WAL`；**WAL 里的内容对读连接可见** |
| ⚠️ 「`notes.db-wal` 里尚未 checkpoint 的事务影响可见性」 | ⚠️ **推断** —— **未实测**（只读连接上禁止发 `wal_checkpoint`） |

### 9.5 与外部软件的关系（第七节）

| 断言 | 证据 |
| --- | --- |
| 工作台**不走**这个接口 | `src-tauri/src/workbench/action.rs:8-26`（只调 `win.show()` / `win.hide()` / `set_focus()`，**无 HTTP**）· `main.rs:49-50` |
| **消费者 = `github_operater`（`src\watch_notes.py`）** | 实测（2026-10-03 收尾段，只读打开）：`:50` `BASE = "http://127.0.0.1:17321"` · `:51` `CHANGES_URL = f"{BASE}/notes/changes"` · `:77` `_get_json(f"{BASE}/notes/{note_id}")` · `:52` `INTERVAL = 3.0`（**每 3 秒**） |
| ⚠️ **本行原先写「消费者未查明」** | 那是第九站/D 段的结论。**收尾段已查明并改正** —— **本文件另一处（9.5 的这一行）原先与第七节的新结论矛盾，本段一并修正** |

---

## 十、未验证与订正

### 10.1 🔴 至今仍未实测的东西

| 项 | 状态 |
| --- | --- |
| **两个端点的真实 HTTP 报文** | 🔴 **一次都没测过**（第九站、第二十站 B 段、D 段，**三次都因禁止碰端口而未测**） |
| **`-wal` 的 checkpoint 状态** | 🔴 **未测**（只读连接上禁止发写语义 PRAGMA —— 见第二十站 A 段纪律） |
| 端口被占用时的真实行为 | ⚠️ **未测** —— 代码路径在 `http.rs:83-89`（绑失败 → 打印一行 → `return`） |

### 10.2 ⚠️ 一处口径变化（B 段之后）

| 位置 | 第九站时 | 现在 |
| --- | --- | --- |
| 第六节「验证方式」 | 写「**未实测**：两个端点的实际返回内容与格式」 | **仍然未实测**，但 **B 段已把字段表按源码写全** → 见 [OUTPUT-SLOT.md](OUTPUT-SLOT.md) 第三节 + 其「证据等级」一节 |

---

## 十一、🔴 2026-10-09 补记：**可达性变了**（消费者必读）

> **本节是第二十一站（窗口行为整改站）的补记。** 见 [log.md](log.md)。

### 11.1 变了什么

| 项 | 改动前 | 改动后 |
| --- | --- | --- |
| 点 × 之后 | 只调 `hide()`，**进程继续活着** | 🔴 **先落盘、成功后进程退出** |
| 17321 端口 | **几乎总是处于监听** | 🔴 **随进程退出而释放；只在窗口开着时可达** |

**改动前的实测（这不是推测）**：那一个便签进程 **PID 37072 从 2026-10-04 10:06:48 一直活到 2026-10-09**，
其间窗口是 `visible=False`（被 × 隐藏），**而 17321 一直可访问**（用 `Invoke-WebRequest` 实测返回 `HTTP 200`）。
→ **即"用户以为关了，服务其实还在"。**

**改动后的实测**：`%APPDATA%\com.myworkbench.notes\trace.log` 里三次关闭都留下 `ui quit: saved, exiting` + `db: quit_notes` 成对记录；
关闭后 `Get-Process my-workbench-notes` **查不到进程**。

### 11.2 对第一节「生命周期」那句话的修订

第一节写的是：「**随应用启动，随应用退出** —— ⚠️ 不是独立进程」。

**这句话现在才真正成立。** 在 2026-10-09 之前，"退出"指的是**进程结束**——而进程几乎从不结束，
所以实际行为是「**一旦启动过，就一直开着**」。**本节之后，它才等于字面意思。**

### 11.3 🔴 对消费者的影响（`github_operater`）

| 项 | 内容 |
| --- | --- |
| **消费者做法** | `src\watch_notes.py` **每 3 秒**轮询 `/notes/changes`（`service.md` 第七节的实测证据） |
| **改动前** | 只有用户显式杀进程才会连不上 —— **"连不上"是异常** |
| **改动后** | 🔴 **用户没开着便签就连不上 —— "连不上"变成常态** |
| **它现在必须能忍** | 不能把"连不上"当成致命错误；**要么重试，要么接受这段空窗** |

> ⚠️ **本节的可行性未验证**：消费者那份脚本**本轮没读、没测**（在 `github_operater` 项目下，禁止碰别的项目）。
> **只登记事实，不断言它会不会坏。** 完整规格见 [OUTPUT-SLOT.md](OUTPUT-SLOT.md)。

### 11.4 第二节端口表的口径

| 端口 | 「何时出现」一列现在应读作 |
| --- | --- |
| **17321** | 原来写「只要 Note 在跑」—— 而"在跑"现在**罕见**（用户开着窗口才在跑）。**"应用运行时常驻"仍然对，但"常驻"的时间窗口短了。** |
