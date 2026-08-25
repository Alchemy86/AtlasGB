# Pokémon Gold and Silver — pending

[← AtlasGB](../../README.md) · [adding an atlas](../../docs/adding-an-atlas.md)

**This is not an atlas yet, and this directory says so on purpose.** An AtlasGB atlas is a
`data/atlas.tsv` with evidence behind every row, produced by a `--pokered`-style extractor
against a built disassembly — see [what an atlas is](../../docs/adding-an-atlas.md#what-an-atlas-is).
Generation 2 (Pokémon Gold and Silver) has none of that here: no `pokegold` extractor, no
`atlas.tsv`, no verification run. Building the real pipeline is genuine work and it is not what
this directory does.

**What this directory holds instead: prose, carrying real cartridge facts, that were already
written down elsewhere and had nowhere honest to live.** [TerminalGB](https://github.com/Alchemy86/TerminalGB),
the emulator project that produces this project's evidence, had accumulated a `docs/gen2/` area
of its own — game knowledge about Gold and Silver's memory layout, save file and one real
Gen 1↔Gen 2 cartridge feature (the Time Capsule) — sitting beside its emulator/tooling docs for
the same reason AtlasGB's own Pokémon Red/Blue material used to. [`docs/discoveries.md`](../pokemon-rb/docs/discoveries.md)
is the precedent this directory follows: a hand-written page, no `atlas.tsv` behind it, never
touched by [`tools/render.py`](../../tools/render.py), carrying facts that are true and citable
today even though the tooling to *verify every address at scale* does not exist yet.

**Because there is no `meta.json` here, none of the project's tooling discovers this
directory.** [`tools/atlases.py`](../../tools/atlases.py) finds atlases by the presence of
`meta.json`; this directory deliberately has none, so `make check` — `validate`, `evidence`,
`render --check`, `export --check` — never runs against it and is never told it is incomplete.
[`tools/checklinks.py`](../../tools/checklinks.py) is the one exception: it walks every Markdown
file in the repository regardless, so the links and anchors on these pages are checked like any
other page's.

## What is here

| page | what it is |
|---|---|
| [the memory map](docs/memory-map.md) | every Gen 2 address this project can currently cite, and what is honestly not established |
| [the save file](docs/the-save-file.md) | the three-run save layout, its checksum, and why editing it cannot move the player |
| [reading the screen](docs/reading-the-screen.md) | the tilemap-as-text property Gen 2 shares with Gen 1, and the three ways it differs |
| [the Time Capsule](docs/time-capsule.md) | the one sanctioned Gen 1 ↔ Gen 2 link: what it requires, what it refuses, and the shiny rule read out of the cartridge |

Every address on these pages is graded loosely in prose as *derived from a byte-identical
disassembly build*, *reached by adding up between two known addresses*, or *measured on a
running cartridge* — the same three ideas [Pokémon Red/Blue's evidence tiers](../pokemon-rb/README.md#the-evidence)
formalise as **R**/**L**/**I**, but stated as sentences rather than as a checked, digested
`verify` column, because there is no [`tools/apply-evidence.py`](../../tools/apply-evidence.py)
run behind any of it. Treat a claim here as **cited, not verified at this atlas's usual bar** —
if you need the bar, that is exactly the work [adding this atlas for real](../../docs/adding-an-atlas.md)
would do.

## What is not here

Everything TerminalGB's own `docs/gen2/` area states plainly it has not established: Gen 2
battle structures, storage, the bag, the RNG, wild encounter tables, and the map system proper
(headers, tilesets, warp/object tables). See each page's own gaps section — an absent structure
is recorded as *absent*, not silently skipped.

Also not here, deliberately: anything about *building* the `pokegold` disassembly, running
TerminalGB's own bot/tooling against a Gold cartridge, or TerminalGB's own two-emulator-instance
architecture for driving a link. Those are TerminalGB's own tooling, not facts about the
cartridge, and stay in TerminalGB's `docs/gen2/`.

## Provenance

Adapted from TerminalGB's `docs/gen2/memory-map.md`, `the-save-file.md`, `reading-the-screen.md`,
`sharp-edges.md` and `time-capsule.md`, each written from a `pokegold` disassembly built and
checked against `roms.sha1` byte for byte — see
[building the disassembly](https://github.com/Alchemy86/TerminalGB/blob/main/docs/gen2/building-the-disassembly.md),
which stays in TerminalGB because it is about that build process, not about the cartridge.
Every description on these pages is written from scratch, exactly as [provenance.md](../../docs/provenance.md)
requires; addresses and structure layouts are facts about the cartridge, cited by `pret/pokegold`
symbol name.

**Licence: CC BY-SA 4.0**, like every other atlas page in this project — see
[licensing.md](../../docs/licensing.md).
