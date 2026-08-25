#!/usr/bin/env python3
"""Validate that local links in tracked Markdown files resolve."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parent.parent
INLINE_LINK = re.compile(r"!?\[[^]]*\]\(\s*(?:<([^>]+)>|([^\s)]+))")
REFERENCE_LINK = re.compile(r"^\s*\[[^]]+\]:\s*(?:<([^>]+)>|([^\s]+))", re.MULTILINE)
IGNORED_SCHEMES = {"data", "http", "https", "mailto"}


def tracked_markdown_files() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z", "--", "*.md"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )
    return [ROOT / name.decode() for name in result.stdout.split(b"\0") if name]


def local_targets(markdown: str) -> list[str]:
    matches = [*INLINE_LINK.finditer(markdown), *REFERENCE_LINK.finditer(markdown)]
    return [match.group(1) or match.group(2) for match in matches]


def resolved_path(source: Path, target: str) -> Path | None:
    parsed = urlsplit(target)
    if parsed.scheme.lower() in IGNORED_SCHEMES or target.startswith("//"):
        return None
    if parsed.scheme:
        return None
    path = unquote(parsed.path)
    if not path:
        return None
    return ROOT / path.removeprefix("/") if path.startswith("/") else source.parent / path


def main() -> int:
    broken: list[str] = []
    for source in tracked_markdown_files():
        markdown = source.read_text(encoding="utf-8")
        for target in local_targets(markdown):
            destination = resolved_path(source, target)
            if destination is not None and not destination.exists():
                broken.append(f"{source.relative_to(ROOT)}: {target}")

    if broken:
        print("Broken repository-relative Markdown links:", file=sys.stderr)
        print("\n".join(f"  {link}" for link in broken), file=sys.stderr)
        return 1

    print("All repository-relative Markdown links resolve.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
