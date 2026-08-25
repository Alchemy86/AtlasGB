# Catching — Pokémon Red/Blue

[← Pokémon Red/Blue](../README.md) · [by address](by-address.md) · [by name](by-name.md) · [discoveries](discoveries.md) · [AtlasGB](../../../README.md)

A ball throw in Pokémon Red and Blue is not the pass/fail roll a modern formula assumes it to
be. `ItemUsePokeBall` (`engine/items/item_effects.asm`) resamples rather than rejects, subtracts
a status bonus that can settle the whole question before the "real" check ever runs, and decides
the shake animation only after the outcome is already fixed. This page disassembles that routine,
checks its closed form against 251 throws on a retail cartridge, and works through how a hunter
would choose what to throw a ball at in the first place. [`battle.md`](battle.md#catching)
carries the short version, folded into the wider battle chapter; this page is the full one.

## The catch check

The bag starts empty. Before any of what follows applies, a ball has to exist in it, and the
cartridge hands over the first five for free rather than making the player buy them: a script
gated behind `wEventFlags`+4 bit 4 — [`wEventFlags`](events.md#s-wEventFlags) at `$D747`,
so byte 4 is `$D74B` — the event named `EVENT_GOT_POKEBALLS_FROM_OAK`, immediately below
`EVENT_GOT_POKEDEX` at bit 5 of the same byte. The script checks the bit before doing anything
else, so replaying the scene a second time hands out nothing. Decoded from this cartridge's own
text engine, the line Oak delivers when the script fires reads:

> OAK: You can't get detailed data on POKéMON by just seeing them. You must catch them!

and the game then calls `GiveItem` for five `POKE_BALL`s. A player who skips or has already
passed that scene can only buy one, and the price table, the mart that stocks it first, and why
that mart is the only one before Pewter with no POTION are covered in [the bag](bag.md#the-first-poké-ball)
rather than repeated here.

Once a ball is in hand and thrown, `ItemUsePokeBall` runs a check that reads nothing like a
single roll:

- **A Great or Ultra Ball's check is a resampling loop, not a threshold you can fail.** The
  routine draws a random byte and compares it against the ball's cutoff — **200** for a Great
  Ball, **150** for an Ultra Ball or a Safari Ball. A draw over the cutoff does not end the
  attempt; it jumps back and draws again. The ball's whole effect is to bias the draw toward low
  values by discarding high ones, not to reject the throw outright — read as a pass/fail hurdle,
  the reading a modern formula invites, every Great and Ultra Ball's real odds are understated.
- **A status condition is subtracted from the roll before the main check runs, and can catch the
  Pokémon outright.** Sleep or freeze is worth 25; any other status condition is worth 12. If the
  random byte the routine draws comes in under that bonus, the Pokémon is caught with no further
  check at all — the bonus is not a nudge added to some other formula, it is itself a chance to
  end the attempt before the ball-cutoff and catch-rate arithmetic is ever reached.
- **The catch rate the routine actually reads is not always the species' base rate.** [`wEnemyMonActualCatchRate`](battle.md#s-wEnemyMonActualCatchRate)
  holds the value `ItemUsePokeBall` consults, and it diverges from the species' own
  [`wEnemyMonCatchRate`](battle.md#s-wEnemyMonCatchRate) exactly when Safari bait or rocks have
  moved it during the encounter.
- **The shake animation is decided after the outcome, not before it.** Whatever produces the
  one-, two- or three-shake wobble on screen runs only once capture or failure has already been
  settled inside the routine. Three shakes followed by the ball popping open is not a near miss —
  the miss, if there was one, happened earlier, and the animation is reporting a verdict rather
  than building suspense toward one.

A caught Pokémon's starting moveset is decided by none of this — it comes from the species'
own base-stats record and its level, the same as any other Pokémon added to the party. See
[the party page](party.md) for `WriteMonMoves` and how a species' `MOVE1`..`MOVE4` entry seeds
it.

## Measured against the retail cartridge

Turned into a closed form and swept across HP, status and ball type against a real cartridge,
the routine's predicted odds and the cartridge's own outcomes were compared bucket by bucket
over 251 throws:

| predicted | n | caught | observed | expected |
|---|---:|---:|---:|---:|
| 0–10 % | 41 | 3 | 7.3 % | 3.1 % |
| 10–20 % | 18 | 6 | 33.3 % | 13.9 % |
| 20–30 % | 2 | 0 | 0.0 % | 27.2 % |
| 30–40 % | 13 | 2 | 15.4 % | 37.0 % |
| 40–50 % | 15 | 5 | 33.3 % | 45.5 % |
| 50–60 % | 29 | 17 | 58.6 % | 53.8 % |
| 60–70 % | 9 | 5 | 55.6 % | 64.8 % |
| 70–80 % | 40 | 32 | 80.0 % | 74.9 % |
| 80–90 % | 5 | 4 | 80.0 % | 82.4 % |
| 90–100 % | 79 | 79 | 100.0 % | 99.6 % |
| **all** | **251** | **153** | **61.0 %** | **59.8 %** |

**z = +0.36** against a binomial standard error of 3.09 %: the disassembled closed form and the
cartridge agree. Individual buckets are noisy on purpose — 6 of 18 in the 10–20 % row, 2 of 13 in
the 30–40 % row — because a bucket that small is *supposed* to wander around its expected value;
the aggregate across all 251 throws is the claim being tested, and the buckets are shown anyway
because an aggregate can land right while every row under it is off, and only the full breakdown
shows that this one did not.

## Wild and trainer battles are the same picture

A ball thrown at a trainer's Pokémon is refused outright, so knowing which kind of battle is
running matters before a ball ever leaves the player's hand — and the battle screen gives almost
nothing away. Traced frame by frame, the wild-battle banner (`Wild <NAME> appeared!`) is on
screen for frames 329–337, nine frames; the trainer banner (`<NAME> wants to fight!`) is on
screen for frames 206–216, eleven frames. Outside those windows the two screens are
indistinguishable down to the pixel. [`battle.md`](battle.md#catching) has the fuller result —
the frame-by-frame comparison over a matched pair of battles, and what a ball thrown at a
trainer actually costs.

## A worked example: choosing a target

None of the arithmetic above says what is worth throwing a ball at, and a cartridge fact answers
part of that too. A gym is a type matchup as much as it is a catch rate, and the two worked
examples below are the same road: Cerulean's gym, [worked in full here](cerulean-gym.md),
against a bench built out of an early catch rather than a starter.

**PARAS is BUG/GRASS**, and that typing is worth more against Misty's STARMIE than it looks on
paper: `TypeEffects` gives a WATER attack ×1.0 against BUG and ×0.5 against GRASS, so PARAS'
GRASS half halves STARMIE's own same-type WATER GUN — coverage neither a pure water-type nor an
electric bench gets against the same attack. PARAS evolves into PARASECT at level 24, and the
level-24 threshold happens to be the same level the Squirtle line's own training already trains
to, so one training leg serves both. PARAS is read out of `WildDataPointers` on three floors of
Mt Moon:

| map | rate | PARAS slots | level |
|---|---:|---:|---:|
| Mt Moon 1F | 10/256 | 1/10 | 8 |
| Mt Moon B1F | 10/256 | 1/10 | 10 |
| Mt Moon B2F | 10/256 | 2/10 | 10–12 |

B2F carries twice the PARAS slots of either floor above it, so it is the floor worth the extra
walking.

**PIKACHU makes the opposite case, and the type chart is the reason.** Its own STARMIE matchup
looks good offensively — ELECTRIC is super-effective on both of Misty's Pokémon — but
`TypeEffects` carries no ELECTRIC row for a WATER attack at all, which means ELECTRIC neither
resists nor is specially threatened by WATER: a STARMIE's WATER GUN lands on Pikachu at full,
unreduced power, the same as it would on almost anything else. Pikachu's offensive advantage does
not buy it a defensive one, and against a Pokémon with base Special 100 hitting a Pokémon with
base HP 35 and base Defense 30, the matchup is decided by bulk long before typing enters into it
again.

## See also

* [`battle.md`](battle.md#catching) — where this page's summary lives inside the wider battle
  chapter, and the wild/trainer frame-matching in full.
* [`bag.md`](bag.md#the-first-poké-ball) — the ball-gift script and the price table for buying
  one instead.
* [`party.md`](party.md) — `WriteMonMoves` and how a caught Pokémon's starting moves are decided.
* [`cerulean-gym.md`](cerulean-gym.md) — Misty's team, her gym's map, and the full type-matchup
  table a bench is chosen against.
* [`discoveries.md`](discoveries.md) — the reasoning behind the findings cited from here.
