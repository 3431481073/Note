# 数据

> **本项目在项目目录内没有【任何】数据。**

输入不在、输出不在、持久状态不在、缓存不在、日志也不在。`data/` 下五个分区
（`input/ output/ store/ cache/ logs/`）**全部为空** —— **这是刻意的，不是遗漏**
（依据 `project_manager/docs/PLAYBOOK.md` 第二节「例外三：数据家完全在项目之外」）。

> ### 🔴 `data/` 五区为空，是【刻意】的
>
> **`data/input` · `data/output` · `data/store` · `data/cache` · `data/logs` 五个目录确实存在，
> 但每一个都是空的（实测 2026-10-03：各 0 项）。**
>
> **真实数据不在这里 —— 它在 `%APPDATA%\com.myworkbench.notes\`**（见下一节）。
> **`data/` 五区的存在，表达的是一条【否定式事实】**：
> 「这个项目的五个数据分区都在，**但它们的作用是声明"这里没有数据"**」。
>
> **→ 不建"空壳标记文件"、不删这五个目录。** 空目录本身就是标记。

**实测佐证**（2026-10-03）：项目内**有** `data/`（**五区齐全，全空**）·
**没有** `*.db` · **没有** `*.env` · **没有** `misc/` 内容（`misc/` 也是空的）。
`.gitignore` 里那几条（`.env` / `*.db` / `*.sqlite` / `*.key` / `*.pem` / `id_rsa*` / `data/` / `models/`）
是项目自己写的**防御性规则** —— 注释注明"本仓库当前**没有**这些文件，以下是防御性的"。

---

## 一、真实数据在哪

```
C:\Users\asus\AppData\Roaming\com.myworkbench.notes\
```

（= `%APPDATA%\com.myworkbench.notes\`。目录名来自 `tauri.conf.json` 的
`identifier: "com.myworkbench.notes"`。）

### 实测规模（2026-10-03，只读）

| 项 | 值 |
| --- | --- |
| **总大小** | **4.39 MB** |
| **总文件数** | **17** |
| **总目录数** | **3** |

**顶层全部内容：**

| 条目 | 大小 | 说明 |
| --- | --- | --- |
| **`notes.db`** | **417,792 B（0.398 MB）** | 🔴 **全部便签**。最后写入 **2026-09-30 17:31:19** |
| **`notes.db-wal`** | **4,144,752 B（3.95 MB）** | SQLite 预写日志 —— **比主库大 10 倍**（应用在跑，WAL 尚未 checkpoint） |
| `notes.db-shm` | 32,768 B | SQLite 共享内存索引文件 |
| `config\` | 0.01 MB / 14 文件 | **运行时的配置副本** —— 由 `config.rs` 从项目 `config/` 或打包 resource 复制过来 |

> **注意这三个文件是一体的**：`notes.db` + `-wal` + `-shm` 是 SQLite 的 WAL 模式三件套。
> **只看 `notes.db` 的 398 KB 会严重低估实际数据量** —— 真正在用的是 **4.56 MB**。
> 备份/迁移时必须**三个一起拿，且要在应用关闭时**（开着的时候 WAL 里可能还有未落盘的事务）。

**谁写的**：`src-tauri/src/notes/db.rs`（51,678 B，整个 SQLite 数据层）；
路径由 `src-tauri/src/notes/mod.rs:15` 决定 —— `app.path().app_data_dir()?.join("notes.db")`。

---

## 二、删除它会发生什么

> **全部便签丢失，且本项目不持有任何备份。**
>
> 回收站能救回来的只有"软删除"的条目（在库里，`deleted_at IS NOT NULL`），
> **删掉整个目录 = 物理消失，连回收站里那些一起没。**

README 自己也写着：「便签数据存在 Tauri 的 appDataDir（`$APPCONFIG`），**不在仓库里** ——
**克隆后是一份空库**。」

---

## 三、项目外还有一处缓存（可再生）

```
C:\Users\asus\AppData\Local\com.myworkbench.notes\EBWebView\
```

**实测：41.04 MB / 447 文件 / 顶层只有 `EBWebView` 一个目录。**

WebView2 内核缓存（GPU cache / Code Cache / Local Storage 等），**可再生**，删了只是下次启动冷一次。

---

## 四、🔴 外部持久状态

### 1. 工作台的工具表里写死了本 exe 的绝对路径

**`history/DEVELOPMENT_LOG.md` §8.4 自己把它登记为待办：**

> - [ ] 便签 exe 绝对路径写死在 `tools.json` 里，**文件夹移动后需同步改**。

**两个 `tools.json`，别搞混：**

| 文件 | 角色 |
| --- | --- |
| `Note\config\tools.json`（171 B） | **本项目自己声明的工具清单**：`{"id":"notes","name":"便签","icon":"📝","command":"toggle_notes","enabled":true}`。这是**随包发布/复制到 `%APPDATA%` 的默认配置** |
| **`Workbench 侧的 tools.json`** | **工作台读取的那份**，里面写着便签 **exe 的绝对路径** → **搬动交付物就会失效** |

**失效模式**：**响亮失败** —— 从工作台点便签，启动不了。

### 2. 注册表

**本项目未使用注册表。** 与 `markdown-reader`（ProgID + 右键菜单）和 `Lattice`（右键菜单）不同 ——
本项目的 Windows 集成走的是 **`windows-sys` 的窗口 API**（窗口定位、点击穿透），**不写注册表**。

> 这点是由代码判定的：`src-tauri/src/notes/hittest.rs` 与 `window.rs` 用的是
> `SetWindowSubclass` / `WM_NCHITTEST` / `ScreenToClient`，没有 `winreg` 依赖。
> **但这只说明"没有主动写"** —— 未逐键排查注册表。

---

## 五、凭据

**无。** 本项目不含任何凭据文件，只读接口**明确不做认证鉴权**
（README：「**无写接口、无鉴权、只绑回环**」）。

所以 PLAYBOOK 步骤 6 的「凭据怎么处理」两条分支在本项目**都不适用**。

---

## 六、一处存疑（未证实）

`src-tauri/src/notes/config.rs` 的 release 分支：

```rust
#[cfg(not(debug_assertions))]
fn copy_default_config(app: &AppHandle, dst: &Path) -> Result<(), String> {
    if let Ok(res) = app.path().resolve("config", tauri::BaseDirectory::Resource) {
        if res.exists() {
            return copy_dir(&res, dst).map_err(|e| e.to_string());
        }
    }
    Ok(())      // ← 找不到就静默放过
}
```

**但 `tauri.conf.json` 的 `bundle` 段里没有 `resources` 字段。**

**推断**：全新安装的机器上 `Resource/config` 不存在 → `res.exists()` 为 false →
**静默返回 `Ok(())`** → 建出一个空 config 目录 → 首次 `read_seasons` 会失败
（读不到 `seasons.json`）。

**⚠️ 本机验证不了**：`%APPDATA%\...\config\` 早就存在（dev 模式建的 14 个文件），
`if !config.exists()` 为 false，**复制分支从来不执行**。

**这是发布问题，不是整理问题。** 已列进 [log.md](log.md) 未解决 2。

### 6.1 🔴 2026-10-14：上面这条已处置（另一半是实测撞出来的）

**改了两处**（见 [log.md](log.md) 第二十二站）：

| # | 改动 | 为什么 |
| --- | --- | --- |
| 1 | `tauri.conf.json` 的 `bundle` 段**补上 `resources: { "../config": "config" }`** | 上面那句"没有 `resources` 字段"是**真的** ⇒ 打包里根本没有默认配置 |
| 2 | release 分支由 `if !config.exists() { 复制 }` 改成 **每次都"补缺复制"**（`copy_dir_missing`，已存在的一律不动） | 🔴 **实测撞到**：`%APPDATA%\...\config\` 早就存在（14 个文件）⇒ 复制永不触发 ⇒ **后来新增的默认配置永远到不了真机**。实测：项目 `config\` 有 `sources.json`、`%APPDATA%\...\config\` 没有 |

**为什么"补缺"而不是"覆盖"**：运行时那份是**用户的**（`README.md` 10.4 说得很清楚，同机任何进程都能写）。
**人改过的主题不能被我盖回去**，所以只补不存在的文件。单测：`config.rs` 的
`copy_dir_missing_fills_gaps_but_never_overwrites`（已存在的 `seasons.json` 保持 `user-edited`）。

**已实测（2026-10-14，`tauri build --no-bundle`）**：映射落点就是 **`resource_dir/config`** ——
构建产物里实测有 `src-tauri\target\release\config\sources.json`（88 B）。
未打包运行（本机就是这种：exe 在 `target\release\`）时 `Resource` = **exe 所在目录** ⇒
`resolve("config", Resource)` 命中它 ⇒ 补缺复制把 `sources.json` 送进 `%APPDATA%\...\config\`。

**仍未验**：真机（release exe）上走完整条路 —— 见 [log.md](log.md) 第二十二站补记的「还没做」。

---

## 七、证据索引

> 格式 `文件:行号`，路径相对 `Note\`。行号是 2026-10-03 第二十站实测值。
> **本文件的数据类断言来自两类来源**：**只读查库/查目录**，和**读源码**。逐条标明。

### 7.1 数据家在哪（第一节）

| 断言 | 证据 |
| --- | --- |
| 数据目录 = `%APPDATA%\com.myworkbench.notes\` | `src-tauri/tauri.conf.json:5`（`"identifier": "com.myworkbench.notes"`）→ Tauri 2 的 `app_data_dir()` 规则 |
| 库文件名 = `notes.db` | `src-tauri/src/notes/mod.rs:15`（`app.path().app_data_dir()?.join("notes.db")`） |
| **`notes.db` 417,792 B** | **实测**（`Get-Item`） |
| **`notes.db-wal` 4,144,752 B** | **实测** |
| **`notes.db-shm` 32,768 B** | **实测** |
| 三个文件是 WAL 三件套 | `src-tauri/src/notes/db.rs:98`（`PRAGMA journal_mode = WAL;`）→ WAL 模式必有 `-wal` / `-shm` |
| 数据层全在 `db.rs` | `src-tauri/src/notes/db.rs`（实测 **51,678 B / 1,331 行**） |
| `config\` 是运行时副本 | `src-tauri/src/notes/config.rs:39-48`（debug：从 `current_dir()/config` 复制）· `:50-57`（release：从 `Resource/config` 复制） |
| 「WAL 尚未 checkpoint」 | **推断** —— 依据：主库 mtime 停在 `2026-09-30 17:31:19`，`-wal` mtime 是 `2026-10-03`，**WAL 比主库新**。**未实际检查 checkpoint 状态**（且只读连接上禁止发 `wal_checkpoint`） |
| 「克隆后是一份空库」 | 仓库根 `README.md:72`（原文） |

### 7.2 项目内零数据（文首 + 第二节）

| 断言 | 证据 |
| --- | --- |
| `data/` 五区存在且全空 | **实测**：`data\{cache,input,logs,output,store}` **各 0 项** |
| `misc/` 空 | **实测**：0 项 |
| 项目内没有 `*.db` | **实测**：无 |
| `.gitignore` 的防御性规则 | `.gitignore:12-23`（`.env` `.env.*` `*.db` `*.sqlite` `*.sqlite3` `*.key` `*.pem` `id_rsa*` `id_ed25519*` `data/` `models/` `browser_data/`） |
| 「注释注明『本仓库当前没有这些文件，以下是防御性的』」 | `.gitignore:9-11`（注释原文：`# 发布前实测：本仓库当前**没有**这些文件。以下是防御性的。`） |
| 回收站能救回来的只有软删除条目 | `src-tauri/src/notes/db.rs:557`（`UPDATE pages SET deleted_at = … WHERE id = ?1 AND deleted_at IS NULL`）· `db.rs:599`（`purge` 是真 `DELETE`） |
| **本项目不持有任何备份** | **实测**：项目内无 `*.bak` / 无备份目录；`_rollback` 在 `D:\MyProject\` 下（**项目外**） |

### 7.3 WebView2 缓存（第三节）

| 断言 | 证据 |
| --- | --- |
| `%LOCALAPPDATA%\com.myworkbench.notes\EBWebView\` | 目录名同 `identifier`（`tauri.conf.json:5`） |
| **41.04 MB / 447 文件** | **实测** |
| 「可再生」 | **推断** —— WebView2 内核缓存的标准行为。**本轮未删、未验证** |

### 7.4 外部持久状态（第四节）

| 断言 | 证据 |
| --- | --- |
| 工作台 `tools.json` 写死便签 exe 绝对路径 | `history/DEVELOPMENT_LOG.md` §8.4（原文，已逐字引用）—— ⚠️ **未打开工作台那份 `tools.json` 核实**（在 `D:\MyProject\Workbench\`，本轮禁止碰其他项目） |
| 本项目 `config/tools.json`（171 B） | **实测**：`Note\config\tools.json` **171 B** |
| 本项目不进注册表 | `src-tauri/Cargo.toml` **无 `winreg` 依赖**（实测：`Cargo.toml:10-22` 里没有）· `notes/hittest.rs:5-8`（只用 `windows-sys` 的窗口 API）· `notes/window.rs:30`（`win_hwnd()` 子类化） |
| 「未逐键排查注册表」 | ⚠️ **原文已自标** —— 本轮**也没查**（禁止改注册表；只读查询本轮也没做） |

### 7.5 凭据（第五节）

| 断言 | 证据 |
| --- | --- |
| 无凭据 | **实测**：项目内无 `.env` / `*.key` / `*.pem` / `id_rsa*` |
| 只读接口不做认证 | `src-tauri/src/notes/http.rs:7`（注释：「**无鉴权**。正因为无鉴权，端点必须只读」）· 全文件**无任何 auth 相关代码** |
| 「无写接口、无鉴权、只绑回环」 | 仓库根 `README.md:55`（原文）+ `specs/README` 那行；**代码佐证**：`http.rs:82`（`SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT)`）· `http.rs:130-132`（非 GET → 405） |

### 7.6 存疑项（第六节）

| 断言 | 证据 |
| --- | --- |
| `copy_default_config` 找不到就静默放过 | `src-tauri/src/notes/config.rs:86-94`（`#[cfg(not(debug_assertions))]` 分支，末尾 `Ok(())`） |
| `tauri.conf.json` 的 `bundle` 段没有 `resources` | `src-tauri/tauri.conf.json:36-45`（`bundle` 段只有 `active` / `targets` / `icon`） |
| 「本机验证不了」 | `src-tauri/src/notes/config.rs:53`（`if !config.exists()`）—— 本机 `%APPDATA%\...\config\` **已存在**（**实测**：14 个文件） |

---

## 八、2026-10-09 新增：`%APPDATA%` 里多了一个文件（`trace.log`）

> **本节是第二十一站（窗口行为整改站）的补记。** 见 [log.md](log.md)。

| 项 | 内容 |
| --- | --- |
| **文件** | 🆕 **`%APPDATA%\com.myworkbench.notes\trace.log`** |
| **与谁同目录** | **`notes.db`**（即第一节那个"数据家"）—— **但两者性质完全不同** |
| **是什么** | 应用的**运行日志**：启动 / 页面加载 / 前端初始化 / 建便签 / 退出 |
| **格式** | `<epoch_ms>\t<消息>`，逐行追加 |
| **🔴 有没有用户数据** | **没有。** 不记正文、不记标题、不记标签名、不记项目路径 —— 只记事件（`create_page` 只带 `id` 与 `kind`） |
| **删了会怎样** | ✅ **可以删**（下次启动会重建）。**它不是数据，是诊断设施** |
| **会不会自己清理** | ❌ **不会。无轮转、无上限**（增长极慢） |
| **完整说明** | **[runtime-log.md](runtime-log.md)** |

### 8.1 它与 `notes.db` 的区别（别混）

| 项 | `notes.db` | `trace.log` |
| --- | --- | --- |
| 是数据吗 | 🔴 **是**（唯一的持久数据，第二节讲"删了会怎样"） | ❌ **不是**（诊断设施，删了无损） |
| 有用户内容吗 | 🔴 **有**（全部便签） | ❌ **没有** |
| 备份吗 | 🔴 **项目自己不持有任何备份**（第二节） | 不需要 |

### 8.2 为什么放在 `%APPDATA%` 而不是项目里

**沿用本项目一贯的做法：项目内零数据**（本文第一节）。
日志落在 `app_data_dir()`，与数据库同处 —— **因此"删项目前必须知道"的清单里要多这一项**（见 [README.md](README.md) 第十一节）。
