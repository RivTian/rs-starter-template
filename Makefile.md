# Makefile

从仓库根目录运行。Cargo 构建产物统一落在 `target/`，覆盖率报告落在
`coverage/`。

| 命令                          | 说明                                       |
| ----------------------------- | ------------------------------------------ |
| `make` / `make help`          | 显示可用目标                               |
| `make build`                  | debug 构建整个 workspace                   |
| `make release`                | 构建 `starter-app` 的 release 二进制          |
| `make run`                    | 以 release 模式运行 `rs-starter-template`           |
| `make run-debug`              | 以 debug 模式运行 `rs-starter-template`             |
| `make test`                   | 运行 workspace 测试                        |
| `make fmt` / `make fmt-check` | Rust 格式化 / 格式检查                     |
| `make lint`                   | 运行 clippy，并将 warning 视为错误         |
| `make coverage`               | 生成 lcov 覆盖率报告和 clippy JSON 报告    |
| `make check`                  | 依次执行格式检查、clippy 和 workspace 测试 |
| `make clean`                  | 清理 Cargo 构建产物                        |
| `make distclean`              | 清理 Cargo 构建产物及 `coverage/`          |

## 变量

| 变量       | 默认值  | 说明                             |
| ---------- | ------- | -------------------------------- |
| `CARGO`    | `cargo` | 指定 Cargo 命令或包装器          |
| `RUN_ARGS` | 空      | 传递给 `rs-starter-template` 的命令行参数 |

例如，打印版本或指定配置文件：

```bash
make run-debug RUN_ARGS=--version
make run RUN_ARGS="--config /path/to/cfg_runtime.toml"
```

构建、运行、测试和 clippy 目标均使用 `Cargo.lock`。`make coverage` 需要预先安装
`cargo-llvm-cov`：

```bash
cargo install cargo-llvm-cov
```

该目标生成：

- `coverage/lcov.info`
- `coverage/clippy.json`
