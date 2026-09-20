#!/usr/bin/env python3
"""Fail if a generated project still contains Liquid markers ({{ or {%).

Usage: check_rendered.py DIR [ALLOWLIST]
ALLOWLIST (default .template-raw-allowlist next to this script's repo root) lists file base
names that legitimately contain such markers (GitHub Actions `${{ }}`, git-cliff templates).
"""
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
allow_path = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else pathlib.Path(__file__).resolve().parent.parent / ".template-raw-allowlist"
allowed = {line.strip() for line in allow_path.read_text().splitlines() if line.strip() and not line.startswith("#")}
bad = []
for path in root.rglob("*"):
    if not path.is_file() or "target" in path.parts or ".git" in path.parts or path.name in allowed:
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        continue
    for lineno, line in enumerate(text.splitlines(), 1):
        if "{{" in line or "{%" in line:
            bad.append(f"{path.relative_to(root)}:{lineno}: {line.strip()[:100]}")
if bad:
    print("unrendered Liquid markers:\n  " + "\n  ".join(bad))
    sys.exit(1)
print(f"ok: no Liquid markers left in {root}")
