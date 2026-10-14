# 输出槽（OUTPUT-SLOT）

> 🔴 **格式：`v2`（§8.4 机器可读格式）** —— **2026-10-11 从旧格式改写**。
> 按 `D:\MyProject\CLAUDE.md` **§8.4**：**程序只读 `## 我往外给` 下面那张两列表**；
> **固定表之后的一切（`## 详情` 及其下）程序不读。**
> 🔴 **认不出固定标题 = 报错，不许跳过。**
>
> ⚠️ **本槽的池子从 2 个扩到 7 个** ——
> 依据 §8：「凡是【你存下来的、别人可能读的数据】都是池子：**数据库 · 数据/输出目录 · 发布的文件 · 端口/HTTP · 二进制工具 · 配置文件**」，
> 且「**一个项目可以有很多个池子 —— 8 个很正常，不要只写一个**」。
> **原有内容一字未删**，全部在 [## 详情](#详情) 之下。

## 我往外给

| 锚点 | 是什么 |
| --- | --- |
| 127.0.0.1:17321 | 便签只读接口（HTTP，只有 GET） |
| ~/AppData/Roaming/com.myworkbench.notes/notes.db | 便签库（SQLite） |
| ~/AppData/Roaming/com.myworkbench.notes/trace.log | 运行日志（纯文本，无用户内容） |
| ~/AppData/Roaming/com.myworkbench.notes/config | 运行时配置与素材（主题色 / 12 个月水印） |
| D:\MyProject\Note\config | 随包发布的配置与素材（对工作台的自声明 + 主题 + 水印） |
| D:\MyProject\Note\services.yaml | 本项目声明的进程（§5）· ⚠️ **内容存疑**，见详情 §0.D |
| D:\MyProject\Note\docs | 对外接口：**文档**（六件套 + 三份槽，按 §8.5） |
| D:\MyProject\Note\dist | 前端构建产物 |
| D:\MyProject\Note\src-tauri\target\release\my-workbench-notes.exe | 便签主程序（二进制） |

## 详情

> ⚠️ **以下为自由说明，程序不读。** 原文档（2026-10-03 起，含 2026-10-09 补记）的内容**一字未删**。

### 0. 本表的三个口径 + 两次调整（2026-10-11）

#### 0.A 口径变化：从「一个接口」到「一串池子」

下面那段原文写的是「按 **§7**，一个输出槽只声明**三项**（① 我放到哪 · ② 放了什么 · ③ 怎么取）」。
**2026-10-11 起父规范改成 §8**：槽 = **池子**，要**一个一个列全**（`## 我往外给` 的表），字段与取法移入 `## 详情`。

**两种口径下"池子"不是一回事**：**表的 9 行 = 我有哪几个池子；下面 §一 的 2 个池子 = 17321 这个接口的两个侧面。**

#### 0.B 🔴 §8.4 新禁令带来的调整（同日第二次改写）

| 调整 | 依据 |
| --- | --- |
| `%APPDATA%\com.myworkbench.notes\...` → **`~/AppData/Roaming/com.myworkbench.notes/...`** | 🔴 **禁令 3**：项目外的路径统一用 `~` 开头，不用 `%VAR%` 写法 |
| 🆕 表里**补上 `D:\MyProject\Note\services.yaml`** | 🔴 **禁令 1**：凡是「我往外给什么」**必须在表里有一行** —— **只写在 `## 详情` 里程序读不到**。本项目根下确有 `services.yaml`，故必须进表 |
| 🆕 表里**补上 `D:\MyProject\Note\docs`** | §8.5：「一个项目的 `docs\` **就是它的对外接口**」。它是一个**具体的东西**，且不是项目目录根 |
| 未使用**通配锚点**（§8.4 第三种写法） | 本项目**没有**"横跨多个项目、逐个列会失效"的池子 —— 通配是给 `service-manager` 那种"吃 24 个项目的 `services.yaml`"用的 |

#### 0.C 锚点写法约定

| 形态 | 写法 | 例 |
| --- | --- | --- |
| 项目内绝对路径 | 反斜杠 | `D:\MyProject\Note\config` |
| 项目外路径 | **`~` 开头，正斜杠** | `~/AppData/Roaming/com.myworkbench.notes/notes.db` |
| 端口 | `127.0.0.1:xxxx` | `127.0.0.1:17321` |

> ⚠️ **`~` 展开的是用户目录**（`C:\Users\asus`），所以 `~/AppData/Roaming/...` = 原来的 `%APPDATA%\...`。
> 按 §8.4 的归一化规则**两种写法本来都能对上**，但**禁令 3 要求统一写成 `~`，不许指望程序兜底**。

#### 0.D ⚠️ `D:\MyProject\Note\services.yaml` 的内容存疑（未动）

按 **§5**，每个项目应在**自己根下**声明**自己的**进程。本项目根下**确有**这个文件，所以它**按禁令 1 进了表**。

🔴 **但它现在的内容不是本项目的进程** —— 里面声明的是 `svcmgr-serve` / `svcmgr-ui`，
那是 **`service-manager` 的**进程；**本项目的 `127.0.0.1:17321`（只读接口）反而不在里面**。

| 项 | 状态 |
| --- | --- |
| 该文件 | **本轮未改**（未跟踪、在途文件）—— 见 [log.md](log.md) 第二十一站·补记 第五节 |
| 表里的那一行 | **按事实登记**：本项目**确实往外给**这个文件；**内容对不对是另一回事** |

---

> **槽的格式版本：`v1`** —— ⚠️ **v1 目前只存在于本文**，响应报文里**没有任何版本字段**。
> 加版本号到线上属于**代码改动**，本轮**没做**（见 [附录 B](#附录-bv1-的落地状态)）。
>
> 🧭 **本文的结构**：按 `D:\MyProject\CLAUDE.md` §7，一个输出槽只声明三项 ——
> **① 我放到哪 · ② 放了什么 · ③ 怎么取**。正文即按这三项组织，其余为附录。
>
> **状态**：本轮（2026-10-03）**只读摸底 + 写规格**，未运行任何代码、未发任何 HTTP 请求。
>
> 🔴 **本文的字段表全部来自【读源码】。两个端点的实际返回报文【一次都没有实测过】**
> （本轮禁止项：禁止碰 17321 端口）。
> **所有类型/可空性都是从 Rust 类型定义推导的，不是从真实 JSON 观测的** —— 见 [附录 A](#附录-a证据等级)。

---

## 一、我放到哪

🔴 **本槽有两个池子** —— 一个**活的 HTTP 接口**（池子 1），一个**它背后的 SQLite 库文件**（池子 2）。
两者读的是**同一份数据**，但**取法不同、可用性不同**。

**池子 1 · 活的 HTTP 接口**

| 项 | 值 | 证据 |
| --- | --- | --- |
| **位置** | **`http://127.0.0.1:17321`** —— **固定端口，写死** | `src-tauri/src/notes/http.rs:30` `pub const PORT: u16 = 17321;` |
| **绑定** | **只绑回环** `SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT)` —— **局域网/公网连不上** | `http.rs:82` |
| **传输** | **手写 `TcpListener`**，不依赖任何 HTTP 库（无 axum/hyper/tiny_http） | `http.rs:8-11`（`use std::net::...`）+ 文件头注释 :9 |
| **监听范围** | `127.0.0.1` —— 实测监听表命中 | `Get-NetTCPConnection` 实测（PID 29608） |
| **生命周期** | **随应用启动，随应用退出**。**不是独立进程**，不能单独起停 | `src-tauri/src/notes/mod.rs:21` `http::spawn(app.handle().clone());` |
| **端口被占用时** | **只打印一行并放弃**，不影响应用其余功能 | `http.rs:85-88` |

**本槽的形态是**：不是文件、不是队列，**是一个"活的窗口"**（直接读同一份 SQLite）。
→ **INTERFACES.md 第二章「待讨论：槽物理上是什么」在这里有了第一个实答：可以先是一个进程内嵌的只读 HTTP。**

**池子 2 · 背后的 SQLite 库文件**

| 项 | 值 | 证据 |
| --- | --- | --- |
| **位置** | **`%APPDATA%\com.myworkbench.notes\notes.db`** | `src-tauri/src/notes/mod.rs:19` —— `app.path().app_data_dir()?.join("notes.db")` |
| **同目录** | `notes.db` ＋ **`notes.db-wal`** ＋ `notes.db-shm` | SQLite WAL 模式三件套。🔴 **`-wal` 比主库大**（实测 417,792 B vs **4,144,752 B**）—— **只复制 `.db` 会丢数据** |
| **绑定** | **只绑文件系统** —— **没有任何端口** | 同 [data.md](data.md) 第一节 |
| **可用性** | ✅ **应用没跑时也能读**（池子 1 不能 —— 它随进程生灭） | 对比上表「生命周期」行 |
| **写入方** | **只有 `Note` 自己**（同一个进程写） | `db.rs` 的全部写操作 |
| **权限** | **没有鉴权 —— 文件系统权限就是权限** | 与池子 1 的「无鉴权」同理 |

> 🔴 **这个池子此前没写进本文件**（2026-10-10 补）。
> 补它的理由：`_data\sync_notes.py` **每 30 分钟直读它一次**（`mode=ro`），
> 而在补之前——**光看槽推不出这条边**（见 `D:\MyProject\project_manager\MAP.md` 第二节）。

> 🔴 **生命周期在 2026-10-09 变过一次** —— 「关闭窗口」从"只隐藏、进程续活"改成"落盘后进程退出"，端口随之释放。
> 完整补记（含实测依据）见 [附录 F](#附录-f2026-10-09-补记第二十一站)。

---

## 二、放了什么

### 2.1 路由表（2 条，只有 GET）

| 方法 | 路径 | 用途 | 实现位置 |
| --- | --- | --- | --- |
| **GET** | `/notes/changes` | **全量变更清单**（轻量，只有 3 列） | `http.rs:137-152` |
| **GET** | `/notes/{id}` | **单条完整内容**（10 字段） | `http.rs:154-163` |
| — | 其他任意路径 | **404** `{"error":"not found"}` | `http.rs:165` |
| — | 非 GET | **405** `{"error":"read-only endpoint: only GET is allowed"}` | `http.rs:130-132` |
| — | `/notes/`（空 id） | **404** `{"error":"not found"}` | `http.rs:155-157` |
| — | id 不存在 | **404** `{"error":"no such note"}` | `http.rs:160` |
| — | 内部错误 | **500** `{"error":"<错误文本>"}` | `http.rs:150,161` |

#### 🔴 查询串被整个丢掉

```rust
// http.rs:134
let path = target.split('?').next().unwrap_or("");
```

**`?` 之后的一切被丢弃。**
→ **没有 `since` / `after` 参数**
→ **没有分页参数**
→ **没有过滤参数**

**每一次请求都是【全量下发的完整清单】。**

### 2.2 `GET /notes/changes` —— 全量变更清单

**SQL 原文**（`db.rs:328`，函数 `changes_all`，`db.rs:325-341`）：

```sql
SELECT id, updated_at, deleted_at FROM pages ORDER BY updated_at DESC, id DESC
```

🔴 **没有 `WHERE`** —— **含已删除（软删）的条目**。这是刻意的（`db.rs:323` 注释：「含已删除条目（`deleted_at` 非空），外部据此识别删除」）。

**返回**：**裸 JSON 数组**（不是带分页元信息的对象）—— `http.rs:140-148` 把 `Vec<ChangeRow>` 直接序列化。

| 字段 | 类型 | 含义 | 可空 | 证据 |
| --- | --- | --- | --- | --- |
| `id` | string | 便签 ID，**21 字符 nanoid**，字母表 `A-Za-z0-9-_` | **否** | `http.rs:40` (`id: String`) + `summary.rs:29-37`（nanoid 定义）+ `db.rs:454`（`nanoid(21)`） |
| `updated_at` | number (i64) | **最后修改时间**，Unix **毫秒** | **否** | `http.rs:41` (`updated_at: i64`) + `db.rs:134`（列定义 `integer not null`） |
| `deleted_at` | number \| null | **进回收站的时间**（Unix 毫秒）。`null` = **不在回收站** | ✅ **可空** | `http.rs:42` (`deleted_at: Option<i64>`) + `db.rs:135`（列定义 `integer`，无 not null） |

**序**：`updated_at DESC, id DESC`（`db.rs:328`）。

**当前体量**（实测，2026-10-03）：**152 条**。

**示例形状**（⚠️ **按类型定义构造，非实测报文**）：

```json
[
  { "id": "V0Nbgx2aLb4XFsi40iJbb", "updated_at": 1790982187410, "deleted_at": null },
  { "id": "AbCdEf0123456789_-xyz", "updated_at": 1790958815217, "deleted_at": 1790958815217 }
]
```

### 2.3 `GET /notes/{id}` —— 单条完整内容

**SQL 原文**（`db.rs:305-314`，函数 `get_locked`，经 `db.rs:316-319` 的 `get` 暴露）：

```sql
SELECT id, content, summary, created_at, updated_at, deleted_at, version,
       title, kind, tags, projects, from_view, nodes_json
FROM pages WHERE id = ?1
```

🔴 **`WHERE` 只有 `id`，没有 `deleted_at IS NULL`** —— **已进回收站的便签，仍可按 id 取到全文**。

**返回**：**单个 JSON 对象**（`FullNote`，`http.rs:45-59`）。

| 字段 | 类型 | 含义 | 可空 | 证据 |
| --- | --- | --- | --- | --- |
| `id` | string | 便签 ID（同 2.2） | 否 | `http.rs:49` |
| `title` | string | 标题。**可为空串**（152 条里只有 15 条有标题） | 否（**空串 ≠ null**） | `http.rs:50` + `db.rs:152`（`title text not null default ''`） |
| `summary` | string | **摘要**：正文前 **20 个 Unicode 字符**，换行→空格，去首尾空白 | 否（空串可能） | `http.rs:51` + `summary.rs:4-8` (`make_summary`) |
| `content` | string | **正文全文** | 否（空串可能） | `http.rs:52` + `db.rs:131` |
| `created_at` | number (i64) | 创建时间，Unix 毫秒 | 否 | `http.rs:53` + `db.rs:133` |
| `updated_at` | number (i64) | 最后修改时间，Unix 毫秒 | 否 | `http.rs:54` + `db.rs:134` |
| `deleted_at` | number \| null | 进回收站的时间；`null` = 不在回收站 | ✅ **可空** | `http.rs:55` + `db.rs:135` |
| `kind` | string | 类型。**实测当前 152 条全是 `"note"`**（无 `"outline"`） | 否 | `http.rs:56` + `db.rs:153`（`default 'note'`）+ 实测 `kind_note=152 / kind_outline=0` |
| `tags` | **string[]** | 🔴 **标签【ID】数组，不是名字** | 否（空数组可能） | `http.rs:57` + `db.rs:154` + `db.rs:185` 注释（「迁移成 tags（ID）+ card_tags，并把 `pages.tags` 改写为 **ID 数组**」） |
| `projects` | **string[]** | 🔴 **项目【绝对路径】数组**（如 `D:\MyProject\xxx`） | 否（空数组可能） | `http.rs:58` + `summary.rs:20-26`（`default_project_dir` = `D:\MyProject`）+ `commands.rs:109-124`（返回 `read_dir` 的完整路径） |

#### 🔴 刻意不对外发的字段

`http.rs:46` 的注释原文：

> **`Page` 里还有 version / from_view / nodes，刻意不往外交。**

| 被剔除的字段 | 它在库里是什么 | 剔除的影响 |
| --- | --- | --- |
| **`version`** | 每条便签自己的编辑计数（i64，实测范围 2–81） | 🔴 **消费者拿不到任何"这条比我上次拉的更新"的序号** —— 只能比 `updated_at` 整数 |
| `from_view` | 从哪个视图导入的（视图名） | 实测 152 条**全为空** |
| `nodes` | 结构树的整棵树（JSON） | 实测 152 条**全为空**（`kind` 全是 note） |

**依据**：`models.rs:18-35`（`Page` 结构体有 `version` / `from_view` / `nodes`）+ `http.rs:45-59`（`FullNote` 没有这三个）。

### 2.4 边界行为（三条硬边界 + 两条缺口）

#### 边界一 · 删除/恢复**不修改** `updated_at`

```sql
-- db.rs:556-560  soft_delete
UPDATE pages SET deleted_at = ?2, version = version + 1
WHERE id = ?1 AND deleted_at IS NULL

-- db.rs:572-576  restore
UPDATE pages SET deleted_at = NULL, version = version + 1
WHERE id = ?1 AND deleted_at IS NOT NULL
```

**两条 SQL 里都没有 `updated_at`。** 对比 `update`（`db.rs:525-530`）与 `save_outline`（`db.rs:908-912`），那两条**都写了 `updated_at = ?`**。

> 🔴 **后果**：**删除和恢复这两件事，在 `updated_at` 上完全不可见** ——
> 只能通过 `deleted_at` 是否为 `null` 来判断。
> **一个只比 `updated_at` 的消费者，会完全漏掉删除和恢复。**

#### 边界二 · `purge` **没有墓碑**

```sql
-- db.rs:599
DELETE FROM pages WHERE id = ?1
```

**是物理 `DELETE`，不是软删除标记。**（同事务里还删 `card_tags`，`db.rs:598`）

> 🔴 **后果**：彻底删除后，**该 id 从 `/notes/changes` 里【整个消失】** ——
> 没有"已删除"的记录留下。
> **消费者只能靠"上次有、这次没有"推断它被彻底删了。**

#### 边界三 · **无增量接口**

见 2.1 末尾：**查询串被丢弃**（`http.rs:134`），**SQL 无 WHERE**（`db.rs:328`）。
**每次调用 = 全量 152 条。**

> 🔴 **后果**：消费者要做增量，**只能自己在本地保存上一次的全量，然后 diff**。
> **服务端不提供任何形式的"只给我变化的部分"。**

#### 缺口一 · 🔴 **`tags` 是 ID，而 API 没有办法把 ID 换成名字**

- `/notes/{id}` 返回的 `tags` 是**标签 ID 数组**（`db.rs:185`）
- **整个槽只有 2 条路由** —— **没有列标签的端点**
- 唯一的标签来源 `get_meta_index` / `list_tags` 是 **Tauri 命令**（`commands.rs:26,36`），**不是 HTTP**

> **→ 外部消费者能拿到 `["jM8e8PtGCLV07g5uNrR3A", ...]`，但拿不到「AI使用体验」这个名字。**

#### 缺口二 · 🔴 **这个槽的消费者不清楚**

`services.yaml` 里未记录消费者（上一轮整理时记录为「**具体是谁未查明**」）。

> 📌 按 §7，**本槽【故意不写】"谁在用"** —— 关系是推出来的，不是写出来的。
> 想知道有没有消费者、是谁，看 `project_manager\MAP.md` 推。

---

### 2.5 池子 2 的表（SQLite schema）

> 读自 `src-tauri/src/notes/db.rs` 的建表与迁移语句（`migrate()`，**v1 → v4**）。**未连库核对。**

| 表 | 列 | 装什么 |
| --- | --- | --- |
| **`pages`** | `id` TEXT 主键 · `content` TEXT · `summary` TEXT · `created_at` INTEGER · `updated_at` INTEGER · `deleted_at` INTEGER NULL · `version` INTEGER · `title` TEXT · `kind` TEXT · `tags` TEXT(JSON) · `projects` TEXT(JSON) · `from_view` TEXT NULL · `nodes_json` TEXT NULL | 🔴 **一张表装两种东西**：`kind='note'` 是卡片、`kind='outline'` 是结构树（树存在 `nodes_json` 这个文本列里） |
| **`tags`** | `id` TEXT 主键 · `name` TEXT · `description` TEXT · `extinguished` INTEGER · `deleted_at` INTEGER NULL | 标签（可"熄灭"、可软删） |
| **`card_tags`** | `card_id` TEXT · `tag_id` TEXT · 主键 `(card_id, tag_id)` | 卡片 ↔ 标签，**多对多** |
| **`schema_migrations`** | `version` INTEGER 主键 · `applied_at` INTEGER | 迁移版本（当前 **v4**） |

**索引**：`idx_active` · `idx_trash`（两个都是**部分索引**）· `idx_card_tags_tag` · `idx_tags_active_name`（**唯一**，只约束未熄灭标签）

**过滤条件**（`db.rs:16-20`，**四者共用一套** —— 保证计数与列表永远一致）：

| 常量 | 条件 |
| --- | --- |
| `NOTE_ACTIVE` | `deleted_at IS NULL AND kind='note'` |
| `NOTE_TRASH` | `deleted_at IS NOT NULL AND kind='note'` |
| `OUTLINE_ACTIVE` | `deleted_at IS NULL AND kind='outline'` |
| `OUTLINE_TRASH` | `deleted_at IS NOT NULL AND kind='outline'` |

> 🔴 **`pages` 没有"表"这一层隔离** —— 想只看卡片**必须自己带 `kind='note'`**；
> 只看未删的还要加 `deleted_at IS NULL`。**两样都不带 = 卡片、结构树、回收站一起出来。**

---

## 三、怎么取

| 项 | 值 | 证据 |
| --- | --- | --- |
| **读取方式** | **HTTP GET + JSON**（`Content-Type: application/json; charset=utf-8`） | `http.rs:193` |
| **权限** | 🔴 **无鉴权**。设计如此：因为无鉴权，端点必须只读 | `http.rs:7` 注释原文：「**无鉴权**。正因为无鉴权，端点必须只读、且只暴露便签内容本身」 |
| **写入能力** | 🔴 **零**。只有 `GET`，**没有任何写路由** | `http.rs:5`（「只有两个 GET 端点，没有任何写路由」）+ `http.rs:130-132`（非 GET 一律 405） |
| **连接模型** | 每连接一线程；**`Connection: close`，无 keep-alive**；不读请求体 | `http.rs:17` 注释 + `http.rs:96-98` + `http.rs:196` |
| **超时** | 单连接总超时 **5 秒**（读写各 5s） | `http.rs:35` `IO_TIMEOUT = Duration::from_secs(5)` + `:104-105` |
| **单行上限** | **8192 字节**（防畸形请求撑爆内存） | `http.rs:33` `MAX_LINE: u64 = 8192` |
| **缓存** | **`Cache-Control: no-store`** —— 明确不许缓存 | `http.rs:195` |

**池子 2 怎么取**（与上表无关，是另一个池子的取法）：

```python
import sqlite3
con = sqlite3.connect(r"file:C:/Users/<你>/AppData/Roaming/com.myworkbench.notes/notes.db?mode=ro", uri=True)
rows = con.execute("SELECT id, title, updated_at FROM pages WHERE deleted_at IS NULL AND kind='note'").fetchall()
con.close()
```

> 🔴 **必须带 `mode=ro`** —— 这是唯一一条"没人在管"的边界（池子 1 是只读设计，池子 2 靠你自己守）。
> 🔴 **应用在跑时 WAL 未 checkpoint** ⇒ 你可能读到**旧快照**。要取"当下的"，
> 要么等应用退出，要么接受延迟。
| **ETag** | **无** | 全文件无 ETag 相关代码 |

**取法汇总**：对**固定的两条路由**发 **HTTP GET**（2.1），拿 JSON（2.2 / 2.3）。
**无鉴权、无写、无缓存、无增量**（增量见 2.4 边界三）。

> 🔴 **取之前得确认应用正开着** —— 2026-10-09 起，关闭窗口即退出进程、端口随之释放，
> **不再有"窗口隐藏但服务还活着"的窗口期**。见 [附录 F](#附录-f2026-10-09-补记第二十一站)。

---

## 附录 A·证据等级

| 等级 | 内容 |
| --- | --- |
| ✅ **读源码确证** | 端口号、绑定地址、路由、SQL 原文、字段名、字段类型、`Option<i64>` 的可空性、HTTP 状态码、响应头、超时与上限、边界三条 |
| ✅ **只读查库确证** | `pages` 表结构（13 列）、`kind` 全为 note、`nodes_json` 全空、152 条 |
| 🔴 ****未实测**** | **两个端点的真实 JSON 报文** —— 键名拼写（snake_case 无 rename）、数组形状、状态码的真实返回、`Content-Length`、序列化时 `null` 的实际表现 |
| ⚠️ **推导，非实测** | 2.2 的示例 JSON **是按类型定义手写的**，不是抓包 |

**为什么键名可以说"是 snake_case"**：`ChangeRow`（`http.rs:37-43`）与 `FullNote`（`http.rs:45-59`）都**只有 `#[derive(Serialize)]`，没有任何 `#[serde(rename_all = ...)]`** —— 所以 serde 用字段名原样输出。**这是读源码的推论，不是实测。**

---

## 附录 B·`v1` 的落地状态

**本规格定义 `v1`。但 `v1` 这个字符串，在当前的代码、响应、配置里【一次都不出现】。**

| 想加在哪 | 现状 | 代价 |
| --- | --- | --- |
| 响应体加 `{"version":"v1", ...}` | **没加** | 改 `http.rs`，**且会改变响应形状**（现在是裸数组）→ **破坏现有任何消费者** |
| 响应头加 `X-Slot-Version: v1` | **没加** | 改 `http.rs` 的 `respond()`，**不改响应体** → 对消费者更安全 |
| 只写在本文档里（**当前做法**） | ✅ **就是现在这样** | **远端无法自证版本** —— 消费者只能靠文档 |

> **本轮不做代码改动。** v1 是否要上到线上、以哪种形式，**是下一轮的决定**。

---

## 附录 C·和 `INTERFACES.md` 的对应

`project_manager/docs/INTERFACES.md` 第五章要求声明四个字段。逐条对照：

| 字段 | 本槽的值 | 状态 |
| --- | --- | --- |
| **位置** | `http://127.0.0.1:17321`（固定端口，写死） | ✅ 已定 |
| **格式** | JSON。`/notes/changes` = 对象数组（3 字段）；`/notes/{id}` = 对象（10 字段） | ✅ 已定 |
| **读取方式** | HTTP GET（轮询）。**无推送、无订阅** | ✅ 已定 |
| **生命周期** | **未定义** —— 数据不落在槽里，槽是**实时读库的视图**；「槽里留多久」这个问题在本槽上**不适用** | ⚠️ **待定** |

**本槽的形态是**：不是文件、不是队列，**是一个"活的窗口"**（直接读同一份 SQLite）。
→ **INTERFACES.md 第二章「待讨论：槽物理上是什么」在这里有了第一个实答：可以先是一个进程内嵌的只读 HTTP。**

---

## 附录 D·未解决的（交接给下一轮）

1. 🔴 **两个端点的真实报文从未实测** —— 所有消费者之前，先 `curl.exe http://127.0.0.1:17321/notes/changes` 验一次。
2. 🔴 **`tags` 是 ID，没有解析成名字的途径** —— 消费者拿到的是不可读的字符串。
3. 🔴 **无版本字段** —— 远端无法自证是 v1。
4. **`purge` 无墓碑** —— 消费者要靠消失推断，且**可能把"被彻底删"误判为"从未存在"**。
5. **删除/恢复不改 `updated_at`** —— 只比时间戳的消费者会漏掉这两类事件。
6. **`vite.config.ts` 的 `host: "0.0.0.0"`** 是另一个端口（1420）的事，**与槽无关**，另行记录。

---

## 附录 E·与 `INTERFACES.md` 第三章三条硬约束的对照

> `project_manager/docs/INTERFACES.md` 第三章是**已定稿**的（不是待讨论项）。
> 本节逐条核对 **`Note` 是否满足**。核对对象是**整个 `Note` 项目**，不只是本槽。

### 约束一 · 项目之间不通过代码耦合

**原文禁止三件事**：不许 `import` 别的项目的包 · 不许硬编码别的项目的绝对路径 · 不许读别的项目的内部文件。

| 子项 | 判定 | 证据 |
| --- | --- | --- |
| 不许 `import` 别的项目的包 | ✅ **满足** | 前端只 import npm 包与自身模块；后端 `use` 只有 crate 内部 + `tauri` / `rusqlite` / `notify` / `serde` / `rand` / `thiserror` / `windows-sys`（`Cargo.toml:10-22`）。**没有 `sys.path` 式的跨项目引入** |
| 不许硬编码别的项目的绝对路径 | ⚠️ **满足，但有一处边界** | 全项目**没有任何一处写出别的项目名**。**但** `summary.rs:20-26` 用**编译期** `env!("CARGO_MANIFEST_DIR")` 上溯两级得到 `D:\MyProject` —— **它是"这些项目的公共父目录"，不是某一个项目** |
| 不许读别的项目的内部文件 | ⚠️ **满足，但有一处边界** | 见下 |

**🔴 边界情况（这是本次核对新挖出来的）**：

```rust
// summary.rs:20-26
pub fn default_project_dir() -> Option<String> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().and_then(|p| p.parent())      // → D:\MyProject
        ...
}

// commands.rs:109-124  list_project_folders
std::fs::read_dir(&root)                        // 读 D:\MyProject 的子目录
    if p.is_dir() { dirs.push(p.to_string_lossy().to_string()) }
```

**它读的是【同级项目的目录名】，并把它们的【绝对路径】存进自己的库**
（`pages.projects` 列，`db.rs:155`），**再通过本槽对外发出**
（`/notes/{id}` 的 `projects` 字段，`http.rs:58`）。

| 判定 | 理由 |
| --- | --- |
| ✅ **不违反约束一** | ① 没有点名任何项目；② 只读目录项（`p.is_dir()`），**没有读别人的内部文件**；③ 存的是**用户在界面上选的**路径（`ProjectPicker.tsx`） |
| 🔴 **但它是一条真实的跨项目耦合** | **本槽的输出里带着同级项目的绝对路径** —— 换机器、换盘符，这些字符串就全失效 |

**→ 约束一判定：✅ 满足（含一条已记录的边界情况）。**

### 约束二 · 每个项目必须声明自己的输入和输出

**原文**：「必须声明这件事现在定死 …… **即使某个维度是空的，也要声明"这里是空的"**」。

| 维度 | 声明了吗 | 在哪 | 判定 |
| --- | --- | --- | --- |
| **输出** | ✅ **声明了** | **本文件**（我放到哪 / 放了什么 / 怎么取 / 边界） | ✅ **满足** |
| **输入** | 🔴 **没有声明** | —— | 🔴 **不满足** |

**→ 约束二判定：⚠️ 部分满足 —— 输出已声明，🔴【输入没有声明】。**

**补法（不在本轮做）**：按 `INTERFACES.md` 的路子，输入也应有对应声明。
**本槽是"活的、无存储的槽"**，输入侧则**不是槽**（就一个 JSON + 12 个 SVG + 人敲的字），
**要不要写成 `INPUT-SLOT.md`、还是并进本文件，待议。**

**`Note` 的输入是什么**（实测，来自读源码）：

| 输入 | 从哪来 | 证据 |
| --- | --- | --- |
| **便签正文 / 标题** | **人在界面上敲** | `Paper.tsx:69-85` |
| **`config\seasons.json`** | 项目内 `config\`（dev）或打包 resource（release），**运行时复制到 `%APPDATA%\...\config\`** | `config.rs:32-60` |
| **`config\assets\watermark\*.svg`**（12 个） | 同上 | `tauri.conf.json:32`（`assetProtocol.scope`）+ 实测 12 个 SVG |
| **`config\tools.json`** | 同上 | `config\tools.json`（171 B） |
| **项目目录**（用户在 `D:\MyProject` 下选） | `list_project_folders` → 系统文件对话框 | `commands.rs:79-88, 109-124` |
| **外部图片/文件** | 🔴 **没有** —— 便签不支持附件 | 全源码无非文本输入 |

> 📌 **`:355-356` 原记的「输入侧要不要写成 `INPUT-SLOT.md`、还是并进本文件，待议」—— 2026-10-09 已处置：写成了 [INPUT-SLOT.md](INPUT-SLOT.md)（见 [附录 F](#附录-f2026-10-09-补记第二十一站) F.3）。**
> **但本节不改上面这张判定表** —— 那是原作者的结论，按本项目纪律（[LESSONS.md](LESSONS.md) #23）**不替别人改结论**，只在 [附录 F](#附录-f2026-10-09-补记第二十一站) 声明事实。

### 约束三 · 消费者只管自己的输入，不管生产者是什么

这条约束**约束的是消费者**，生产者侧能做的是**把契约固定下来**。

| 视角 | 判定 | 理由 |
| --- | --- | --- |
| **生产者侧（`Note`）** | ⚠️ **部分满足** | 位置 / 格式 / 读取方式**都固定了**（本文「我放到哪 / 放了什么 / 怎么取」三节）；**但有一处不达标 —— 见下** |
| **消费者侧** | ⚪ **不适用** | **消费者还没接**（§7 下本槽不登记"谁在用"），无法评估 |

**🔴 不达标的那一处，正是 2.4 的「缺口一」**：

> `/notes/{id}` 返回的 `tags` 是 **Note 自己的 `tags` 表主键**（21 字符 nanoid）。
> **消费者想把它变成人能读的名字，唯一的办法是去读 `Note` 的内部表 —— 而本槽没有提供任何别的途径。**

**这直接撞上约束三的精神**：「取数据的人**不需要知道数据怎么产生的**」——
**而这里，取数据的人【必须】知道 `Note` 内部有个 `tags` 表、以及它的 ID 长什么样。**

**→ 约束三判定：⚠️ 部分满足 —— 契约固定了，但 `tags` 字段把生产者内部泄漏给了消费者。**

### 汇总

| 约束 | 判定 |
| --- | --- |
| **一 · 不通过代码耦合** | ✅ **满足**（含一条已记录的边界：输出里带同级项目的绝对路径） |
| **二 · 必须声明输入和输出** | ⚠️ **部分满足** —— 输出已声明，🔴 **输入未声明**（判定表见上，2026-10-09 已写 [INPUT-SLOT.md](INPUT-SLOT.md)，但本表不改结论） |
| **三 · 消费者不管生产者内部** | ⚠️ **部分满足** —— 契约固定，🔴 **`tags` 泄漏内部主键** |

**一条缺口指向的根因**：**这个槽是从"本项目自己用"演进出来的**，
`tags` 直接透了内部 ID —— **它是事实上的"槽"，但还不是【按槽的标准设计】的槽。**

> 📌 本轮**不改 `Note`**（用户拍板）。这条缺口记为**范式待办「槽的自洽性」**。

---

## 附录 F·2026-10-09 补记（第二十一站）

> ⚠️ **本节按 `D:\MyProject\CLAUDE.md` §7 的要求补写：输出有变化 → 当场更新槽。**
> **变化不在字段上**（两条路由、10 个字段一字未改），**而在"什么时候取得"。**

### F.1 🔴 槽的可用性变了 —— 消费者必读

| 项 | 改动前 | 改动后 |
| --- | --- | --- |
| 点 × 之后 | 只 `hide()`，**进程继续活着** | 🔴 **落盘成功后进程退出** |
| **`127.0.0.1:17321` 可达吗** | **几乎总是可达** | 🔴 **只在应用窗口开着时可达；关掉即不可达** |

**本文「怎么取」那一项现在要加一句**：**取之前得确认应用正开着**，否则连不上。

**实测依据（不是推断）**：

| 事实 | 证据 |
| --- | --- |
| 改动前：窗口隐藏而端口仍活 | 那一个进程 **PID 37072 从 2026-10-04 10:06:48 活到 2026-10-09**，其间 `visible=False`，而我实测 `Invoke-WebRequest http://127.0.0.1:17321/notes/changes` **返回 HTTP 200** |
| 改动后：关闭即退出、端口释放 | `%APPDATA%\com.myworkbench.notes\trace.log` 三次关闭均留下 `ui quit: saved, exiting` + `db: quit_notes`；关闭后 `Get-Process` 查不到进程 |

**对"什么时候有得取"的影响**：**改动前"连不上"是异常，改动后"连不上"是常态。**
详见 [service.md](service.md) 第十一节。
⚠️ **本轮只登记槽这一侧的事实，未读、未测任何消费者脚本**（在别的项目下）。

### F.2 契约本身未变

| 项 | 状态 |
| --- | --- |
| 路由（2 条，只有 GET） | ✅ 未变 |
| 字段表（`changes` 3 字段 / `{id}` 10 字段） | ✅ 未变 |
| 三条硬边界（删除不改 `updated_at` / `purge` 无墓碑 / 无增量接口） | ✅ 未变 |
| 两条缺口（`tags` 是内部主键 / 消费者不清） | ✅ 未变 |

**→ 所以本槽的「放了什么」与「怎么取」的取法都没变，只是"什么时候有得取"变了。**

### F.3 附录 E「约束二」的缺口已补

附录 E 把「输入侧要不要写成 `INPUT-SLOT.md`」记为**待议**，判定是「🔴 **输入未声明**」。

**2026-10-09 已处置：写成了 [INPUT-SLOT.md](INPUT-SLOT.md)**（结论：**本项目无输入槽**），并补登了两条"非槽但能改变行为"的通道。
