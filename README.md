<div align="center">

# 便签

**一个接入工作台的便签应用 —— 卡片记事，结构树理思路。**

![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=flat-square&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-61DAFB?style=flat-square&logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white)

![License](https://img.shields.io/badge/license-MIT-4B5563?style=flat-square)
![Last Commit](https://img.shields.io/github/last-commit/3431481073/Note?style=flat-square&color=4B5563)
![Repo Size](https://img.shields.io/github/repo-size/3431481073/Note?style=flat-square&color=4B5563)
![Status](https://img.shields.io/badge/status-active-4B5563?style=flat-square)

</div>

---

## 这个项目解决什么问题

随手记下来的东西会散。记得越多，越找不回、越理不清。

这个便签给同一份数据两种看法：**卡片模式**用来快速记和快速找，**结构树模式**把一堆零散便签整理成一棵可折叠的树。随时切换，不用搬数据。

它不是独立软件 —— 作为插件挂进「工作台」。工作台是一个 56×64 的无边框竖条，贴在屏幕边缘；点一下浮出便签窗口，不占桌面、不进任务栏。

## 快速开始

```bash
npm install
npm run tauri dev
```

打包成可执行文件用 `npm run tauri build`。
跑测试：`npm test`（vitest，17 条）+ `cd src-tauri && cargo test`（Rust，12 条）。

> 首次 `tauri dev` 会编译约 462 个 crate（约 2 分钟），之后增量很快。

## 功能

| 功能 | 说明 |
| --- | --- |
| 卡片模式 | 快速记事，虚拟列表滚动，按时间倒序，游标分页 |
| 结构树模式 | 把便签组织成可折叠的树，节点可增删改、卡片可导入 |
| 标签与项目 | 便签可打多个标签、绑定项目；过滤为标签 AND / 项目 OR，在后端完成以保证分页正确 |
| 搜索 | 同时搜标题与正文，后端转义 LIKE 通配符 |
| 回收站 | 删除进回收站，可恢复或彻底清除；回收站里的纸面只读 |
| 墨色主题 | 墨色按月切换 12 色（`config/seasons.json`）+ 当季节气水印 SVG |
| 光带特效 | CSS `@property` + `conic-gradient` 旋转流光，`prefers-reduced-motion` 下自动降级 |
| 单实例 | 重复启动不会开出第二个窗口 |
| 工作台集成 | 注册进工作台，通过事件同步便签数角标 |

## 文档

| 文件 | 内容 |
| --- | --- |
| `UI_SPEC.md` | 界面与交互规格：布局、组件清单、状态机、动效、复现要点 |
| `DEVELOPMENT_LOG.md` | 开发日志：项目背景、踩过的坑、验证结果、沉淀的认知 |
| `IMPLEMENTATION_PLAN.md` | 实施计划 |
| `PORTING_REPORT.md` | v12 移植进现有 Tauri 应用的可行性 / 冲突 / 落地分析 |
| `123.txt` / `456.txt` | UI 规格导出文档与移植讨论的往来记录 |

## 已知限制

- **仅 Windows** —— 依赖 `windows-sys`（窗口定位、点击热区）
- 需要 **Rust 工具链**（MSVC）与 Node.js
- 工作台是**另一个独立项目**；本仓库只含便签插件，`src/workbench/` 是开发用的最小外壳
- 便签数据存在 Tauri 的 appDataDir（`$APPCONFIG`），**不在仓库里** —— 克隆后是一份空库
- 透明窗口叠加 transform 动画在 Windows 上仍有合成器伪影，**未闭环**（详见 `DEVELOPMENT_LOG.md` §19.8）

## 许可证

MIT

本项目采用 MIT 许可证，详见 LICENSE。
