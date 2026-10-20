#!/usr/bin/env python3
"""Check relative Markdown links in project documentation."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
SKIP = ("http://", "https://", "mailto:", "#")


def markdown_files() -> list[Path]:
    files = [ROOT / "GLOSSARY.md", ROOT / "prototype" / "README.md"]
    files.extend((ROOT / "docs").rglob("*.md"))
    return [path for path in files if path.is_file()]


def broken_links(path: Path) -> list[str]:
    broken: list[str] = []
    for match in LINK.finditer(path.read_text(encoding="utf-8")):
        target = match.group(1).split("#", 1)[0].strip()
        if not target or target.startswith(SKIP):
            continue
        if not (path.parent / target).resolve().exists():
            broken.append(target)
    return broken


def main() -> int:
    failures: list[tuple[Path, str]] = []
    for path in markdown_files():
        for target in broken_links(path):
            failures.append((path.relative_to(ROOT), target))

    if failures:
        for path, target in failures:
            print(f"broken link: {path}: {target}", file=sys.stderr)
        return 1
    print(f"documentation links ok ({len(markdown_files())} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
