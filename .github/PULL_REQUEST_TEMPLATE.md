<!-- 标题按 Conventional Commits：fix(template): … · feat(generated): … · chore: … · docs: … · ci: … -->

Closes #

## 改了什么

## 为什么

## 验证

- [ ] `generated/acme-svc`：`just ci` 通过（fmt · clippy -D warnings · nextest · doctests · rustdoc · deny · typos）
- [ ] `template/` 与快照在同一提交里，`just drift` 零差异（或模板渲染结果与快照逐字节一致）
- [ ] `CHANGELOG.md` 的 `[Unreleased]` 已更新（用户可见的改动）
- [ ] 改了生成项目的行为 → 新增或更新了测试

## 影响范围

- [ ] template/
- [ ] generated/acme-svc
- [ ] template-ci / justfile / scripts
- [ ] docs/
