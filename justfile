# 模板仓库（rs-starter-template）的维护任务。生成项目自己的任务在 generated/acme-svc/justfile。
set shell := ["bash", "-euo", "pipefail", "-c"]

snapshot := "generated/acme-svc"
# 固定的默认答案：快照必须在任何机器上可复现
common := '--vcs none --silent -d project-description="A Rust service built from rs-starter-template" -d repository="" -d authors="Template Authors" -d year=2026'

default:
    @just --list

# ── 文档 ─────────────────────────────────────────────────
# 设计文档的表格必须对齐（CI 同款检查）
docs-check:
    python3 scripts/align_tables.py --check docs/design/*.md docs/design/references/agent-report-*.md

docs-align:
    python3 scripts/align_tables.py docs/design/*.md docs/design/references/agent-report-*.md

# ── 快照 ─────────────────────────────────────────────────
# 用默认答案重新生成快照（P4 起可用）
regen:
    rm -rf {{snapshot}}
    cargo generate --path template --destination generated --name acme-svc {{common}} -d license=MIT -d include_docker=true
    just check-rendered {{snapshot}}

# 快照必须能过它自己的 ci
verify:
    cd {{snapshot}} && just ci

# 漂移检查：regen 后工作区不得有变化
drift: regen
    git diff --exit-code --stat -- generated/

# 生成结果里不得残留未渲染的 Liquid 标记（允许列表：.template-raw-allowlist）
check-rendered DIR:
    python3 scripts/check_rendered.py "{{DIR}}"

# 三组答案矩阵：生成到临时目录并各跑一遍 ci（本地版 template-ci）
# Different names on purpose: they catch hard-coded `acme-svc` and lockfile ordering issues
matrix:
    just _gen-one zz-widget 'license=Apache-2.0' 'include_docker=false'
    just _gen-one my-api 'license=MIT OR Apache-2.0' 'include_docker=true'

_gen-one NAME LICENSE DOCKER:
    tmp=$(mktemp -d) && cargo generate --path template --destination "$tmp" --name {{NAME}} {{common}} -d '{{LICENSE}}' -d '{{DOCKER}}' \
      && just check-rendered "$tmp/{{NAME}}" && (cd "$tmp/{{NAME}}" && just ci) && rm -rf "$tmp"

# 冒烟：构建快照 → 启动 → /healthz → SIGTERM → 退出码 0
smoke:
    cd {{snapshot}} && just smoke
