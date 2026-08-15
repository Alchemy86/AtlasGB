#!/usr/bin/env python3
"""Export each atlas's `atlas.tsv` to the other shapes people consume it in.

The TSV is the source of truth and everything else is derived from it, so a
consumer never has to choose which file is right.  Two derived forms ship per
atlas, beside the TSV in `atlases/<id>/data/`:

* `atlas.json` — the whole atlas as one object: a `meta` block naming the game
  and carrying the counts, then `entries`, an array of objects with the TSV's
  columns typed (`addr` and `len` become integers, `verify` becomes a list).
  This is the form a web tool or a save editor wants; `schema/atlas.schema.json`
  describes it, and it is shared by every atlas.
* `atlas.min.json` — the same entries with the empty fields dropped and no
  indentation, for anything that has to fetch it over a network.

`meta` says **which cartridge** the numbers are about, because a count with no
game attached is meaningless once there is more than one atlas.

    tools/export.py                     # rewrite both, for every atlas
    tools/export.py --atlas pokemon-rb  # just that one
    tools/export.py --check             # fail if anything is stale

`--check` runs in CI, so the JSON cannot drift away from the TSV the way a
hand-maintained second copy always eventually does.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import atlases as atlases_mod  # noqa: E402

REPO = atlases_mod.REPO

# The regions that are storage.  ROM entries are addresses of *tables in the
# cartridge*, and the evidence tiers mean something different for them (a ROM
# table is never "observed live"), so the headline counts exclude them — the
# same split the pages use.
STORAGE = ("VRAM", "SRAM", "WRAM0", "HRAM")

INT_COLUMNS = ("len",)
TOKEN_COLUMNS = ("verify",)


def read_tsv(path: str) -> tuple[list[str], list[dict]]:
    with open(path, encoding="utf-8") as handle:
        lines = [line.rstrip("\n") for line in handle if line.strip()]
    header = lines[0].split("\t")
    rows = []
    for n, line in enumerate(lines[1:], start=2):
        fields = line.split("\t")
        if len(fields) != len(header):
            raise SystemExit(
                f"{path}:{n}: {len(fields)} columns, header has {len(header)}"
            )
        rows.append(dict(zip(header, fields)))
    return header, rows


def typed(row: dict) -> dict:
    """One TSV row as the JSON object a consumer wants.

    `addr` keeps its `$XXXX` spelling *and* gains an integer, because a tool
    that indexes by address wants the number and a tool that prints one wants
    the form every other Gen 1 document uses.
    """
    out = dict(row)
    out["addr_int"] = int(row["addr"].lstrip("$"), 16)
    for c in INT_COLUMNS:
        out[c] = int(row[c] or 0)
    for c in TOKEN_COLUMNS:
        out[c] = [t for t in row[c].split(",") if t]
    return out


def meta(atlas, entries: list[dict]) -> dict:
    """The `meta` block: which cartridge this is, then the counts.

    The identity fields come from the atlas's own `meta.json`, so the JSON, the
    pages and the citation cannot end up disagreeing about which game 2,898
    claims are claims *about*.
    """
    storage = [e for e in entries if e["region"] in STORAGE]

    def tier(tok: str) -> int:
        return sum(1 for e in storage if tok in e["verify"])

    return {
        "project": "AtlasGB",
        "atlas": atlas.id,
        "title": atlas.title,
        "platform": atlas.meta.get("platform", "Game Boy"),
        "description": atlas.summary,
        "games": atlas.games,
        "engine": atlas.meta.get("engine", {}).get("name", ""),
        "source_of_truth": atlas.rel(atlas.tsv),
        "schema": atlas.rel(atlases_mod.SCHEMA),
        "documentation": "docs/schema.md",
        "pages": atlas.rel(atlas.docs) + "/",
        "entries": len(entries),
        "storage_entries": len(storage),
        "evidence": {
            "rom": tier("rom"),
            "live": tier("live"),
            "inv": tier("inv"),
            "none": sum(1 for e in storage if not e["verify"]),
        },
        "regions": sorted({e["region"] for e in entries}),
        "chapters": sorted({e["group"] for e in entries}),
    }


def build(atlas) -> tuple[str, str]:
    _, rows = read_tsv(atlas.tsv)
    entries = [typed(r) for r in rows]
    full = json.dumps(
        {"meta": meta(atlas, entries), "entries": entries}, indent=1, ensure_ascii=False
    ) + "\n"
    slim = json.dumps(
        {
            "meta": meta(atlas, entries),
            "entries": [
                {k: v for k, v in e.items() if v not in ("", [], 0) or k == "len"}
                for e in entries
            ],
        },
        separators=(",", ":"),
        ensure_ascii=False,
    ) + "\n"
    return full, slim


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    atlases_mod.add_argument(ap)
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()

    selected = atlases_mod.select(args.atlas)
    stale, wrote = [], []
    for atlas in selected:
        full, slim = build(atlas)
        for path, want in ((atlas.json, full), (atlas.min_json, slim)):
            have = open(path, encoding="utf-8").read() if os.path.exists(path) else None
            if have != want:
                stale.append(atlas.rel(path))
            if not args.check:
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write(want)
                wrote.append(f"{atlas.rel(path)} ({len(want):,} bytes)")

    if args.check:
        if stale:
            print("FAIL — the exported data is stale; run `make data`:")
            for s in stale:
                print(f"  {s}")
            return 1
        print(f"OK — the JSON of {len(selected)} atlas(es) matches its TSV")
        return 0

    for w in wrote:
        print(f"wrote {w}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
