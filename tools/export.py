#!/usr/bin/env python3
"""Export `data/atlas.tsv` to the other shapes people consume it in.

The TSV is the source of truth and everything else is derived from it, so a
consumer never has to choose which file is right.  Two derived forms ship:

* `data/atlas.json` — the whole atlas as one object: a `meta` block with the
  counts, then `entries`, an array of objects with the TSV's columns typed
  (`addr` and `len` become integers, `verify` becomes a list).  This is the
  form a web tool or a save editor wants; `data/atlas.schema.json` describes it.
* `data/atlas.min.json` — the same entries with the empty fields dropped and no
  indentation, for anything that has to fetch it over a network.

    tools/export.py            # rewrite both
    tools/export.py --check    # fail if either is stale

`--check` runs in CI, so the JSON cannot drift away from the TSV the way a
hand-maintained second copy always eventually does.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ATLAS = os.path.join(REPO, "data", "atlas.tsv")
FULL = os.path.join(REPO, "data", "atlas.json")
MIN = os.path.join(REPO, "data", "atlas.min.json")

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


def meta(entries: list[dict]) -> dict:
    storage = [e for e in entries if e["region"] in STORAGE]

    def tier(tok: str) -> int:
        return sum(1 for e in storage if tok in e["verify"])

    return {
        "name": "AtlasGB",
        "description": (
            "Every address in Pokemon Red and Blue, with the evidence for it."
        ),
        "source_of_truth": "data/atlas.tsv",
        "schema": "data/atlas.schema.json",
        "documentation": "docs/schema.md",
        "games": ["Pokemon Blue (USA/Europe)", "Pokemon Red (USA/Europe)"],
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


def build() -> tuple[str, str]:
    _, rows = read_tsv(ATLAS)
    entries = [typed(r) for r in rows]
    full = json.dumps(
        {"meta": meta(entries), "entries": entries}, indent=1, ensure_ascii=False
    ) + "\n"
    slim = json.dumps(
        {
            "meta": meta(entries),
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
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()

    full, slim = build()
    stale = []
    for path, want in ((FULL, full), (MIN, slim)):
        have = open(path, encoding="utf-8").read() if os.path.exists(path) else None
        if have != want:
            stale.append(os.path.relpath(path, REPO))
        if not args.check:
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(want)

    if args.check:
        if stale:
            print("FAIL — the exported data is stale; run `make data`:")
            for s in stale:
                print(f"  {s}")
            return 1
        print("OK — data/atlas.json and data/atlas.min.json match data/atlas.tsv")
        return 0

    print(f"wrote {os.path.relpath(FULL, REPO)} ({len(full):,} bytes) and "
          f"{os.path.relpath(MIN, REPO)} ({len(slim):,} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
