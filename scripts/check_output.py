#!/usr/bin/env python3
"""Check generated HTML structure, local links, and size budgets."""

import sys
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urlparse


class PageParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.tags = []
        self.links = []

    def handle_starttag(self, tag, attrs):
        self.tags.append(tag)
        for key, value in attrs:
            if key == "href" and value:
                self.links.append(value)


def target_path(root, href):
    parsed = urlparse(href)
    if parsed.scheme or parsed.netloc:
        if parsed.netloc and parsed.netloc != "dl-alexandre.github.io":
            return None
    path = unquote(parsed.path)
    if not path.startswith("/bible/"):
        return None
    relative = path.removeprefix("/bible/")
    if not relative or relative.endswith("/"):
        relative += "index.html"
    return root / relative


def main():
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "out/bible")
    errors = []
    for page in root.rglob("*.html"):
        # Psalm 119 is the only intentionally larger chapter; its 176 verses
        # need a higher ceiling while ordinary chapters remain under 50 KB.
        limit = 64_000 if page.parts[-2:] == ("Psalms", "119.html") else 50_000
        if page.stat().st_size > limit:
            errors.append(f"HTML budget exceeded: {page}")
        parser = PageParser()
        parser.feed(page.read_text(encoding="utf-8"))
        for required in ("html", "title", "main", "h1"):
            if required not in parser.tags:
                errors.append(f"{page}: missing <{required}>")
        for href in parser.links:
            target = target_path(root, href)
            if target and not target.exists():
                errors.append(f"{page}: broken link {href}")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Checked generated HTML under {root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
