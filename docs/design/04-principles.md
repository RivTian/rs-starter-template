# 04 · 设计原则：六个架构问题的回答

> 研究提纲（`references/pumpkin-vs-pingora-source-study-focus.md` §5）把最终沉淀定为 **6 个架构问题**。本文逐条回答，每条给出：来自 Pumpkin / Pingora 的证据 → 提炼的原则 → 在本模板中的落点（指向 `05`/`06`）。
> 提纲的原话："Pumpkin 重点学怎么组织一个大型 Rust 应用，Pingora 重点学怎么设计一个可扩展 Rust 基础设施框架。"

## 0. 两个仓库的分工（提纲 §3 的图，标注落点）

```text
                        Rust 工程能力
                              │
                 ┌────────────┴────────────┐
                 ▼                         ▼
             Pumpkin                    Pingora
                 │                         │
     Application Architecture     Infrastructure Architecture
                 │                         │
      Domain Modeling  ──┐         ┌── Framework Design
      Crate Organization │         │   Trait / Extension
      Module Boundary    │         │   Runtime
      Event / ECS        │         │   Network
      Business Flow    ──┘         └── Connection
                 │                         │
                 └────────────┬────────────┘
                              ▼
                  rs-starter-template（本设计）
                  ├─ 05 §1  crate 分层 ◄── Pumpkin 的拆分 + Pingora 的叶子/门面
                  ├─ 05 §4  生命周期   ◄── Pingora Server / ShutdownWatch / Readiness
                  ├─ 05 §5  扩展点     ◄── Pingora ProxyHttp / ServerApp；Pumpkin 事件
                  ├─ 05 §6  错误       ◄── Pumpkin 分层枚举 + Pingora 分类三元组
                  ├─ 05 §7  配置       ◄── Pumpkin config crate + Pingora validate/-t
                  └─ 05 §9  事件       ◄── Pumpkin 事件系统的最小子集
```

## 1. Crate 怎么拆？

**证据**

| 仓库    | 观察                                                                                                                                                                        | 出处                                  |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| Pumpkin | 20 个成员按**能力**拆：`protocol`、`world`、`inventory`、`command`、`config`、`nbt`、`codecs`、`util`、`data`、`macros`、`plugin-*`；工具在 `tools/`（`codegen`、`fuzzer`） | 根 `Cargo.toml` `[workspace] members` |
| Pumpkin | 主 crate `pumpkin` 同时是 lib + bin，约 26 万行，包含 `block/ command/ entity/ item/ net/ plugin/ server/ world/` 等全部业务模块                                            | `crates/pumpkin/src/`                 |
| Pingora | 23 个成员按**基础设施层**拆：`error` → `http` → `core` → `proxy` / `cache` / `load-balancing` → 门面 `pingora`；TLS 后端各自一个 crate                                      | 根 `Cargo.toml`；`pingora/src/lib.rs` |
| Pingora | 叶子 crate（`error`、`timeout`、`runtime`、`pool`、`lru`、`ketama`）不依赖任何其他 pingora crate                                                                            | 各 crate `Cargo.toml`                 |

**原则 P1 —— 按"变化原因"和"依赖重量"拆，而不是按文件多少拆。**

一个 crate 应当满足其一：① 有独立的变化原因（协议版本、存储后端、传输方式）；② 有明显不同的依赖重量（纯逻辑 vs 拖着 tokio/openssl）；③ 需要被 ≥ 2 个 crate 复用。Pumpkin 主 crate 26 万行说明"应用 crate"天然会吞掉一切；对策不是拆得更细，而是让组合根薄到没有业务可放。

**落点**：`05` §1 的 8 个 crate；bin 只有 `main.rs` / `cli.rs` / `bootstrap.rs`；`05` §12 的拆分触发条件表。

## 2. Module 边界怎么定？

**证据**

| 仓库    | 观察                                                                                                                                                                                                                          | 出处                                                 |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------- |
| Pumpkin | 主 crate 内按**领域概念**分模块：`entity/{ai,mob,player,projectile,…}`、`block/{blocks,entities,fluid}`、`net/{java,bedrock,proxy,rcon}`；横切设施单独成文件：`crash.rs`、`error.rs`、`logging.rs`、`telemetry.rs`            | `crates/pumpkin/src/` 目录树                         |
| Pingora | `pingora-core` 内按**角色**分模块：`apps`（应用接口）、`services`（长期运行单元）、`listeners` / `connectors`（两个方向的连接）、`protocols`（l4 / tls / http）、`upstreams`、`server`（生命周期）、`modules`（可插拔处理链） | `pingora-core/src/lib.rs` 的 `pub mod` 列表          |
| Pingora | 每个 crate 提供 `prelude` 模块，门面 crate 再汇总                                                                                                                                                                             | `pingora-core/src/lib.rs` 末尾；`pingora/src/lib.rs` |

**原则 P2 —— 模块是一个概念的完整闭包；对外只暴露最小面；跨模块只走 trait 或事件。**

- 领域模块 = `model`（类型与不变量）+ `repository`（port）+ `service`（规则与流程）三件套，放在同一目录。
- `mod.rs` / `lib.rs` 只做 `pub use` 与文档，不放逻辑。
- 横切设施（错误、日志、配置）不散落，集中在 `core` / `config`。
- 每个 crate 有 `prelude`，用户 `use acme_svc_core::prelude::*` 即可获得常用类型。

**落点**：`06` §1 中 `domain/src/todo/{model,repository,service}.rs`、`core/src/*` 按角色命名、各 crate `prelude.rs`。

## 3. 依赖方向怎么控制？

**证据**

| 仓库    | 观察                                                                                                                                              | 出处                                                         |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| Pumpkin | 单向链：`codecs` → `nbt` → `util` → `config` / `data` → `protocol` → `inventory` / `command` → `world` → `pumpkin`；无环                          | 各 `Cargo.toml` 的 `pumpkin-*` 依赖（`01` §A 依赖图）        |
| Pumpkin | 所有第三方版本只在 `[workspace.dependencies]` 声明，且一律 `default-features = false`；lint 只在 `[workspace.lints]` 声明                         | 根 `Cargo.toml`                                              |
| Pingora | `pingora-core` 不依赖 `pingora-proxy`；`pingora-error` / `pingora-http` 无 runtime 依赖；门面 crate 用 optional dependency + `?/feature` 语法组合 | `pingora/Cargo.toml` `[features]`；`pingora-core/Cargo.toml` |
| Pingora | `clippy.toml` 固定 `msrv`，CI 用 nightly / msrv / stable 三个工具链                                                                               | `clippy.toml`；`.github/workflows/build.yml`                 |

**原则 P3 —— 依赖方向是架构的一部分，要写下来、要机械检查。**

- 分层 L0–L4，只向下依赖，同层不可见（`05` §1.1 R1–R2）。
- 叶子层（`util`、`domain`、`config`）不得依赖 tokio。
- 版本、feature、lint、profile 全部在 workspace 根统一（R7–R8）。
- 用 `cargo metadata` 写一个架构守卫测试，违反即红。

**落点**：`05` §1.1 规则表与守卫伪代码；`06` §2.1 workspace `Cargo.toml`；`06` §1 `tests/layering.rs`。

## 4. Core 与业务怎么解耦？

**证据**

| 仓库    | 观察                                                                                                                                                  | 出处                                                      |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| Pingora | `ServerApp::process_new(self: &Arc<Self>, session: Stream, shutdown: &ShutdownWatch) -> Option<Stream>`：框架持有连接与关停信号，应用只实现这一个方法 | `pingora-core/src/apps/mod.rs:39-64`                      |
| Pingora | `Service` 只是"跑到关停为止的东西"，两类实现：监听端点的、后台的                                                                                      | `pingora-core/src/services/mod.rs:15-24`                  |
| Pingora | `ProxyHttp` 定义在 `pingora-proxy` 而不是 `pingora-core`：core 只提供协议与生命周期，"代理"是一种应用                                                 | `pingora-proxy/src/proxy_trait.rs`                        |
| Pumpkin | 插件 API 独立成 crate `pumpkin-plugin-api`（2.4 万行）供插件依赖；主程序在 `plugin/` 加载并回调                                                       | `crates/pumpkin-plugin-api`、`crates/pumpkin/src/plugin/` |

**原则 P4 —— Core 定义 trait 与生命周期，业务实现 trait；Core 永远不 `use` 业务 crate；唯一同时认识两者的地方是组合根。**

```text
            ┌──────────────── core ────────────────┐
            │  trait Service { run(shutdown, ready) } │
            │  trait HealthCheck { check() }          │
            │  EventBus<E>   Server   ShutdownToken   │
            └──────────────┬───────────────────────┘
                           │ 实现（依赖箭头向上指向 core）
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
   api::HttpService   infra::CleanupJob   core::EventLogSubscriber
        ▲                  ▲                  ▲
        └──────────────────┴──────────────────┘
                       bootstrap.rs（组合根）把它们塞进 Server
```

**落点**：`05` §5 扩展点表；`06` §2.3 `bootstrap.rs`；`06` §2.4 `Service` trait。

## 5. Trait / Extension Point 怎么设计？

**证据**

| 仓库    | 观察                                                                                                                                                                                      | 出处                                                                            |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| Pingora | `ProxyHttp` 有 40 余个钩子，**每个都有默认实现**，用户只覆盖关心的阶段；`type CTX` + `fn new_ctx()` 每请求一份上下文贯穿所有阶段；`request_filter` 返回 `Result<bool>` 表示"已响应，短路" | `pingora-proxy/src/proxy_trait.rs:56-764`；`docs/user_guide/ctx.md`、`phase.md` |
| Pingora | 钩子签名统一为 `(&self, &mut Session, &mut Self::CTX)`：框架状态可变借用、应用状态共享借用、请求状态独占                                                                                  | 同上                                                                            |
| Pingora | `HttpModules` 用 builder 组装有序处理链；`ConnectionFilter` 用 feature 门控，关闭时是零开销 no-op                                                                                         | `pingora-core/src/modules/http/`；`pingora-core/src/lib.rs:44-80`               |
| Pingora | 阶段顺序有官方 mermaid 图                                                                                                                                                                 | `docs/user_guide/phase_chart.md`                                                |
| Pumpkin | 插件通过事件注册回调，事件带优先级与可取消（详见 `01` §F）                                                                                                                                | `crates/pumpkin/src/plugin/api/`                                                |

**原则 P5 —— 扩展点 = 带默认实现的 trait + 每请求 CTX + 明确的阶段顺序；数量要少，每个都要有一张"何时被调用"的图。**

新增扩展点前的检查清单：

| 问题                           | 要求                                                                                                                                  |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------- |
| 它在生命周期的哪个阶段被调用？ | 能在 `05` §4 的时序图上指出位置                                                                                                       |
| 不实现它会怎样？               | 必须有默认实现或默认行为                                                                                                              |
| 它需要哪些状态？               | 只传 `&self`、`&mut ctx`、必要的框架句柄；不传全局                                                                                    |
| 同步还是异步？                 | 有 IO 就 `async`（`async_trait` 保证对象安全，与 Pingora 一致）                                                                       |
| 多个实现的顺序？               | 注册顺序即调用顺序，写进文档                                                                                                          |
| 怎么测？                       | 提供一个 no-op 实现作为测试替身                                                                                                       |
| 参数会不会变？                 | 用 `XxxArgs<'a>` / `Ctx<'a>` 结构体承载参数，而不是长参数列表：加字段不破坏既有实现（[Pumpkin] `block/mod.rs:219-441` 的 27 个 Args） |
| 只有一个函数？                 | 用闭包 `Arc<dyn Fn>`，不逼用户定义空壳类型；有一组函数才用 trait（[Pingora] `PeerOptions` `peer.rs:557-569`）                         |
| 注册错了怎么办？               | 重复注册、建好上下文后再注册 → **启动期** panic/错误，绝不在请求期爆炸（[Pingora] `modules/http/mod.rs:125,143`）                     |

服务骨架不照搬 40 个钩子：HTTP 阶段已由 tower 中间件模型覆盖，本模板只保留 `Service`、`HealthCheck`、port、中间件、事件 5 类扩展点（`05` §5）。

**落点**：`05` §5；`06` §2.8 `HttpService` / 中间件栈；`05` §11 请求时序图。

## 6. Runtime / Infrastructure 能力怎么抽象？

**证据**

| 仓库    | 观察                                                                                                                         | 出处                                                           |
| ------- | ---------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Pingora | `pingora-runtime` 提供"多线程但不 work-stealing"的第三种 runtime 口味；`RuntimeOpts` 来自配置；每个 Service 可有自己的线程数 | `pingora-runtime/src/lib.rs:15-24`；`pingora-core/src/server/` |
| Pingora | `pingora-timeout` 把超时做成独立 crate；`pingora-pool` 连接池独立                                                            | 各 crate                                                       |
| Pingora | 信号语义明确：SIGINT 立即、SIGTERM 优雅、SIGQUIT 热升级                                                                      | `docs/user_guide/start_stop.md`                                |
| Pumpkin | tokio 负责 IO，rayon 负责 CPU 密集，两者之间用 `mpsc` 传递，禁止在 tokio 上阻塞 rayon                                        | `CONTRIBUTING.md`「Working with Tokio and Rayon」              |
| Pumpkin | `rust-toolchain.toml` 只锁 `stable` + 组件；profile 区分 dev / release / profiling                                           | 根目录文件                                                     |

**原则 P6 —— 运行时能力由 core 以"设施"形式提供，配置驱动，可替换，但不过早抽象。**

- 一个 runtime，从 `RuntimeConfig` 构建，**配置先于 runtime**（`main` 保持同步）。
- 关停用标准件 `CancellationToken` + `TaskTracker`，不自研 `watch` 协议。
- 超时、限流、指标先用成熟 crate（`tower-http`、`metrics`），不做自己的 `pingora-timeout`。
- CPU 密集任务的规则写进 `CONTRIBUTING.md`：`spawn_blocking` 或 rayon，禁止阻塞 worker。
- 热升级、自定义 runtime 明确列为非目标（`00` N4）。
- 生命周期对外可观测：`Phase` 枚举经 `watch` 发布，测试与运维订阅它而不是 `sleep`（[Pingora] `ExecutionPhase`）。
- 配置与注册类错误一律在启动期失败；进入请求期后不再有"配置错了"这种错误。

**落点**：`05` §4 生命周期与信号表；`06` §2.2 同步 `main`；`06` §2.4 `Server::run`。

## 7. 原则 → 落点总表

| 原则        | 一句话                                | `05`        | `06`                      |
| ----------- | ------------------------------------- | ----------- | ------------------------- |
| P1 拆 crate | 按变化原因与依赖重量                  | §1、§2、§12 | §1 目录树                 |
| P2 定模块   | 概念闭包 + 最小暴露 + prelude         | §2.1        | `domain/todo/*`、`core/*` |
| P3 控依赖   | 分层 + 允许边 + 守卫测试              | §1.1        | §2.1、`tests/layering.rs` |
| P4 解耦     | core 定义 trait，业务实现，组合根装配 | §5          | §2.3、§2.4                |
| P5 扩展点   | 默认实现 + CTX + 阶段图 + 少而精      | §5、§11     | §2.8                      |
| P6 运行时   | 设施化、配置驱动、标准件              | §4、§7、§8  | §2.2、§2.4                |

## 8. 研究提纲问题清单的逐条回答

**Pumpkin 五问（提纲 §4）**

| 问题                                   | 回答                                                                                                                        |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| 一个大型 Rust Application 应该如何拆？ | 按能力/变化原因拆成 crate，主程序保持薄；工具链（codegen、fuzzer）放 `tools/`，与 `crates/` 分开                            |
| 一个 Domain 应该放在哪里？             | 静态数据（Pumpkin `pumpkin-data`，代码生成）与行为（主 crate 的领域模块）分离；本模板：类型与规则在 `domain`，IO 在 `infra` |
| 一个功能应该属于哪个 Crate？           | 看它拖来的依赖与被谁复用：纯规则 → `domain`；需要外部系统 → `infra`；对外暴露 → `api`；跨 crate 纯函数 → `util`             |
| Crate 之间应该如何控制依赖？           | 单向分层 + workspace 统一版本/feature/lint + 守卫测试                                                                       |
| 业务模块之间应该如何通信？             | 同步、需要返回值 → trait 直接调用；异步、只通知事实 → `EventBus`；绝不互相 `use` 对方内部模块                               |

**Pingora 六问（提纲 §4）**

| 问题                                       | 回答                                                                                                    |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| 一个 Rust Framework 应该如何设计？         | 叶子（error/http）→ core（协议、生命周期）→ 应用型 crate（proxy/cache）→ 门面（feature 组合 + prelude） |
| Core 与 Application 如何解耦？             | Core 只暴露 trait 与句柄（`Stream`、`ShutdownWatch`），应用实现 trait；Core 不依赖应用 crate            |
| Trait 应该在哪里定义？                     | 在"调用方"所在的最低层：生命周期 trait 在 core，HTTP 代理钩子在 proxy，领域 port 在 domain              |
| Extension Point 应该如何设计？             | 全默认实现、每请求 CTX、统一签名、有阶段图、可 feature 门控为零开销                                     |
| Runtime / Network / HTTP 如何分层？        | runtime（线程与调度）→ l4（TCP/UDS）→ tls → http（v1/v2 session）→ apps（业务接口）；每层只依赖下层     |
| 如何让上层业务在不修改 Core 的情况下扩展？ | 实现 trait + 注册到 Server；模块流水线（`HttpModules`）按注册顺序执行；配置分节由上层新增               |
