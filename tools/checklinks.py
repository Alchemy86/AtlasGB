#!/usr/bin/env python3
"""Every link in every Markdown file must land on something that exists.

This repository is browsed far more often than it is cloned, so a dead link is
a real defect rather than a tidiness complaint — and there are more than 2,400
anchored links in the two indexes alone.  Three kinds are checked:

* **relative file links** — the target path exists;
* **fragments** — `page.md#s-wPartyCount` lands on an `<a id="s-wPartyCount">`
  the renderer actually emitted, or on a heading GitHub will slugify to that;
* **in-page fragments** — `#coverage` resolves against the same file's headings.

External `http(s)` links are listed but not fetched: a network call would make
this check flaky and slow, and a link rotting on somebody else's server is not
something CI can fix.  Run with `--external` to fetch them anyway.

    tools/checklinks.py              # the offline check, run by `make check`
    tools/checklinks.py --external   # also fetch every http(s) link
"""

from __future__ import annotations

import argparse
import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")
ANCHOR = re.compile(r'<a id="([^"]+)"')
HEADING = re.compile(r"^#{1,6}\s+(.+?)\s*$", re.MULTILINE)

SKIP_DIRS = {".git", ".github/workflows"}


def markdown_files() -> list[str]:
    out = []
    for root, dirs, files in os.walk(REPO):
        dirs[:] = [d for d in dirs if d != ".git"]
        for f in files:
            if f.endswith(".md"):
                out.append(os.path.join(root, f))
    return sorted(out)


def slug(heading: str) -> str:
    """GitHub's heading slug: lowercase, punctuation dropped, spaces to dashes.

    Each space becomes its own dash — GitHub does *not* collapse runs. That
    matters here because several headings contain an em dash, which is dropped
    and leaves two spaces behind: `## I — an invariant covers it` is
    `#i--an-invariant-covers-it`, with two.
    """
    s = heading.strip().lower()
    s = re.sub(r"[`*\[\]()]", "", s)
    s = re.sub(r"[^\w\s-]", "", s)
    return re.sub(r"\s", "-", s).strip("-")


def targets(path: str) -> set[str]:
    text = open(path, encoding="utf-8").read()
    out = set(ANCHOR.findall(text))
    for h in HEADING.findall(text):
        out.add(slug(h))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--external", action="store_true",
                    help="also fetch every http(s) link (needs the network)")
    args = ap.parse_args()

    files = markdown_files()
    cache: dict[str, set[str]] = {}
    broken: list[str] = []
    external: set[str] = set()
    checked = 0

    for path in files:
        rel = os.path.relpath(path, REPO)
        text = open(path, encoding="utf-8").read()
        for target in LINK.findall(text):
            checked += 1
            if target.startswith(("http://", "https://")):
                external.add(target)
                continue
            if target.startswith(("mailto:", "tel:")):
                continue
            file_part, _, frag = target.partition("#")
            if file_part:
                dest = os.path.normpath(os.path.join(os.path.dirname(path), file_part))
                if not os.path.exists(dest):
                    broken.append(f"{rel}: {target} — no such file")
                    continue
            else:
                dest = path
            if frag and dest.endswith(".md"):
                if dest not in cache:
                    cache[dest] = targets(dest)
                if frag not in cache[dest]:
                    broken.append(f"{rel}: {target} — no such anchor")

    print(f"{checked:,} links in {len(files)} Markdown files "
          f"({len(external)} external, not fetched)")

    if args.external:
        import urllib.error
        import urllib.request
        for url in sorted(external):
            req = urllib.request.Request(url, method="HEAD",
                                         headers={"User-Agent": "atlasgb-checklinks"})
            try:
                urllib.request.urlopen(req, timeout=15)
            except urllib.error.HTTPError as exc:
                if exc.code not in (403, 405):  # some hosts refuse HEAD
                    broken.append(f"(external) {url} — HTTP {exc.code}")
            except Exception as exc:  # noqa: BLE001 — any failure is a report
                broken.append(f"(external) {url} — {exc}")

    if broken:
        print(f"\nFAIL — {len(broken)} dead link(s):")
        for b in broken:
            print(f"  {b}")
        return 1
    print("OK — every link resolves.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
