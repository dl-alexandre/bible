#!/usr/bin/env python3
"""Check dataset presence and broad structural invariants before generation."""

import re
import sys
from pathlib import Path


def main():
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "datasets")
    errors = []
    for name in ("kjv", "asv", "web", "bsb"):
        path = root / f"{name}.txt"
        if not path.is_file() or path.stat().st_size == 0:
            errors.append(f"missing or empty dataset: {path}")
            continue
        text = path.read_text(encoding="utf-8")
        chapters = len(re.findall(r"^Chapter\s+\d+", text, re.MULTILINE))
        verses = len(re.findall(r"^\s*\d+\s+\S", text, re.MULTILINE))
        if chapters != 1189:
            errors.append(f"{path}: expected 1189 chapters, found {chapters}")
        if verses < 29000:
            errors.append(f"{path}: suspiciously low verse count ({verses})")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Dataset quality check passed for {root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
