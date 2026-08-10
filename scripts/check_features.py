#!/usr/bin/env python3
"""Smoke-test the generated feature entry points without a browser dependency."""

import sys
from pathlib import Path


def main():
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "out/bible")
    index = (root / "index.html").read_text(encoding="utf-8")
    chapter = next(root.glob("*/Genesis/1.html")).read_text(encoding="utf-8")
    required_index = ("search-input", "search-version", "static/search.js", "static/study.html")
    required_chapter = ("comparison-panel", "bookmark-button", "verse-note", "clear-offline")
    missing = [item for item in required_index if item not in index]
    missing.extend(item for item in required_chapter if item not in chapter)
    if not (root / "static/search-worker.js").is_file():
        missing.append("static/search-worker.js")
    if missing:
        print(f"Missing generated feature markers: {', '.join(missing)}", file=sys.stderr)
        return 1
    print("Generated feature smoke test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
