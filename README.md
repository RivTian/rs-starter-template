# rs-starter-template

一条命令，得到一个优雅、开箱即用的多 crate Rust 服务骨架：

```bash
cargo generate RivTian/rs-starter-template --name acme-svc
cd acme-svc && just dev
```

三个问题（描述、许可证、是否要 Docker），其余全部有默认值；`--silent -d ...` 可零交互生成。

## 生成后长什么样

```text
acme-svc/
├── crates/
│   ├── acme-svc/             bin：CLI · 配置 · runtime · 依赖装配（bootstrap.rs）
│   ├── acme-svc-api/         HTTP：router · 中间件 · handlers · DTO · problem+json 错误
│   ├── acme-svc-infra/       适配器：内存仓储 · 系统时钟 · 事件总线桥 · 后台任务
│   ├── acme-svc-core/        运行时核心：Server · Service · 优雅关停 · 就绪 · 健康 · telemetry · EventBus
│   ├── acme-svc-domain/      纯模型：实体 · 端口 · DomainError · DomainEvent
│   ├── acme-svc-config/      配置 schema · 分层加载 · 校验
│   ├── acme-svc-util/        共享内核：错误分类 · id · Secret<T>
│   └── acme-svc-test-utils/  仅测试：进程内测试服务器 · 假时钟 · 事件记录器
├── config/default.toml       每个键 + 默认值 + 注释
├── docs/architecture.md      规则、图、如何加功能；docs/adr/ 决策记录
├── justfile                  dev · ci · smoke · docker-build …
├── Dockerfile                cargo-chef 缓存 + distroless + 非 root + 自带 HEALTHCHECK 探针
└── .github/workflows/        ci（fmt/clippy/nextest/msrv/doc/deny/typos）· release · docker
```

开箱即有：`/healthz` `/readyz` `/metrics` `/version`、结构化日志（pretty/json）、request-id、SIGTERM 优雅关停（grace period）、就绪状态机、进程内事件总线、示例领域（todo）+ 端到端测试 + 架构守卫测试。依赖方向只能向下（`bin → api/infra → core → domain/config → util`），违规即编译失败。

| 层  | crate                                         | 只允许依赖                    |
| --- | --------------------------------------------- | ----------------------------- |
| L4  | `acme-svc`（bin）                             | 全部                          |
| L3  | `acme-svc-api` / `acme-svc-infra`（互不可见） | domain · core · config · util |
| L2  | `acme-svc-core`                               | config · util                 |
| L1  | `acme-svc-domain` / `acme-svc-config`         | util                          |
| L0  | `acme-svc-util`                               | —                             |

## 五分钟体验

```bash
cargo generate RivTian/rs-starter-template --name acme-svc
cd acme-svc
cp .env.example .env
just dev                        # 第一行日志：version / commit / build；最后一行：all services ready
curl -s localhost:8080/healthz  # {"status":"ok"}
curl -s localhost:8080/readyz   # {"status":"ready","checks":{"todo_repository":{"status":"ok"}}}
curl -s -XPOST localhost:8080/api/v1/todos -H 'content-type: application/json' -d '{"title":"ship it"}'
just ci                         # fmt · clippy -D warnings · nextest · doctests · rustdoc · deny · typos
# Ctrl-C → "draining" → "clean shutdown"
```

需要：stable Rust ≥ 1.96、[`just`](https://just.systems)、`cargo-nextest`、`cargo-deny`、`typos`（`cargo install cargo-nextest cargo-deny typos-cli`，或用系统包管理器）。

## 零交互生成

```bash
cargo generate RivTian/rs-starter-template --name acme-svc --silent \
  -d project-description="Orders service" -d license=MIT -d include_docker=true -d repository=
```

| 占位符                | 类型                                                                   | 默认                                            |
| --------------------- | ---------------------------------------------------------------------- | ----------------------------------------------- |
| `project-description` | string                                                                 | `A Rust service built from rs-starter-template` |
| `license`             | `MIT` / `Apache-2.0` / `None`（不生成 LICENSE，也不写 `license` 字段） | `MIT`                                           |
| `include_docker`      | bool                                                                   | `true`                                          |
| `repository`          | string，可空                                                           | 空                                              |

## 仓库布局（模板维护者）

```text
template/     cargo-generate 模板源（含 Liquid 占位符，本身不可编译）
generated/    用默认答案生成的快照：独立 workspace，永不手改，CI 在此编译
docs/design/  设计文档：研究（Pumpkin / Pingora / ratatui-templates）→ 原则 → 架构 → 模板工程
scripts/      文档与模板维护脚本
justfile      regen / verify / drift / matrix / smoke
```

```bash
just regen      # 用默认答案重新生成 generated/acme-svc
git diff generated/
just verify     # 快照跑自己的 just ci
just drift      # 模板改了但快照没更新 → 失败
just matrix     # zz-widget / my-api 两组不同答案各生成一遍并跑 ci
```

设计从 [Pumpkin-MC/Pumpkin](https://github.com/Pumpkin-MC/Pumpkin)（大型 Rust 应用如何组织）、[cloudflare/pingora](https://github.com/cloudflare/pingora)（生产级框架如何设计）与 [ratatui/templates](https://github.com/ratatui/templates)（模板工程链）提炼而来，见 [`docs/design/`](docs/design/README.md)。

## 许可证

模板仓库：MIT。生成项目的许可证由生成时的 `license` 占位符决定。
