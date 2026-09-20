#!/usr/bin/env python3
"""Align Markdown pipe tables in place (East-Asian-width aware). Skips fenced code blocks.

Usage:
    python3 scripts/align_tables.py FILE...            # rewrite files in place
    python3 scripts/align_tables.py --check FILE...    # exit 1 if any file would change (CI)
"""
import re
import sys
import unicodedata


def width(s: str) -> int:
    w = 0
    for ch in s:
        if unicodedata.combining(ch):
            continue
        w += 2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1
    return w


def split_row(line: str):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|") and not s.endswith("\\|"):
        s = s[:-1]
    cells, cur, i = [], "", 0
    while i < len(s):
        c = s[i]
        if c == "\\" and i + 1 < len(s) and s[i + 1] == "|":
            cur += "\\|"
            i += 2
            continue
        if c == "`":
            j = s.find("`", i + 1)
            if j == -1:
                cur += c
                i += 1
                continue
            cur += s[i : j + 1]
            i = j + 1
            continue
        if c == "|":
            cells.append(cur.strip())
            cur = ""
            i += 1
            continue
        cur += c
        i += 1
    cells.append(cur.strip())
    return cells


def is_sep(cells) -> bool:
    non_empty = [c.strip() for c in cells if c.strip()]
    return bool(non_empty) and all(re.fullmatch(r":?-+:?", c) for c in non_empty)


def fmt_table(rows):
    ncol = max(len(r) for r in rows)
    rows = [r + [""] * (ncol - len(r)) for r in rows]
    aligns = []
    for c in rows[1]:
        c = c.strip()
        if c.startswith(":") and c.endswith(":"):
            aligns.append("c")
        elif c.endswith(":"):
            aligns.append("r")
        else:
            aligns.append("l")
    widths = [3] * ncol
    for ri, r in enumerate(rows):
        if ri == 1:
            continue
        for i, c in enumerate(r):
            widths[i] = max(widths[i], width(c))
    out = []
    for ri, r in enumerate(rows):
        cells = []
        for i, c in enumerate(r):
            w, a = widths[i], aligns[i]
            if ri == 1:
                if a == "c":
                    cells.append(":" + "-" * (w - 2) + ":")
                elif a == "r":
                    cells.append("-" * (w - 1) + ":")
                else:
                    cells.append("-" * w)
                continue
            pad = w - width(c)
            if a == "r":
                cells.append(" " * pad + c)
            elif a == "c":
                left = pad // 2
                cells.append(" " * left + c + " " * (pad - left))
            else:
                cells.append(c + " " * pad)
        out.append("| " + " | ".join(cells) + " |")
    return out


def process(text: str) -> str:
    lines = text.split("\n")
    out, i, in_fence = [], 0, False
    while i < len(lines):
        line = lines[i]
        if re.match(r"^\s*(```|~~~)", line):
            in_fence = not in_fence
            out.append(line)
            i += 1
            continue
        if (
            not in_fence
            and line.lstrip().startswith("|")
            and i + 1 < len(lines)
            and lines[i + 1].lstrip().startswith("|")
            and is_sep(split_row(lines[i + 1]))
        ):
            block = []
            while i < len(lines) and lines[i].lstrip().startswith("|"):
                block.append(split_row(lines[i]))
                i += 1
            out.extend(fmt_table(block))
            continue
        out.append(line)
        i += 1
    return "\n".join(out)


if __name__ == "__main__":
    args = sys.argv[1:]
    check = "--check" in args
    files = [a for a in args if a != "--check"]
    dirty = 0
    for p in files:
        with open(p, encoding="utf-8") as f:
            t = f.read()
        n = process(t)
        if n == t:
            print(f"ok      : {p}")
            continue
        dirty += 1
        if check:
            print(f"UNALIGNED: {p}")
        else:
            with open(p, "w", encoding="utf-8") as f:
                f.write(n)
            print(f"aligned : {p}")
    if check and dirty:
        print(f"{dirty} file(s) need `python3 scripts/align_tables.py <file>`")
        sys.exit(1)
