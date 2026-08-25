# The save file — Pokémon Gold and Silver

[← Gold/Silver](../README.md) · [AtlasGB](../../../README.md)

A `.sav` is raw cartridge RAM, bank 0 first, no header, so bank *N* starts at file offset
*N* × `$2000` and a bank/address pair *is* a file offset — the same rule
[Pokémon Red/Blue's own save](../../pokemon-rb/docs/save.md) follows. Everything past that is
different: Generation 1 saves one contiguous block behind a complemented 8-bit checksum;
Generation 2 saves **three separate runs**, checksums them with a 16-bit word sum, and keeps a
second copy.

## Three runs, not one base plus an offset

Three WRAM regions are copied into three SRAM regions independently. Mapping a WRAM address to
a save offset is therefore defined **run by run**:

| WRAM run | WRAM range | SRAM destination | bank 1 offset |
|---|---|---|---|
| `wPlayerData` | `$D1A1`–`$D9ED` | `sPlayerData` (= `sGameData`) | `$009` |
| `wCurMapData` | `$D9EE`–`$DA21` | `sCurMapData` | `$856` |
| `wPokemonData` | `$DA22`–`$DF00` | `sPokemonData` | `$88A` |

The map closes exactly, which is the proof it is right rather than merely plausible: the first
saved byte lands on `sGameData`, a known symbol, and the last saved byte lands one position
before `sGameDataEnd` / `sChecksum`, also a known symbol — 3,424 bytes of WRAM into 3,424 bytes
of SRAM, with neither endpoint chosen to make the arithmetic work.

**On this cartridge the three runs happen to be contiguous on both sides**, so the run-by-run
mapping coincides with a single base plus offset — `$D9EE − $D1A1 = $84D`, and
`$009 + $84D = $856`; the same check holds for the second boundary. That is a fact about
*today's* numbers, not a licence to treat the save as one contiguous block: a future cartridge,
revision or generation that moves one of the three runs would make the three-run form correct
and the shortcut silently wrong, in a way the checksum cannot catch — it covers the whole
region and does not care which byte inside it moved.

## The checksum is verified, not tolerated

A 16-bit sum of every byte of `sGameData`, stored little-endian at `sChecksum` (bank 1
`$AD69`). Edit anything inside the saved region without recomputing it and the game rejects the
save outright — not a soft warning, a refusal to load. Generation 1 uses a *complemented 8-bit*
sum instead; carrying either algorithm's assumptions into the other generation produces a file
that will not load.

**A second copy exists and is not exercised here.** Gold keeps a backup save with its own
checksum at `sBackupChecksum` (bank 3 `$BE6D`). A save edited to touch only `sGameData` and its
own checksum loads without the backup ever being written — so *when* the backup is consulted,
and whether a stale one can cause trouble later, is not established by this page.

## You cannot teleport the player by editing the save

Both generations refuse this, for different reasons, and the Generation 2 reason is the more
surprising one: **a continue deliberately does not reload the map's objects**, on the
assumption that the save was made on the map it names. Editing the map identity in the save
lands the player on the right map with the *previous* map's people — nobody to talk to, a
stale camera, a room that draws correctly and does nothing. Moving the player's own position
inside the save does not fix this either; whatever else the object load would have set up stays
unset.

**The route that works is the same one Generation 1 uses**: override the destination of a real,
in-game warp at the moment the player steps onto it, and let the engine perform its own full
warp path — objects, camera and scene scripts included, because as far as the engine is
concerned an ordinary warp fired. The write has to land in the instruction-wide window between
the destination being read and being consumed; a per-frame write loses that race roughly half
the time. [Pokémon Red/Blue's own map header](../../pokemon-rb/docs/overworld.md#the-room-format)
states the identical principle for the earlier cartridge: move the world's pointers, not the
player.

## See also

- [the memory map](memory-map.md) — where the runs and the checksum symbols live.
- [Pokémon Red/Blue's own save](../../pokemon-rb/docs/save.md) — the equivalent page for the
  cartridge this project's evidence pipeline actually covers.
