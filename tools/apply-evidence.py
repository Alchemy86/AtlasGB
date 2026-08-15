#!/usr/bin/env python3
"""Land a verification run into the atlas, and keep the tiers from going stale.

An entry's evidence tier is this repository's most valuable property, and it is
the one thing this repository cannot produce on its own: proving an address
against a running cartridge needs an emulator, and the emulator lives
elsewhere.  So the tiers arrive here as a **report**, and the loop is closed by
making the `verify` column unwritable by hand.

    tools/apply-evidence.py report.json                       # land a run
    tools/apply-evidence.py report.json --atlas pokemon-rb    # ... into that atlas
    tools/apply-evidence.py --check              # every atlas: is the verify column
                                                 # still the one its last run produced?

`--check` runs in CI.  Each atlas's `data/evidence.json` records the provenance of
the last run landed into it *and a digest of the verify column it produced*; if anybody edits an
evidence token by hand, or adds a symbol without re-running the verification,
the digest stops matching and CI is red.  A tier in this atlas therefore always
names a real run against a real cartridge — which is the whole claim.

## The report

A producer writes one JSON object.  See `docs/verification.md` for the prose and
`docs/schema.md` for the evidence tokens.

    {
      "schema": "atlasgb-evidence/1",
      "atlas": "pokemon-rb",
      "produced_by": {"repo": "...", "commit": "...", "harness": "..."},
      "cartridge":   {"title": "POKEMON BLUE", "sha1": "..."},
      "script":      {"name": "opening+overworld", "frames": 3600},
      "run":         {"date": "2026-08-15"},
      "verify":      {"wPartyCount": ["rom", "live", "inv"], ...}
    }

`atlas` names which atlas the run is about.  It is **optional** — a report
without it lands into `--atlas`, which is how reports written before the project
grew a second cartridge keep working — but emitting it is the better habit, and
if it is present it must agree with the atlas being landed into.

Every symbol in the atlas must appear in `verify`, and every symbol in `verify`
must be in the atlas: a partial report would silently downgrade whatever it left
out, which is exactly the failure this file exists to prevent.  Use an empty
list to say "this entry has no evidence" — that is a claim the report is making,
not an omission.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import atlases as atlases_mod  # noqa: E402

REPO = atlases_mod.REPO

SCHEMA = "atlasgb-evidence/1"
TOKENS = ("rom", "live", "inv")
STORAGE = ("VRAM", "SRAM", "WRAM0", "HRAM")


def read_atlas(ATLAS) -> tuple[list[str], list[dict]]:
    with open(ATLAS, encoding="utf-8") as handle:
        lines = [line.rstrip("\n") for line in handle if line.strip()]
    header = lines[0].split("\t")
    return header, [dict(zip(header, line.split("\t"))) for line in lines[1:]]


def digest(rows: list[dict]) -> str:
    """A hash of exactly the claim we are protecting: symbol → evidence.

    Deliberately *not* a hash of the whole file — a new description or a
    reworded chapter must not invalidate a verification run, because neither
    changes what was proved.
    """
    body = "".join(f"{r['symbol']}\t{r['verify']}\n" for r in rows)
    return "sha256:" + hashlib.sha256(body.encode("utf-8")).hexdigest()


def load_report(path: str) -> dict:
    with open(path, encoding="utf-8") as handle:
        report = json.load(handle)
    if report.get("schema") != SCHEMA:
        raise SystemExit(f"{path}: schema is {report.get('schema')!r}, expected {SCHEMA!r}")
    for key in ("produced_by", "cartridge", "script", "run", "verify"):
        if key not in report:
            raise SystemExit(f"{path}: missing {key!r}")
    for sym, toks in report["verify"].items():
        if not isinstance(toks, list) or any(t not in TOKENS for t in toks):
            raise SystemExit(f"{path}: {sym}: {toks!r} is not a list of {TOKENS}")
    return report


def apply(path: str, atlas) -> int:
    report = load_report(path)
    named = report.get("atlas")
    if named is not None and named != atlas.id:
        raise SystemExit(
            f"{path}: the report is for atlas {named!r}, but this is {atlas.id!r}. "
            "Landing it here would attribute one cartridge's evidence to another."
        )
    ATLAS, RECORD = atlas.tsv, atlas.evidence
    header, rows = read_atlas(ATLAS)
    have = {r["symbol"] for r in rows}
    want = set(report["verify"])
    if have - want:
        raise SystemExit(
            f"{path}: {len(have - want)} atlas symbols are not in the report, "
            f"starting with {sorted(have - want)[:5]} — a partial report would "
            "silently downgrade them"
        )
    if want - have:
        raise SystemExit(
            f"{path}: {len(want - have)} reported symbols are not in the atlas, "
            f"starting with {sorted(want - have)[:5]} — refresh the atlas first"
        )

    moved = []
    for r in rows:
        now = ",".join(t for t in TOKENS if t in report["verify"][r["symbol"]])
        if now != r["verify"]:
            moved.append(f"{r['addr']} {r['symbol']}: {r['verify']!r} -> {now!r}")
            r["verify"] = now

    with open(ATLAS, "w", encoding="utf-8") as handle:
        handle.write("\t".join(header) + "\n")
        for r in rows:
            handle.write("\t".join(r.get(c, "") for c in header) + "\n")

    # The published tiers count *storage* entries, because "observed live" means
    # something different for a table in the cartridge than for a byte of RAM.
    # Both figures are recorded so neither can be quoted as the other.
    storage = [r for r in rows if r["region"] in STORAGE]

    def tally(rs):
        return {tok: sum(1 for r in rs if tok in r["verify"].split(","))
                for tok in TOKENS}

    record = {
        "schema": SCHEMA,
        "atlas": atlas.id,
        "title": atlas.title,
        "produced_by": report["produced_by"],
        "cartridge": report["cartridge"],
        "script": report["script"],
        "run": report["run"],
        "entries": len(rows),
        "storage_entries": len(storage),
        "evidence": tally(storage),
        "evidence_including_rom_tables": tally(rows),
        "unevidenced": sorted(r["symbol"] for r in storage if not r["verify"]),
        "verify_digest": digest(rows),
    }
    with open(RECORD, "w", encoding="utf-8") as handle:
        json.dump(record, handle, indent=1, ensure_ascii=False)
        handle.write("\n")

    print(f"landed {os.path.basename(path)}: {len(moved)} entries moved tier")
    for m in moved[:20]:
        print(f"  {m}")
    if len(moved) > 20:
        print(f"  ... and {len(moved) - 20} more")
    print(f"wrote {os.path.relpath(RECORD, REPO)} "
          f"({record['verify_digest'][:23]}…)")
    print("Now run `make docs data` — the pages carry these counts.")
    return 0


def check(atlas) -> int:
    _, rows = read_atlas(atlas.tsv)
    RECORD = atlas.evidence
    if not os.path.exists(RECORD):
        print(f"FAIL — {atlas.rel(RECORD)} is missing: the evidence "
              f"tiers in the {atlas.id} atlas name no run at all")
        return 1
    with open(RECORD, encoding="utf-8") as handle:
        record = json.load(handle)
    now = digest(rows)
    if now != record.get("verify_digest"):
        print(f"FAIL — {atlas.id}: the verify column is not the one the last "
              "landed run produced.")
        print(f"  recorded {record.get('verify_digest')}")
        print(f"  actual   {now}")
        print("  Evidence is not editable by hand: land a report with")
        print(f"  `tools/apply-evidence.py report.json --atlas {atlas.id}`.")
        print("  See docs/verification.md.")
        return 1
    run = record["run"].get("date", "?")
    cart = record["cartridge"].get("title", "?")
    print(f"OK — {atlas.id}: evidence tiers match the run of {run} against {cart} "
          f"({record['entries']:,} entries, {len(record['unevidenced'])} unevidenced)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("report", nargs="?", help="a verification report to land")
    ap.add_argument("--check", action="store_true",
                    help="verify the tiers still match the last landed run")
    atlases_mod.add_argument(ap)
    args = ap.parse_args()
    if args.check:
        # Every atlas by default: a run that stops checking an atlas because
        # nobody named it is the drift this file exists to prevent.
        return max(check(a) for a in atlases_mod.select(args.atlas))
    if not args.report:
        ap.error("give a report to land, or pass --check")
    return apply(args.report, atlases_mod.one(args.atlas))


if __name__ == "__main__":
    sys.exit(main())
