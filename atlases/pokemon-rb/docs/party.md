# The party — Pokémon Red/Blue

[← Pokémon Red/Blue](../README.md) · [by address](by-address.md) · [by name](by-name.md) · [the structures](structures.md) · [AtlasGB](../../../README.md)

Six Pokémon, described three times over. `wPartyCount` says how many; `wPartySpecies` is a flat
list of species with an `$FF` terminator; and each 44-byte [`party_struct`](structures.md)
carries its own species byte. The game writes those three at different moments, so **the fact
that they agree is what makes all three addresses trustworthy** — it is the atlas's strongest
invariant and it is checked on every run against a real save.

Three traps live in the structure and all three have cost real time:

* **Everything multi-byte is big-endian**, on a little-endian console.
* **The HP DV is not stored.** It is assembled from bit 0 of the other four DVs, most
  significant first. Sixteen bits of DV are the whole of what distinguishes one Pokémon from
  another of its species and level, and one of the five is not a field.
* **`Species` is the internal index, not the Pokédex number.** `MonsterNames` is indexed by
  internal index and `BaseStats` by Pokédex number; `PokedexOrder` converts. Read one with the
  other's index and you get a plausible, wrong Pokémon.

That trap is not academic: PARAS' own internal index is **109** against a Pokédex number of
**46**, and reading either table with the other's number raises no error — it returns a
different, entirely plausible Pokémon. Every symbol in this atlas that stores a species carries
the internal index. A lookup by *name* instead of by index has its own version of the same
trap: [two internal indices print as one identical name](discoveries.md#two-species-share-one-printable-name),
so a name is not a safe key either.

The party block ends exactly where the saved block begins — `wPartyDataEnd` and
`wMainDataStart` are the same address — which is why the party survives a reset and most of
what is above it in memory does not.

**A caught Pokémon's moves are a pure function of its species and its level, nothing else.**
`WriteMonMoves` (`engine/pokemon/add_mon.asm`) builds the four bytes at `wPartyMon1Moves` from the
species' own starting moves (`BaseStats`' `MOVE1`..`MOVE4`) and its level-up learnset, oldest move
pushed out once the four slots are full. Two Pokémon of the same species at the same level always
arrive holding the same four moves — nothing about the battle that produced one, or the trainer
that owns it, enters into it. A wild or trainer Pokémon's exact moveset is therefore knowable
before it is ever seen.

**Experience from a battle goes to whichever Pokémon was sent out, not to the party as a whole.**
Gen 1 always sends out the first party member with any HP, so a Pokémon left on the bench earns
nothing until it leads — the only way to move it up is `SwitchPartyMon`
(`engine/menus/party_menu.asm`), reached from the START menu's `POKéMON` list. `wPartyFoughtCurrentEnemyFlags`
and `wPartyGainExpFlags` record who was actually in the fight and is therefore owed a share of the
experience; neither says anything about a Pokémon still waiting in slot 6.

Levelling up checks a single per-species table, `EvosMovesPointerTable` (bank `$0E`), for a
`(level, move)` entry, and the whole cartridge holds **728** of them across its 190 internal
indices — indexed the same way `wPartySpecies` is, [and just as easy to misread against the wrong
table](discoveries.md#the-learnset-table-is-indexed-by-internal-number-not-by-dex-number). A
"which move should be forgotten?" prompt only fires once the four slots above are already full;
walking every species from level 1 to level 100 finds **840** such prompts, and collapsing them to
"these four moves held, this one offered" leaves only **581** distinct shapes. The learning system
is closed and enumerable, not open-ended — which is also why
[crossing two level boundaries in one battle still loses the move that belonged to the level in
between](discoveries.md#crossing-two-levels-loses-the-move-in-between): the game checks the
learnset once, for the level it ends the battle on, not for every level passed through.

One line driven on the real cartridge, a BULBASAUR raised from level 1 with
[every evolution taken](discoveries.md#cancelling-an-evolution-costs-one-level-and-nothing-else),
shows the shape of it. `held` is what `WriteMonMoves` had already given it; `offered` is
`EvosMovesPointerTable`'s entry for that level. Which of the two the game keeps afterwards is left
to the player — the cartridge only asks — so `chosen` below is one stated answering rule's output,
not anything the game requires:

| level | form | offered | held before the prompt | chosen |
|---:|---|---|---|---|
| 22 | IVYSAUR | POISONPOWDER | TACKLE, GROWL, LEECH SEED, VINE WHIP | forget LEECH SEED |
| 30 | IVYSAUR | RAZOR LEAF | TACKLE, GROWL, VINE WHIP, POISONPOWDER | forget POISONPOWDER |
| 43 | VENUSAUR | GROWTH | TACKLE, GROWL, VINE WHIP, RAZOR LEAF | decline |
| 55 | VENUSAUR | SLEEP POWDER | TACKLE, GROWL, VINE WHIP, RAZOR LEAF | decline |
| 65 | VENUSAUR | SOLARBEAM | TACKLE, GROWL, VINE WHIP, RAZOR LEAF | forget GROWL |

A decline plays out as two further prompts — offer to replace, then offer to abandon — and ends
with the offered move simply not learned; nothing about the four held moves changes. The three
screens, verbatim: `Delete an older move to make room for <MOVE>?`, then either
`Which move should be forgotten?` or, on a decline, `Abandon learning <MOVE>?`.

**The first of those three is easy to misidentify, because it can scroll itself off the
screen** — see [a decline box can scroll its own question off the
screen](discoveries.md#a-decline-box-can-scroll-its-own-question-off-the-screen).

### Findings behind these bytes

Several entries below carry a sentence that cost somebody a campaign to establish. The reasoning —
the messy symptom, the wrong turns, the measurement that settled it — is recorded once, in
[this atlas's own discoveries page](discoveries.md), and linked from here rather than repeated.

| the finding | the bytes it is about |
|---|---|
| [the learnset table is indexed by internal number, not by dex number](discoveries.md#the-learnset-table-is-indexed-by-internal-number-not-by-dex-number) | [`wPartySpecies`](#s-wPartySpecies), [`wPartyMon1Species`](#s-wPartyMon1Species) |
| [two species share one printable name](discoveries.md#two-species-share-one-printable-name) | [`wPartyMon1Species`](#s-wPartyMon1Species) |
| [crossing two levels loses the move in between](discoveries.md#crossing-two-levels-loses-the-move-in-between) | [`wPartyMon1Level`](#s-wPartyMon1Level) |
| [cancelling an evolution costs one level and nothing else](discoveries.md#cancelling-an-evolution-costs-one-level-and-nothing-else) | [`wPartyMon1Species`](#s-wPartyMon1Species) |

<!-- atlas:begin (table) — generated by tools/render.py from the atlas data; edit the data, not the table -->

**179 entries** · 31 distinct addresses · **30 with a written description** · 141 repeated slots folded onto slot 1.

| address | bytes | symbol | ev | what it is |
|---|---:|---|:--:|---|
| `$AF2C` b1 | 404 | <a id="s-sPartyData"></a>`sPartyData` | R | The saved party, the same layout as `wPartyCount` onwards. |
| `$CC2B` | 1 | <a id="s-wPartyAndBillsPCSavedMenuItem"></a>`wPartyAndBillsPCSavedMenuItem` | RL |  |
| `$CCF5` | 1 | <a id="s-wPartyFoughtCurrentEnemyFlags"></a>`wPartyFoughtCurrentEnemyFlags` | RL | One bit per party slot, set for anyone who faced the current opponent. Used to decide the experience split. Measured: written together with `wPartyGainExpFlags` from the same instruction context (bank 3 `$7692`) — the two flag arrays are set as one act, not two. |
| `$D058` | 1 | <a id="s-wPartyGainExpFlags"></a>`wPartyGainExpFlags` | RL | One bit per party slot: who was in the battle and therefore earns experience. |
| `$D163` | 1 | <a id="s-wPartyCount"></a>`wPartyCount`<br><a id="s-wPartyDataStart"></a>`wPartyDataStart` | RLI | **`wPartyCount`** — How many Pokemon you are carrying, 0 to 6. **Verified live**: the harness reads it, walks both parallel species lists, and requires them to agree and to be `$FF`-terminated. **`wPartyDataStart`** — Label for the first byte of the party block; the same address as `wPartyCount`. |
| `$D164` | 7 | <a id="s-wPartySpecies"></a>`wPartySpecies` | RLI | Six species indices and an `$FF` terminator — a flat list beside the structures, written at a different moment from the copy inside each `party_struct`. The two agreeing is what makes both addresses trustworthy. |
| `$D16B` | 1 | <a id="s-wPartyMon1Species"></a>`wPartyMon1Species`<br><a id="s-wPartyMon1"></a>`wPartyMon1` <a id="s-wPartyMons"></a>`wPartyMons` | RL | **`wPartyMon1Species`** — Species **internal index** — not the Pokedex number. `PokedexOrder` converts. **`wPartyMon1`** — Party slot 1, the whole 44-byte `party_struct`. **`wPartyMons`** — Six 44-byte `party_struct`s. See [the structures](structures.md). The species here is the **internal index**, not the Pokedex number. *(×6, stride 44)* |
| `$D16C` | 2 | <a id="s-wPartyMon1HP"></a>`wPartyMon1HP` | RL | Current HP, **big-endian**. *(×6, stride 44)* |
| `$D16E` | 1 | <a id="s-wPartyMon1BoxLevel"></a>`wPartyMon1BoxLevel` | L | The level as stored. Kept in step with `Level` for a party Pokemon; for a stored one it is the only level there is. *(×6, stride 44)* |
| `$D16F` | 1 | <a id="s-wPartyMon1Status"></a>`wPartyMon1Status` | RL | Sleep, poison, burn, freeze, paralysis, as one byte of flags and a sleep counter. *(×6, stride 44)* |
| `$D170` | 1 | <a id="s-wPartyMon1Type1"></a>`wPartyMon1Type1`<br><a id="s-wPartyMon1Type"></a>`wPartyMon1Type` | L | **`wPartyMon1Type1`** — First type. **`wPartyMon1Type`** — The two type bytes as a pair, `Type1` then `Type2`. *(×6, stride 44)* |
| `$D171` | 1 | <a id="s-wPartyMon1Type2"></a>`wPartyMon1Type2` | L | Second type; equal to `Type1` for a single-typed Pokemon. |
| `$D172` | 1 | <a id="s-wPartyMon1CatchRate"></a>`wPartyMon1CatchRate` | L | The species' base catch rate, copied in at capture. Held item in later generations; here it is the ball maths. *(×6, stride 44)* |
| `$D173` | 4 | <a id="s-wPartyMon1Moves"></a>`wPartyMon1Moves` | RL | Four move ids, 0 for an empty slot. *(×6, stride 44)* |
| `$D177` | 2 | <a id="s-wPartyMon1OTID"></a>`wPartyMon1OTID` | RL | The original trainer's ID, **big-endian**. Compared against `wPlayerID` to decide whether this Pokemon was traded to you. *(×6, stride 44)* |
| `$D179` | 3 | <a id="s-wPartyMon1Exp"></a>`wPartyMon1Exp` | L | Experience, **three bytes big-endian**. *(×6, stride 44)* |
| `$D17C` | 2 | <a id="s-wPartyMon1HPExp"></a>`wPartyMon1HPExp` | L | Stat experience for HP, big-endian. Gained per battle and folded into the stat by `CalcStat`. *(×6, stride 44)* |
| `$D17E` | 2 | <a id="s-wPartyMon1AttackExp"></a>`wPartyMon1AttackExp` | L | Stat experience for Attack. *(×6, stride 44)* |
| `$D180` | 2 | <a id="s-wPartyMon1DefenseExp"></a>`wPartyMon1DefenseExp` | L | Stat experience for Defense. *(×6, stride 44)* |
| `$D182` | 2 | <a id="s-wPartyMon1SpeedExp"></a>`wPartyMon1SpeedExp` | L | Stat experience for Speed. *(×6, stride 44)* |
| `$D184` | 2 | <a id="s-wPartyMon1SpecialExp"></a>`wPartyMon1SpecialExp` | L | Stat experience for Special. *(×6, stride 44)* |
| `$D186` | 2 | <a id="s-wPartyMon1DVs"></a>`wPartyMon1DVs` | L | Two bytes, four nybbles: attack, defense, speed, special. **The HP DV is not stored** — it is assembled from bit 0 of the other four, most significant first. These sixteen bits are the whole of what makes one Pokemon differ from another of its species and level. *(×6, stride 44)* |
| `$D188` | 4 | <a id="s-wPartyMon1PP"></a>`wPartyMon1PP` | RL | Four bytes: current PP in the low 6 bits, PP Ups applied in the top 2. *(×6, stride 44)* |
| `$D18C` | 1 | <a id="s-wPartyMon1Level"></a>`wPartyMon1Level` | RL | The level the stats below were computed at. *(×6, stride 44)* |
| `$D18D` | 2 | <a id="s-wPartyMon1MaxHP"></a>`wPartyMon1MaxHP`<br><a id="s-wPartyMon1Stats"></a>`wPartyMon1Stats` | RL | **`wPartyMon1MaxHP`** — Computed maximum HP, big-endian. **`wPartyMon1Stats`** — Label for the five computed stats as one run. *(×6, stride 44)* |
| `$D18F` | 2 | <a id="s-wPartyMon1Attack"></a>`wPartyMon1Attack` | RL | Computed Attack, big-endian. *(×6, stride 44)* |
| `$D191` | 2 | <a id="s-wPartyMon1Defense"></a>`wPartyMon1Defense` | RL | Computed Defense, big-endian. *(×6, stride 44)* |
| `$D193` | 2 | <a id="s-wPartyMon1Speed"></a>`wPartyMon1Speed` | RL | Computed Speed, big-endian. *(×6, stride 44)* |
| `$D195` | 2 | <a id="s-wPartyMon1Special"></a>`wPartyMon1Special` | RL | Computed Special, big-endian. One stat here; Gen 2 splits it in two. *(×6, stride 44)* |
| `$D273` | 11 | <a id="s-wPartyMon1OT"></a>`wPartyMon1OT`<br><a id="s-wPartyMonOT"></a>`wPartyMonOT` | RL | **`wPartyMon1OT`** — Party slot 1's original-trainer name, 11 bytes. **`wPartyMonOT`** — Six 11-byte original-trainer names, one per party slot. Kept outside the structure because the trade protocol sends them as their own block. *(×6, stride 11)* |
| `$D2B5` | 11 | <a id="s-wPartyMon1Nick"></a>`wPartyMon1Nick`<br><a id="s-wPartyMonNicks"></a>`wPartyMonNicks` | RL | **`wPartyMon1Nick`** — Party slot 1's nickname, 11 bytes. **`wPartyMonNicks`** — Six 11-byte nicknames. A Pokemon with no nickname carries its species name here, which is why renaming and evolving both have to rewrite it. *(×6, stride 11)* |
| `$D2F7` | — | <a id="s-wPartyDataEnd"></a>`wPartyDataEnd` | RL | One past the party block. Numerically the same address as `wMainDataStart`, which is why the party is saved: it sits immediately below the block a save writes out. |

<!-- atlas:end (table) -->
