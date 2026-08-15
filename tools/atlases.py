#!/usr/bin/env python3
"""Where the atlases are, and what each one is.

AtlasGB is a **project**, not a file.  One cartridge's map is one *atlas*, and
an atlas is a directory under `atlases/` holding its own data, its own pages and
a `meta.json` saying which game it is about:

    atlases/<id>/meta.json          identity: title, games, engine, sources
    atlases/<id>/data/atlas.tsv     the source of truth for that cartridge
    atlases/<id>/data/atlas.json    generated from it by tools/export.py
    atlases/<id>/data/evidence.json which verification run its tiers came from
    atlases/<id>/README.md          that atlas's front page
    atlases/<id>/docs/*.md          its chapters and indexes

Everything above that line is shared and lives once: `schema/`, `tools/`,
`docs/` and the brand.  Adding a second cartridge is a new sibling directory,
never a merge into a global file — which is the whole reason the data is
namespaced by game.  See `docs/adding-an-atlas.md`.

Every tool in `tools/` takes `--atlas <id>` and defaults to acting on *all* of
them, so nothing has to be taught about a new atlas beyond creating its
directory.

    tools/atlases.py        # list what is published, as the tools see it
"""

from __future__ import annotations

import json
import os

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIR = os.path.join(REPO, "atlases")
SCHEMA = os.path.join(REPO, "schema", "atlas.schema.json")

META_SCHEMA = "atlasgb-atlas/1"

# The one every tool acts on when a single atlas is required and none is named.
DEFAULT = "pokemon-rb"


class Atlas:
    """One cartridge's atlas: where its files are, and what it is about."""

    def __init__(self, ident: str) -> None:
        self.id = ident
        self.dir = os.path.join(DIR, ident)
        self.data = os.path.join(self.dir, "data")
        self.docs = os.path.join(self.dir, "docs")
        self.tsv = os.path.join(self.data, "atlas.tsv")
        self.json = os.path.join(self.data, "atlas.json")
        self.min_json = os.path.join(self.data, "atlas.min.json")
        self.evidence = os.path.join(self.data, "evidence.json")
        self.readme = os.path.join(self.dir, "README.md")
        self.meta_path = os.path.join(self.dir, "meta.json")
        with open(self.meta_path, encoding="utf-8") as handle:
            self.meta = json.load(handle)
        if self.meta.get("schema") != META_SCHEMA:
            raise SystemExit(
                f"{self.rel(self.meta_path)}: schema is "
                f"{self.meta.get('schema')!r}, expected {META_SCHEMA!r}"
            )
        if self.meta.get("id") != ident:
            raise SystemExit(
                f"{self.rel(self.meta_path)}: id is {self.meta.get('id')!r} but the "
                f"directory is named {ident!r}"
            )

    # -- identity -----------------------------------------------------------

    @property
    def title(self) -> str:
        return self.meta["title"]

    @property
    def short_title(self) -> str:
        return self.meta.get("short_title", self.meta["title"])

    @property
    def games(self) -> list[str]:
        return self.meta.get("games", [])

    @property
    def summary(self) -> str:
        return self.meta.get("summary", "")

    # -- paths --------------------------------------------------------------

    @staticmethod
    def rel(path: str) -> str:
        return os.path.relpath(path, REPO).replace(os.sep, "/")

    def __repr__(self) -> str:  # pragma: no cover — diagnostics only
        return f"<Atlas {self.id}>"


def ids() -> list[str]:
    if not os.path.isdir(DIR):
        return []
    return sorted(
        name
        for name in os.listdir(DIR)
        if os.path.isfile(os.path.join(DIR, name, "meta.json"))
    )


def discover() -> list[Atlas]:
    """Every atlas in the repository, in a stable order.

    Discovery is by the presence of `meta.json`, not by a list kept somewhere:
    a list is a thing somebody forgets to add to, and the failure mode is an
    atlas that quietly stops being validated.
    """
    found = [Atlas(name) for name in ids()]
    if not found:
        raise SystemExit(f"no atlases found under {Atlas.rel(DIR)}/")
    return found


def select(ident: str | None) -> list[Atlas]:
    """The atlases a `--atlas` argument names — all of them when it is absent."""
    if ident is None:
        return discover()
    if not os.path.isfile(os.path.join(DIR, ident, "meta.json")):
        raise SystemExit(
            f"no such atlas: {ident!r}. Published: {', '.join(ids()) or '(none)'}"
        )
    return [Atlas(ident)]


def one(ident: str | None) -> Atlas:
    """Exactly one atlas: the one named, or the default."""
    return Atlas(ident or DEFAULT)


def add_argument(parser) -> None:
    """The `--atlas` flag, spelled the same way by every tool."""
    parser.add_argument(
        "--atlas",
        metavar="ID",
        help=f"act on one atlas ({', '.join(ids())}); default: all of them",
    )


def main() -> int:
    for atlas in discover():
        print(f"{atlas.id}  —  {atlas.title}")
        print(f"    {atlas.summary}")
        print(f"    data  {Atlas.rel(atlas.tsv)}")
        print(f"    pages {Atlas.rel(atlas.docs)}/")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
