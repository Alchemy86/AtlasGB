# Where Pokémon Gold keeps what is known so far

[← Gold/Silver](../README.md) · [AtlasGB](../../../README.md)

Every address below is for **English Pokémon Gold**, derived from a `pokegold` disassembly
built and checked byte-for-byte against its own `roms.sha1`. This page is deliberately narrow —
it states what has actually been derived or measured, and lists what has not, by name, so an
absent structure reads as *absent* rather than as *simple*. See [the front page](../README.md)
for why this is prose rather than a generated table.

---

## The player and the party

| symbol | address | size | how it is known |
|---|---|---|---|
| `wPlayerID` | `$D1A1` | 2, big-endian | walked from `wPlayerName − 2`, landing on `wPlayerData`'s own first byte — see [the save file](the-save-file.md) |
| `wPlayerName` | `$D1A3` | 11 | disassembly |
| `wPartyCount` | `$DA22` | 1 | disassembly + measured |
| `wPartySpecies` | `$DA23` | 6, `$FF`-terminated | disassembly + measured |
| `wPartyMons` | `$DA2A` | 6 × 48 | disassembly + measured |
| `wPartyMonOTs` | `$DB4A` | 6 × 11 | disassembly |
| `wPartyMonNicks` | `$DB8C` | 6 × 11 | disassembly + measured |

The party structure is **48 bytes**, not Generation 1's 44 — a constant read out of the
assembler rather than hand-counted. Three of the addresses above check each other:
`wPartyMons + 6×48 = wPartyMonOTs`, and `wPartyMonOTs + 6×11 = wPartyMonNicks`, both exactly.

### The 48-byte party structure, as far as this page establishes it

This is not a transcription of the disassembly's own struct macro — it is the set of offsets
that have actually been exercised by a real cartridge-to-cartridge trade, so what is listed is
measured and what is missing is honestly missing rather than filled in from memory of a
different game:

```text
 0  Species        1     Gen 2 dex number — NOT Gen 1's internal index
 1  Item           1     held item; 0, or the Time Capsule refuses the party
 2  Moves          4
 6  OTID           2     big-endian
 8  Exp            3     big-endian
11  HPExp          2 |
13  AttackExp      2 |
15  DefenseExp     2 |   stat experience, big-endian
17  SpeedExp       2 |
19  SpecialExp     2 |
21  DVs            2     4 nybbles: attack, defense, speed, special
23  PP             4
27  Happiness      1
28  ...            3     not established here
31  Level          1
32  ...            2     not established here
34  HP             2 |
36  MaxHP          2 |
38  Attack         2 |
40  Defense        2 |   big-endian
42  Speed          2 |
44  SpclAtk        2 |
46  SpclDef        2 |
```

**Four differences from Generation 1's 44-byte structure matter to anything that touches
both, and all four are visible above:**

1. **A held item exists**, at offset 1 — a concept Generation 1 does not have at all.
2. **Special splits** into Special Attack and Special Defense, four bytes longer even though
   nothing is dropped.
3. **Happiness exists**, at offset 27 — another Generation 2 concept.
4. **Level moves to offset 31**, not 33. This is the classic way to build a nonsense Pokémon
   when porting code between generations — the same class of mistake as confusing Generation
   1's own party and battle structures.

Multi-byte fields are big-endian in both generations, on a little-endian console. The DV pair
at offset 21 is the field the whole Time Capsule chain exists to follow — it crosses the wire
unchanged and is what [the shiny rule](time-capsule.md#the-shiny-rule) reads.

---

## The map, and where the player is

| symbol | address | notes |
|---|---|---|
| `wMapGroup` | `$DA00` | **a map is `(group, number)` in Gen 2, not one id** |
| `wMapNumber` | `$DA01` | |
| `wYCoord` | `$DA02` | |
| `wXCoord` | `$DA03` | |
| `wObjectStructs` (= `wPlayerStruct`) | `$D1FD` | the player's own object; map Y at offset 16, map X at offset 17 |
| `wMapObjects` | `$D445` | the player is object 0; map Y at offset 2, map X at offset 3 |
| `wNextWarp` | `$D043` | +1 the destination map group, +2 the destination map number |

**`wNextWarp` is runtime-only — it is not part of the saved state, and overwriting it is how a
scripted tool moves the player without touching the save.** The routine that reads it back does
so a handful of instructions after loading it, which makes the write window instruction-wide
rather than frame-wide; a per-frame write loses that race roughly half the time. This is
Generation 2's version of the same principle
[Pokémon Red/Blue's own map header](../../pokemon-rb/docs/overworld.md) rests on: move the world's
pointers, not the player, and let the engine's own warp path do the rest of the work.

Pokémon Center 2F is one shared map for every Center in the game, 8×4 blocks. Its Time
Capsule room holds two consoles, at (4,4) and (5,4), each a direction-restricted event — one
fires only facing right, the other only facing left — so the two player seats are (3,4) facing
right and (6,4) facing left. Which seat is actually free is decided by a byte, not by geometry
— see [the Time Capsule](time-capsule.md).

---

## Flags

| symbol | address | notes |
|---|---|---|
| `wEventFlags` | `$D7B7` | 2,048 bits, 256 bytes |
| `wDailyFlags1` | `$D968` | zeroed once a day has elapsed on the cartridge clock |

`EVENT_MET_BILL` is bit 2 of `wEventFlags` byte 226; `DAILYFLAGS1_TIME_CAPSULE_F` is bit 3 of
`wDailyFlags1`. Both are read the "wrong way round" from what their names suggest — see
[the Time Capsule](time-capsule.md).

---

## The link

| symbol | address | notes |
|---|---|---|
| `hSerialConnectionStatus` | `$FFCD` | **Generation 1's is `$FFAA` — a different byte with the same name and purpose** |

The clock-role values are shared with Generation 1: `$01` is the external clock, `$02` is the
internal clock. Which Cable Club seat is free is read from this byte, not guessed from an
arrival coordinate — see [the Time Capsule](time-capsule.md).

The link timeout is **767 frames** (`$2FF`), about 13 seconds — a value read out of the
disassembly, though its address is not established here.

---

## The screen

| symbol | address | size |
|---|---|---|
| `wTilemap` | `$C3A0` | 20×18 |

The same address as Generation 1's `wTileMap`, and the same property: a tile id is a character
code, so the buffer reads back as the text a player can see. See
[reading the screen](reading-the-screen.md) for what differs from Generation 1. The menu
cursor's own variable is deliberately not published here — see that page for why.

---

## The save file

Cartridge RAM, banked, 8 KiB per bank. A `.sav` is raw cartridge RAM, bank 0 first, no header,
so bank *N* starts at file offset *N* × `$2000`.

| symbol | address | notes |
|---|---|---|
| `sGameData` (= `sPlayerData`) | bank 1 `$A009` | |
| `sCurMapData` | bank 1 `$A856` | |
| `sPokemonData` | bank 1 `$A88A` | |
| `sGameDataEnd` / `sChecksum` | bank 1 `$AD69` | |
| `sBackupChecksum` | bank 3 `$BE6D` | disassembly only — not exercised here |

The three-run mapping, the checksum, and why editing the save cannot move the player are
[the save file](the-save-file.md).

---

## What is not on this page

Named, so a gap reads as a gap rather than as an oversight:

* **Battle structures.** No Generation 2 equivalent of `wBattleMon` / `wEnemyMon` /
  `wIsInBattle` / `wCurOpponent` is established here. Nothing behind this page starts, reads or
  scores a Generation 2 battle.
* **Storage and the bag.** Boxes and the item list are untouched.
* **The map system proper** — the map header format, tileset pointers, the block-map buffer,
  the collision table, and the warp/sign/object tables. [Pokémon Red/Blue's own room
  format](../../pokemon-rb/docs/overworld.md#the-room-format) is the closest existing model of the
  shape this would take; nobody has done the Generation 2 derivation.
* **The RNG.** Generation 2's `Random` is not read here. Do not assume Generation 1's
  behaviour — [everything this project says about `rDIV`](../../pokemon-rb/docs/rng.md) is a
  Generation 1 claim.
* **Wild encounter tables**, and whatever Generation 2 does about DVs at capture.

---

## See also

- [Pokémon Red/Blue's own memory map](../../pokemon-rb/README.md) — the equivalent page for the
  cartridge this project's evidence pipeline actually covers, with every address behind an
  evidence tier.
- [Adding an atlas](../../../docs/adding-an-atlas.md) — what turning this page into a real,
  evidenced atlas would take.
