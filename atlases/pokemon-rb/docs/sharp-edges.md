# Sharp edges — Pokémon Red/Blue

[← Pokémon Red/Blue](../README.md) · [by address](by-address.md) · [by name](by-name.md) · [discoveries](discoveries.md) · [AtlasGB](../../../README.md)

**This page is the warning; [`discoveries.md`](discoveries.md) is the reasoning.** Everything
below is a trap a reader of this atlas can walk into — a byte that means something other than
its name suggests, a table indexed a way that looks safe and isn't, a mechanic the community
states one way and the cartridge states another. Where the trap is a fault or a corrected belief
that already has a full write-up — the messy symptom, the wrong turns, the evidence — this page
gives one line and a link rather than the story again. Grouped by what you were reading, writing
or watching when it would bite.

---

## Reading state

**Species indices are not Pokédex numbers.** [`MonsterNames`](rom-data.md#s-MonsterNames) is
indexed by *internal* index, [`BaseStats`](rom-data.md#s-BaseStats) by *Pokédex* number, and
[`PokedexOrder`](rom-data.md#s-PokedexOrder) is the only safe way to convert between them —
`GetMonHeader` is the game's own conversion routine, and it special-cases Mew because Mew's base
stats sit outside the main table in Red and Blue. Reading one table with the other's index gives
a plausible, wrong Pokémon: PARAS is internal index 109 and Pokédex 46. See
[the party](party.md) and [rom-data](rom-data.md).

**Every multi-byte stat is big-endian**, on a little-endian console — HP, stats, stat experience,
money, trainer ID. Read one the wrong way round and a Pokémon's level reads as 6,400. See
[the party](party.md#s-wPartyMon1HP).

**The party structure and the battle structure are not the same shape.** `party_struct` is 44
bytes; `battle_struct` is 29 and drops original trainer, experience and stat experience, so level
and the five stats sit at different offsets in the two. See [the battle](battle.md#s-wBattleMon).

**`wIsInBattle == $FF` means the player blacked out, not "trainer battle."** A catch-all arm that
reads any non-zero value as a trainer battle draws a stale battle panel in the overworld with the
same confidence as live data. [Discoveries](discoveries.md#a-documented-sentinel-read-as-anything-else) ·
[`wIsInBattle`](battle.md#s-wIsInBattle).

**`wEnemyMons` shares storage with the map's wild-encounter tables.** Reading it outside a
trainer battle returns encounter data, not an opposing team — check
[`wIsInBattle`](battle.md#s-wIsInBattle) first. See [the battle](battle.md#s-wEnemyMon1Species).

**A trainer's Pokémon have fixed DVs.** `LoadEnemyMonData` writes the same
`ATKDEFDV_TRAINER`/`SPDSPCDV_TRAINER` nybbles into every trainer Pokémon, so anything that reads
DVs and draws a conclusion from them has to exclude trainer battles or it reports the same answer
forever. See [the battle](battle.md#s-wEnemyMon1DVs).

---

## Writing state

**Well-formed is not the same as accepted.** `CheckForDisobedience` compares a party entry's OT
ID against [`wPlayerID`](player.md#s-wPlayerID) and treats a mismatch as a *traded* Pokémon,
obeyed only up to a level the player's badges permit — so an injected Pokémon that passes every
checksum can still answer "won't obey!" and waste the turn. See [who you are](player.md).

**You cannot move the player by editing `wCurMap` alone.** It is saved state, restored together
with the map header below it; writing a different map id without the matching header makes the
engine read the new map's blocks through the old map's pointers — observed as an illegal opcode
within a second. Redirecting one of the room's own `wWarpEntries` and letting the engine load the
destination itself is the only path that works.
[Discoveries](discoveries.md#wcurmap-is-not-a-loaded-map) · [the map](overworld.md#s-wCurMap).

**A battle started with an empty party is an instant black-out**, and it looks exactly like a
crash rather than a loss. See [`wPartyCount`](party.md#s-wPartyCount).

---

## The engine

**`wCurOpponent` is polled only by the overworld loop.** Set it while a menu or a text box is
open and nothing happens until the player regains control — the byte just sits there armed.
`NewBattle` also refuses to start on a dungeon warp, while a cutscene is moving the player, and
on a map whose script has set `BIT_NO_BATTLES`. See [the battle](battle.md).

**A and B are not interchangeable, and the asymmetry is the trap.** Both advance a text box, but
**B also closes a menu and can never open one** — a B press against an open FIGHT menu backs out
to the menu above it, invisibly. And an A press one frame after a text box closes opens the START
menu, which then absorbs every direction key until it is closed again.

**Gen 1's menu code does not register a button that arrives too soon.** `HandleMenuInput` delays
several frames after accepting one press before it will see the next, so two presses close
together look, from outside, exactly like a dropped input. See [the screen](screen.md).

**Fleeing a battle leaves the "Got away safely!" box open**, and an open text box swallows every
direction key — the overworld does not move again until it is dismissed.

**A directional press only turns the player if they are not already facing that way.** The first
press of a new direction changes facing without moving a tile; only the next press in the same
direction actually steps.

**A tileset's collision list is not the whole of its walkability.** Beside the ordinary
`$FF`-terminated per-tile list ([`wTilesetCollisionPtr`](overworld.md#s-wTilesetCollisionPtr)), Gen 1 keeps a
second table, `TilePairCollisionsLand`, of three-byte `(tileset, tile, tile)` records naming
pairs of tiles that are **each walkable on their own** and that `CheckForTilePairCollisions`
still refuses to let the player step *between*. Mt Moon 1F's one-tile corridor is exactly this
shape: the corridor tile and the floor tile beside it are both in tileset 17's ordinary walkable
list, and the pair is blocked anyway. A walkability grid built from the per-tile list alone
routes straight through a wall that the cartridge does not.

**Oak's Lab can lock the cartridge up permanently between the starter speech and taking a
starter**, if the player walks into the wrong two rows of the room.
[Discoveries: the Oak's Lab soft-lock](discoveries.md#the-oaks-lab-soft-lock).

**`wCurMap` is populated at the top of the main menu**, by `TryLoadSaveFile`, long before CONTINUE
is chosen — it cannot be read as "the game is in progress."
[Discoveries](discoveries.md#wcurmap-is-not-a-loaded-map) · [the map](overworld.md#s-wCurMap).

**Cancelling an evolution needs B held through the whole animation, not tapped once.** Gen 1
reads some inputs through a routine that only sees a button transition, so a single tap is long
gone by the time the animation checks for it, and the Pokémon evolves anyway.
[Discoveries](discoveries.md#cancelling-an-evolution-costs-one-level-and-nothing-else) ·
[the party](party.md).

**A battle that crosses two level boundaries loses the move that belonged to the level in
between.** The game announces one level-up per battle and checks the learnset for the level it
*ended* on, never for a level only passed through.
[Discoveries](discoveries.md#crossing-two-levels-loses-the-move-in-between) · [the party](party.md).

**Setting `wEventFlags` bit 0 skips the whole introduction.** `PalletTownDefaultScript` checks
`EVENT_FOLLOWED_OAK_INTO_LAB` before it lets the player walk north out of Pallet Town, so that one
bit gates the starter choice and the rival battle behind it entirely. See [the story](events.md).

---

## Reading the screen

**Gen 1 draws the whole screen through the WINDOW layer, not the background map.** The BG tilemap
at `$9800` reads back as 20×18 spaces; the screen a player actually sees is
[`wTileMap`](screen.md#s-wTileMap) in WRAM. See [the screen](screen.md).

**A decoded screen is not a fixed-length string.** Some tiles expand to whole words — `$4A` is
`PKMN`, `$54` is `POKé`, `$5D` is `TRAINER` — so 360 tiles are not 360 characters and slicing by
byte index panics on a multi-character tile. See [the screen](screen.md).

**Rows are padded to twenty columns and a message can wrap across them**, so matching text
against a decoded screen means collapsing runs of whitespace first. See [the screen](screen.md).

---

## Files and saves

**A `.sav` is raw cartridge RAM with no header.** A file offset is a straight offset into cartridge
SRAM, and bank 1 begins at file offset `$2000`. See [the save](save.md).

**A hand-built save can hang the game.** Map, tileset, sprite and warp state are consistent state
the engine maintains together, and the saved block carries a checksum over it —
[`sMainDataCheckSum`](save.md#s-sMainDataCheckSum), computed by `CalcCheckSum` — so a save edited
outside the game's own SAVE routine has to recompute the checksum or the game treats it as
corrupt.

---

## Timing and determinism

**`BattleRandom` folds the hardware divider register into its seed on every call**, so the exact
species and DVs a scripted sequence produces are not reproducible from a starting seed alone —
only a whole run, replayed bit-identically from the same starting state, is. See
[the dice](rng.md).

**Even a move with a "100%" accuracy byte misses about once in 256.** `MoveHitTest` compares a
random byte against the (possibly stage-scaled) accuracy value; when the accuracy byte is at its
maximum the comparison still fails for one value the random byte can take, so nothing in the
cartridge is truly unmissable. This is separate from — and stacks with — the stage scaling
covered in
[discoveries: accuracy is not the number in the move table](discoveries.md#accuracy-is-not-the-number-in-the-move-table).

---

## See also

- [Discoveries](discoveries.md) — the full reasoning behind every entry above that links here:
  the symptom, the wrong turns, the citation and the evidence.
- [The party](party.md), [the battle](battle.md), [the map you are standing on](overworld.md),
  [the screen, and why it reads back as text](screen.md), [who you are](player.md),
  [the story](events.md), [the save](save.md), [the dice](rng.md) — the chapter pages these edges
  are drawn from, each with the full address table behind it.
