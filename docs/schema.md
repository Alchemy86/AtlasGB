# The schema

[← AtlasGB](../README.md) · [provenance](provenance.md) · [verification](verification.md) · [consuming it](consuming.md)

**[`data/atlas.tsv`](../data/atlas.tsv) is the product.** Everything else in this
repository — the chapter pages, the two indexes, the structure tables, the JSON — is
generated from it. This page is its contract: every column, what it holds, what units it
is in, and every value it is allowed to take. You should be able to write a parser from
this page alone, without opening the generator.

If you want the machine-readable version, [`data/atlas.schema.json`](../data/atlas.schema.json)
is a JSON Schema for [`data/atlas.json`](../data/atlas.json), which carries the same rows
with the types applied.

---

## The file

Tab-separated, UTF-8, LF line endings, **one header row and then one row per entry**.

There are **no comment lines and no preamble** — the file opens with its header. That is
deliberate and it is worth stating because the file used to open with six lines of
provenance: GitHub's CSV/TSV viewer has no notion of a comment line, read the preamble as
one-column data, and refused to render the file at all. Rendered, GitHub gives a TSV a
sortable, searchable table view, which is most of what makes this data browsable without
cloning it. The provenance those lines carried is in [provenance.md](provenance.md).

Fields are **never quoted and never escaped**: no cell contains a tab or a newline, so
`line.split("\t")` is a complete parser. Descriptions may contain commas, backticks and
Markdown; treat `desc` as Markdown and everything else as plain text.

```python
import csv
with open("data/atlas.tsv", encoding="utf-8", newline="") as f:
    rows = list(csv.DictReader(f, delimiter="\t"))
```

---

## The columns

Ten columns, in this order. **Eight are derived** from a built `pret/pokered` checkout by
[`tools/extract.py`](../tools/extract.py) and must not be hand-edited; **two are ours** —
`verify` and `desc`.

| # | column | derived? | what it holds |
|--:|---|:--:|---|
| 1 | [`region`](#region) | derived | which of the CPU's address ranges the entry lives in |
| 2 | [`bank`](#bank) | derived | the bank number, or `-` |
| 3 | [`addr`](#addr) | derived | the CPU address |
| 4 | [`len`](#len) | derived | how many bytes the entry owns |
| 5 | [`symbol`](#symbol) | derived | the label, as the disassembly spells it |
| 6 | [`role`](#role) | derived | what kind of row this is |
| 7 | [`sect`](#sect) | derived | the linker section it was declared in |
| 8 | [`group`](#group) | rules | our chapter |
| 9 | [`verify`](#verify) | **ours** | the evidence for the entry |
| 10 | [`desc`](#desc) | **ours** | what it is, in our own words |

---

### `region`

<a id="region"></a>The address range, as the Game Boy's address decoder sees it. One of:

| value | span | what it is |
|---|---|---|
| `VRAM` | `$8000-$9FFF` | video RAM — the tiles and tilemaps the picture is made of |
| `SRAM` | `$A000-$BFFF` | cartridge RAM, battery-backed and banked; this is what a `.sav` is |
| `WRAM0` | `$C000-$DFFF` | work RAM — everything the game is currently thinking |
| `HRAM` | `$FF80-$FFFE` | high RAM, 127 bytes reachable one byte faster than anything else |
| `ROM0` | `$0000-$3FFF` | the always-mapped home bank |
| `ROMX` | `$4000-$7FFF` | one of 63 switched banks; where the cartridge's data tables live |

**Every entry's `addr` lies inside the span its `region` claims** — checked by
[`tools/validate.py`](../tools/validate.py) on every push.

`WRAM0` is Gen 1's whole work RAM: the game is a DMG title and never uses the CGB's
banked `WRAMX`.

### `bank`

<a id="bank"></a>The bank number as a decimal string (`0`, `1`, `2`, `3`, …), or `-`
where the region is not banked. Only `SRAM` and `ROMX` are ever banked here; `VRAM`,
`WRAM0`, `HRAM` and `ROM0` always carry `-`.

`SRAM` addresses repeat across banks, so **`(region, bank, addr)` is the identifying
triple**, not `addr` alone. `$A000` in bank 1 and `$A000` in bank 2 are different bytes.

### `addr`

<a id="addr"></a>The CPU address: a `$` followed by exactly four uppercase hex digits —
`$D163`. Parse with `int(value[1:], 16)`.

This is the address **as the CPU sees it**, not a file offset. For a `.sav` file, which
is raw cartridge RAM with bank 0 first and no header, the file offset of an `SRAM` entry
is `bank * 0x2000 + (addr - 0xA000)`.

### `len`

<a id="len"></a>**Bytes**, as a decimal integer. How many bytes this entry owns: the
distance from its address to the next distinct address in the same region and bank, or —
for the last entry in a section — the remainder of that section.

`len` is a *span*, not a type width. `wEventFlags` has a length of `320` because that is
what sits there before the next label — 2,560 one-bit event flags — not because it is one
320-byte value. Where an entry is a repeated structure, the slot-1 row's length is the
stride of one slot: `wPartyMon1Nick` is `11`, and the next eleven bytes are
`wPartyMon2Nick`, an [`instance`](#role).

A `len` of `0` means the entry owns nothing: it is an end marker, a label the disassembly
puts one past the last byte of a run so that the run's size can be written as a
difference. `wPartyDataEnd`, `wMainDataEnd` and `wBoxDataEnd` are the ones that matter —
`wMainDataEnd` in particular is the boundary a save is written from.

### `symbol`

<a id="symbol"></a>The label, spelled exactly as `pret/pokered` spells it — `wPartyCount`,
`hRandomAdd`, `sBox1`. **Unique across the whole file**, which is what lets a verification
run key on it (see [verification.md](verification.md)).

The leading letter is pokered's own convention and is worth knowing: `w` work RAM, `h`
high RAM, `s` cartridge RAM (saved), `v` video RAM. Names are the community's, not ours —
see [provenance.md](provenance.md).

Two `role`s carry a synthetic name instead of a label, because there is no label to carry:
`gap:WRAM00:$DF00` and `free:SRAM1:$B524` — the kind, the region and bank, and the address
the run starts at. They are stable and unique, but they are identifiers, not names anybody
uses, and a consumer displaying them raw is displaying the wrong thing. Render them as
*"reached by no label"* and *"unallocated"*, which is what the pages do.

### `role`

<a id="role"></a>What kind of row this is. One of:

| value | meaning |
|---|---|
| `entry` | the canonical row for this address — the first label declared there |
| `alias` | a second name for the *same bytes*. `wPartyMons`, `wPartyMon1` and `wPartyMon1Species` are one byte under three names |
| `instance` | slot 2..N of a repeated structure. `wBoxMon14Status` is an instance; its slot-1 row `wBoxMon1Status` carries the count and the stride |
| `gap` | bytes inside a declared section that **no label reaches**. Not a hole — a run somebody looked at and found unnamed |
| `free` | a run the linker reported as `EMPTY`: RAM the game declares no storage for at all |

`gap` and `free` are what make the coverage add up, and they are the reason this atlas
can say there are no holes. There is exactly one `gap` in work RAM and it is not an
oversight: the `Stack` section is `$DF00-$DFFF` and its only label sits at `$DFFF`,
because the stack pointer starts at the top and grows *downwards* through all 256 bytes.

**If you are building an index, filter to `role == "entry"`.** If you are resolving a
name a user typed, use every role — a name somebody searched for should be findable
whether or not it is the one this atlas chose as canonical.

### `sect`

<a id="sect"></a>The linker section the symbol was declared in, verbatim from the RGBDS
map file — `"Main Data"`, `"Stack"`, `"Saved Boxes 1"`, `"(unallocated)"` for `free` rows.

It is here because it is a fact about the build and because it is what
[`group`](#group) falls back to. It is *not* a subject grouping: a section is about where
bytes fit, and several of pokered's mix half a dozen unrelated subsystems.

### `group`

<a id="group"></a>Our chapter — the subject the entry belongs to, and the page it is
rendered onto (`docs/<group>.md`). One of:

`player` · `party` · `storage` · `battle` · `bag` · `pokedex` · `events` · `overworld` ·
`sprites` · `screen` · `graphics` · `audio` · `link` · `rng` · `save` · `system` ·
`scratch` · `rom-data` · `misc`

**This column is editorial judgement, and it is the only derived column that is ours
rather than the disassembly's.** It is produced by reviewable rules in
[`tools/chapters.py`](../tools/chapters.py) — first match wins — rather than typed into
2,898 cells, so it can be argued with and re-run. `misc` is an admission, not a category:
it means nobody has given that entry a home yet. The atlas currently has none.

An `alias` and every `instance` are forced into the same chapter as their `entry`, so a
name and the bytes it refers to are never on different pages.

### `verify`

<a id="verify"></a>**The evidence for this entry**, and the reason this atlas exists.
Zero or more tokens, comma separated, no spaces, in this order: `rom,live,inv`. An empty
cell means *no evidence yet* — stated, not hidden.

| token | badge | what it establishes |
|---|:--:|---|
| `rom` | **R** | the address appears in the cartridge image as the operand of an instruction that takes one (`ld [nn],a` is `EA lo hi`; `ldh [n],a` is `E0 n`) |
| `live` | **L** | the byte changed while the cartridge ran a **fixed** button script on a cycle-accurate emulator, from a cold boot with no save |
| `inv` | **I** | a named, hand-written invariant proved the entry means what it says, not merely that it exists |

**These are not interchangeable, and each has a limit worth knowing:**

- `rom` is a byte scan, not a disassembly. Two bytes of graphics data preceded by a byte
  that happens to be `$EA` look exactly like `ld [nn],a`, so a single hit is weak
  evidence. What it *cannot* produce is a false negative: an address the game really does
  load will be found.
- `live` only marks what one script happened to touch. The script plays the opening and
  then walks and opens menus, so it never reaches a battle or a PC. **An unmarked entry
  means "not observed", never "not used".**
- `inv` is the strongest tier and the only one that can catch an address which is real,
  live, and describing the wrong thing — but an invariant is only as strong as the save
  it runs against. The invariants that can pass vacuously say so rather than reporting a
  pass they have not earned.

**You cannot edit this column by hand.** It is written by landing a verification report,
and CI checks that the column is still the one the last landed run produced. See
[verification.md](verification.md) — that is the whole loop, and it is what stops a tier
from quietly becoming a stale claim.

### `desc`

<a id="desc"></a>What the entry is, **in our own words**, as Markdown. May be empty, and
often is: about a third of the distinct addresses carry a written description and the
rest carry their symbol name and their evidence and nothing more, which is the honest
state of them. Every chapter page reports its own figure at the top of its table.

Descriptions are written from scratch. `pret/pokered`'s prose and comments are not ours
to copy — see [provenance.md](provenance.md). Where a description asserts behaviour
rather than restating a name, either an invariant covers it or it is hedged.

---

## Derived forms

Generated from the TSV by [`tools/export.py`](../tools/export.py) and checked in CI, so
they cannot drift:

| file | what it is |
|---|---|
| [`data/atlas.json`](../data/atlas.json) | `{"meta": …, "entries": […]}` — every row as an object, indented, with the types applied |
| [`data/atlas.min.json`](../data/atlas.min.json) | the same, minified, with empty fields dropped |
| [`data/atlas.schema.json`](../data/atlas.schema.json) | a JSON Schema for `atlas.json` |

The JSON rows carry the TSV's columns plus two conveniences:

- `addr_int` — the address as an integer, so a tool can index by it without parsing
  `$D163` (`addr` keeps its `$XXXX` spelling, because that is how every other Gen 1
  document writes it);
- `verify` — a **list** of tokens rather than a comma-joined string.

`len` is an integer. Everything else is a string.

---

## Stable anchors

Every entry has a permanent link, and the anchors are derived from the data rather than
from the position of anything in a file, so they survive regeneration:

| link | goes to |
|---|---|
| `docs/<group>.md#s-<symbol>` | the entry on its chapter page — [`docs/party.md#s-wPartyCount`](party.md#s-wPartyCount) |
| `docs/by-address.md#a-<addr>` | the entry in the address index — [`docs/by-address.md#a-D163`](by-address.md#a-D163) |

`<addr>` is the four hex digits without the `$`, plus ` b<bank>` collapsed to `b<bank>`
where the region is banked (`#a-A000b2`). Every one of those links is checked on every
push by [`tools/checklinks.py`](../tools/checklinks.py); an index of 2,400 anchored links
is exactly the kind of thing that rots silently, and it once did.

---

## What is covered

**Pokémon Blue and Pokémon Red, English (USA/Europe).** The disassembly builds both from
one source and the work RAM map is shared, so every address here is Red's too; the
differences between them are in ROM data — which Pokémon appear where, a handful of
tables — and not in where anything lives.

**Yellow is not covered and is not approximated.** Its work RAM shifted, so an address
from this file is *wrong* there rather than approximately right, and the failure mode is
a write landing in the middle of somebody else's data and passing its own checksum.
Yellow needs its own build, its own extraction and its own verification run.

**Generation 2 is not covered.** Different engine, different map.
