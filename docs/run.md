# 运行与构建

> 所有命令都在**仓库根**跑（不是 `app/` —— 与 Lattice 不同，本项目根就是 npm 工程根）。

---

## 一、安装

```powershell
cd D:\MyProject\Note
npm install
```

运行时依赖 4 个 + 开发 7 个。见 [deps.md](deps.md)。

---

## 二、开发

```powershell
npm run tauri dev
```

> 首次会编译约 **462 个 crate**（约 2 分钟），之后增量很快。（README 原文）

窗口是 **56×64 的无边框透明竖条**（工作台壳），点一下浮出便签窗口。

---

## 三、构建

```powershell
npm run build        # = tsc --noEmit && vite build  → dist/
npm run tauri build  # 打包
```

---

## 四、🔴 重编之前，先确认 `custom-protocol` 的状态

**这一条必须单独读，因为它是这个项目最反直觉的地方。**

`Cargo.toml` 里写着：

```toml
tauri = { version = "2", features = ["protocol-asset", "custom-protocol"] }
```

**这是有意的，不是笔误。** 完整背景见 [deps.md](deps.md) 第三节，这里只放结论：

> **决定 dev/prod 的不是 debug/release，是 `custom-protocol` feature。**
>
> — `history/DEVELOPMENT_LOG.md` §8.3

| 情形 | 后果 |
| --- | --- |
| `custom-protocol` **没开** | **即使是 `--release` 也走 dev 模式** → 去连 `http://localhost:1420` → **"无法访问此页面"** |
| `custom-protocol` **开了** | 自包含，`dist` 资产内嵌进 exe |

**本项目的选择是「一直开着」** —— 因为作者踩过上面第一个坑
（用裸 `cargo build --release` 重编，`Cargo.toml` 里当时是 `features = []`，结果 exe 变回 dev 模式）。

**发布 checklist 里的一条**（见 `history/IMPLEMENTATION_PLAN.md` §6 / `history/PORTING_REPORT.md`）：

> release exe 重编（**确认 `custom-protocol` 仍在**，避免退回 dev 连 `localhost:1420`）

**验证方法**（`history/DEVELOPMENT_LOG.md` §8.3）：**在 exe 里搜 vite 的 hash chunk 文件名**
（如 `notes-CtBn0kik.js`）= 资产已内嵌；`localhost:1420` 字符串只是配置里的 `devUrl` 残留、不再被使用。

> ⚠️ **已知的代价**：`custom-protocol` 常开意味着 `cfg(dev)` 常关，
> **`tauri dev` 时不会走 `devUrl`，而是加载 `dist/`**。
> 也就是说**改了前端源码，`npm run tauri dev` 可能看不到变化，要先 `npm run build`**。
> **本条未经实测** —— 见 [log.md](log.md) 未解决 1。
>
> 能同时满足"dev 能热重载"和"裸 cargo 重编不掉链子"的写法是
> `[features] default = ["custom-protocol"]`（见 `deps.md` 第三节），**本项目未采用**。

---

## 五、运行

**双击**工作台注册的入口（工作台竖条点一下），或直接跑打包出的 exe。

> **仓库根没有任何 exe** —— 交付物在 `npm run tauri build` 的产物目录里，或由工作台持有。
> 见 [data.md](data.md) 的"外部持久状态"——**工作台的 `config/tools.json` 里写死了本 exe 的绝对路径**。

---

## 六、测试

```powershell
npm test                        # vitest，17 条（实测）
cd src-tauri; cargo test        # Rust —— ⚠️ 条数未实测，见第十一节
```

> Rust 侧的单测写在源码里（如 `summary.rs` 末尾有 6 条 `#[cfg(test)]`）——
> `cargo test` 跑的是它们，不是 `test/` 目录。

---

## 七、端口

| 端口 | 谁 | 常驻 | 何时 |
| --- | --- | --- | --- |
| **17321** | **只读对外接口** | ✅ **应用运行时常驻** | 只要 Note 在跑就在（实测：PID 29608 正在监听） |
| 1420 | vite dev server（`strictPort: true`） | ❌ | 仅 `npm run tauri dev` / `npm run dev` |
| 1421 | vite HMR | ❌ | 仅 dev（`vite.config.ts` 里写死） |

> ⚠️ **`vite.config.ts` 里 `host: "0.0.0.0"`** —— dev server **绑所有网卡**，不是只绑回环。
> 本机开发无所谓；**但这台机器若在不可信网络上，dev 模式会把源码暴露给局域网**。

详见 [service.md](service.md)。

---

## 八、已知限制

| # | 限制 | 出处 |
| --- | --- | --- |
| 1 | **仅 Windows** —— 依赖 `windows-sys`（窗口定位、点击热区） | README |
| 2 | **需要 Rust 工具链（MSVC）与 Node.js** | README |
| 3 | **工作台是另一个独立项目** —— 本仓库只含便签插件；`src/workbench/` 是**开发用的最小外壳** | README |
| 4 | **透明窗口叠加 transform 动画在 Windows 上仍有合成器伪影，未闭环** | README（详见 `history/DEVELOPMENT_LOG.md`） |
| 5 | **`custom-protocol` 常开 → `tauri dev` 可能不走热重载**（未实测） | 本文第四节的推演 |
| 6 | **便签数据不在仓库里** —— 克隆后是一份空库 | README |

---

## 九、项目名对照（省得你以为是别的项目）

| 地方 | 名字 |
| --- | --- |
| 目录 / GitHub | `Note` |
| `package.json` / `Cargo.toml` | `my-workbench-notes` |
| `tauri.conf.json` 的 `productName` | `MyWorkbench` |
| `tauri.conf.json` 的 `identifier` | `com.myworkbench.notes` |
| README 标题 | `便签` |

**同一个东西。** 见 [README.md](README.md) 第四节。

---

## 十、证据索引

> 格式 `文件:行号`，路径相对 `Note\`。行号是 2026-10-03 第二十站实测值。

### 10.1 安装 / 开发 / 构建（第一～三节）

| 断言 | 证据 |
| --- | --- |
| 仓库根就是 npm 工程根 | `package.json` 在仓库根（实测） |
| 5 个 npm scripts | `package.json:6-12`（`dev` `build` `preview` `test` `tauri`） |
| 运行时依赖 4 个 | `package.json:13-18`（`@tanstack/react-virtual` `@tauri-apps/api` `react` `react-dom`） |
| 开发依赖 7 个 | `package.json:19-27`（`@tauri-apps/cli` `@types/react` `@types/react-dom` `@vitejs/plugin-react` `typescript` `vite` `vitest`） |
| `npm run build` = `tsc --noEmit && vite build` | `package.json:8` |
| 窗口是 56×64 无边框透明竖条 | `src-tauri/tauri.conf.json:18-24`（`width:56` `height:64` `decorations:false` `transparent:true`） |
| ⚠️ 「首次编译约 **462 个 crate**（约 2 分钟）」 | 仓库根 `README.md:40` —— **本轮未跑构建，未独立验证** |
| `dist/` 是构建产物 | 实测 `dist/` 7 文件 / 0.2 MB（`index.html` `notes.html` + `assets/` 5 个） |
| Rust 工具链在 | 实测 `cargo` = `C:\Users\asus\.cargo\bin\cargo.exe` |

### 10.2 `custom-protocol`（第四节）

| 断言 | 证据 |
| --- | --- |
| `Cargo.toml` 写死两个 feature | `src-tauri/Cargo.toml:11`（`tauri = { version = "2", features = ["protocol-asset", "custom-protocol"] }`） |
| **没有 `[features]` 段** | `Cargo.toml` 全文（1-27 行）**无 `[features]`** |
| 作者自己的记录 | `history/DEVELOPMENT_LOG.md` §8.3 / §10.1（原文已在 [deps.md](deps.md) 第三节逐字引用） |
| 能同时满足两边的写法 | `[features] default = ["custom-protocol"]` —— **本项目未采用**（`Cargo.toml` 无 `[features]`） |
| 验证方法：在 exe 里搜 vite 的 hash chunk 文件名 | 实测 `dist/assets/` 里的 chunk 名 = **`notes-D9mOxkb4.js`**（85,526 B）· `event-uN3fBrrf.js`（144,604 B）· `main-BtsBY_Pk.js` · `main-BAzd9ioo.css` · `notes-DAaNLt_e.css` —— **这 5 个名字就是"要搜的字符串"** |
| ⚠️ 「`tauri dev` 时不会走 `devUrl`，而是加载 `dist/`」 | **推导，未实测** —— 原文已自标"未经实测"（第四节末）✅ |

### 10.3 运行 / 测试（第五、六节）

| 断言 | 证据 |
| --- | --- |
| 仓库根没有任何 exe | 实测根下 **20 项**，全为目录或文本文件 |
| 交付物由工作台持有 | 见 [data.md](data.md) 第四节（工作台 `tools.json` 写死 exe 绝对路径） |
| `npm test` = vitest | `package.json:10`（`"test": "vitest run"`） |
| **vitest 17 条** | 实测 `test/opSeq.test.ts` 里 `it(` = **17** ✅ **与原文一致** |
| `cargo test` 跑的是源码里的 `#[cfg(test)]` | `summary.rs:63-106`（`mod tests`）· `db.rs` 末尾的 `mod tests` |
| 🔴 **「Rust，12 条」**（**E 段改前的写法**） | 实测 `#[test]` 属性 = **23 个**（`db.rs` **17** + `summary.rs` **6**）。⚠️ **与改前的"12 条"不符** —— **但没跑过 `cargo test`，无法判定是 12 还是 23**。**已改成不写数字，见第十一节 11.1** |

### 10.4 端口（第七节）

| 断言 | 证据 |
| --- | --- |
| **17321 固定、写死** | `src-tauri/src/notes/http.rs:30`（`pub const PORT: u16 = 17321;`） |
| **17321 实测在监听** | `Get-NetTCPConnection -State Listen` 实测：`127.0.0.1:17321`，OwningProcess = **29608** |
| 1420 + `strictPort` | `vite.config.ts:5`（`const port = 1420;`）· `:13`（`strictPort: true`） |
| **1421 HMR** | `vite.config.ts:18`（`port: 1421`）· `:17`（`host: "localhost"`） |
| ⚠️ **`host: "0.0.0.0"`** | `vite.config.ts:14` ✅ **原文正确** |
| 1420 / 1421 当前**未监听** | `Get-NetTCPConnection` 实测：两个端口**都不在**监听表里（dev server 没跑） |

### 10.5 已知限制（第八节）

| # | 断言 | 证据 |
| --- | --- | --- |
| 1 | 仅 Windows | `Cargo.toml:21-22`（`[target.'cfg(windows)'.dependencies] windows-sys`）· `notes/window.rs:29`（`#[cfg(target_os = "windows")]`） |
| 2 | 需要 Rust + Node | 实测 `cargo` 存在；`package.json` + `Cargo.toml` 都需要 |
| 3 | 工作台是另一个独立项目 | `src/workbench/`（3 文件：`App.tsx` `main.tsx` `workbench.css`）是**最小外壳**；真正的 `my-workbench.exe` 在 `D:\MyProject\Workbench\src-tauri\target\release\`（实测 PID 12104 的 `Path`） |
| 4 | 合成器伪影未闭环 | 仓库根 `README.md:73` → `history/DEVELOPMENT_LOG.md` §19.8 —— ⚠️ **主观观察，未独立验证** |
| 5 | `custom-protocol` 常开 → dev 可能不热重载 | 见 10.2 末条 —— **原文已自标"未实测"** ✅ |
| 6 | 数据不在仓库里 | `src-tauri/src/notes/mod.rs:15`（`app.path().app_data_dir()?.join("notes.db")`） |

---

## 十一、未验证与订正

### 11.1 🔴 实测发现的疑似错误（1 处）—— **已于 E 段修正**

| 位置 | 原文 | 实测 | 处置 |
| --- | --- | --- | --- |
| 第六节 | `cargo test # Rust，12 条` | 源码里 `#[test]` 属性 = **23 个**（`db.rs` 17 + `summary.rs` 6） | ✅ **E 段已改成 `Rust —— ⚠️ 条数未实测，见第十一节`**（**不写数字** —— 因为没跑过 `cargo test`） |

**依据**：`Select-String -Pattern '#\[test\]'` 逐文件计数 = 23（实测）。
**⚠️ 但 12 与 23 谁对，判不了** —— 两种可能：① 原文的 12 是旧数、后来加了测试；
② 部分测试在 `cargo test` 默认不跑（需 feature / 被 `#[ignore]`）。
**本轮禁止跑 `cargo test`（用户拍板），所以按纪律【不写数字】。**
已列进 [log.md](log.md) 第二十站未解决 4。

### 11.2 ⚠️ 本轮未独立验证的断言

| 断言 | 出处 | 状态 |
| --- | --- | --- |
| 首次编译约 462 个 crate / 约 2 分钟 | 仓库根 `README.md:40` | ⚠️ 未跑构建 |
| 合成器伪影未闭环 | 仓库根 `README.md:73` / `DEVELOPMENT_LOG.md` §19.8 | ⚠️ 主观观察 |
| `tauri dev` 加载 `dist/` 而非 `devUrl` | 本文第四节 | ⚠️ 推导（原文已自标） |
| 「双击工作台注册的入口」的运行方式 | 本文第五节 | ⚠️ **未实测启动** |

---

## 十二、2026-10-09 补记（第二十一站）

> **本节是第二十一站（窗口行为整改站）的补记。** 见 [log.md](log.md)。

### 12.1 🔴 第十一节那个「不写数字」的空白，**已实测填上**

本文 §六 与 §11.1 都记着同一件事：源码里 `#[test]` = **23 个**，而 `DEVELOPMENT_LOG.md` §6 写 **12/12**，
**「谁对，判不了」**（第九站、第二十站两轮都禁止跑 `cargo test`）。

**2026-10-09 实测结果**：

| 项 | 结果 |
| --- | --- |
| 命令 | `cargo test --release` |
| 结果 | ✅ **23 passed; 0 failed; 0 ignored**（含标签生命周期与四个迁移测试） |
| **→ 结论** | **源码的 23 是对的**；`DEVELOPMENT_LOG.md` §6 的 **12 是旧数** |

⚠️ 该数字是 **2026-10-09** 的实测值；本轮同时新增了代码（见 §12.3），**再往后要重新数**。

### 12.2 已知限制新增一条（第八节那张表补第 7 行）

| # | 限制 | 出处 |
| --- | --- | --- |
| 7 | 🔴 **透明窗口长时间隐藏后重新显示，会出现约 0.5～1 秒的合成停摆**（与第 4 条**同源**）—— 表现为"点不动、过一会儿又好了"。**已用「关闭即退出进程」规避**（窗口不再被隐藏 → 不再需要恢复合成），**但不是修好了它** | [log.md](log.md) 第二十一站第六节 · [runtime-log.md](runtime-log.md) |

> ⚠️ **第 4 条不动、不重编号**（重编号会让别处的引用失效）。本条与第 4 条**是同一个未闭环问题的两个面向**：
> 第 4 条记"有伪影"，本条记"伪影在隐藏→显示时表现为卡顿"。
>
> **第八节那张表本身未改** —— 按本项目纪律，旧内容不改，只在补记里追加。

### 12.3 🔴 行号漂移提示（用前请重新定位）

| 本文引用 | 为什么失效 |
| --- | --- |
| §10.5 的 `notes/window.rs:29`（`#[cfg(target_os = "windows")]`） | 2026-10-09 改了 `window.rs`（移除 `always_on_top`、重写 `on_page_load`）后**行号位移** |

**引用 `window.rs` / `main.rs` / `commands.rs` / `hittest.rs` 的行号，用前请重新定位**（完整清单见 [log.md](log.md) 第二十一站第五节）。

### 12.4 构建状态

| 项 | 值 |
| --- | --- |
| 当前 exe | `src-tauri\target\release\my-workbench-notes.exe`，**2026-10-09 11:15:39** |
| 前端产物 | `dist\` 同期重建，已嵌入 exe |
| `tsc --noEmit` / `cargo check --release` / `cargo test --release` | ✅ 全部通过 |
