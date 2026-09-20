# rs-starter-template

> **状态：实施中（P0 已开始）。** 模板本身尚未可用；设计文档见 [`docs/design/`](docs/design/README.md)。

一条命令，得到一个优雅、开箱即用的多 crate Rust 服务骨架：

```bash
cargo generate riotian/rs-starter-template --name acme-svc
```

生成的项目（示例名 `acme-svc`）包含：

| 层 | crate | 职责 |
|----|-------|------|
| L4 | `acme-svc`（bin） | CLI、配置加载、runtime、依赖装配 |
| L3 | `acme-svc-api` / `acme-svc-infra` | HTTP 传输 / 端口适配器（互不可见） |
| L2 | `acme-svc-core` | Server、Service、优雅关停、就绪、telemetry、事件总线、错误分类 |
| L1 | `acme-svc-domain` / `acme-svc-config` | 纯领域模型与端口 / 配置 schema |
| L0 | `acme-svc-util` | 共享小工具 |
| dev | `acme-svc-test-utils` | 进程内测试服务器、假时钟 |

开箱即有：`/healthz` `/readyz` `/metrics` `/version`、结构化日志、request-id、SIGTERM 优雅关停、`just ci`（fmt / clippy / nextest / doc / deny / typos）、Dockerfile、GitHub Actions。

## 仓库布局

```text
template/     cargo-generate 模板源（含 Liquid 占位符，本身不可编译）
generated/    用默认答案生成的快照（独立 workspace，永不手改，CI 在此编译）
docs/design/  设计文档：研究（Pumpkin / Pingora / ratatui-templates）→ 原则 → 架构 → 模板工程
scripts/      文档与模板维护脚本
justfile      模板作者任务：regen / verify / drift / matrix / smoke
```

## 模板作者的工作循环

```bash
just regen      # 用默认答案重新生成 generated/
git diff generated/
just verify     # 快照跑自己的 just ci
just matrix     # 三组答案各生成一遍并跑 ci
```

## 许可证

模板仓库：MIT。生成项目的许可证由生成时的 `license` 占位符决定。
