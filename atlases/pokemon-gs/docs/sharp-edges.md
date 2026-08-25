# Sharp edges — Pokémon Gold and Silver

[← Gold/Silver](../README.md) · [AtlasGB](../../../README.md)

Traps in the cartridge itself, as far as they are known. Several are shared with
[Pokémon Red/Blue's own sharp edges](../../pokemon-rb/docs/sharp-edges.md); two are **inverted**
between the generations, and those are marked, because carrying a Generation 1 habit into
Generation 2 is the specific way to get hurt by them.

## Addresses are only exact for the cartridge a build reproduces

A symbol's address is a claim about a specific build — for a `pokegold` disassembly, byte for
byte against `roms.sha1`. **A Gold symbol is not a Silver symbol and is not a Crystal symbol.**
Nothing on these pages verifies a Generation 2 address against a running cartridge the way
[Pokémon Red/Blue's own tooling](../../pokemon-rb/README.md) does — that gap is the highest-value
piece of work missing from this whole area.

## The save

**A Generation 2 save is three WRAM runs into three SRAM runs**, not one contiguous block —
mapping an address means knowing which of the three runs it falls in, and an address outside
all three is not a valid saved address at all. See [the save file](the-save-file.md).

**The checksum is a 16-bit word sum, little-endian, and a stale one is rejected outright.**
Generation 1's is a complemented 8-bit sum. Carrying either generation's algorithm into the
other produces a file the game refuses to load.

**⚠️ Inverted from Generation 1.** Generation 1's save cannot be used to teleport the player
because `LoadSAV` skips the map-header load on continue. **Generation 2 refuses for a related
but distinct reason**: `LoadMapAttributes_SkipObjects` — a continue does not reload the map's
*objects*, on the assumption the save was made on the map it names. Editing the map identity in
the save lands the player on the right map with the previous map's people still standing there.
See [the save file](the-save-file.md#you-cannot-teleport-the-player-by-editing-the-save).

## The engine

**⚠️ Inverted from Generation 1: dialogs are dismissed with A, never B.** B backs a Generation 2
dialog out through a continue-style menu rather than closing it; Generation 1's B is the *safe*
dismissal, because it closes a menu and can never open one. See
[reading the screen](reading-the-screen.md#dismissing-a-dialog-is-inverted-from-generation-1).

**The map identity is loaded into WRAM before the continue panel is dismissed**, and it is
noise — not a stable value — until then. The same trap exists in Generation 1, where `wCurMap`
is populated at the top of the main menu before CONTINUE is chosen.

**The overworld renders no letters at all**, which is a stronger test for "does a menu or
dialog currently own the screen" than checking for any particular expected phrase — see
[reading the screen](reading-the-screen.md#the-overworld-renders-no-letters-at-all).

**The menu cursor's position is not reliably held in the variable that looks like it should
hold it.** See [reading the screen](reading-the-screen.md#the-menu-cursor-is-a-tile-not-a-variable).

## Linking

**⚠️ `hSerialConnectionStatus` is a different byte from Generation 1's.** `$FFCD` on Generation
2, `$FFAA` on Generation 1 — same name, same purpose, unrelated value if the wrong one is read.

**Which Cable Club seat is free is decided by that byte, not by geometry** — see
[the Time Capsule](time-capsule.md#which-seat-is-yours-is-a-byte-not-geometry).

**A Generation 2 console answers a link handshake from its overworld serial handler.** A
Generation 1 partner that initiates while the Generation 2 side is still walking around, rather
than already at the relevant receptionist, completes a handshake against nothing useful and
desyncs. The window during which the Generation 2 side is ready is a fixed number of frames —
see [the Time Capsule](time-capsule.md#the-link-window).

**Generation 2 refuses a Generation 1 partner at its ordinary trade desk outright.** The Time
Capsule desk is the only sanctioned door between the generations — see
[the Time Capsule](time-capsule.md).

## Reading and writing Pokémon

**⚠️ The Generation 2 party structure is 48 bytes with level at offset 31; Generation 1's is 44
bytes with level at offset 33.** Porting a reader between generations without moving that
offset is the classic way to produce a nonsense Pokémon. See
[the memory map](memory-map.md#the-48-byte-party-structure-as-far-as-this-page-establishes-it).

**A held item exists at offset 1, and Special splits into Special Attack and Special Defense** —
neither concept exists in Generation 1 at all.

**The Time Capsule rejects a whole party for one incompatible member, not just that member.**
Any Generation-2-only species, any move introduced after Generation 1, or any held mail refuses
the entire trade. All three Johto starters are Generation 2 species, so a normally-played Gold
save is very likely to be refused on its party even with both gate conditions already open —
see [the Time Capsule](time-capsule.md#the-in-game-prerequisites).

**A Pokémon arriving from Generation 1 holds an item whose id equals its Generation 1 catch
rate.** Documented Time Capsule behaviour — the two structures place different concepts at the
same offset — not a conversion bug. See [the Time Capsule](time-capsule.md#quirks-and-honest-gaps).

**Generation 1 has no shiny Pokémon.** A DV-derived verdict about a Generation 1 Pokémon is
always "would be shiny if traded to Generation 2", never "is shiny".

## What has not bitten yet, because nobody has been there

Stated so an empty section does not read as a safe one: nothing behind these pages starts,
reads or scores a Generation 2 **battle**; nothing touches the **box** or the **bag**; nothing
has read Generation 2's **RNG**, so nothing this project says about Generation 1's `rDIV` and
reproducibility applies to it; and the **map system** proper (headers, tilesets, warp tables)
is not established here at all.

## See also

- [Pokémon Red/Blue's own sharp edges](../../pokemon-rb/docs/sharp-edges.md) — the shared and
  contrasting traps for the cartridge this project's evidence pipeline actually covers.
