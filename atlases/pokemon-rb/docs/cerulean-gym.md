# Cerulean Gym, worked — Pokémon Red/Blue

[← Pokémon Red/Blue](../README.md) · [by address](by-address.md) · [by name](by-name.md) ·
[discoveries](discoveries.md) · [AtlasGB](../../../README.md)

**One gym, read out of the cartridge rather than recalled: the leader's own team, the room's
layout and its trainers, what beating it writes to the save, the town's mart, and the grass on
the way there.** Nothing on this page is about anything driven from outside the game — it is a
worked example of what this atlas's addresses and structures mean when they are read together
against one place in the cartridge, in the same spirit as [discoveries](discoveries.md) but
without a fault or a corrected belief behind any of it.

---

## Misty's team

Built from `TrainerDataPointers` (bank `0E`, offset `$5D3B`, trainer class 35 — `TrainerNames`'
own 35th record is `MISTY`), with the stats computed the way [every trainer's Pokémon
is](battle.md#s-wEnemyMon1DVs): fixed DVs (`$98`/`$88`), no stat experience.

| | level | HP | Attack | Defense | Speed | Special | moves |
|---|---:|---:|---:|---:|---:|---:|---|
| STARYU | 18 | 41 | 24 | 27 | 38 | 33 | TACKLE, WATER GUN |
| STARMIE | 21 | 59 | 40 | 44 | 56 | 50 | TACKLE, WATER GUN, HARDEN |

**Her whole offence is one 40-power special WATER move.** STARYU is WATER-typed; STARMIE is
PSYCHIC/WATER, so it also takes double damage from BUG — but it learns nothing at all by level,
so both Pokémon carry only the four moves their base-stats record starts them with.

`TypeEffects` (bank `0F`, offset `$6474`) gives the starter lines three different answers to
that WATER move: BULBASAUR's line resists it outright from level 13 (VINE WHIP, GRASS); a
SQUIRTLE resists WATER itself, and by level 24 carries BITE at 60 power against its next-best
40; a CHARMANDER's line never resists it at any level — EMBER is FIRE, which WATER halves, and
the whole line is doubly weak to WATER GUN.

## The room

Map `$41`, 10×14 walk tiles, two warps out at `(4,13)` and `(5,13)`. Three trainers, and the
pools are water:

| at | facing | who |
|---|---|---|
| `(4,2)` | down | opponent `235` = `200 + 35` = **MISTY** |
| `(2,3)` | right | opponent `206` = `200 + 6` = **JR. TRAINER♀**, roster 1: GOLDEEN Lv19 |
| `(8,7)` | left | opponent `215` = `200 + 15` = **SWIMMER**, roster 1: HORSEA Lv16, SHELLDER Lv16 |

```text
   0 ##########
   1 ....##....
   2 .###M.###.     MISTY, facing down
   3 .#J.....#.     the junior trainer, facing right
   4 .######.#.
   5 .#......#.
   6 .#.##.###.
   7 .#.##.##S.     the swimmer, facing left — see below
   8 .#....###.
   9 .###..###.
  10 .###..#G#.     the gym guide
  11 .###..#.#.
  12 ..........
  13 ....WW....     the way in
```

**The junior trainer cannot be avoided, and that is geometry rather than scripting.** Misty's
only free neighbours are `(4,3)` and `(5,2)`; `(5,2)` is reachable only from `(5,3)`, because row
1 is walled at `x=4` and `x=5`; so every route to her crosses row 3, which is the junior
trainer's own line — and `(1,3)` is a wall, so there is no tile on that row to her left to slip
past on. A breadth-first search over the room's walkable tiles with row 3 blocked from `x=3` to
`x=7` finds no path to Misty at all.

**She then parks on `(4,3)`** once beaten — a beaten trainer walks to the tile beside the player
and stays there — so her body ends up standing on the one tile a route along row 3 still needed.
`(5,2)` is her other neighbour, reachable once she has moved.

**The swimmer at `(8,7)` sees through two walls.** [A trainer's sight line does not care about
walls](discoveries.md#a-trainers-sight-line-does-not-care-about-walls) is the general finding
this specific trainer produced, and it is written there rather than repeated here.

## What beating her writes

Snapshotted before and after the fight, every one of the 320 bytes of
[`wEventFlags`](events.md#s-wEventFlags) plus the badge bytes. Exactly three bytes move, and
nothing is cleared:

| address | before | after | what changed |
|---|---:|---:|---|
| [`wObtainedBadges`](player.md#s-wObtainedBadges) `$D356` | `$01` | `$03` | bit 1 — the Cascade Badge |
| `wBeatGymFlags` `$D72A` | `$01` | `$03` | bit 1 |
| `$D75E` (byte 23 of `wEventFlags`) | `$00` | `$CC` | bits 2, 3, 6, 7 |

`$D75E`'s four moved bits decode against `constants/event_constants.asm`'s own `const` list as
`EVENT_BEAT_CERULEAN_GYM_TRAINER_0` (2), `EVENT_BEAT_CERULEAN_GYM_TRAINER_1` (3),
`EVENT_GOT_TM11` (6) and `EVENT_BEAT_MISTY` (7) — the same counting that puts
`EVENT_BEAT_PEWTER_GYM_TRAINER_0`, `EVENT_GOT_TM34` and `EVENT_BEAT_BROCK` on `$D755` bits 2, 6
and 7, and `EVENT_FOLLOWED_OAK_INTO_LAB` on `wEventFlags` bit 0 — three flags this atlas states
elsewhere from independent addresses, so the agreement is a cross-check on the counting rather
than an assumption behind it. Pewter's own gym additionally *clears* two bits of `$D7EB` (it
rearms a rival encounter); Cerulean's clears nothing.

## The Pokémon Center's PP refill

A party's move counters, read before and after the nurse's counter: TACKLE `$14 → $23` (20 of 35
PP to 35 of 35), VINE WHIP `$00 → $0A` (0 of 10 to 10 of 10), HP full both times. **`HealParty`
refills PP as well as HP.** "The party is whole" is therefore a weaker statement than it looks —
VINE WHIP's whole pool is 10 PP, and this gym alone puts five WATER-typed Pokémon in front of it.

## The mart

Cerulean's is mart 2 in the cartridge's own list (file offset `$02442`): POKE_BALL, POTION,
REPEL, ANTIDOTE, BURN_HEAL, AWAKENING, PARLYZ_HEAL — the same seven medicines Pewter's mart
stocks, with a REPEL standing where Pewter's ESCAPE_ROPE does, and no SUPER_POTION (that is
Vermilion's). [The bag](bag.md) has Viridian's list, the first the player can reach; later marts
including these two are not covered there.

## The grass on Route 4

Sixty walkable tiles at `x=64-73, y=10-15` on map `$0F`, the tileset's own grass tile `$52`. It
is a closed room with one door: walled at row 9 and row 16 and at `x=62` and `x=75`, entered
along row 14 through `x=75-77`, with column 77 the only tile joining rows 12 and 14 without a
ledge (row 13 is solid at `x=75, 76, 78, 79`). No trainer stands in it.

Experience per step, from `WildDataPointers` and each species' own base-experience byte:

| grass | map | mean exp/encounter | encounter rate | exp per step |
|---|---|---:|---:|---:|
| Route 4, `x=64-73` | `$0F` | 96.6 | 20/256 | **7.55** |
| Route 5 | `$10` | 138.3 | 15/256 | 8.10 |
| Route 24 | `$23` | 111.1 | 25/256 | **10.85** |
| Route 25 | `$24` | 108.1 | 15/256 | 6.33 |
| Route 3 | `$0E` | 54.0 | 20/256 | 4.22 |
| Mt Moon B2F | `$3D` | 98.7 | 10/256 | 3.86 |

The two richer patches each cost something Route 4's does not: Route 5's grass sits below a
one-way ledge descent — `LedgeTiles` entries at rows 3, 7, 11 and 15 mean three patches can be
dropped into and never climbed back out of — and Route 24's sits on the far side of Nugget
Bridge, past five trainers in a column and a Team Rocket grunt at the top, on a two-tile-wide
bridge.

---

## See also

- [Catching](catching.md) — the odds, and the wild-versus-trainer signal, that this gym's own
  trainers do not change.
- [Learning moves](party.md#findings-behind-these-bytes) — every level-up prompt the game can
  ask, and the rule this atlas states for answering one.
- [Discoveries](discoveries.md) — the trainer-sight-line finding this gym's swimmer produced,
  and the rest of what this atlas found out about the cartridge.
