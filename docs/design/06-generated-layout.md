# 06 · 生成后的项目：目录逐文件说明与关键文件伪代码

> 以 `cargo generate riotian/rs-starter-template --name acme-svc` 的默认答案（`license = MIT`、`include_docker = true`）为准。
> 每个文件后面的注释说明"为什么存在"；带 ★ 的文件在 §2 给出伪代码。

## 1. 目录树

```text
acme-svc/
├── Cargo.toml                       ★ workspace 根：members、workspace.package、workspace.dependencies、workspace.lints、profiles
├── Cargo.lock                         应用项目提交 lockfile
├── rust-toolchain.toml                channel = "stable"；components = [rustfmt, clippy, rust-analyzer, rust-src]   [Pumpkin]
├── rustfmt.toml                       edition = "2024"（只用 stable 选项）                                          [Pumpkin]
├── clippy.toml                        msrv = "1.96"                                                                [Pingora]
├── deny.toml                          cargo-deny：advisories / licenses / bans / sources
├── typos.toml                         拼写检查白名单                                                                 [Pumpkin]
├── cliff.toml                         git-cliff：Conventional Commits → CHANGELOG                                    [Pingora]
├── .editorconfig
├── .gitignore
├── .dockerignore
├── .env.example                       ACME_SVC__HTTP__BIND=0.0.0.0:8080 …（直接 `cp .env.example .env`）             [ratatui] .envrc 的替代
├── .config/nextest.toml               nextest 配置
├── justfile                         ★ dev / check / lint / test / ci / doc / docker / bump
├── README.md                          生成项目自己的 README（模板里是 README.md.liquid）
├── CHANGELOG.md                       由 git-cliff 维护
├── CONTRIBUTING.md                    提交规范、lint 说明、如何加功能                                                 [Pumpkin]
├── LICENSE                            按占位符选择
├── Dockerfile                       ★ 多阶段：chef 缓存依赖 → 构建 → distroless/cc，非 root，HEALTHCHECK              [Pumpkin] 非 root + healthcheck
├── docker-compose.yml                 本地起服务 + 环境变量示例
├── config/
│   ├── default.toml                   全部键 + 默认值 + 注释（真相来源）
│   └── local.toml.example             本地覆盖示例（.gitignore 了 local.toml）
├── docs/
│   ├── architecture.md                分层图（mermaid）、依赖规则、扩展点、如何加功能（从本设计精简而来）
│   └── adr/
│       ├── 0001-record-architecture-decisions.md
│       └── 0002-crate-layering.md
├── .github/
│   ├── workflows/ci.yml             ★ fmt → clippy → nextest → doc → deny → typos                                     [Pumpkin] + [Pingora]
│   ├── workflows/release.yml          tag 触发：多平台二进制 + GitHub Release                                          [Pumpkin] 精简版
│   ├── workflows/docker.yml           push main / tag：构建并推送镜像（include_docker 时生成）                          [Pumpkin]
│   ├── dependabot.yml                 cargo + github-actions + docker，每周                                            [Pumpkin]
│   └── PULL_REQUEST_TEMPLATE.md       Conventional Commits 提示                                                       [Pumpkin]
└── crates/
    ├── acme-svc/                      ── L4 bin ──
    │   ├── Cargo.toml
    │   ├── build.rs                   vergen-gix：git sha / describe / build date                                      [ratatui]
    │   ├── src/main.rs              ★ 同步 main：panic hook → CLI → Config → Runtime → block_on(bootstrap::run)
    │   ├── src/cli.rs                 clap：--config、--check-config、--print-config、--bind；version() 拼 vergen 信息   [ratatui]
    │   ├── src/bootstrap.rs         ★ 装配：adapters → domain services → AppState → services → Server
    │   └── tests/
    │       ├── lifecycle.rs           启动 → ready → 触发关停 → 在 grace 内退出
    │       ├── http_smoke.rs          /healthz /readyz /version /metrics /api/v1/todos
    │       └── layering.rs            cargo metadata 依赖边守卫
    ├── acme-svc-core/                 ── L2 ──
    │   ├── Cargo.toml                 features: default = []; otel = [dep:…]
    │   └── src/
    │       ├── lib.rs · prelude.rs
    │       ├── runtime.rs             build(&RuntimeConfig) -> Runtime
    │       ├── server.rs            ★ Server / ServerOptions / run()
    │       ├── service.rs           ★ trait Service / ServiceError
    │       ├── shutdown.rs            ShutdownToken / signal listener
    │       ├── readiness.rs           Readiness / wait_all
    │       ├── health.rs              HealthCheck / HealthRegistry / HealthReport
    │       ├── event.rs             ★ EventBus<E> / Publisher<E> / Subscription<E>
    │       ├── error.rs               ErrorKind / ErrorSource / Classify
    │       └── telemetry/{mod,tracing,metrics,panic}.rs
    ├── acme-svc-config/               ── L1 ──
    │   └── src/
    │       ├── lib.rs               ★ Config / load() / validate() / redacted()
    │       ├── sections/{app,runtime,server,http,telemetry,events,jobs}.rs
    │       └── error.rs               ConfigError
    ├── acme-svc-domain/               ── L1 ──
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs               DomainError
    │       ├── event.rs               DomainEvent
    │       ├── ports.rs               trait Clock（跨领域的 port）
    │       └── todo/                  示例领域（可整目录删除）
    │           ├── mod.rs
    │           ├── model.rs         ★ Todo / TodoId / Title
    │           ├── repository.rs    ★ trait TodoRepository
    │           └── service.rs       ★ TodoService<R, C>
    ├── acme-svc-infra/                ── L3 ──
    │   └── src/
    │       ├── lib.rs · error.rs      InfraError
    │       ├── clock.rs               SystemClock: Clock
    │       ├── memory/todo_repository.rs   InMemoryTodoRepository: TodoRepository + HealthCheck
    │       └── jobs/cleanup.rs        CleanupJob: Service（周期任务示例）
    ├── acme-svc-api/                  ── L3 ──
    │   └── src/
    │       ├── lib.rs
    │       ├── service.rs           ★ HttpService: Service
    │       ├── router.rs            ★ router(state) -> Router
    │       ├── state.rs               AppState
    │       ├── context.rs             RequestContext
    │       ├── middleware/{mod,request_id,trace}.rs
    │       ├── error.rs             ★ ApiError: IntoResponse（problem+json）
    │       ├── dto/todo.rs            CreateTodoRequest / TodoResponse
    │       └── routes/{mod,health,version,metrics,todos}.rs
    ├── acme-svc-util/                 ── L0 ──
    │   └── src/{lib,secret,duration}.rs
    └── acme-svc-test-utils/           ── dev only ──
        └── src/{lib,app,clock}.rs     TestApp::spawn(cfg) / FakeClock
```

文件数量一览（不含 `target/`）：

| 区域                | 文件数（约） | 说明                                                                    |
| ------------------- | ------------ | ----------------------------------------------------------------------- |
| 根目录元文件        | 18           | 工具链、lint、发布、容器                                                |
| `.github/`          | 5            | 3 workflows + dependabot + PR 模板                                      |
| `config/` + `docs/` | 5            | 配置真相源 + 架构文档 + 2 ADR                                           |
| `crates/*`          | 55           | 8 个 crate                                                              |
| 合计                | ≈ 83         | 与 Pumpkin 的 20 个 crate、Pingora 的 23 个相比，是"够用的最小多 crate" |

## 2. 关键文件伪代码

### 2.1 `Cargo.toml`（workspace 根）

```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
license = "MIT"                                  # 由占位符决定
repository = "https://github.com/acme/acme-svc"  # 可空
publish = false

[workspace.dependencies]
# 内部 crate（成员用 acme-svc-core.workspace = true）
acme-svc-core   = { path = "crates/acme-svc-core" }
acme-svc-config = { path = "crates/acme-svc-config" }
acme-svc-domain = { path = "crates/acme-svc-domain" }
acme-svc-infra  = { path = "crates/acme-svc-infra" }
acme-svc-api    = { path = "crates/acme-svc-api" }
acme-svc-util   = { path = "crates/acme-svc-util" }
acme-svc-test-utils = { path = "crates/acme-svc-test-utils" }

# 第三方：统一 default-features = false，按需开 feature                       [Pumpkin]
tokio        = { version = "1", default-features = false, features = ["rt-multi-thread", "macros", "signal", "net", "time", "sync"] }
tokio-util   = { version = "0.7", default-features = false, features = ["rt"] }
axum         = { version = "0.8", default-features = false, features = ["http1", "json", "tokio", "tower-log", "query"] }
tower-http   = { version = "0.6", default-features = false, features = ["trace", "request-id", "timeout", "catch-panic", "limit"] }
serde        = { version = "1", default-features = false, features = ["derive", "std"] }
serde_json   = { version = "1", default-features = false, features = ["std"] }
thiserror    = { version = "2", default-features = false }
anyhow       = { version = "1", default-features = false, features = ["std"] }
tracing      = { version = "0.1", default-features = false, features = ["std", "attributes"] }
tracing-subscriber = { version = "0.3", default-features = false, features = ["env-filter", "fmt", "json", "ansi", "std"] }
metrics      = "0.24"
metrics-exporter-prometheus = { version = "0.17", default-features = false }
config       = { version = "0.15", default-features = false, features = ["toml"] }
clap         = { version = "4", default-features = false, features = ["std", "derive", "help", "usage", "error-context"] }
async-trait  = "0.1"
uuid         = { version = "1", default-features = false, features = ["v7", "serde", "std"] }
time         = { version = "0.3", default-features = false, features = ["std", "serde", "formatting"] }
humantime-serde = "1"
dashmap      = { version = "6", default-features = false }
# 版本号以实施时 `cargo add` 解析结果为准；此处只表达 major

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]                          # 与 Pumpkin / Mikan-rs 同款全量 deny 集
all      = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
nursery  = { level = "deny", priority = -1 }
cargo    = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
# … 其余 deny / allow 项按 Pumpkin 列表逐条评估后写入（见 01 §3.3）
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
multiple_crate_versions = "allow"
cargo_common_metadata = "allow"

[profile.release]
lto = true
codegen-units = 1
strip = "debuginfo"

[profile.profiling]
inherits = "release"
debug = true
strip = false

[profile.dev]
debug = false                                    # 更快的增量编译                                   [Pumpkin]
```

### 2.2 `crates/acme-svc/src/main.rs`

```rust
//! 组合根。只做四件事：panic hook、CLI、配置、把控制权交给 bootstrap。
mod bootstrap;
mod cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    acme_svc_core::telemetry::install_panic_hook();        // 1. 先于一切                       [ratatui]
    let cli = cli::Cli::parse();                            // 2.
    let cfg = match acme_svc_config::Config::load(cli.sources()) {   // 3. 文件 + env + cli
        Ok(cfg) => cfg,
        Err(err) => return exit_with("configuration error", &err),
    };
    if cli.check_config || cli.print_config {               // 3b. Pingora 的 -t
        println!("{}", cfg.redacted());
        return ExitCode::SUCCESS;
    }
    let runtime = match acme_svc_core::runtime::build(&cfg.runtime) {   // 4. 配置决定 runtime   [Pingora]
        Ok(rt) => rt,
        Err(err) => return exit_with("runtime error", &err),
    };
    match runtime.block_on(bootstrap::run(cfg)) {           // 5.
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => exit_with("fatal", &err),
    }
}

fn exit_with(stage: &str, err: &dyn std::error::Error) -> ExitCode {
    // telemetry 可能还没初始化，所以用 eprintln!（此文件对 print_stderr 局部 allow）
    eprintln!("{stage}: {err:#}");
    ExitCode::FAILURE
}
```

### 2.3 `crates/acme-svc/src/bootstrap.rs`

```rust
pub async fn run(cfg: Config) -> anyhow::Result<()> {
    let _telemetry = acme_svc_core::telemetry::init(&cfg.telemetry)?;   // 返回 guard，drop 时 flush
    tracing::info!(version = %build_info::VERSION, git = %build_info::GIT_SHA, "starting {}", cfg.app.name);

    // ── adapters（L3 infra）
    let clock = Arc::new(SystemClock);
    let todo_repo = Arc::new(InMemoryTodoRepository::default());

    // ── core 服务设施
    let events = EventBus::<DomainEvent>::new(cfg.events.capacity);
    let health = HealthRegistry::new().register("todo_repository", todo_repo.clone());

    // ── domain 服务（只认识 port）
    let todo_service = Arc::new(TodoService::new(todo_repo.clone(), clock.clone(), events.publisher()));

    // ── api
    let state = AppState { todo_service, health: health.clone(), metrics: telemetry_handle() };
    let router = acme_svc_api::router(state);

    // ── 启动
    Server::new(cfg.server)
        .service(HttpService::new(cfg.http, router))
        .service(CleanupJob::new(cfg.jobs.cleanup, todo_repo, clock))
        .service(EventLogSubscriber::new(events.subscribe()))
        .run()
        .await
        .map_err(Into::into)
}
```

### 2.4 `crates/acme-svc-core/src/service.rs` 与 `server.rs`

```rust
// service.rs
#[async_trait]
pub trait Service: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    async fn run(self: Box<Self>, shutdown: ShutdownToken, ready: Readiness) -> Result<(), ServiceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("failed to bind {addr}")]          Bind { addr: SocketAddr, #[source] source: std::io::Error },
    #[error("service {name} failed: {reason}")] Failed { name: &'static str, reason: String },
}

// server.rs
pub struct Server { opts: ServerOptions, services: Vec<Box<dyn Service>> }

impl Server {
    pub fn new(opts: ServerOptions) -> Self { … }
    pub fn service(mut self, svc: impl Service) -> Self { self.services.push(Box::new(svc)); self }

    pub async fn run(self) -> Result<(), ServerError> {
        let root = ShutdownToken::new();
        let tracker = TaskTracker::new();
        let mut readiness = Vec::with_capacity(self.services.len());

        for svc in self.services {
            let (notifier, rx) = Readiness::new();
            readiness.push(rx);
            let name = svc.name();
            let child = root.child();
            let root_for_task = root.clone();
            tracker.spawn(async move {
                if let Err(err) = svc.run(child, notifier).await {
                    tracing::error!(service = name, error = %err, "service failed; requesting shutdown");
                    root_for_task.cancel();
                } else {
                    tracing::info!(service = name, "service stopped");
                }
            });
        }
        tokio::spawn(shutdown::listen_for_signals(root.clone()));

        if tokio::time::timeout(self.opts.readiness_timeout, readiness::wait_all(readiness)).await.is_err() {
            tracing::warn!("readiness timeout elapsed; some services are not ready");
        } else {
            tracing::info!("all services ready");
        }

        root.cancelled().await;
        tracing::info!("shutdown requested; draining");
        tracker.close();
        match tokio::time::timeout(self.opts.grace_period, tracker.wait()).await {
            Ok(()) => { tracing::info!("clean shutdown"); Ok(()) }
            Err(_) => { tracing::warn!("grace period exceeded"); Err(ServerError::GraceExceeded) }
        }
    }
}
```

### 2.5 `crates/acme-svc-core/src/event.rs`

```rust
pub struct EventBus<E> { tx: broadcast::Sender<E> }
pub struct Publisher<E> { tx: broadcast::Sender<E> }
pub struct Subscription<E> { rx: broadcast::Receiver<E> }

impl<E: Clone + Send + 'static> EventBus<E> {
    pub fn new(capacity: usize) -> Self { … }
    pub fn publisher(&self) -> Publisher<E> { … }
    pub fn subscribe(&self) -> Subscription<E> { … }
}
impl<E: Clone + Send + 'static> Publisher<E> {
    pub fn publish(&self, event: E) { let _ = self.tx.send(event); /* 无订阅者不是错误 */ }
}
impl<E: Clone + Send + 'static> Subscription<E> {
    /// 返回 None 表示总线已关闭；Lagged 时记指标并继续
    pub async fn next(&mut self) -> Option<E> { loop { match self.rx.recv().await { Ok(e) => return Some(e), Err(Lagged(n)) => { metrics::counter!("events_lagged_total").increment(n); } Err(Closed) => return None } } }
}
```

### 2.6 `crates/acme-svc-config/src/lib.rs`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub app: AppConfig, pub runtime: RuntimeConfig, pub server: ServerConfig,
    pub http: HttpConfig, pub telemetry: TelemetryConfig, pub events: EventsConfig, pub jobs: JobsConfig,
}

pub struct Sources { pub file: Option<PathBuf>, pub env_prefix: &'static str, pub overrides: Vec<(String, String)> }

impl Config {
    pub fn load(sources: Sources) -> Result<Self, ConfigError> {
        let mut b = config::Config::builder();
        if let Some(path) = &sources.file { b = b.add_source(config::File::from(path.as_path()).required(true)); }
        else { b = b.add_source(config::File::with_name("config/default").required(false))
                    .add_source(config::File::with_name("config/local").required(false)); }
        b = b.add_source(config::Environment::with_prefix(sources.env_prefix).prefix_separator("__").separator("__"));
        for (k, v) in sources.overrides { b = b.set_override(k, v)?; }
        let cfg: Self = b.build()?.try_deserialize()?;
        cfg.validate()?;                       // 聚合所有问题一次报告
        Ok(cfg)
    }
    pub fn validate(&self) -> Result<(), ConfigError> { … }
    pub fn redacted(&self) -> String { … }     // Secret 字段打 ***
}
```

### 2.7 `crates/acme-svc-domain/src/todo/*.rs`

```rust
// model.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TodoId(Uuid);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title(String);                      // 构造即校验：非空、≤ 200 字符
impl Title { pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> { … } }
#[derive(Debug, Clone)]
pub struct Todo { pub id: TodoId, pub title: Title, pub done: bool, pub created_at: OffsetDateTime }

// repository.rs（port）
#[async_trait]
pub trait TodoRepository: Send + Sync + 'static {
    async fn insert(&self, todo: Todo) -> Result<(), DomainError>;
    async fn get(&self, id: TodoId) -> Result<Option<Todo>, DomainError>;
    async fn list(&self) -> Result<Vec<Todo>, DomainError>;
    async fn update(&self, todo: Todo) -> Result<(), DomainError>;
}

// service.rs
pub struct TodoService<R: TodoRepository, C: Clock> { repo: Arc<R>, clock: Arc<C>, events: Publisher<DomainEvent> }
impl<R: TodoRepository, C: Clock> TodoService<R, C> {
    pub async fn create(&self, title: Title) -> Result<Todo, DomainError> {
        let todo = Todo { id: TodoId::new(), title, done: false, created_at: self.clock.now() };
        self.repo.insert(todo.clone()).await?;
        self.events.publish(DomainEvent::TodoCreated { id: todo.id });
        Ok(todo)
    }
    pub async fn complete(&self, id: TodoId) -> Result<Todo, DomainError> {
        let mut todo = self.repo.get(id).await?.ok_or(DomainError::NotFound(id))?;
        if todo.done { return Err(DomainError::AlreadyDone(id)); }
        todo.done = true;
        self.repo.update(todo.clone()).await?;
        self.events.publish(DomainEvent::TodoCompleted { id });
        Ok(todo)
    }
}
```

### 2.8 `crates/acme-svc-api/src/{service,router,error}.rs`

```rust
// service.rs
pub struct HttpService { cfg: HttpConfig, router: Router }
#[async_trait]
impl Service for HttpService {
    fn name(&self) -> &'static str { "http" }
    async fn run(self: Box<Self>, shutdown: ShutdownToken, ready: Readiness) -> Result<(), ServiceError> {
        let listener = TcpListener::bind(self.cfg.bind).await.map_err(|source| ServiceError::Bind { addr: self.cfg.bind, source })?;
        tracing::info!(addr = %listener.local_addr()?, "http listening");
        ready.ready();
        axum::serve(listener, self.router.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
            .map_err(|e| ServiceError::Failed { name: "http", reason: e.to_string() })
    }
}

// router.rs
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(routes::health::live))
        .route("/readyz",  get(routes::health::ready))
        .route("/version", get(routes::version::get))
        .route("/metrics", get(routes::metrics::get))
        .nest("/api/v1", routes::todos::router())
        .layer(middleware::stack(&state.http))       // request-id → trace → timeout → body-limit → catch-panic
        .with_state(state)
}

// error.rs
pub struct ApiError { kind: ErrorKind, title: &'static str, detail: Option<String>, request_id: Option<String> }
impl From<DomainError> for ApiError { fn from(e: DomainError) -> Self { Self::from_classified(&e) } }
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.kind.http_status();
        if self.kind == ErrorKind::Internal { tracing::error!(?self, "internal error"); }
        (status, [(CONTENT_TYPE, "application/problem+json")], Json(ProblemDetails::from(self))).into_response()
    }
}
```

### 2.9 `justfile`（生成项目）

```just
set dotenv-load := true
set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# ── 日常 ─────────────────────────────────────────────
dev *ARGS:                       # 本地启动（读取 .env）
    cargo run -p acme-svc -- {{ARGS}}
watch:                           # 需要 cargo-watch / bacon
    bacon run
check:
    cargo check --workspace --all-targets --all-features
fmt:
    cargo fmt --all
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings
test *ARGS:
    cargo nextest run --workspace --all-features {{ARGS}}
    cargo test --workspace --doc
doc:
    cargo doc --workspace --no-deps --all-features

# ── 质量门（CI 用同一条命令）──────────────────────────
ci: fmt-check lint test doc deny typos
fmt-check:
    cargo fmt --all -- --check
deny:
    cargo deny check
typos:
    typos
machete:                         # 未使用依赖（可选）
    cargo machete

# ── 发布 / 容器 ───────────────────────────────────────
docker-build TAG="acme-svc:dev":
    docker build -t {{TAG}} .
docker-run TAG="acme-svc:dev":
    docker run --rm -p 8080:8080 --env-file .env {{TAG}}
changelog:
    git cliff -o CHANGELOG.md
bump LEVEL="patch":              # 需要 cargo-release 或手工
    cargo release {{LEVEL}} --no-publish --execute
```

### 2.10 `.github/workflows/ci.yml`（生成项目）

```yaml
name: CI
on: { push: { branches: [main] }, pull_request: {} }
concurrency: { group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true }
env: { CARGO_TERM_COLOR: always, RUSTFLAGS: -Dwarnings }
jobs:
  fmt:     { steps: [checkout, rust-toolchain@stable(rustfmt), cargo fmt --all --check] }
  clippy:  { steps: [checkout, rust-toolchain@stable(clippy), rust-cache, cargo clippy --workspace --all-targets --all-features -- -D warnings] }
  test:    { needs: [fmt, clippy], strategy: { matrix: { os: [ubuntu-latest, macos-latest] } },
             steps: [checkout, rust-toolchain@stable, install-action@nextest, rust-cache, cargo nextest run --workspace, cargo test --doc] }
  doc:     { steps: [checkout, rust-toolchain@stable, cargo doc --workspace --no-deps --all-features (RUSTDOCFLAGS=-Dwarnings)] }
  deny:    { steps: [checkout, EmbarkStudios/cargo-deny-action] }
  typos:   { steps: [checkout, crate-ci/typos] }
```

与参考仓库的关系：`RUSTFLAGS=-Dwarnings` + `concurrency` + nextest 来自 [Pumpkin] `rust.yml`；`cargo doc` 单独 job、`audit`/`deny` 来自 [Pingora] `docs.yml` / `audit.yml`；不采用 Pumpkin 的"按受影响包裁剪"脚本（`ci_packages.py`）——8 个 crate 规模不需要。

### 2.11 `Dockerfile`

```dockerfile
# syntax=docker/dockerfile:1
FROM rust:1.96-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json          # 依赖层缓存
COPY . .
RUN cargo build --release -p acme-svc

FROM gcr.io/distroless/cc-debian12:nonroot                       # 非 root、无 shell     [Pumpkin] 非 root 用户
COPY --from=builder /app/target/release/acme-svc /usr/local/bin/acme-svc
COPY config/default.toml /etc/acme-svc/default.toml
ENV ACME_SVC__HTTP__BIND=0.0.0.0:8080
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD ["/usr/local/bin/acme-svc", "--probe", "http://127.0.0.1:8080/healthz"]
ENTRYPOINT ["/usr/local/bin/acme-svc"]
CMD ["--config", "/etc/acme-svc/default.toml"]
```

说明：distroless 没有 `curl`/`nc`，所以二进制自带 `--probe <url>` 子命令做健康检查（Pumpkin 用 alpine + `nc -z`，本模板选择更小的攻击面）。这是一个可讨论的决策点（见 `00` D16 附注）。

## 3. 生成后 5 分钟体验

```bash
cargo generate riotian/rs-starter-template --name acme-svc
cd acme-svc
cp .env.example .env
just dev                    # 日志第一行：version / git / build；最后一行：all services ready
curl -s localhost:8080/healthz          # {"status":"ok"}
curl -s localhost:8080/readyz           # {"status":"ready","checks":{"todo_repository":"ok"}}
curl -s -XPOST localhost:8080/api/v1/todos -H 'content-type: application/json' -d '{"title":"ship it"}'
just ci                     # fmt / clippy / nextest / doc / deny / typos 全绿
# Ctrl-C → "shutdown requested; draining" → "clean shutdown"
```
