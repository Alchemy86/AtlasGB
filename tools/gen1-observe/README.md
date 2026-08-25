# `gen1-observe` — descriptions and linkage from watching the cartridge run

A standalone tool, deliberately outside TerminalGB's own repository: it never writes there,
only depends on its published library crate (`terminalgb`, `default-features = false` — the
"embedding build," core only, which also sidesteps the panic-strategy build-cache collision
documented in TerminalGB's own `docs/pitfalls/builds.md`).

**What it answers that reading alone cannot.** The well of *existing* evidence — sibling
entries, TerminalGB's own plugin source comments — was found nearly dry after seventeen
descriptions from mining it. The well was never going to refill itself; this tool generates
fresh evidence instead of searching for old evidence, by watching what the cartridge's own
code actually does to an address during a real playthrough — who writes it, when, to what
values, and (this round) which *other* addresses the same routine writes — rather than
guessing from a name or hoping a coincidence surfaces a relationship.

Full results and provenance for each round of runs: [`docs/observation.md`](../../docs/observation.md).
This page is the mechanism; that one is what it found.

## Layout

- **`src/lib.rs`** — the deterministic script, shared by both passes. One driver
  (`Driver::tick`/`press`/`probe_menu`), called from `run_script`. Both binaries below replay
  this exact function; they must never diverge in what buttons are pressed or what state is
  forced, only in how they observe a frame, or a frame number from one pass stops meaning the
  same thing in the other.
- **`src/main.rs`** (`gen1-observe`) — pass 1: frame-granularity sweep.
- **`src/bin/pass2.rs`** — pass 2: instruction-granularity trace, targeted.
- **`src/bin/investigate_levelup.rs`** — a small, one-question trace: replays `run_script`
  watching only `wIsInBattle`, `wCurOpponent`, `wCurEnemyLevel` and the party's own species/
  HP/exp/level bytes every frame, to answer a specific hand-off question (why did the forced
  level-up in round three never fire) rather than sweeping everything. See
  `docs/observation.md` for what it found.

## How it works

**Pass 1** steps frame by frame — the same granularity, and the same `Gameboy::frame()` call,
TerminalGB's own `testharness/gen1atlas.rs` Tier C uses — and for every VRAM/WRAM/HRAM byte
records which frames it changed on and which values it took. Cheap: a full run in single-digit
seconds.

**Pass 2** re-runs the identical script, but for the specific frames pass 1 flagged as
containing a change to a target address, single-steps *instruction by instruction* instead of
calling `frame()` wholesale — replicating `Gameboy::frame()`'s own loop with the same public
methods it uses internally (`begin_frame`/`step_instruction`/`check_and_reset_gpu_updated`/
`end_frame`), so behaviour is identical and only the observation is finer-grained. For each
instruction it records the program counter and ROM bank *before* stepping, then diffs the
target addresses afterward. Determinism plus targeting only the frames that matter keeps this
fast: measured at roughly 27 million `step_instruction()` calls per second on this machine, so
even tracing every entry in the atlas (931 addresses, 2,252 frames) finishes well inside a
couple of minutes.

**A third step, in Python, is not a Rust program**: it cross-references every recorded
`(bank, pc)` against this atlas's own already-named ROM routines (to try to *name* a writer)
and, separately, groups target addresses by their *shared* writer (to find linkage — see
below). Three real pitfalls this step found and now corrects for, all worth knowing before
trusting a `(bank, pc)` at face value:

1. **A ROM entry's `len` is "distance to the next atlas entry in the same bank," which is
   exhaustively accurate for `WRAM0`/`HRAM` but can be wildly inflated for a sparsely-covered
   ROM bank.** `ItemNames`'s recorded `len` is 13,598 bytes — the rest of its bank, not its
   own true size — and an unguarded range match against it credited it with writing dozens of
   unrelated bytes. Fix: cap any ROM entry's matching window to its first 80 bytes.
2. **`ROM0` ($0000-$3FFF) is always mapped, regardless of the bank register — grouping its
   writers by `(bank, pc)` instead of by `pc` alone silently splits one shared routine into
   several apparently-different ones**, by the accident of whatever bank happened to be
   switched in when each write happened. Fix: for any PC below `$4000`, group and look it up
   by PC alone.
3. **This tool's own `debug_write()` setup calls (forcing a battle, stocking the bag, and so
   on) can be misattributed to whatever instruction happens to run first in the next traced
   frame**, because the "before" value they are diffed against is the state at the start of
   the whole run, not immediately before the `debug_write()`. Fix: never let a target address
   this tool itself directly writes seed a writer-group or a description — exclude it from
   that analysis entirely, even though its *later*, game-code-driven writes (if any) are still
   real data.

## Linkage: group by writing routine

Two addresses written by the *same* routine are related by construction — a fact this tool
measures directly (the PC behind every write), not a correlation it has to go looking for.
Frame-level and instruction-level *co-occurrence* (do two addresses change at the same moment)
were tried first and mostly failed — see `docs/observation.md` for why — but grouping by
*writer* instead, once the three pitfalls above are corrected for, found real structure: a
sound-engine channel-init routine that writes nine per-channel state fields together, a
stat-stage reset routine distinct from the one already credited in an existing description,
two shadow-OAM slots that are never touched independently regardless of which of several
routines is doing the touching. Full list in `docs/observation.md`.

## Reproducing it

```bash
cd tools/gen1-observe
export POKE_ROM=~/roms/pokeblue.gb        # your own cartridge; none is distributed here
export POKE_SAVE=~/roms/pokeblue.sav      # optional; a real save reaches more of the game

cargo run --release --bin gen1-observe    # pass 1 -> observe-pass1.json
# build pass 2's input from pass 1's output + whichever atlas addresses you want traced
# (a short Python script against atlas.json; see docs/observation.md for what this round used)
cargo run --release --bin pass2           # pass 2 -> observe-pass2.json

cargo run --release --bin investigate_levelup   # frame-by-frame trace to stdout, no JSON
```

Nothing here touches TerminalGB's own tree — the `Cargo.toml` path dependency reads its
published library surface, the same way `testharness/gen1atlas.rs` does from inside that
repository.

## What this does not answer

**Reads are not traced, only writes.** `step_instruction()` plus a before/after diff can only
see a byte *change*, which is a write. Seeing a *read* needs decoding which opcode ran at each
traced PC (this tool doesn't have a disassembler) or a bus-level hook inside TerminalGB itself
(a real change there — tell the captain and he dispatches it). Every description this tool has
produced says "written from," never "read by," for exactly this reason.

**`SRAM` (the save file) is not watched at all.** `Gameboy` exposes `debug_rom_bank()` for ROM
but nothing equivalent for the save file's four SRAM banks, so a change there cannot be
attributed to the right physical bank — worse than not looking, for an atlas whose `(region,
bank, addr)` triple is the identifying key. See the hand-off in `docs/observation.md`.

**Coverage depends entirely on what the script drives the game to do.** Two rounds of runs
have covered 911 of 936 watchable addresses (see `docs/observation.md` for the exact list of
the 25 that have not) — every one of those 25 has a specific, plausible reason (a two-console
link session, the Game Corner, a late-game area) rather than looking like a wrong address, but
that is a property of *this playthrough's coverage*, not a property of the tool.
