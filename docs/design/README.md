# rs-starter-template · 设计文档集

> **状态：方案设计稿，第一次交付，等待确认。** 本阶段只有调研、方案与文档；没有任何实现代码、Cargo 配置、生成工具链或 CI 被创建或修改。
> 目标：让新用户通过一条 `cargo generate` 得到一个**优雅、开箱即用的多 crate Rust 服务骨架**。

## 阅读路径

| 读者       | 时间    | 顺序                                                                          |
| ---------- | ------- | ----------------------------------------------------------------------------- |
| 只想拍板   | 30 分钟 | `00` §1–§3 → `00` §7 决策清单 → `05` §1 一张图 → `06` §3 五分钟体验 → `08` §2 |
| 想看架构   | 2 小时  | `04` → `05` → `06` → 回头看 `01` / `02` 的 Adopt/Adapt/Avoid                  |
| 要维护模板 | 1 小时  | `03` → `07` → `08`                                                            |
| 想核对证据 | 按需    | `01` / `02` 每条结论都带 `path:line`；`03` §10 带 cargo-generate 源码行号     |

## 文档清单

| 文件                                                                                                         | 内容                                                                                   | 主要图表                                                                                               |
| ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| [`00-goals-and-decisions.md`](00-goals-and-decisions.md)                                                     | 目标、非目标、硬约束、术语、验收标准、**18 条待确认决策**                              | 表格                                                                                                   |
| [`01-research-pumpkin.md`](01-research-pumpkin.md)                                                           | Pumpkin：大型 Rust Application 如何组织与演进                                          | crate 依赖 mermaid、启动/事件伪代码、Adopt/Adapt/Avoid                                                 |
| [`02-research-pingora.md`](02-research-pingora.md)                                                           | Pingora：生产级 Framework 如何设计                                                     | 分层 mermaid、请求阶段 mermaid、`run_forever` 伪代码、连接流水线 ASCII                                 |
| [`03-research-ratatui-templates.md`](03-research-ratatui-templates.md)                                       | 模板工程链 + cargo-generate 能力边界（源码核实）                                       | 三件套 mermaid、hook 伪代码、能力表                                                                    |
| [`04-principles.md`](04-principles.md)                                                                       | 六个架构问题的回答 → 六条原则 → 落点                                                   | 分工 ASCII、解耦 ASCII、对照表                                                                         |
| [`05-architecture.md`](05-architecture.md)                                                                   | 生成项目的架构：分层、依赖规则、生命周期、扩展点、错误、配置、可观测、事件、测试、演进 | 分层 mermaid + ASCII、启动时序、就绪状态机、错误流、请求时序                                           |
| [`06-generated-layout.md`](06-generated-layout.md)                                                           | 生成后的目录逐文件说明 + 关键文件伪代码                                                | 目录树、`main.rs` / `Server::run` / `EventBus` / `Config` / `HttpService` / justfile / CI / Dockerfile |
| [`07-template-engineering.md`](07-template-engineering.md)                                                   | 模板仓库：布局、占位符、hooks、Liquid 规范、快照、justfile、CI                         | 定位流程、CI mermaid                                                                                   |
| [`08-roadmap.md`](08-roadmap.md)                                                                             | 确认后的实施阶段、完成定义、风险                                                       | 阶段 mermaid                                                                                           |
| [`references/pumpkin-vs-pingora-source-study-focus.md`](references/pumpkin-vs-pingora-source-study-focus.md) | 原始研究提纲（任务输入）                                                               | —                                                                                                      |
| [`references/agent-report-pumpkin.md`](references/agent-report-pumpkin.md)                                   | Opus 子 agent 的 Pumpkin 深读报告原文（1077 行，证据附录）                             | crate 依赖 mermaid、伪代码                                                                             |
| [`references/agent-report-pingora.md`](references/agent-report-pingora.md)                                   | Opus 子 agent 的 Pingora 深读报告原文（1585 行，证据附录）                             | 分层 mermaid、阶段 mermaid、主链路 ASCII                                                               |

## 图例与约定

| 约定                                          | 含义                                                  |
| --------------------------------------------- | ----------------------------------------------------- |
| **[Pumpkin]** / **[Pingora]** / **[ratatui]** | 该设计点的来源仓库                                    |
| `crates/xxx/src/file.rs:LINE`                 | 研究基线 commit 上的证据位置                          |
| `05 §4.2`                                     | 交叉引用：文档编号 + 章节                             |
| mermaid                                       | 结构与流程图（GitHub 原生渲染）                       |
| ASCII                                         | 可复制进 README / 终端的图                            |
| 伪代码                                        | Rust 风格但省略细节，只表达结构与顺序；不是可编译代码 |
| 表格                                          | 全部由脚本按东亚字宽对齐                              |

## 研究基线

| 仓库               | commit                                  | 日期       | 说明                                     |
| ------------------ | --------------------------------------- | ---------- | ---------------------------------------- |
| Pumpkin-MC/Pumpkin | `e413623`                               | 2026-09-20 | 20 个 workspace 成员，主 crate ≈ 26 万行 |
| cloudflare/pingora | `4487f7b`                               | 2026-09-03 | 23 个 workspace 成员，core ≈ 4.9 万行    |
| ratatui/templates  | `5903368`                               | 2026-09-06 | 6 个子模板 + 6 个生成快照                |
| cargo-generate     | 本机 0.24.0；文档源码 0.25.0            | 2026-09    | 子模板定位与默认忽略规则已核实源码       |
| 本机工具链         | rustc / cargo 1.96.0，cargo-deny 0.20.2 | 2026-09-20 | 未安装 `just`、`cargo-nextest`           |

## 本阶段边界

- 工作目录当前**不是 git 仓库**；`08` §5 给出确认后的第一条命令（`git init -b main`）。
- `docs/prompts/` 保持原样。
- 研究用的仓库克隆位于 `/tmp/rs-template-research/`，不在项目内。

## 如何确认

回复 `00` §7 中每条决策的"同意 / 改为 …"即可；未提及的按推荐值执行。也欢迎直接在文档上批注。
