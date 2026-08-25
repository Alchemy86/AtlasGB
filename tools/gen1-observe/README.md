# `gen1-observe` — descriptions and linkage from watching the cartridge run

A standalone tool, deliberately outside TerminalGB's own repository: it never writes there,
only depends on its published library crate (`terminalgb`, `default-features = false` — the
"embedding build," core only, which also sidesteps the panic-strategy build-cache collision
documented in TerminalGB's own `docs/pitfalls/builds.md`).

**What it answers that reading alone cannot.** Round two of the descriptions work found the
well of *existing* evidence — sibling entries, TerminalGB's own plugin source comments — was
nearly dry: seventeen new descriptions, most of the atlas's remaining 2,300 gaps untouched.
The well was never going to refill itself; this tool generates fresh evidence instead of
searching for old evidence, by watching what the cartridge's own code actually does to an
address during a real playthrough — who writes it, when, and to what values — rather than
guessing from its name.

## How it works

Two passes, both against the same deterministic script and (optionally) the same real save,
so a re-run reproduces the same frame numbers and the same findings:

**Pass 1** (`src/main.rs`) steps frame by frame — the same granularity, and the same
`Gameboy::frame()` call, TerminalGB's own `testharness/gen1atlas.rs` Tier C uses — and for
every WRAM/HRAM byte records which frames it changed on and which values it took. Cheap:
6,030 frames (a walk through the intro, a forced battle, and menus — see below) in under
five seconds.

**Pass 2** (`src/bin/pass2.rs`) re-runs the identical script, but for the specific frames
Pass 1 flagged as containing a change to one of the target addresses, it single-steps
*instruction by instruction* instead of calling `frame()` wholesale — replicating
`Gameboy::frame()`'s own loop with the same public methods it uses internally
(`begin_frame`/`step_instruction`/`check_and_reset_gpu_updated`/`end_frame`), so behaviour is
identical and only the observation is finer-grained. For each instruction it records the
program counter and ROM bank *before* stepping, then diffs the target addresses afterward —
so a change is attributed to the exact instruction that produced it. Determinism plus
targeting only the frames that matter (1,531 of 6,030 in this round's run) keeps this fast:
well under a second of wall-clock time at roughly 27 million `step_instruction()` calls per
second measured on this machine.

**A third step, in Python, is not a Rust program**: it cross-references every recorded
`(bank, pc)` against this atlas's own already-named ROM routines and tables (`region` is
`ROM0`/`ROMX`, `role` is `entry`) to try to *name* the writer, not just cite its address. This
mostly fails — only 55 ROM addresses are named in the atlas at all, so most writes come from
code nothing here has a name for yet — and it very nearly produced a wrong answer worth
recording: the atlas's `len` column is "distance to the next atlas entry in this bank," which
is exhaustively accurate for WRAM/HRAM but wildly inflated for a sparsely-covered ROM bank
(`ItemNames`'s recorded `len` is 13,598 bytes — the rest of its bank, not its own true size —
because nothing else in that bank is named yet). An unguarded range match credited `ItemNames`
with writing dozens of unrelated WRAM bytes. The fix: cap any ROM entry's matching window to
its first 80 bytes, which turns a 94%-but-wrong hit rate into a 3-of-79,258-but-honest one.
**Do not trust a ROM entry's `len` for anything except the WRAM/HRAM completeness invariant it
was built for.**

## Reproducing it

```bash
cd tools/gen1-observe
export POKE_ROM=~/roms/pokeblue.gb        # your own cartridge; none is distributed here
export POKE_SAVE=~/roms/pokeblue.sav      # optional; a real save reaches more of the game

cargo run --release --bin gen1-observe    # pass 1 -> observe-pass1.json
# then build the pass-2 input from pass 1's output + the atlas's own missing-desc list
# (a short Python script; see atlas-observed-descriptions handover for the exact one used)
cargo run --release --bin pass2           # pass 2 -> observe-pass2.json
```

Nothing here touches TerminalGB's own tree — the `Cargo.toml` path dependency reads its
published library surface, the same way `testharness/gen1atlas.rs` does from inside that
repository.

## What this round's run actually covered

The script: mash A/START through the intro (1,400 frames, same shape as TerminalGB's own
Tier C script) → walk and open menus (1,800 frames, same shape) → **force a wild battle** by
writing `wCurOpponent`/`wCurEnemyLevel` directly — the same technique the game's own debug
menu uses, and both addresses are already fully evidenced, described entries in this atlas,
not a new guess — and fight it out by pressing A (≈490 frames) → open the START menu and walk
its PARTY/ITEM screens (≈1,340 frames). 6,030 frames, roughly 100 seconds of game time, one
real save (the same retail Pokémon Blue cartridge and save used for this atlas's own
cartridge verification). It never reaches the PC/Bill's PC, catching, or a second kind of
battle — see the hand-off.

Of 505 atlas entries with no description that this playthrough's Pass 1 could even see move,
481 moved at least once. Of those, 90 had a write history clean enough — one to three
distinct `(bank, pc)` writers, once a boot-time bulk memory-clear loop (bank 1, `$1F80`,
observed as the sole writer of 307 *different* addresses — not a semantically interesting
fact about any one of them, and excluded) was recognised and filtered out — to write a
description from with real confidence. Every one of those 90 descriptions says so in its own
text and cites what was measured: the writer's location, the game phase, and the values
actually seen.

## What this does not answer

**Reads are not traced, only writes.** `step_instruction()` plus a before/after diff can only
see a byte *change*, which is a write. Seeing a *read* needs decoding which opcode ran at each
traced PC (this tool doesn't have a disassembler) or a bus-level hook inside TerminalGB itself
(which this project cannot add — see the hand-off). Every description above says "written
from," never "read by," for exactly this reason.

**Co-occurrence at frame granularity is mostly noise.** Pass 1 also records which addresses
changed on the same *frame*, which was meant to surface "written together" relationships for
linkage. In this run it mostly surfaced things that already change every frame regardless of
each other (the RNG seed bytes, a free-running counter) — a true fact, but not the kind of
relationship a reader benefits from being told. The stronger, instruction-level version of
the same idea (`instr_co_occurrence` in pass 2's own output) found nothing in this run because
none of the 505 targeted addresses happened to be written by the exact same instruction as
another target — plausible, since 16-bit writes pairing two *targeted* bytes are a small
fraction of all writes. See the hand-off for how to point this deliberately at known-adjacent
pairs (a structure's own consecutive fields) rather than at whatever the target list happens
to contain.
