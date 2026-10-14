# 整理记录 · 2026-10-03

> 依据 `project_manager/docs/PLAYBOOK.md`。
> **代码零改动，运行中的进程全程未中断，`git` 未做任何操作。**

---

## 一、背景

`Note`（便签）是保留项目、**用户说"非常重要"**，从未整理过。

摸底定下的四件事：

1. **8,536.67 MB 里 99.0% 是构建缓存** —— `src-tauri\target\` 占 8,452.03 MB（其中 **`debug\` 6,193.11 MB + `release\` 2,258.93 MB**）。
   **真正不可再生的只有 0.58 MB（0.007%）。**
2. **它是一个移植项目** —— 把一份叫「便签 v12」的单文件 HTML 应用移植进一个已有的 Tauri 应用。6 份文档构成一条完整的移植链。
3. **它正在运行** —— PID 29608，exe 在 `src-tauri\target\release\` 里，**监听固定的 17321 端口**。
4. **P-1 反模式是有意选择** —— 见 [deps.md](deps.md) 第三节。**本次不改代码。**

**用户拍板**：代码零改动（含 `Cargo.toml`）· 只删 `target\debug\` · `123.txt`/`456.txt` 搬进 `docs\` 保持原名 · `test\` 留根 · `config\` 留根 · **绝不跑 `git clean`**。

---

## 二、结构前后对照

### 删除 / 移动 / 新建

| # | 动作 | 原位置 | 现位置 |
| --- | --- | --- | --- |
| 1 | **删除** | `src-tauri\target\debug\`（9,105 文件 / 6,193.11 MB） | —— |
| 2 | **移动** | `DEVELOPMENT_LOG.md`（42,756 B） | `docs\DEVELOPMENT_LOG.md` |
| 3 | **移动** | `IMPLEMENTATION_PLAN.md`（14,502 B） | `docs\IMPLEMENTATION_PLAN.md` |
| 4 | **移动** | `PORTING_REPORT.md`（16,402 B） | `docs\PORTING_REPORT.md` |
| 5 | **移动** | `UI_SPEC.md`（18,694 B） | `docs\UI_SPEC.md` |
| 6 | **移动** | `123.txt`（35,671 B） | `docs\123.txt`（**保持原名**） |
| 7 | **移动** | `456.txt`（9,693 B） | `docs\456.txt`（**保持原名**） |
| 8 | **新建** | —— | `docs\{README,run,data,deps,service,log}.md` |
| 9 | **新建** | —— | `data\{input,output,store,cache,logs}\`（**五区全空**）· `misc\` · `services.yaml` |

**搬迁方式**：6 份用 `Move-Item` 逐个搬，逐份核 SHA256。

### 明确**未搬**的

| 位置 | 为什么不动 |
| --- | --- |
| **`src-tauri\target\release\`** | 🔴 **运行中的 exe 在里面**，且含手动备份 `my-workbench-notes.exe.bak-20261001` |
| `src-tauri\`（其余） | Tauri 约定位置 |
| **`config\`** | 🔴 **既在源码里、又被运行时复制出去当默认值** —— 搬了 debug 分支就断（见第五节） |
| `src\` `test\` `node_modules\` `dist\` | 四区已覆盖 / 约定位置 / 依赖环境 |
| `index.html` `notes.html` | vite 双入口，被 `rollupOptions.input` 写死 |
| `package.json` `package-lock.json` `tsconfig.json` `vite.config.ts` | 工具约定位置 |
| `README.md` `LICENSE` | 「位置即语义」（GitHub 从仓库根读） |
| `.git\` `.gitignore` | 仓库根 |

### 体积对照

| | 整理前 | 整理后 |
| --- | --- | --- |
| **总计** | **8,536.67 MB / 17,341 文件** | **2,343.61 MB / 8,243 文件** |
| `src-tauri\target\` | 8,452.03 MB / 14,048 文件 | **2,258.94 MB / 4,943 文件** |
| **不可再生部分** | 0.58 MB / 80 文件 | **0.61 MB / 86 文件**（+6 份新文档） |
| **D 盘剩余** | 7.282 GB | **12.411 GB** |

---

## 三、代码改动清单

> # **代码改动 = 无。**

**理由**：

| 检查项 | 结果 |
| --- | --- |
| 搬的 6 份 `.md`/`.txt` 被代码引用吗 | **不被任何代码引用** |
| `src\` / `src-tauri\src\` 动了吗 | **没动**（`src` 28 文件、`src-tauri\src` 13 文件，前后一致） |
| `config\` 动了吗 | **没动**（14 文件，前后一致） |
| 配置文件动了吗 | `package.json` / `tsconfig.json` / `vite.config.ts` / `tauri.conf.json` / `Cargo.toml` **一行未改** |

**🔴 特别声明：`Cargo.toml` 里的 `custom-protocol` 写法没有改。**
它是**有意选择**（见 [deps.md](deps.md) 第三节），不是笔误。**在没有跑过一次 `tauri dev` 之前，改它是猜，不是修。**

---

## 四、模型路由

> # 对 `_shared/models/` 与 `_shared/routes.yaml` **零贡献**。

本项目**没有任何模型文件** —— 无权重、无数据集、无 AI 调用。
（`.gitignore` 里的 `models/` 是**防御性**规则，注释写明本仓库没有。）

**跳过也是一种结论。**

---

## 五、验证结果表

| # | 验证项 | 方法 | 证伪标准 | 结果 |
| --- | --- | --- | --- | --- |
| V-1 | **备份完整** | 备份目录逐项核验 | 缺文件或 SHA 不符 | ✅ **71 文件 / 0.44 MB**（6 md/txt + 9 根散文件 + `src` 28 + `src-tauri\src` 13 + `config` 14 + `test` 1） |
| V-2 | **🔴 `http.rs` 已备份**（git 未跟踪） | SHA256 比对 | 不备份 → 丢失即不可恢复 | ✅ 源与备份**均为 `A8C99A91…A378`** |
| V-3 | **6 份文档搬迁一致** | 搬前搬后 SHA256 | 不符 → 内容被改坏 | ✅ 六份全部一致 |
| V-4 | **`target\debug` 与运行中的 exe 不在同一子树** | 路径比对 | 若在同一子树 → 可能误删活文件 | ✅ 是兄弟目录 |
| V-5 | **删前确认 release 完好** | 存在性 + 大小 | exe 不在 → 不能删 | ✅ `target\release\my-workbench-notes.exe` = **10,526,720 B** |
| V-6 | **用户进程未中断** | `Win32_Process` + 端口 | PID 变或端口消失 | ✅ 仍是 **PID 29608**，`ExecutablePath` 未变，**17321 仍在监听** |
| V-7 | **`target\release` 完好** | 删后复核 | 被误删 | ✅ 2,258.93 MB / 4,941 文件，exe 大小未变 |
| V-8 | **`.bak-20261001` 未动** | 存在性 | —— | ✅ 仍在（10,493,952 B） |
| V-9 | **文件数守恒** | 推算 17,341 − 9,105 + 7 = **8,243** | 不符 → 有文件丢失或多出 | ✅ 实测 **8,242**（写 `log.md` 前）+ 本文件 = **8,243** ✓ |
| V-10 | **`data\` 五区全空** | 逐区计数 | 非 0 → 与例外三矛盾 | ✅ 各 **0 文件** |
| V-11 | **编码未写坏** | 逐字节验 UTF-8 / 查 BOM | 出现 BOM 或非法序列 | ✅ 全部合法 UTF-8、**无 BOM** |
| V-12 | **未做 git 写操作** | 复核 | —— | ✅ **未跑 `git add` / `commit` / `clean` / `checkout`** —— 只跑过 `git status`（只读） |
| V-13 | **未改动其他项目** | 仅写入本项目 + `project_manager` + 根地图 | —— | ✅ |

### ⚠️ 一处未能完全对上的数字（如实记录）

- `target\debug\` **删除前实测体积 = 6,193.11 MB（6.05 GiB）**
- **D 盘可用空间只涨了 5.129 GB**（7.282 → 12.411）
- **差约 0.92 GiB 未能归因。**
- 可能原因：NTFS 硬链接（cargo 在 `target/debug/deps/` 下会产生硬链接，逻辑大小会重复计数）、
  或删除窗口期 `D:` 上其他项目的写入（**本机同时跑着 Lattice 的 9 个进程**）。
- **可确认的是**：项目自身的体积差与删除量一致（8,536.67 → 2,343.60），删除动作本身无异常。

---

## 六、未解决与拿不准

1. **🔴 P-1 反模式：本次未测 `tauri dev`，待以后验证。**
   `Cargo.toml` 把 `custom-protocol` 写死在 `[dependencies]` 里（**有意**，见 `DEVELOPMENT_LOG.md` §8.3/§10.1）。
   按机制推演，它会让 `tauri dev` **加载 `dist/` 而不是 `localhost:1420`** —— 即**热重载失效**。
   **但本轮禁止运行代码，无法实测。**
   **判定方法**：跑一次 `npm run tauri dev`，看 webview 加载的是哪个 URL。
   **可能的修法**（P-1 推荐，能同时满足两边）：改成
   `[features] default = ["custom-protocol"]` + `custom-protocol = ["tauri/custom-protocol"]`。
   **但改之前必须先测** —— 现在改是猜。

2. **`bundle.resources` 缺失 —— `config.rs` 的 release 分支可能静默失败。**
   `copy_default_config` 从 `BaseDirectory::Resource` 找 `config`，找不到就 `Ok(())`；
   而 `tauri.conf.json` 的 `bundle` 段**没有 `resources` 字段**。
   **推断**：全新安装的机器上，首次启动会建出**空的 config 目录**，随后 `read_seasons` 失败。
   **本机永远走不到那条分支**（config 早就存在了），**无法证实也无法证伪**。
   **这是发布问题，不是整理问题。**

3. **数据口的实际消费者是谁 —— 未查明。**
   工作台不走 HTTP（它调 Tauri 命令 `toggle_notes`）。那 `/notes/changes` 到底有没有人在调，**本次没查**。

4. **🔴 `http.rs` 是 git 未跟踪的新文件 —— 这是本轮最该记住的一件事。**
   `git status` 显示 4 条未提交：`M README.md`、`M src-tauri/src/notes/db.rs`、`M src-tauri/src/notes/mod.rs`、
   **`?? src-tauri/src/notes/http.rs`**。
   它们是 **2026-10-01 18:56 写的只读 HTTP 接口** —— **即运行中那个 17321 服务的实现**。
   **仓库只有一个 commit（`e62511f`）**，所以 **git 在这里不是可靠的备份**。
   本次**已把它单独备份**。

5. **编译期路径：`env!("CARGO_MANIFEST_DIR")`。**
   `summary.rs:20-26` 的 `default_project_dir()` 由编译期常量算出来，结果落在 **`D:\MyProject`**（上溯两级）。
   **`grep` 源码搜 `D:\` 搜不到它**，但它把项目和位置绑在了一起。
   本次**未改动**（它只是个"新建便签的默认项目"提示值，不是硬依赖）。
   已补进 PLAYBOOK 第五节。

6. **`123.txt` / `456.txt` 是否该改名 —— 未动。**
   它们现在是 `docs\123.txt` / `docs\456.txt`，**保持原名**（用户拍板）。
   但 **README 的文档表里它们是正式条目**，名字却是 `123`/`456`。**是否改名留给以后。**

7. **`my-workbench-notes.exe.bak-20261001`（10,493,952 B）的用途不明。**
   它躺在 `target\release\` 里，与当前 exe 差 32,768 B。**本次未动。**
   可能是发布前的手动留档 —— **但没有任何文档提到它**。

8. **`vite.config.ts` 的 `host: "0.0.0.0"`** —— dev server 绑所有网卡。
   本机开发无所谓，**但这是本次整理发现的一处攻击面**（dev 模式会把源码暴露给局域网）。
   **未改**（改了会影响开发行为）。

---

## 七、未改动清单

**以下内容本次一行未动（实测佐证见第五节）：**

- **`src-tauri\target\release\`** —— 含运行中的 exe 与 `.bak-20261001`
- **`src\`（28 文件）· `src-tauri\src\`（13 文件）· `config\`（14 文件）· `test\`（1 文件）**
- `src-tauri\` 的其余部分：`capabilities\` `icons\` `gen\` `Cargo.toml` `Cargo.lock` `tauri.conf.json` `build.rs`
- `node_modules\` · `dist\`
- `index.html` · `notes.html` · `package.json` · `package-lock.json` · `tsconfig.json` · `vite.config.ts`
- `README.md` · `LICENSE` · `.gitignore`
- **`.git\`** —— **未做任何 git 写操作**（未 `add` / `commit` / `clean` / `checkout`）
- **本项目以外的任何项目** —— 未碰 `_shared` / `shared_models` / `_archive` / `video-workbench` /
  Lattice 的发布侧 / `_rollback` 里已有的五份备份（**只新建了 `note_before_20261003`**）
- **未运行本项目任何代码，未杀任何进程**

---
---

# 第二十站（2026-10-03）：范式验证站 · A–D 段

> **这一站不是普通整理站。** 它要验证三件事：
> ① **四区分层能不能承载一个真在用的项目**
> ② **能不能把 17321 接口正式化为「输出槽」**（`INTERFACES.md` 第一次落地）
> ③ **文档里写的每一条，能不能都被探查到**（不许有幽灵）
>
> **三条特别纪律**：文档与实物必须对应（每条附证据）· 边做边反馈分层问题 · 不做没验证的断言。
>
> ⚠️ **上面「第一站」（第九站）的记录一字未改** —— 本节只做追加。
> **上面第 127 行 / 第 158-159 行对 `DEVELOPMENT_LOG.md` / `123.txt` 的提及是【历史记录】**，
> 记的是当时搬到了 `docs\`，**不反映 C 段之后的新位置**（见下）。

---

## 八、A 段：全量摸底

### 8.1 源码（实测）

| 项 | 值 |
| --- | --- |
| **文件总数** | **42**（前端 28 · 后端 13 · 测试 1） |
| **总行数** | **7,873** |
| **前端入口** | **2 个** —— `index.html` → `src/workbench/main.tsx`；`notes.html` → `src/notes/main.tsx`（`vite.config.ts:23-27`） |
| **后端模块** | **2 个** —— `src-tauri/src/notes/`（10 文件）+ `src-tauri/src/workbench/`（2 文件）+ `main.rs` |
| **Tauri 命令数** | **26**（`main.rs:24-51` 逐行计数：24 个 `notes::commands::*` + 2 个 `workbench::action::*`） |

### 8.2 数据（只读 `mode=ro` 实测）

| 表 | 行数 |
| --- | --- |
| `pages` | **152** |
| `tags` | **12** |
| `card_tags` | **50** |
| `schema_migrations` | **3**（v2/v3/v4） |

**画像**：活跃 **134** / 回收站 **18** · `kind` 全为 `note`（**`outline` = 0**）·
`version` 2–81 · `deleted_at` 非空 18 · `updated_at == created_at` 4 条 ·
有标题 15 条 · `nodes_json` 非空 **0** · `from_view` 非空 **0** · 重复 id **0** ·
`PRAGMA integrity_check` = **ok**

🔴 **`kind_outline = 0` 且 `nodes_json` 非空 = 0** —— **两条互证：「结构树」这个功能在这 152 条数据里一次都没被用过。**

### 8.3 只读纪律

**全程 `-readonly` 打开**；🔴 **未在只读连接上发任何有写语义的 PRAGMA**
（无 `wal_checkpoint` / 无 `journal_mode=` / 无 `VACUUM`）。
（**第九站之后、本站之前**，另有一次排查误发过 `wal_checkpoint`，**实测被拒**（`disk I/O error`），**已记纪律**。）

---

## 九、B 段：输出槽规格

| 动作 | 内容 |
| --- | --- |
| **新建** | **`docs\OUTPUT-SLOT.md`**（本站 B 段新建；C/D 段有追加） |
| **代码改动** | **零** —— `v1` **没有**写到线上（`http.rs` 一行未动） |

**内容**：槽的位置 · 两条路由的 SQL 原文 · 逐字段表（名/类型/含义/可空）·
边界三条 · 缺口两条 · 证据等级 · `v1` 落地状态 · 三条硬约束对照。

**两条【源码级】边界 + 两条缺口**（都附行号）：

| # | 内容 |
| --- | --- |
| 边界一 | **删除/恢复不修改 `updated_at`**（`db.rs:556-560` / `:572-576` 两条 SQL 里都没有它，而 `update` `:525-530` 与 `save_outline` `:908-912` 都有） |
| 边界二 | **`purge` 没有墓碑**（`db.rs:599` 是 `DELETE FROM pages`） |
| 边界三 | **无增量接口**（`http.rs:134` 把查询串整个丢掉；`db.rs:328` 的 SQL 无 `WHERE`） |
| 🔴 缺口一 | **`tags` 是 ID 数组，而整个槽没有把 ID 换成名字的途径** |
| 🔴 缺口二 | **槽里没有版本字段** —— `v1` 只存在于文档，远端无法自证版本 |

---

## 十、C 段：四区整理

### 10.1 `docs\` 分层（本轮新增两级）

```
docs\
  README.md  run.md  data.md  deps.md  service.md  log.md   ← 六件套，留 docs\ 根
  OUTPUT-SLOT.md                                            ← 输出槽规格，留 docs\ 根
  specs\    123.txt · 456.txt                               ← 移植当时的往来记录
  history\  DEVELOPMENT_LOG.md · IMPLEMENTATION_PLAN.md ·
            PORTING_REPORT.md · UI_SPEC.md
```

**搬迁**：6 份逐个用 `[System.IO.File]::Move`，**逐份核 SHA256，6/6 一致**，源文件全部消失。
（**不是 `Move-Item`** —— 它对目录非原子，且 `MV` 是 PowerShell 内置别名。）

> ⚠️ **`specs\` / `history\` 这两个目录名有争议**，已记进 `project_manager\docs\log.md` 待议：
> `DEVELOPMENT_LOG.md` 被 `run.md` / `deps.md` **实时引用着**，它不是"history"，是"活的外部参考"。

### 10.2 空壳（刻意的）

`misc\` **0 项** · `data\{input,output,store,cache,logs}` **各 0 项**。**全留，未建标记文件。**
理由写进了 [data.md](data.md) 文首：**`data/` 五区的存在表达的是一条【否定式事实】。**

### 10.3 三条硬约束核对（C4）

| 约束 | 判定 |
| --- | --- |
| **一 · 不通过代码耦合** | ✅ **满足**（含一条边界：`summary.rs:20-26` 的编译期 `env!("CARGO_MANIFEST_DIR")` 上溯到 `D:\MyProject`） |
| **二 · 必须声明输入和输出** | ⚠️ **部分满足** —— 输出已声明（`OUTPUT-SLOT.md`）；🔴 **输入没有声明** |
| **三 · 消费者不管生产者内部** | ⚠️ **部分满足** —— 🔴 **`tags` 直接返回内部主键** |

**两条缺口同一个根因**：**这个槽是从"本项目自己用"演进出来的** ——
它是事实上的「槽」，**但还不是按槽的标准设计的槽**。

---

## 十一、D 段：六件套 + 证据标注

### 11.1 修复悬空引用（D1）

C2 搬迁后 `docs\` 里产生 **28 处**指向旧位置的引用。本轮修了 **21 行**：

| 文件 | 改了几行 | 改了什么 |
| --- | --- | --- |
| `README.md` | **12** | 移植链 ASCII 图 6 处 + 文档索引表 6 条 Markdown 链接 |
| `deps.md` | 4 | `DEVELOPMENT_LOG.md` / `IMPLEMENTATION_PLAN.md` / `PORTING_REPORT.md` 的路径 |
| `run.md` | 4 | 同上 |
| `data.md` | 1 | `DEVELOPMENT_LOG.md` §8.4 |

**未改**：`docs\log.md` 里的 **10 处历史提及**（那是历史记录，改了就是篡改）。

**验证**：跑脚本逐个解析 `docs\**\*.md` 全部 Markdown 链接 → **29 条全部 `Test-Path` 通过，断链 0**。

### 11.2 加证据索引（D2）

在 **5 份文档末尾各加一节「证据索引」**（`README.md` 第八节 · `run.md` 第十节 ·
`data.md` 第七节 · `deps.md` 第七节 · `service.md` 第九节）。

- **格式**：表格，`断言 | 证据`，证据是 `文件:行号` 或"实测"
- **原则**：🔴 **不改断言本身，只加证据** —— 发现的错只登记，不改
- **未验证的**：另设「未验证与订正」小节（`README` 第九 · `run` 第十一 · `deps` 第八 · `service` 第十）

### 11.3 🔴 D 段实测发现的错 —— **E 段已全部修正**

| # | 位置 | 原文 | 实测 | E 段处置 |
| --- | --- | --- | --- | --- |
| 1 | `README.md` 第五节架构图 | `components/(16 个)` | **14 个** | ✅ 已改为 **14** |
| 2 | `run.md` 第六节 | `cargo test # Rust，12 条` | `#[test]` 属性 = **23 个**（`db.rs` 17 + `summary.rs` 6）—— **没跑过 `cargo test`，12 与 23 谁对判不了** | ✅ 已改为 **`Rust —— ⚠️ 条数未实测，见第十一节`**（**不写数字**） |
| 3 | `deps.md` 第四节 | `thiserror = "1"` 用途列「错误类型」 | 🔴 **全项目源码零使用点**（`thiserror` / `derive(Error)` / `#[error(` **三样全零命中**） | ✅ 已改为 **「全项目无使用点」** |
| 4 | `README.md` 第六节目录树 | 未含 `specs\` / `history\` | C2 搬迁造成的过期描述 | ✅ 已补上两个子目录 |

**🔴 E 段推翻的口径**：**D 段的「只加证据不改断言」在 E 段被改为「已知错当场改」** ——
**理由（用户拍板原话）：已知错留在文档里，比没写更坏。**
**每一处都在各自文档的「未验证与订正」小节里记了「已于 E 段修正 + 依据」。**

### 11.4 🔴 D 段推翻的【我自己的】错（这条是自纠）

**A 段我报了两条"原文错了"，实测后两条都站不住：**

| 我当时说 | 实测 | 真相 |
| --- | --- | --- |
| 「命令数 = **25**（原文写 26 是错的）」 | **26** | 🔴 **我错了** —— 原文的 26 是对的，**我 A 段数漏了一个** |
| 「各文件字节数已过期」 | `db.rs` **51,678 B** · `commands.rs` **8,848 B** · `http.rs` **7,008 B** · `config.rs` **4,103 B** · `summary.rs` **3,556 B** · `hittest.rs` **2,091 B** | 🔴 **我错了** —— **原文的字节数与实测逐项一致** |

**→ 已在 `README.md` 的证据索引里改回正确表述，并把这条自纠写在这里。**

### 11.5 🔴 C2 搬迁造成、但**未改**的两处（D 段纪律：不改断言）

| 位置 | 现状 |
| --- | --- |
| `README.md` 第六节目录树 | **未列出 `docs/specs/` 与 `docs/history/`** |
| `README.md` 第六节首行 `├── docs/  ← 本目录（6 份移植档案 + 六件套）` | 那 6 份**已不在 `docs/` 根** |

**已登记在 `README.md` 第九节「未验证与订正」的 9.3 小节。**

---

## 十二、代码改动清单（第二十站）

> # **代码改动 = 零。**

`Note` 的 `.rs` / `.ts` / `.tsx` / `.json` **一个字节未动**。
（`git status` 里 `Note` 的源代码改动 —— `M db.rs` / `M mod.rs` / `?? http.rs` ——
**都是 2026-10-01 写的、本站之前就存在的**，与本站无关。）

---

## 十三、验证结果（第二十站）

| # | 验证项 | 方法 | 证伪标准 | 结果 |
| --- | --- | --- | --- | --- |
| W-1 | **`docs\` 无断链** | 解析全部 Markdown 链接 + `Test-Path` | 任一条 404 | ✅ **29 条全通过，断链 0** |
| W-2 | **无未加前缀的裸引用** | 正则扫 6 个文件名 | 出现裸名 | ✅ 只剩 `log.md` 的 10 条历史（**禁止改**）+ `README.md` 3 条自解释句 |
| W-3 | **C2 搬迁无损** | 搬前搬后 SHA256 | 不符 | ✅ **6/6 一致**，源文件全部消失 |
| W-4 | **空壳未被误删** | `misc` + `data` 五区计数 | 少一个 | ✅ `misc` 0 项；五区**全在**、各 0 项 |
| W-5 | **进程未中断** | PID + 端口 | PID 变或端口消失 | ✅ 仍 **PID 29608**；**17321 仍监听** |
| W-6 | **编码未写坏** | 读前 3 字节 | 出现 BOM | ✅ `README` `run` `data` `deps` `service` `log` `OUTPUT-SLOT` **全部无 BOM** |
| W-7 | **历史未被篡改** | 复核 `docs\log.md` | 第一站记录被改 | ✅ **185 行原文一字未动**，只追加 |
| W-8 | **未碰 17321** | 全程 | 发了 HTTP 请求 | ✅ **一次都没发** |
| W-9 | **未跑 git 写操作** | 复核 | —— | ✅ 只跑过 `git status` / `diff` / `log` / `remote`（**全是读**） |
| W-10 | **代码零改动** | 复核 | 任一处 `.rs`/`.ts`/`.tsx`/`.json` 被写 | ✅ **零** |

---

## 十四、第二十站新增的未解决

1. 🔴 **两个端点的真实 HTTP 报文 —— 至今零实测。**
   第九站、第二十站 B 段、D 段，**三次都因禁止碰端口而未测**。
   `OUTPUT-SLOT.md` 的字段表**全部来自读源码**（该文件已自标「证据等级」）。

2. 🔴 **输入侧没有声明。** `OUTPUT-SLOT.md` 只声明了输出。
   输入实测为：人敲的正文 + `config\seasons.json` + 12 个水印 SVG + `tools.json` + 用户选的项目目录。

3. 🔴 **`tags` 返回内部主键**，消费者拿到的是不可读的字符串。

4. ⚠️ **`cargo test` 到底几条** —— 源码 `#[test]` = 23，`run.md` 写 12。**未跑过，判不了。**

5. ⚠️ **`thiserror` 依赖声明了但零使用。**

6. ⚠️ **`docs\history\` 这个目录名可能不贴** —— `DEVELOPMENT_LOG.md` 是"活的外部参考"，不是 history。

7. ⚠️ **`docs\README.md` 第六节目录树未更新**（未含 `specs\` / `history\`）。

8. ⚠️ **`README.md` 的 `components/(16 个)` 应为 14** —— **登记未改**。

9. **`mergeOps` 疑为死代码**（`src/notes/store/opSeq.ts`）——
   定义 + 8 条测试都在，**运行时代码零调用**。**是预留还是忘了删，判不了。**

10. **`data\` 五区与 `misc\` 是"结构性空壳"** —— 磁盘上「有意为空」和「漏建」长得一样。

---

## 十五、E 段：范式验证总结（+ 收尾段）

> **本节是追加。** 上面第十四节的第 7、8 两条（"未更新""登记未改"）
> **是 D 段末尾的状态** —— **E 段已经把它们修掉了**，见下。
> **第十四节作为历史记录，一字未改。**

### 15.1 E 段改了 `Note` 哪三份

**E 段拍板 9：「已知错留在文档里，比没写更坏」—— 所以 D 段"只登记不改"的口径被推翻，直接改。**

| # | 文件 | 改了什么 | 对应 D 段第十四节的哪一条 |
| --- | --- | --- | --- |
| 1 | **`README.md`** 第五节架构图 | `components/(16 个)` → **`(14 个)`** | 第 8 条（**已解决**） |
| 2 | **`README.md`** 第六节目录树 | 补上 `docs/specs/` 与 `docs/history/` 及其内容 | 第 7 条（**已解决**） |
| 3 | **`run.md`** 第六节 | `cargo test # Rust，12 条` → **`# Rust —— ⚠️ 条数未实测`**（**不写数字**） | —— |
| 4 | **`deps.md`** 第四节 | `thiserror` 用途「错误类型」→ **「🔴 全项目无使用点」** | —— |

**每处都在各自文档的「未验证与订正」小节里记了「已于 E 段修正 + 依据」。**

**详细记录**：`project_manager\docs\log.md` →《阶段 E + 本站完整总结》一节的 **3.7 全站实测**。

### 15.2 收尾段（2026-10-03）：改了哪些

| # | 文件 | 改了什么 | 依据 |
| --- | --- | --- | --- |
| 1 | **`OUTPUT-SLOT.md` 第八节** | 「消费者信息来自简报、全项目没有找到」→ **实测结果 `github_operater\src\watch_notes.py`** | 只读打开该文件：`:50` `BASE = "http://127.0.0.1:17321"` · `:51` `CHANGES_URL` · `:77` `_2` 条路由都用 |
| 2 | **`service.md` 第七节** | 「实际消费者本次未查明」→ **同上** | 同上 |
| 3 | `service.md` **第九节 9.5** | 该行也写着「消费者未查明」→ **一并改正**（**超出"只改第七节"的字面范围，已报备**） | 同上 |
| 4 | `README.md` **第八节 8.4** | **补上 `specs/` / `history/` 的证据行**（E 段改了第六节但没同步证据索引） | 段末自检发现 |
| 5 | `run.md` **第十节 10.3** | 「与原文的 12 条不符」→ **「与【改前】的 12 条不符」**（E 段已改，措辞需跟上） | 段末自检发现 |
| 6 | `deps.md` **第七节 7.4** | 交叉引用「见第七节末」→ **「见第八节 8.2」**（**指错了节**） | 段末自检发现 |
| 7 | **本节** | 补 E 段 + 收尾段记录 | —— |

**🔴 第 1、2 两处修的是一条【跨文档矛盾】**：

| 文档 | 原先的说法 |
| --- | --- |
| `project_manager\docs\PLAYBOOK.md:2131` | ✅ **`github_operater` 的 `watch_notes.py` 轮询 17321** |
| 本文件 `docs\service.md` §七 | 🔴 「实际消费者本次未查明」 |
| 本文件 `docs\OUTPUT-SLOT.md` §八 | 🔴 「这条消费者信息来自本轮简报，不是从代码或配置里查到的」 |

**→ 矛盾已消除。`OUTPUT-SLOT.md` 第八节现在带完整实测证据。**

### 15.3 收尾段的实测发现（对输出槽两条缺口的印证）

`watch_notes.py` 的行为**正好印证了输出槽的两条已知缺口**：

| 缺口 | 它的实际做法 |
| --- | --- |
| **缺口一：`tags` 是内部主键** | 它把 `tags` **原样存进自己的 `candidates.jsonl`**（`:29` docstring）—— 🔴 **存进去的是一串它无法解析的 ID** |
| **缺口二：无增量接口** | 它**自己实现了全量 diff**：每 3 秒拉全量（`:52` `INTERVAL = 3.0`），与本地 `note_state.json` 的 `id → (updated_at, deleted_at)` 比对（`:7` docstring） |

### 15.4 🔴 反向核对（实物 → 文档）：发现【大量未登记】

**以前做的都是"文档 → 实物"。收尾段做了反方向：从源码列出关键结构，逐个查文档有没有登记。**

**结果（详见 `project_manager\docs\log.md`）：**

| 类 | 数量 | 未登记 |
| --- | ---: | ---: |
| **Tauri 命令** | **26** | 🔴 **15** |
| **数据表** | **4** | 🔴 **4（`data.md` 里完全没有 schema）** |
| **`notes://` 事件** | **3** | 🔴 **2** |
| 对外路由 | 2 | 0 |
| 配置文件 | 3 类 | 0 |

**→ 完整清单见 `project_manager\docs\log.md` 的收尾段记录（2d 只报告，未补文档）。**

### 15.5 本段代码改动

> # **零。**

---

# 第二十一站（2026-10-09）：窗口行为整改站

> ⚠️ **与前面各站不同：本站在改代码**（前几站多为只读整理）。
> **起因**：用户报告便签窗口出现「一大块黑、点不动、关不掉、连任务管理器也盖不住」，以及「打开时卡 0.5～1 秒」。
> **本站不是"整理"，是"修"** —— 因此记录方式按 [../CLAUDE.md](../../CLAUDE.md) §5（改了啥 / 怎么验的 / 还剩啥），而不是整理站的 A–E 段。

## 一、背景：事故的三个条件

用户最初的困境**不是偶发 bug，是三个缺陷叠加的必然结果，缺一不可**：

| # | 缺陷 | 后果 |
| --- | --- | --- |
| 1 | 窗口的透明**完全依赖 JS 加载的 CSS**（`notes.css` 由 `main.tsx` 用 `import` 引入） | 页面没渲染出来时露出 webview 底色 = **黑块** |
| 2 | `hittest.rs` 的 `HIT_RECTS` **初始为空**，而空 = 「全部穿透」 | 前端没上报过 → **整窗点不动，连关闭按钮都点不到** |
| 3 | `always_on_top(true)` + `decorations(false)` + 无托盘无热键 | **盖不住、关不掉**，没有逃生口 |

## 二、改了哪七处

| # | 改动 | 文件 |
| --- | --- | --- |
| 1 | **窗口不再置顶**（`always_on_top` 移除）—— 这是用户唯一要求的那一条 | `src-tauri/src/notes/window.rs` |
| 2 | 透明**不再依赖 JS**：`notes.html` 的 `<head>` 内联透明样式 | `notes.html` |
| 3 | 命中矩形**未就绪时按普通窗口处理**（原为整窗穿透）；空数组上报被忽略；持锁调 `DefSubclassProc` 的重入死锁隐患一并去掉 | `src-tauri/src/notes/hittest.rs` |
| 4 | `on_page_load` **只记日志**，不再 `hide()` 窗口（曾在启动路径上动窗口，已撤回） | `src-tauri/src/notes/window.rs` |
| 5 | **点 × / Alt+F4 = 先落盘、成功才真正退出进程**（原为 `hide()` 常驻） | `src-tauri/src/main.rs` · `src-tauri/src/notes/commands.rs` · `src/notes/store/notesStore.ts` |
| 6 | **卡片项目标记**：只在绑定「比默认根更具体」的目录时显示，位置在卡片内部最底部 | `src/notes/components/Card.tsx` · `CardList.tsx` · `notes.css` |
| 7 | **复制整条便签**：没选中文本时 Ctrl+C 得到「标题+正文」 | `src/notes/components/Paper.tsx` |

**另外修了一个数据 bug**：`init()` 读列表失败时**静默吞错**，随后把"看起来是空的"当成"空库"→ **凭空自动新建空白便签**。
实测背景：本站工作期间反复重启应用，**每次启动各送一张空白便签**，用户删了一批。
已修：`loadFirstPage` 返回成功与否，**读失败绝不新建**；且**只在真·空库时**才自动建。
见 `src/notes/store/notesStore.ts` 的 `init()` / `loadFirstPage()`。

## 三、🔴 契约变化：**消费者必读**

| 项 | 改动前 | 改动后 |
| --- | --- | --- |
| 点 × 之后 | 只 `hide()`，**进程常驻、17321 继续监听** | 🔴 **进程退出、端口释放** |
| 17321 可达性 | **几乎总是可达**（实测：那一个进程常驻了 5 天） | 🔴 **只在窗口开着时可达** |

**对消费者 `github_operater`（每 3 秒轮询）的影响**：以前"连不上"是异常；**现在"连不上"是常态**（用户没开着便签就是连不上）。
→ 已同步登记到 [service.md](service.md) 与 [OUTPUT-SLOT.md](OUTPUT-SLOT.md)。

## 四、验证结果

| 项 | 结果 |
| --- | --- |
| `tsc --noEmit` | ✅ 通过 |
| `cargo check --release` | ✅ 通过 |
| `cargo test --release` | ✅ **23/23 通过**（含标签生命周期与迁移测试）—— 🔴 **这条同时回答了 [LESSONS.md](LESSONS.md) 第三节"`cargo test` 到底几条"那个空白：23** |
| 前端 `npm run build` | ✅ 通过；且**实测 `dist/notes.html` 里内联 `<style>` 排在 CSS/JS 之前** |
| "点 × 真的退出进程" | ✅ **实测**：`trace.log` 三次开/关都留下 `ui quit: saved, exiting` + `db: quit_notes` 成对记录；关闭后 `Get-Process` 查不到 |
| 冷启动耗时 | **实测 1,763 ms**（`Start-Process` → `IsWindowVisible`）；其中**属于本项目的部分 = `setup` → `init: done` ≈ 0.26～0.29 秒**，其余是 WebView2 自身引导 |
| 取消置顶是否生效 | ✅ **实测**：`GetWindowLong(GWL_EXSTYLE)` 的 `TOPMOST = False` |

## 五、🔴 行号漂移（本轮改动导致既有文档的引用失效）

**本项目纪律**（[LESSONS.md](LESSONS.md) #23）：路径/行号是**写入时刻的快照，不该改**（改了就是篡改历史）。
**故本节只声明，不改旧引用。**

| 文件 | 为什么漂了 | 受影响的可疑引用 |
| --- | --- | --- |
| `src-tauri/src/notes/commands.rs` | **新增两条命令**（`trace` 在 `:130`、`quit_notes` 在 `:139`） | 任何引用该文件 `:NN` 的断言 |
| `src-tauri/src/main.rs` | `invoke_handler` 列表新增 2 行 + `on_window_event` 改写 | `deps.md:195` 的 `main.rs:49-50` |
| `src-tauri/src/notes/window.rs` | 删 1 行、`on_page_load` 改写 | `run.md:198` 的 `notes/window.rs:29`（`#[cfg(target_os = "windows")]`） |
| `src-tauri/src/notes/hittest.rs` | 重写 | `data.md:198` 的 `notes/hittest.rs:5-8` |

> 📌 **未逐条核对全部引用**（工作量大于本轮范围）。**引用这些文件的断言，用前请重新定位。**

## 六、未解决：合成器伪影（**已知限制 4** 仍未闭环）

**用户报告的"卡 0.5～1 秒"本轮未能定位。** 但有一条重要澄清：

> 🔴 **它不是本轮引入的新 bug** —— 它就是本项目**早就记录在案的已知限制 4**
> （`run.md` 第八节 / 仓库根 `README.md:73` / `history/DEVELOPMENT_LOG.md` §19.8 的「**仍未闭环**」）。
> 本轮做的是**移除它发生的条件**（窗口不再被隐藏 → 页面不再被冻结 → 合成不再需要恢复），**不是修好了它。**

**本轮排除掉的九个假设**（全部有反证，不是"觉得不像"）：

| 假设 | 反证 |
| --- | --- |
| JS 死循环 | 0% CPU |
| UI 线程死锁 | `Responding=True` |
| 数据库锁被占 | 用 17321 探测，**秒回 200** |
| `http.rs` 跨 I/O 持锁 | 读源码：先取数再写 socket；有 5s 超时 |
| `db.rs` 双重加锁 | 读 diff：干净的只读查询 |
| `loading` 遮罩卡住 | 死字段，全项目从不读写 |
| `opSeq` 二次方爆炸 | 纯函数 O(n) |
| 渲染进程崩溃/回收 | Crashpad `metadata` 为空；渲染进程**存活 5 天**未重建 |
| 页面重载风暴 | WebView2 `History` 实测：**每次启动仅一次导航**，`visit_count` 与启动次数一一对应 |

**唯一留下的异常**：WebView2 的 **`gpu-process` 在 2026-10-04 10:18:57 被重建过**（浏览器/渲染/utility 三个进程都是 10:06:48 启动，只有它是 10:18:57）。
**GPU 进程被重建 = 它崩过或驱动被重置**，而 GPU 进程一挂，**整个窗口的合成立刻停摆** —— 时间量级与用户描述的 0.5～1 秒吻合。
⚠️ **但这只是假设，没有抓到"卡顿与 GPU 重建同时发生"的现场。**

**下一步（若要继续）**：只读监视器，每 200ms 记一次 GPU 进程 PID 与窗口探针阻塞时长；证伪标准 —— 若卡顿当刻两者都没变，这条线也排除。

## 七、🔴 我自己犯的错（自纠）

| 错 | 代价 | 处置 |
| --- | --- | --- |
| 🔴 **把"取消置顶"这一个改动，扩写成了一整套几何追踪机制**（`requestAnimationFrame` 节流 + 结果去重 + `transitionend/animationend` 捕获监听 + `ResizeObserver` + 对整个 `document.body` 的 `MutationObserver`） | 用户明确反馈"**你改完之后它非常不稳定**"。那套逻辑**每帧、每次 DOM 变动都在跑**，且压在启动路径上 | ✅ **全部拆除**，恢复成原来的**同步直报**。`App.tsx` 的改动量从 **107 行降到 10 行** |
| 🔴 **在没抓到现场的情况下连续改了多轮** | 消耗用户大量耐心，且每轮都要重启应用验证 | 本站末尾改为**先测量、后修改**：用 `WM_NCHITTEST` 探针 + `trace.log` 取证，**九个假设全部先排除再动手** |
| ⚠️ **一度误判"守护进程 svcmgr 在反复拉起便签"** | 白杀了几次用户正在用的窗口 | 读 `service-manager\config\projects\*.yaml` 后**当场收回**：它托管的是 catcher/infra/proxy_pool，与本项目无关 |

> **一句元教训**（呼应 [LESSONS.md](LESSONS.md) #10「一个 bug 反复修不好时，先问这其实是不是好几个 bug」）：
> 🔴 **本轮是它的反面 —— "一个小需求被扩写成一个大改造，而它引入了新的不稳定"。**
> **用户要的是"别总在最顶层"，那就只做那一件事。**

## 八、本轮代码改动清单

| 文件 | 改动 |
| --- | --- |
| `notes.html` | 新增内联透明样式 |
| `src-tauri/src/main.rs` | 注册 `trace`/`quit_notes`；`on_window_event` 由 `hide()` 改为广播 `notes://close-requested` |
| `src-tauri/src/notes/window.rs` | 移除 `always_on_top`；`on_page_load` 改为只记日志 |
| `src-tauri/src/notes/hittest.rs` | `HIT_RECTS` 改 `Option`；空数组忽略；锁范围收窄 |
| `src-tauri/src/notes/commands.rs` | 新增 `trace`、`quit_notes`；`create_page` 加留痕 |
| `src-tauri/src/notes/models.rs` | `MetaIndex` 加 `default_project` |
| `src-tauri/src/notes/db.rs` | `meta_index()` 填 `default_project: None`（由命令层填值） |
| `src-tauri/src/notes/mod.rs` | 挂 `trace` 模块与 `trace::init` |
| `src-tauri/src/notes/trace.rs` | 🆕 **新增**（诊断日志实现） |
| `src/notes/store/notesStore.ts` | `doFlush`/`flushNow` 返回布尔；`closeNotes` 改为落盘后退出；`init` 加闸门与"空库"判据 |
| `src/notes/store/notesApi.ts` | 新增 `trace`、`quitNotes` |
| `src/notes/events/notesEvents.ts` | 新增 `notes://close-requested` 监听 |
| `src/notes/components/Card.tsx` · `CardList.tsx` · `Paper.tsx` · `ProjectPicker.tsx` · `format.ts` · `styles/notes.css` · `types.ts` | 卡片项目页脚、复制整条、`projectName()` 抽取、`default_project` 类型 |

---

# 第二十一站 · 补记（2026-10-11）：两份槽改成 §8.4 机器可读格式

> 🔴 **本轮只改文档：零代码改动、零数据改动、零删除。**

## 一、背景：父规范改版了

`D:\MyProject\CLAUDE.md` 于 2026-10-11 改版，与本项目相关的有四处：

| 项 | 变化 |
| --- | --- |
| **§8** | 重写。槽 = **池子**（数据库 / 数据目录 / 发布的文件 / 端口 / 二进制工具 / 配置文件），**「一个项目可以有很多个池子 —— 8 个很正常，不要只写一个」** |
| **§8** | 三份槽：`INPUT-SLOT` / `OUTPUT-SLOT` / 🆕 **`TRIGGER-SLOT`**（我怎么被启动 —— **控制流**） |
| **§8.4** | 🆕 **机器可读固定格式**：**固定标题 + 正好两列表 + `## 详情`**；程序**只读固定标题下的那张表**，其余一律不管；**认不出固定标题 = 报错，不许跳过** |
| **§8** | 🔴 **「输出槽不写『谁在用』，输入槽不写『从谁那拿』」** —— 关系是推出来的，写了就是耦合；关系看 `project_manager\MAP.md` |

**判据（§8.4 末尾）**：锚点只写两种 —— **绝对路径**（`D:\MyProject\...`，反斜杠）或 **端口**（`127.0.0.1:xxxx`）；**项目外的原样写**（`%APPDATA%\...`）；**不写项目名**。

## 二、改了哪两份

| 文件 | 改动 |
| --- | --- |
| [INPUT-SLOT.md](INPUT-SLOT.md) | 加固定标题 `## 我从哪拿` + 两列表（**2 行锚点**）；原有 §一~§八**一字未删**，降级挂到 `## 详情` 之下；新增 §0「**口径变化**」解释为什么表里有 2 行而原文写「🔴 无」 |
| [OUTPUT-SLOT.md](OUTPUT-SLOT.md) | 加固定标题 `## 我往外给` + 两列表（**7 行锚点**，原为 2 个池子）；原有全部内容**一字未删**，落在 `## 详情` 之下；详情开头标注口径变化（旧 §7 三项 → 新 §8 池子） |

## 三、🔴 OUTPUT-SLOT 为什么从 2 个池子变成 7 个

**不是新增事实，是"原来就存在、但没有按池子登记"的东西被登记了。** 每一行都能在既有文档里找到出处：

| 新增锚点 | 它是什么 | 原先记在哪 |
| --- | --- | --- |
| `%APPDATA%\com.myworkbench.notes\trace.log` | 运行日志 | [runtime-log.md](runtime-log.md)（2026-10-09 新增的文件） |
| `%APPDATA%\com.myworkbench.notes\config` | 运行时配置目录 | [README.md](README.md) 第十节 10.4（**外部可写**） |
| `D:\MyProject\Note\config` | 随包发布的配置与素材 | [deps.md](deps.md) 第五节（`tools.json` 自声明） |
| `D:\MyProject\Note\dist` | 前端构建产物 | [README.md](README.md) 11.1 产出 #1 |
| `D:\MyProject\Note\src-tauri\target\release\my-workbench-notes.exe` | 主程序二进制 | [README.md](README.md) 11.1 产出 #2 · [deps.md](deps.md) 第五节（**别人按绝对路径启动它**） |

**INPUT-SLOT 那 2 行同理**：原先写在第五节「非槽、但能改变行为的外部通道」（5.1 配置目录 / 5.2 默认项目根），**新口径下升格为输入锚点**。

## 四、验证

| 项 | 结果 |
| --- | --- |
| 固定标题是否存在 | ✅ `## 我从哪拿` / `## 我往外给` 各自存在且**唯一** |
| 表是否正好两列 | ✅ 列名分别是 `\| 锚点 \| 拿什么 \|` 与 `\| 锚点 \| 是什么 \|` |
| 锚点是否合规 | ✅ 只有绝对路径与端口两种；项目外的按原样写（`%APPDATA%\...`）；**无项目名** |
| 原有内容是否丢失 | ✅ 逐节核对：**两处原有小节标题与正文全部保留**，只降了一级标题层级 |
| 代码/数据 | ✅ **未改一行** |

## 五、还剩什么（未做）

| # | 未做 | 为什么 |
| --- | --- | --- |
| 1 | 🔴 **`TRIGGER-SLOT.md` 不存在** | §1/§8 现在要求**三份**槽。本项目**有控制流可写**（工作台调 `toggle_notes`、双击 exe、单实例回调——见 [deps.md](deps.md) 第五节）。**本轮用户未要求，故未做** |
| 2 | 🔴 **槽里仍写着"谁在用 / 从谁那拿"** | §8 新规：「输出槽不写谁在用，输入槽不写从谁那拿」。但 ① 本轮用户要求**不丢信息** ② **`project_manager\MAP.md` 在本会话沙箱外**（只能访问 `D:\MyProject\Note`），搬不过去 → **只保留 + 在详情里标注**。**待办：关系应迁往 `MAP.md`** |
| 3 | ⚠️ **§2.2 与 §3 的新要求未核对** | §2.2 要求 `deps.md` 写清**用了什么外部资源 · 在哪 · 从哪来**；§3 要求**核心做成接口、UI 只是壳**（判据：把 UI 关掉，命令行能不能跑出同样结果？）。**本轮范围只有两份槽，未核对这两条** |
| 4 | ⚠️ **`Note\services.yaml` 可能错位** | §5 要求每个项目在**自己根下**声明**自己的**进程；而该文件（未跟踪、用户在途）里声明的是 `svcmgr-serve` / `svcmgr-ui` —— **是 `service-manager` 的进程，不是本项目的 17321**。**未动** |

## 六、本轮文档改动清单

| 文件 | 改动 |
| --- | --- |
| `docs/INPUT-SLOT.md` | 重排为 §8.4 格式（新增固定表 + `## 详情` + §0 口径变化），原文全留 |
| `docs/OUTPUT-SLOT.md` | 顶部插入 §8.4 固定表 + `## 详情`，原文全留 |
| `docs/log.md` | **本节** |

---

# 第二十一站 · 补记二（2026-10-11 同日）：按 §8.4 四条禁令重改两张锚点表

> 🔴 **只改文档：零代码、零数据、零删除。** 这是同日**第二次**改写（第一次见上一节）。

## 一、触发：§8.4 当天又更新了，而且**实测点名的就是本项目**

| 新增内容 | 与本项目的关系 |
| --- | --- |
| 锚点第三种写法：**通配**（`D:\MyProject\*\services.yaml`） | 本项目**未用** —— 没有"横跨多个项目、逐个列会失效"的池子 |
| 🔴 **禁令 1**：声明不许只写在 `## 详情` 里 | 本项目根下的 `services.yaml` **原来只在详情里被提过** → **本轮补进表** |
| 🔴 **禁令 2**：锚点不许写「项目目录根」或「生态根」 | 🔴 **spec 的实测例子就是本项目**：「`Note` 写 `D:\MyProject`（生态根）→ 前缀匹配给它凑出 **28 条假边**」 |
| 🔴 **禁令 3**：项目外路径统一 `~` 开头，不用 `%VAR%` | 本项目原写 `%APPDATA%\...` → **本轮全改 `~/AppData/Roaming/...`** |
| 🔴 **禁令 4**：读什么写什么 | 逐条核对见第三节 |
| 归一化新增：`%VAR%` 展开 · `~` 展开 | ⚠️ 那是**兜底**；**禁令 3 明确要求不指望它兜底** |

## 二、改了哪两处（逐条对照）

| # | 改动 | 依据 |
| --- | --- | --- |
| 1 | **删掉 INPUT 表里的 `D:\MyProject`** | 禁令 2 |
| 2 | `%APPDATA%\com.myworkbench.notes\*` → **`~/AppData/Roaming/com.myworkbench.notes/*`**（INPUT 1 行、OUTPUT 4 行） | 禁令 3 |
| 3 | OUTPUT 表**补 `D:\MyProject\Note\services.yaml`** | 禁令 1（本项目根下确有该文件） |
| 4 | OUTPUT 表**补 `D:\MyProject\Note\docs`** | §8.5：「一个项目的 `docs\` **就是它的对外接口**」 |
| 5 | OUTPUT 表 **7 行 → 9 行** | 即 #3 + #4 |

## 三、🔴 一处两条禁令冲突：`D:\MyProject` 那个读取**现在无处可写**

`list_project_folders()` **确实会 `read_dir(D:\MyProject)`**（只取直接子目录名，不读内容）。
但它**没有"具体的东西"可锚** —— 读的是**目录名**，不是文件 / 表 / 端口；而 **禁令 2 又禁止锚生态根**。

| 选项 | 结果 |
| --- | --- |
| **只写详情**（本轮采用） | ⚠️ 这条读取**对程序不可见** —— 正是**禁令 1** 警告的「**内容没丢 ≠ 声明没丢**」 |
| 写 `D:\MyProject` | ❌ 实测 **28 条假边** |
| 写 `D:\MyProject\*` | ❌ `*` = **各项目的目录根**，等于禁令 2 点名的"别人项目的根" |

**→ 处置：只写详情，并在 [INPUT-SLOT.md](INPUT-SLOT.md) §0.B 如实登记这次冲突。**
**内容没丢、声明丢了 —— 这是明知而为的取舍，写在这里，免得下次被当成疏漏。**

## 四、验证

| 项 | 结果 |
| --- | --- |
| 固定标题唯一 | ✅ `INPUT-SLOT.md:9`（`## 我从哪拿`）/ `:15`（`## 详情`）；`OUTPUT-SLOT.md:13`（`## 我往外给`）/ `:27`（`## 详情`）—— 各一次 |
| 表正好两列 | ✅ `\| 锚点 \| 拿什么 \|` · `\| 锚点 \| 是什么 \|` |
| **表内无 `%VAR%` 写法** | ✅ 表内 0 处；`%APPDATA%` 只剩在**说明性正文**里 |
| **表内无生态根锚点** | ✅ 无 `\| D:\MyProject \|` |
| 原有内容 | ✅ **一字未删，只增** |

## 五、还剩什么

| # | 未做 | 为什么 |
| --- | --- | --- |
| 1 | 🔴 **`TRIGGER-SLOT.md` 仍不存在** | §1/§8 要求**三份**槽。本项目**有控制流可写**（工作台调 `toggle_notes` · 双击 exe · 单实例回调）。**用户未要求，故未做** |
| 2 | 🔴 **上一节（补记一）的结论已被本轮取代** | 那一节写「锚点只写两种…项目外的原样写 `%APPDATA%`」—— **当时是对的**；§8.4 当天更新后作废。按 [LESSONS.md](LESSONS.md) **#23 不改历史记录**，只在此声明 |
| 3 | ⚠️ `Note\services.yaml` **内容存疑** | 它声明的是 `svcmgr-serve` / `svcmgr-ui`（`service-manager` 的进程），**本项目的 17321 反而不在里面**。本轮**只登记进表，未改内容**（未跟踪的在途文件） |
| 4 | ⚠️ 槽里仍写着"谁在用 / 从谁那拿" | §8 新规不该写（关系看 `project_manager\MAP.md`）—— 该目录在**本会话沙箱外**（只能访问 `D:\MyProject\Note`），**搬不过去** |

---

# 第二十二站（2026-10-14）：手机 catcher 文本 → 便签卡片

> 🔴 **这是本项目第一次有【程序】写入的数据。** 此前每一字节都来自人的手指（[README.md](README.md) 10.5）。
> 本轮起，`pages` 表里会多出**不是人敲进去**的行 —— 改的是这个项目的身份，不只是加一个功能。

## 一、需求与危险边界（原文要点）

**目标**：手机上 catcher 采到的文本，在 Note 里变成**普通卡片** —— 打开能看见、能按标签找到，发过一次就一直在。
**来源**：`D:\MyProject\_data\index.db` 的 `texts` 表（`WHERE source='catcher' AND deleted=0`，主键 `text_id`），
即 `_data\docs\OUTPUT-SLOT.md` 的 `## 我往外给` 表里声明过的那一个池子。

**六条方向**：① 后台拉、不在启动路径上；② 认 `text_id` 去重，账本**独立存、不依附卡片**；
③ 只新增、不覆盖，本地删除只删本地；④ 标签镜像原名（重名即合并）+ 自动挂「手机」；
⑤ 按数据自带的 `src_created_at` 排，不是导入时刻；⑥ 读 `_data` 失败 → 这一轮不导，Note 照常起。

**危险边界**（原文判据是反的 —— 列那些"看起来不危险"的）：
读 `_data` 会撞 `database is locked`（那个库 `journal_mode=delete` + 每 30 分钟有人写）；
放在启动路径上 = Note 起不来；插标签 = 写进用户的标签体系；导入 = **Note 第一次有程序写入的数据**。

## 二、六条方向各落在哪

| # | 方向 | 落点 |
| --- | --- | --- |
| 1 | **后台拉，不在启动路径上** | `notes::setup` 里 `import::spawn()` **只起线程就返回**；`run()` 里读配置/读源/写库的任何失败都只写状态，**不冒泡**。窗口创建在它之前，完全不受影响 |
| 2 | **已拉过的不再拉** | 🆕 表 `imported_texts`（迁移 **v5**）。**不建指向 `pages` 的外键、`purge` 也不级联删它** —— 卡片彻底删了账本还在 |
| 3 | **只新增不覆盖 / 本地删除只删本地** | 账本在 ⇒ **从不重导**，所以「覆盖用户改过的卡」在结构上不可能发生。上游软删后本地卡片**保留**（发过一次就一直在） |
| 4 | **标签镜像原名 + 自动挂「手机」** | `Database::import_texts` 里查**活标签**建 名字→ID 图，重名即复用；没有才新建。每张卡先挂 `MOBILE_TAG`（`"手机"`）再去重合并上游标签 |
| 5 | **按 `src_created_at` 排** | `import::parse_iso_utc_ms()`（自写解析器，**不加新依赖**）→ `pages.created_at`。排的是那段时间，不是导入时刻 |
| 6 | **读失败不拖垮 Note** | 状态置 `failed` + 写 `trace.log`，`run()` 返回。前端 `reconcileImport` 读不到状态也**静默** |

**另外两条硬要求**：

- **路径不硬编码** —— 🆕 `config/sources.json`（`{"version":1,"catcher":{"index_db":"..."}}`），由 `config::read_sources()` 读。
  换机器/挪目录只改配置，不用重编译。
- **`database is locked` 要重试** —— `read_new_texts()` 只对 `SQLITE_BUSY`/`SQLITE_LOCKED` 重试
  （4 次、每次隔 1.5 秒，另给连接设 2 秒 `busy_timeout`）。**别的错误重试也没用，直接放弃这一轮。**
  只读打开：`mode=ro` + `SQLITE_OPEN_READ_ONLY` —— **库不存在就是打不开，绝不会把它建出来**。

## 三、事件与竞态（为什么还需要一次"对账"）

Rust 导完发一次 `notes://catcher-imported`，前端收到就刷新列表/标签/计数。
🔴 **但导入是后台线程，很可能在前端挂上监听之前就跑完了** —— 那一次事件没人听见。
所以前端 `init` 收尾时再问一次 `catcher_import_status`（纯读内存状态，不碰数据库），
`imported > 0` 就补一次刷新。两条路走同一个 `notesStore.refreshAfterImport()`，**幂等**。

**不做提示**：不加「本轮新导入 N 条」这类提示 / 角标（用户明确拒掉了）—— 只静默刷新。

## 四、改了哪些文件

| 文件 | 改动 |
| --- | --- |
| `config/sources.json` | 🆕 外部池子路径（唯一一处写 `_data` 的地方） |
| `src-tauri/src/notes/import.rs` | 🆕 读源（含重试、只读打开）、ISO 时间解析、后台线程、状态机 |
| `src-tauri/src/notes/db.rs` | 迁移 **v5**（`imported_texts`）；`import_texts()` / `imported_text_count()`；`ImportedText` / `ImportOutcome` |
| `src-tauri/src/notes/config.rs` | `SourcesConfig` / `read_sources()`；**debug 分支的 `copy_dir` 加 `COPY_LOCK`**（见第六节） |
| `src-tauri/src/notes/mod.rs` | `pub mod import`；`setup` 里挂 `import::spawn` |
| `src-tauri/src/notes/commands.rs` | 🆕 命令 `catcher_import_status` |
| `src-tauri/src/main.rs` | 注册该命令（26 → **27** 条） |
| `src/notes/types.ts` | `CatcherImportStatus` |
| `src/notes/store/notesApi.ts` | `catcherImportStatus()` |
| `src/notes/store/notesStore.ts` | `refreshAfterImport()` + `reconcileImport()`；`init` 收尾对账 |
| `src/notes/events/notesEvents.ts` | 监听 `notes://catcher-imported` |
| `docs/log.md` | **本节** |

## 五、验证（全部实测）

| 项 | 结果 | 命令 |
| --- | --- | --- |
| Rust 单元测试 | ✅ **34 passed / 0 failed**（其中导入相关 6 个：落卡与账本 · 幂等 · 空正文不记账 · 标签重名合并 · purge 后不重导 · 上游时序） | `cargo test` |
| 时间解析 | ✅ 9 个断言：微秒/毫秒/1 位小数/零偏移/纪元原点/闰日；**并拒绝**非 UTC 偏移与格式错误 | 同上 |
| **真实源只读** | ✅ 从 `_data\index.db` 读出 **41** 条（`catcher AND deleted=0` 实测 = 41），`text_id` 形状、时间戳年份全部断言通过 | 同上（`reads_real_source_when_available`） |
| **端到端演练** | ✅ 真实 41 行 → **一次性临时库**（不是用户那份）：`imported: 41`；**第二遍 `imported: 0, skipped_existing: 41`**；`active=41`；标签实测 `["手机"]` / `["手机","待作"]`；最新一条 `created_at=1791798693740` = `2026-10-12T09:51:33.740Z`（正是上游最新一条） | 一次性测试，**验完已删** |
| 前端类型 | ✅ `tsc --noEmit` 通过 | `npx tsc --noEmit` |
| 前端单测 | ✅ 17 passed | `npm test` |
| release 编译 | ✅ `cargo check --release` exit 0 | —— |
| **未做** | 🔴 **没有在用户真实的 `notes.db` 上跑过** —— 见第七节 | —— |

## 六、一处必要的新增：`COPY_LOCK`

`config_dir()` 在 debug 下**每次调用都把项目里的 `config/` 覆盖复制到 appConfigDir**。
本轮多了一个后台线程（导入）也会走这里 —— 两个线程同时复制同一批文件，**可能复制到半截内容**（`seasons.json` 922 B，坏了就是主题失效）。
处置：**只把"复制"这一段串行化**（`static COPY_LOCK: Mutex<()>`，`#[cfg(debug_assertions)]`），读取不加锁，行为不变。

## 七、还没做 / 未验（如实登记）

| # | 项 | 说明 |
| --- | --- | --- |
| 1 | 🔴 **真机未跑** | 用户真实的 `notes.db` **未被写入**。导入会在**用户下次启动便签**时发生 —— 那才是「打开 Note 能看见」那一刻。本轮只验到"真实上游数据 → 真实 schema → 正确的卡" |
| 2 | 🔴 **回环（撞见了，没解）** | 导入的卡进了 `pages` 就是普通卡，`_data\sync_notes.py` 会把它们抄回 `_data.notes` ⇒ **同一段文本在 `_data` 里躺两条**。按指令**未碰 `_data` 那边的任何东西**，只报上来 |
| 3 | 🔴 **两份文档现在与事实相反** | `README.md` 10.2/10.5 与 [INPUT-SLOT.md](INPUT-SLOT.md) 都写「**Note 没有输入槽 / 只有输出、没有输入**」。本轮**没有改它们**（本轮范围只有代码）。<br>**但事实已经变了**：`_data\index.db` 的 `texts` 表现在是一条**读进来的**边 |
| 4 | ⚠️ **release 下 `sources.json` 可能不被复制** | `config.rs` 只在 `!config.exists()` 时才复制默认配置 —— 已装机的 `%APPDATA%\...\config\` 早就存在。**与 `seasons.json` 是同一个老问题**（[data.md](data.md) 第六节） |
| 5 | ⚠️ **空正文**：不落卡、**也不记账** | 本机实测 **0 条命中**。记账会让上游之后补的正文永远进不来，所以选择不记 |
| 6 | ⚠️ **熄灭的同名标签不被复用** | 只在**活标签**里找重名；用户熄灭过「待作」再遇到「待作」会新建一枚活的。（理由是活标签唯一索引 + 与 `create_tag` 的语义一致） |
| 7 | ⚠️ **「手机」标签会出现在用户的标签体系里** | 这是需求第 4 条要求的，但它确实往用户的标签面板里加了一枚标签 |
| 8 | ⚠️ **行号漂移** | `commands.rs` / `main.rs` / `db.rs` / `config.rs` / `notesStore.ts` / `notesEvents.ts` 都有改动，旧文档里引用它们的 `:NN` 需要重新定位（同第二十一站第五节的口径：**只声明，不改历史**） |

## 八、自纠

| 错 | 处置 |
| --- | --- |
| ⚠️ 我给"上游时间解析"写的**期望值算错了**（把 `2026-10-12` 的 epoch 写成了 `1760262693`，实为 `1791798693`） | **当场改**：重算并写进注释（`2026-01-01 = 1767225600`，+284 天 +9:51:33） |

---

# 第二十二站 · 补记（2026-10-14 同日）：让**装机那份**真的能看见

> 🔴 上一节停在「导入会在用户下次启动便签时发生」——**那句话当时不成立**。
> 本节把两个缺口补上，并实测到能建的边界为止。

## 一、两个缺口（实测确认）

| # | 缺口 | 实测 |
| --- | --- | --- |
| 1 | **`sources.json` 到不了 release 真机** | 项目 `Note\config\` **有**、`%APPDATA%\com.myworkbench.notes\config\` **没有**（14 个文件里没它）。原因：release 分支只有 `if !config.exists()` 才复制，而那个目录**早就存在** ⇒ 复制永不触发 |
| 2 | **装机的 exe 里没有导入代码** | `target\release\my-workbench-notes.exe` = **2026-10-09 11:15:39**，10,544,640 B —— 今天做的东西不在里面 |

## 二、改了什么

| # | 改动 | 为什么 |
| --- | --- | --- |
| 1 | `tauri.conf.json` 的 `bundle` 段**补 `resources: { "../config": "config" }`** | 打包里**本来没有**默认配置（[data.md](data.md) 第六节的存疑项，实测为真）⇒ 无从补缺 |
| 2 | release 分支：`if !config.exists() { 复制 }` → **每次都"补缺复制"**（🆕 `copy_dir_missing`） | 目录一存在，后加的默认文件就永远到不了。**只补缺、绝不覆盖** —— 运行时那份是用户的 |
| 3 | 🆕 单测 `copy_dir_missing_fills_gaps_but_never_overwrites` | 把「补缺不覆盖」钉死（已存在的 `seasons.json` 保持 `user-edited`） |
| 4 | 🆕 单测 `bundled_defaults_fill_an_existing_install` | **拿真实构建产物**（`target/release/config`）补进"已装机、只有旧文件"的目录，断言 `sources.json` 真的到得了 |
| 5 | 读源失败**不再无声**：`read_sources` 由 `Ok(None)` 改成 `SourceConfig::Ready/Unconfigured(原因)`；源库不存在单独判 | 三种"没配"以前全压成 `None`。现在每种都带一句人话 ⇒ `catcher_import_status.message` + `trace.log` |
| 6 | 死代码清理：`copy_dir` 加 `#[cfg(debug_assertions)]`；`imported_text_count` 加 `cfg_attr(not(test), allow(dead_code))` | 首轮 release 构建报了 **2 个 warning**，清掉 |
| 7 | 备份旧 exe → `my-workbench-notes.exe.bak-20261014`（10,544,640 B） | **不复制就删**是禁项 —— 重编会覆盖它 |
| 8 | 重编 release（`npm run tauri build -- --no-bundle`） | 缺口 2 |

## 三、实测

| 项 | 结果 |
| --- | --- |
| release 构建 | ✅ **exit 0，0 warning**（1m58s） |
| 新 exe | ✅ **2026-10-14 12:32:19 · 10,646,528 B**（旧：10-09 11:15:39 · 10,544,640 B） |
| **打包落点** | ✅ 实测 `src-tauri\target\release\config\sources.json`（88 B）—— 正是 `Resource/config` 解析到的那个 |
| Rust 测试 | ✅ **36 passed / 0 failed**（新增上面两条） |
| exe 里的新代码 | ✅ 明文搜到 `catcher_import: begin` · `imported_texts` · `sources.json` · `notes://catcher-imported` |
| 前端 | ✅ `dist/assets/notes-*.js` 里有 `catcher_import_status` · `notes://catcher-imported` · `refreshAfterImport`；`tauri build` 先跑 `npm run build`（vite 输出新 hash）再编译，故已随包嵌入 |

## 四、还没做 / 没验（如实登记）

| # | 项 | 说明 |
| --- | --- | --- |
| 1 | 🔴 **没有启动真机** | **"卡片真的出现"这一步没验。** 理由是：本机没有合规的关闭手段 —— **按进程名杀进程在禁用清单里**，而它是 GUI 程序、启动后会留在用户屏幕上。<br>🔴 **已决（2026-10-14）：不做演练。** 「**用户自己开一次**」本身就是端到端验证，而且那才是真实路径。已核：工作台 `D:\MyProject\Workbench\config\tools.json` 的 notes 那条 `action` 指向的正是**本次重编的那一份 exe**。<br>**开完若"没东西"**：先看 `catcher_import_status` 的 message 与 `trace.log` 里的 `catcher_import:` 行，**别猜**。 |
| 2 | ⚠️ **补 `sources.json` 发生在 app 启动那一刻** | 不是构建期就位。顺序是对的：`read_sources` **先**调 `config_dir()`（补缺）**再**查文件在不在 —— 实测代码顺序如此 |
| 3 | ⚠️ `%APPDATA%\...\config\` **此刻仍缺 `sources.json`** | 实测（写本节时）：那 14 个文件里没有它。**下一次 release 启动会补上** |
| 4 | ⚠️ 真机是否走通「补缺 → 读到 → 导入」 | **未验**，见 #1。降级证据：打包落点实测命中 + 补缺单测用真实产物通过 + 读源/落卡各有单测与真实数据演练（上一节第五节） |
| 5 | ⚠️ 旧 exe 备份留在 `target\release\` | 与既有 `.bak-20261001` 同一个位置、同一命名法。**用途**：这次是真的回滚点 |

## 五、自纠

| 错 | 处置 |
| --- | --- |
| ⚠️ 我第一次用 `Select-String` 搜 exe，看到 `notes://catcher-imported` 就以为"前端是明文嵌入的"，接着又因为搜不到 `catcher_import_status` 而困惑 | **改用逐字节搜索 + 看上下文**，当场弄清：那几个命中的**全是 Rust 字面量**；**前端资源是压缩嵌入的**，明文本来就搜不到。结论不变（前端确实在包里），但**推理路径当时是错的** |
