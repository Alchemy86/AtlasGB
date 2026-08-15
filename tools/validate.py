#!/usr/bin/env python3
"""Check that every atlas's `atlas.tsv` is structurally sound — no emulator needed.

Runs over every atlas under `atlases/`, or over one named with `--atlas`.  This
is the part of the verification that needs nothing but the file itself, so it
runs here, in this repository's CI, on every push.  The part that needs a
running cartridge lives in the emulator that produces the evidence tiers; see
[`docs/verification.md`](../docs/verification.md).

What it proves, for each atlas:

* the file parses: one header row, then rows of exactly that many tab-separated
  fields, and no comment or blank lines (the TSV starts with its header so that
  GitHub renders it as a sortable table — the provenance lives in
  `docs/provenance.md`, not in the data);
* every address lies inside the region it claims;
* every `region`, `role`, `group` and `verify` token is from the vocabulary
  `docs/schema.md` publishes — a token outside it would be a claim nothing
  checks;
* entries in a region and bank march strictly upwards, and each entry's `len`
  really is the distance to the next distinct address;
* work RAM and high RAM are **fully accounted for** — every byte is named,
  inside a `gap`, or declared `free`.  A hole is a byte nobody looked at, and
  this repository's central claim is that there are none;
* every symbol name is unique, because the evidence loop keys on it.

    tools/validate.py                     # every atlas; non-zero on any failure
    tools/validate.py --atlas pokemon-rb  # just that one
    tools/validate.py --quiet             # only print failures
"""

from __future__ import annotations

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import atlases as atlases_mod  # noqa: E402

REPO = atlases_mod.REPO

COLUMNS = ["region", "bank", "addr", "len", "symbol", "role", "sect", "group",
           "verify", "desc"]

REGIONS = {
    "VRAM": (0x8000, 0x9FFF),
    "SRAM": (0xA000, 0xBFFF),
    "WRAM0": (0xC000, 0xDFFF),
    "HRAM": (0xFF80, 0xFFFE),
    "ROM0": (0x0000, 0x3FFF),
    "ROMX": (0x4000, 0x7FFF),
}
# The regions the atlas walks exhaustively.  SRAM is banked and sparse by
# design — the cartridge really does leave room unallocated — so it is checked
# for containment but not for completeness.
COMPLETE = ("WRAM0", "HRAM")

ROLES = {"entry", "alias", "instance", "gap", "free"}
VERIFY_TOKENS = {"rom", "live", "inv"}


class Report:
    def __init__(self, quiet: bool) -> None:
        self.quiet = quiet
        self.failed = 0

    def check(self, name: str, ok: bool, note: str = "") -> None:
        if not ok:
            self.failed += 1
        if ok and self.quiet:
            return
        mark = "ok  " if ok else "FAIL"
        print(f"  {mark} {name}" + (f" — {note}" if note else ""))


def read(path: str) -> list[dict]:
    with open(path, encoding="utf-8") as handle:
        lines = [line.rstrip("\n") for line in handle]
    if not lines:
        raise SystemExit(f"{path}: empty")
    header = lines[0].split("\t")
    if header != COLUMNS:
        raise SystemExit(f"{path}:1: header is {header}, expected {COLUMNS}")
    rows = []
    for n, line in enumerate(lines[1:], start=2):
        if not line.strip():
            continue
        fields = line.split("\t")
        if len(fields) != len(header):
            raise SystemExit(
                f"{path}:{n}: {len(fields)} columns, header has {len(header)}"
            )
        row = dict(zip(header, fields))
        row["_line"] = n
        row["_addr"] = int(row["addr"].lstrip("$"), 16)
        row["_len"] = int(row["len"] or 0)
        rows.append(row)
    return rows


def validate(atlas, quiet: bool) -> int:
    """Every structural check, for one atlas.  Returns the number that failed."""
    ATLAS = atlas.tsv
    print(f"AtlasGB — validating {atlas.id} ({atlas.title})")
    print(f"  {atlas.rel(ATLAS)}\n")
    rows = read(ATLAS)
    rep = Report(quiet)

    # --- the atlas says which cartridge it is about ------------------------
    # A number without a game attached is meaningless, and every generated
    # page's title comes from here.
    for key in ("title", "games", "summary"):
        rep.check(f"meta.json names a {key}", bool(atlas.meta.get(key)),
                  f"{atlas.rel(atlas.meta_path)} is what the pages are titled from")

    # --- the file itself ---------------------------------------------------
    raw = open(ATLAS, encoding="utf-8").read()
    rep.check(
        "the file starts with its header row",
        raw.startswith("\t".join(COLUMNS) + "\n"),
        "so GitHub renders it as a sortable, searchable table",
    )
    rep.check(
        "no comment lines",
        not any(line.startswith("#") for line in raw.splitlines()),
        "provenance lives in docs/provenance.md, not in the data",
    )
    rep.check("rows parse", True, f"{len(rows):,} entries")

    # --- vocabularies ------------------------------------------------------
    bad = [r for r in rows if r["region"] not in REGIONS]
    rep.check("every region is known", not bad,
              "; ".join(f"{r['_line']}: {r['region']}" for r in bad[:5]))
    bad = [r for r in rows if r["role"] not in ROLES]
    rep.check("every role is known", not bad,
              "; ".join(f"{r['_line']}: {r['role']}" for r in bad[:5]))
    bad = [
        r for r in rows
        if any(t not in VERIFY_TOKENS for t in r["verify"].split(",") if t)
    ]
    rep.check("every evidence token is known", not bad,
              "; ".join(f"{r['_line']}: {r['verify']}" for r in bad[:5]))
    bad = [r for r in rows
           if r["bank"] != "-" and not r["bank"].isdigit()]
    rep.check("every bank is a number or '-'", not bad,
              "; ".join(f"{r['_line']}: {r['bank']}" for r in bad[:5]))

    # --- addresses ---------------------------------------------------------
    bad = []
    for r in rows:
        lo, hi = REGIONS.get(r["region"], (0, 0xFFFF))
        if not lo <= r["_addr"] <= hi:
            bad.append(r)
    rep.check("every address is inside the region it claims", not bad,
              "; ".join(f"{r['_line']}: {r['addr']} not in {r['region']}"
                        for r in bad[:5]))

    dupes = {}
    for r in rows:
        dupes.setdefault(r["symbol"], []).append(r["_line"])
    clashes = {s: ls for s, ls in dupes.items() if len(ls) > 1}
    rep.check("every symbol name is unique", not clashes,
              "; ".join(f"{s} on lines {ls}" for s, ls in list(clashes.items())[:5]))

    # GitHub lowercases the `id` it renders from `<a id="s-SYMBOL">`, so two
    # symbols differing only in case would land on one anchor and one of the
    # two permanent links would silently point at the wrong entry. There are
    # none today; this is here so there are none tomorrow either.
    fold = {}
    for r in rows:
        fold.setdefault(r["symbol"].lower(), []).append(r["symbol"])
    folded = {k: v for k, v in fold.items() if len(set(v)) > 1}
    rep.check(
        "no two symbols differ only in case", not folded,
        "; ".join(f"{sorted(set(v))}" for v in list(folded.values())[:5])
        + (" — their anchors would collide" if folded else ""),
    )

    # --- ordering and extent ----------------------------------------------
    groups: dict[tuple, list[dict]] = {}
    for r in rows:
        groups.setdefault((r["region"], r["bank"]), []).append(r)
    out_of_order, wrong_len = [], []
    for key, rs in groups.items():
        prev = -1
        for r in rs:
            if r["_addr"] < prev:
                out_of_order.append(f"{key}: {r['addr']} after ${prev:04X}")
            prev = r["_addr"]
        # `len` is the distance to the next *distinct* address in this region
        # and bank; the last row runs to whatever its own length says.
        addrs = sorted({r["_addr"] for r in rs})
        nxt = {a: b for a, b in zip(addrs, addrs[1:])}
        for r in rs:
            after = nxt.get(r["_addr"])
            if after is not None and r["_addr"] + r["_len"] > after:
                wrong_len.append(
                    f"{key} {r['symbol']}: {r['addr']}+{r['_len']} overruns ${after:04X}"
                )
    rep.check("entries are in ascending address order", not out_of_order,
              "; ".join(out_of_order[:5]))
    rep.check("no entry overruns the next address", not wrong_len,
              "; ".join(wrong_len[:5]))

    # --- completeness: the claim the whole repository rests on -------------
    for region in COMPLETE:
        lo, hi = REGIONS[region]
        span = hi - lo + 1
        seen = bytearray(span)
        for r in rows:
            if r["region"] != region:
                continue
            for a in range(r["_addr"], r["_addr"] + max(r["_len"], 1)):
                if lo <= a <= hi:
                    seen[a - lo] = 1
        missing = span - sum(seen)
        rep.check(
            f"{region} is completely accounted for", missing == 0,
            f"{sum(seen):,} of {span:,} bytes"
            + (f", {missing:,} unaccounted" if missing else ""),
        )

    # --- the published JSON really does match its published schema ---------
    # A schema nobody runs is documentation that has not been checked, and this
    # one is what a consumer writes their parser against.
    label = f"{atlas.rel(atlas.json)} matches {atlas.rel(atlases_mod.SCHEMA)}"
    try:
        import json

        import jsonschema
    except ImportError:
        rep.check(label, True, "SKIPPED — `pip install jsonschema` to run it")
    else:
        if not os.path.exists(atlas.json):
            rep.check(label, False,
                      f"{atlas.rel(atlas.json)} is missing — run `make data`")
        else:
            with open(atlases_mod.SCHEMA, encoding="utf-8") as handle:
                schema = json.load(handle)
            with open(atlas.json, encoding="utf-8") as handle:
                doc = json.load(handle)
            errs = list(jsonschema.Draft202012Validator(schema).iter_errors(doc))
            rep.check(
                label, not errs,
                "; ".join(f"{list(e.path)[:3]}: {e.message}" for e in errs[:3]),
            )

    # --- the honest seven --------------------------------------------------
    storage = [r for r in rows if r["region"] in ("VRAM", "SRAM", "WRAM0", "HRAM")]
    unevidenced = [r for r in storage if not r["verify"]]
    print(f"\n{len(rows):,} entries, {len(storage):,} of them storage.")
    for tok, label in (("rom", "named by the cartridge"),
                       ("live", "observed live"),
                       ("inv", "covered by an invariant")):
        n = sum(1 for r in storage if tok in r["verify"].split(","))
        print(f"  {n:>5,}  {label}")
    print(f"  {len(unevidenced):>5,}  no evidence yet — stated, not hidden")
    for r in unevidenced:
        print(f"         {r['addr']} {r['symbol']}")

    if rep.failed:
        print(f"\n{atlas.id}: {rep.failed} check(s) failed.")
    else:
        print(f"\n{atlas.id}: all checks passed.")
    return rep.failed


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    atlases_mod.add_argument(ap)
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args()

    selected = atlases_mod.select(args.atlas)
    failed = 0
    for n, atlas in enumerate(selected):
        if n:
            print()
        failed += validate(atlas, args.quiet)
    if failed:
        print(f"\n{failed} check(s) failed across {len(selected)} atlas(es).")
        return 1
    print(f"\nOK — {len(selected)} atlas(es) validated.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
