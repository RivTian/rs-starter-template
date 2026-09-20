# 00 · 目标、范围、约束与决策清单

> 状态：**方案设计稿（第一次交付）**。本阶段只产出调研与设计文档；任何实现（Cargo 配置、模板源、生成工具链、CI）都等待确认后才开始。

## 1. 一句话目标

新用户执行一条命令：

```bash
cargo generate RivTian/rs-starter-template --name acme-svc
```

得到一个**优雅、开箱即用、多 crate** 的 Rust 服务骨架：目录一眼能懂，`just dev` 立刻跑起来，`curl /healthz` 有回应，`Ctrl-C` 优雅退出，`just ci` 一次通过，且骨架本身已经把"往后怎么长大"的路铺好。

## 2. 目标用户与关键旅程

| 角色                       | 旅程                                                                                    | 成功信号                                         |
| -------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------ |
| 第一次用模板的开发者       | `cargo generate` → 回答 3–4 个问题 → `cd` → `just dev`                                  | 5 分钟内看到 `/healthz` 返回 200，日志结构清晰   |
| 要加第一个业务功能的开发者 | 读 `docs/architecture.md` → 在 `domain` 加类型与 port → 在 `infra` 实现 → 在 `api` 暴露 | 不需要改 `core`；知道每个文件该放哪              |
| 要上线的开发者             | `just ci` → `docker build` → 配置环境变量 → 部署                                        | 优雅关停、就绪探针、版本信息、结构化日志开箱即有 |
| 模板维护者（你）           | 改 `template/` → `just regen` → 看 `git diff generated/` → CI 通过                      | 模板改动的真实效果可审查、可编译                 |

## 3. 目标与非目标

**目标**

| #   | 目标                                                                               | 度量                                             |
| --- | ---------------------------------------------------------------------------------- | ------------------------------------------------ |
| G1  | 多 crate、依赖方向受控的 workspace                                                 | `cargo tree` 无环；层级规则在 CI 可检查          |
| G2  | Core 与业务解耦：加功能不改 `core`                                                 | 示例功能只触及 `domain`/`infra`/`api`            |
| G3  | 生产级生命周期：启动顺序、就绪、优雅关停、超时兜底                                 | 集成测试覆盖 SIGTERM 路径                        |
| G4  | 分层错误模型：领域错误 / 基础设施错误 / API 错误分离，且能分类（可重试？谁的错？） | 每层一个错误类型；API 层统一映射                 |
| G5  | 配置分层：默认值 → 文件 → 环境变量 → CLI，且可 `--check-config`                    | 单元测试覆盖覆盖顺序                             |
| G6  | 可观测性默认开启：tracing、request-id、`/healthz` `/readyz` `/metrics` `/version`  | 生成即有                                         |
| G7  | 严格但可解释的 lint 策略                                                           | `cargo clippy --all-targets -- -D warnings` 通过 |
| G8  | 模板工程链可验证：生成 → 编译 → 测试 → 快照漂移检查                                | 模板仓库 CI 全绿                                 |
| G9  | 文档随骨架生成：`docs/architecture.md`、ADR 目录、`CONTRIBUTING.md`                | 生成物内含 mermaid 依赖图                        |

**非目标（本期明确不做）**

| #   | 非目标                                                 | 原因                                                                |
| --- | ------------------------------------------------------ | ------------------------------------------------------------------- |
| N1  | 不内置数据库（sqlx/sea-orm）                           | 会把 CI 绑到外部服务；先用 in-memory adapter 演示 port/adapter 模式 |
| N2  | 不内置 gRPC / WebSocket                                | 传输层留下扩展缝，v1 只做 HTTP                                      |
| N3  | 不内置认证/授权                                        | 业务差异太大                                                        |
| N4  | 不做 Pingora 那种自定义 tokio runtime、fd 传递式热升级 | 收益/复杂度比不适合骨架                                             |
| N5  | 不做插件系统（dylib/wasm）                             | Pumpkin 的插件机制面向游戏生态                                      |
| N6  | 不提供多套子模板（cli / lib / tui）                    | 需求是"一个多 crate 服务骨架"                                       |
| N7  | 不引入 nix / devcontainer                              | 可后补，不影响架构                                                  |

## 4. 来自任务书的硬约束

| 约束                                                           | 本文档如何遵守                                                                                                       |
| -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| 在 `main` 上从零设计；不是修补旧模板、不恢复备份分支           | 当前目录只有 `.gitignore` 与空 `docs/prompts/`；所有设计不引用任何旧实现                                             |
| 本阶段仅调研 + 方案 + 文档；首次交付后停止等待确认             | 本次只写 `docs/design/**`；`08-roadmap.md` 列出确认后的实施步骤                                                      |
| 必须研究并吸收 Pumpkin、Pingora、ratatui/templates             | `01`/`02`/`03` 三份研究文档，每份以 Adopt / Adapt / Avoid 收口                                                       |
| ratatui/templates 只研究模板工程链                             | `03` 明确不引入 TUI 业务、不照搬多子模板根结构                                                                       |
| 文档必须含 mermaid、ASCII、对齐表格、伪代码                    | 每份文档均含；表格由脚本按东亚字宽对齐                                                                               |
| 允许有限数量的 Opus 子 agent                                   | 使用 2 个 Opus 子 agent 分别深读 Pumpkin 与 Pingora；原始报告存于 `references/agent-report-*.md`，关键引用已抽查核实 |
| 整合《pumpkin-vs-pingora-source-study-focus.md》到仓库研究部分 | 原件存于 `references/`；其研究方向表、阅读路线、6 个架构问题被 `01`/`02`/`04` 逐项承接                               |

## 5. 术语表

| 术语             | 本文档中的含义                                                                  |
| ---------------- | ------------------------------------------------------------------------------- |
| Workspace        | Cargo workspace；生成项目的根 `Cargo.toml`                                      |
| Crate            | workspace 成员；命名 `<project>-<role>`                                         |
| Layer（层）      | 依赖方向的分组：`util` → `domain` / `config` → `core` → `infra` / `api` → `app` |
| Core             | 与业务无关的运行时与生命周期能力（Service、Shutdown、Telemetry、ErrorKind）     |
| Port / Adapter   | 领域层定义的 trait（port）与基础设施层的实现（adapter）                         |
| Extension Point  | 用户不改 `core` 就能接入行为的 trait / hook / 注册点                            |
| Service          | 一个随进程生命周期运行的组件（HTTP 监听、后台任务），实现 `Service` trait       |
| ShutdownToken    | 传播关停信号的 `CancellationToken` 句柄（对应 Pingora 的 `ShutdownWatch`）      |
| CTX              | 单次请求的上下文（request-id、span、租户等；对应 Pingora `ProxyHttp::CTX`）     |
| Snapshot（快照） | 用默认答案从 `template/` 生成并提交到仓库的真实项目 `generated/`                |
| Placeholder      | `cargo-generate.toml` 中声明的输入变量                                          |
| Hook             | cargo-generate 的 Rhai 脚本（init / pre / post）                                |

## 6. 验收标准（实施完成的定义）

```text
[ ] cargo generate --path template --name acme-svc --silent -d ...   零交互成功
[ ] 生成目录内 grep -rn '{{' 为空（无未渲染占位符）
[ ] cd acme-svc && just ci  通过：fmt / clippy -D warnings / nextest / doc / deny
[ ] just dev 启动；curl :8080/healthz → 200；/readyz 在依赖就绪前 503、之后 200
[ ] kill -TERM 后：停止接受新连接 → 等在途请求 ≤ grace → 退出码 0；超时则强制退出并记录
[ ] docker build 成功，镜像以非 root 运行，HEALTHCHECK 生效
[ ] 模板仓库 CI：矩阵生成（默认 / 全关 / 全开 三组答案）+ 快照漂移检查 全绿
[ ] docs/architecture.md 里的依赖图与 cargo tree 一致
```

## 7. 决策清单（请逐条确认或改写）

每条给出**推荐值**与备选；推荐值是后续文档的默认假设。

| #   | 决策                  | 推荐                                                                                                                                                            | 备选                                         | 依据                                                    |
| --- | --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------------------------------- |
| D1  | 模板仓库布局          | 根目录无 `cargo-generate.toml`；`template/` + 提交的 `generated/` 快照                                                                                          | ratatui 式多子模板；或根目录即模板           | `03` §10.5、§12；`07` §1                                |
| D2  | 生成项目的 crate 集合 | 8 个：`<p>`(bin) `-core` `-config` `-domain` `-infra` `-api` `-util` `-test-utils`                                                                              | 去掉 `-util`（5–7 个）；或加 `-macros`       | `05` §2                                                 |
| D3  | crate 目录命名        | 目录名 = 包名（`crates/<p>-core`）                                                                                                                              | 目录不带前缀（`crates/core`）                | Pumpkin / Pingora 均为前缀式                            |
| D4  | HTTP 框架             | axum 0.8                                                                                                                                                        | actix-web / poem                             | 生态默认；Pumpkin 自身也依赖 axum                       |
| D5  | 配置加载              | `config` crate + serde 默认值 + `APP__A__B` 环境变量                                                                                                            | `figment`；手写 toml + env                   | ratatui 有实证；语义足够                                |
| D6  | 错误模型              | 每 crate `thiserror` 枚举 + `core::Classify`（kind / source / retryable）+ `api::ApiError` 映射；`anyhow` 仅限 bin 启动路径                                     | Pingora 式单一富结构 `Error`                 | `01` §11（`PumpkinError` 策略 trait）、`02` §7、`05` §6 |
| D7  | Lint 策略             | Pumpkin 全量 `deny` 集（与 Mikan-rs 现状一致），`[lints] workspace = true`                                                                                      | pedantic 降为 warn                           | `01` §3.3                                               |
| D8  | 任务运行器            | `just`                                                                                                                                                          | `cargo xtask`                                | 任务书点名 Justfile                                     |
| D9  | 测试运行器            | `cargo nextest`（CI）+ `cargo test` 兼容                                                                                                                        | 仅 `cargo test`                              | Pumpkin CI 已用 nextest                                 |
| D10 | 关停机制              | `tokio-util` `CancellationToken` + `TaskTracker` + grace timeout                                                                                                | 手写 `watch` 通道（Pingora 式）              | 标准库级方案，无自研                                    |
| D11 | 事件总线              | `core::EventBus`（`tokio::sync::broadcast`）+ `domain::DomainEvent`                                                                                             | 不内置                                       | 回答"模块间如何通信"                                    |
| D12 | 占位符集合            | `project-description`、`license`（MIT/Apache-2.0/MIT OR Apache-2.0）、`include_docker`(bool)、`repository`(可空)                                                | 增加 `http_port`、`include_ci`               | 越少越好；其余生成后可改                                |
| D13 | 版本信息              | `vergen-gix` 注入 git/构建信息                                                                                                                                  | `git describe` 手写 build.rs                 | ratatui 实证                                            |
| D14 | Edition / MSRV        | edition 2024、`rust-version = "1.96"`、`resolver = "3"`、`rust-toolchain.toml` = stable                                                                         | 固定版本号 channel                           | Pumpkin 同款                                            |
| D15 | 可观测性范围          | tracing（pretty/json）+ `metrics` + `/metrics` Prometheus 文本 + request-id；OpenTelemetry 作为 feature `otel` 留缝不实现                                       | 直接内置 otel                                | 依赖体积                                                |
| D16 | 发布工作流            | `ci.yml` 与 `release.yml`（多平台二进制）必有；`docker.yml` + `Dockerfile` + compose 跟随 `include_docker`；镜像基于 distroless，健康检查用二进制自带 `--probe` | 全部默认生成；alpine + `nc -z`（Pumpkin 式） | 减少新用户噪音；更小攻击面                              |
| D17 | 许可证                | 生成物许可证由占位符决定；模板仓库自身 MIT                                                                                                                      | —                                            | —                                                       |
| D18 | `missing_docs`        | 生成物默认 `allow`，模板自身代码全部有文档；用户想要时改一行 `[workspace.lints.rust]`。**例外**：`config` crate 单独 `deny`，配置字段必须有文档                 | `warn`（会被 CI 的 `-D warnings` 升为错误）  | 不让新用户第一天就被文档 lint 拦住；配置文档是用户界面  |

## 8. 文档地图

| 文件                                | 内容                                                                                | 读者            |
| ----------------------------------- | ----------------------------------------------------------------------------------- | --------------- |
| `README.md`                         | 索引、阅读顺序、图例                                                                | 所有人          |
| `00-goals-and-decisions.md`（本文） | 目标、非目标、约束、术语、验收、决策                                                | 决策者          |
| `01-research-pumpkin.md`            | Pumpkin：大型 Rust Application 如何组织                                             | 架构            |
| `02-research-pingora.md`            | Pingora：生产级 Framework 如何设计                                                  | 架构            |
| `03-research-ratatui-templates.md`  | 模板工程链 + cargo-generate 能力边界                                                | 模板维护者      |
| `04-principles.md`                  | 6 个架构问题的回答 → 设计原则                                                       | 架构            |
| `05-architecture.md`                | 生成项目的架构：crate、依赖、生命周期、配置、错误、可观测、扩展点、事件、测试、演进 | 实施者          |
| `06-generated-layout.md`            | 生成后的目录逐文件说明 + 关键文件伪代码                                             | 实施者          |
| `07-template-engineering.md`        | 模板仓库：布局、占位符、hooks、快照、justfile、CI                                   | 模板维护者      |
| `08-roadmap.md`                     | 确认后的实施阶段与验收                                                              | 决策者 / 实施者 |
| `references/`                       | 原始研究提纲                                                                        | 溯源            |
