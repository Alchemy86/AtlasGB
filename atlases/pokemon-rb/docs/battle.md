# The battle — Pokémon Red/Blue

[← Pokémon Red/Blue](../README.md) · [by address](by-address.md) · [by name](by-name.md) · [the structures](structures.md) · [AtlasGB](../../../README.md)

Everything on this page exists only while a battle is running, which is why so much of it
shares storage with something else.

**One byte starts a battle.** The overworld loop polls `wCurOpponent` every frame; non-zero
means fight. Below 200 it is a species and you get a wild battle at `wCurEnemyLevel`; at or
above 200 it is a trainer class plus that offset, with `wTrainerNo` choosing the roster. This
is not a discovered trick — it is what the game's own debug menu writes. The full technique is
[battle control](https://github.com/Alchemy86/TerminalGB/blob/main/docs/gen1/battle-control.md).

**The in-battle form is not the party form.** `wBattleMon` and `wEnemyMon` are 29-byte
[`battle_struct`](structures.md)s: original trainer, experience and stat experience are gone,
so the level and the five stats sit at different offsets. Confusing the two is the classic way
to produce a level 6,400 Pokémon. Two files in
[TerminalGB](https://github.com/Alchemy86/TerminalGB) disagreed about whether the
structure was 28 or 29 bytes; the atlas settles it at 29, and the harness proves it by showing
that `wTrainerClass` follows immediately.

Two more things a reader will otherwise get wrong:

* **`wIsInBattle` has four values, not three.** 0 none, 1 wild, 2 trainer — and **`$FF` means
  the player blacked out**. A catch-all arm that reads anything non-zero as a trainer battle
  shows a battle panel, full of stale data, in the overworld.
* **Stat stages are 1-13 with 7 neutral, and they start at 0.** The block is zeroed work RAM
  until `InitBattleVariables` writes the sevens, so a sample taken on the first frames of a
  battle reads zero everywhere and looks exactly like −6 on everything.

The opposing trainer's team is built into `wEnemyMons` in the *party* shape, and it **shares
storage with the map's wild encounter tables**. That is not a bug — the game cannot be in a
trainer battle and consulting the wild tables at once — but it does mean reading `wEnemyMons`
in the overworld returns encounter data, so check `wIsInBattle` first.

**A battle does not start just because `wCurOpponent` is non-zero — three conditions can refuse
it first**, and all three are checked before the byte is ever read: a dungeon-warp already in
progress, the player character being moved by the game rather than by input, and a per-map
flag some scripts set explicitly (Mt Moon B2F and Pokémon Tower 5F both set it). An empty party
does not refuse a battle at all — it starts, and ends on the first turn as an immediate
black-out, which reads like a crash if the cause is not known.

**A traded Pokémon can refuse an order, and the check is the original trainer ID, not a
flag.** `CheckForDisobedience` compares [`wPartyMon1OTID`](party.md#s-wPartyMon1OTID) against
[`wPlayerID`](player.md#s-wPlayerID); a mismatch counts the Pokémon as traded, and a traded
Pokémon's level ceiling scales with badge count — 10 with none held, rising to 30, 50, 70 and
101 as more are earned. Nothing about the Pokémon's own data marks it as traded; the game
decides it fresh, every time it might disobey, from whichever ID is currently in
`wPlayerID`.

### Findings behind these bytes

Several entries below carry a sentence that cost somebody a campaign to establish. The
reasoning — the messy symptom, the wrong turns, the measurement that settled it — is recorded
once, in [this atlas's own discoveries page](discoveries.md), and linked from here rather than
repeated. The *fact* belongs on the entry; the *story* belongs one link away.

| the finding | the bytes it is about |
|---|---|
| [a screen does not survive a switch](discoveries.md#a-screen-does-not-survive-a-switch) | [`wPlayerBattleStatus3`](#s-wPlayerBattleStatus3), [`wPlayerStatsToDouble`](scratch.md#s-wPlayerStatsToDouble) |
| [a documented sentinel read as anything else](discoveries.md#a-documented-sentinel-read-as-anything-else) | [`wIsInBattle`](#s-wIsInBattle) |
| [the blackout carve-out](discoveries.md#the-blackout-carve-out) | [`wCurOpponent`](#s-wCurOpponent), [`wCurMap`](overworld.md#s-wCurMap) |
| [a critical hit is the worst case](discoveries.md#a-critical-hit-is-the-worst-case) | [`wBattleMon`](#s-wBattleMon), [`wMonHBaseSpeed`](scratch.md#s-wMonHBaseSpeed) |
| [accuracy is not the number in the move table](discoveries.md#accuracy-is-not-the-number-in-the-move-table) | [`wPlayerMonStatMods`](#s-wPlayerMonStatMods), [`wPlayerMonAccuracyMod`](#s-wPlayerMonAccuracyMod) |
| [a missed trapping move leaves its counter standing](discoveries.md#a-missed-trapping-move-leaves-its-counter-standing) | [`wPlayerNumAttacksLeft`](#s-wPlayerNumAttacksLeft) |
| [an escape is not free](discoveries.md#an-escape-is-not-free) | [`wEnemyMon`](#s-wEnemyMon) |

**`wPlayerBattleStatus1`, `2` and `3` are three different bytes, not three slots.** They sit
consecutively and are one byte each, so the extractor reads them as a repeated structure — but
they hold different bits, and only the block from `wPlayerStatsToDouble` clears on a switch. A
described slot is listed on its own row below for exactly that reason.

### Known battle-engine bugs

Gen 1's battle engine has real defects, not just surprising-but-correct rules, and every one
below is cited to the routine that produces it — a model of this game that "fixed" them would be
a model of a game nobody played.

**A critical hit's own rate is computed from base Speed, not the in-battle stat.** The roll
reads a species' base-stats Speed byte directly rather than the staged, in-battle value the
damage formula itself uses — so a Speed-boosted Pokémon's crit chance does not move with the
boost. **And for the enemy specifically, a crit recomputes its stats from scratch, at zero
stat experience, rather than reading the values already sitting in its battle structure** —
outside a link battle, `GetEnemyMonStat` looks the species up fresh from its base stats, DVs
and level, ignoring whatever stat experience it has actually earned. The two sides of a battle
are not treated symmetrically by this one calculation.

**A critical hit discards stat stages in the damage calculation too, not only in its own rate.**
`GetDamageVarsForPlayerAttack` doubles the level on a crit, but pulls Attack and Defense from the
unmodified stat rather than the stage-adjusted one — a crit landed through your own SWORDS DANCE
can do *less* damage than an ordinary hit would have with the boost counted.

**A move can be reported as a miss after its accuracy roll already passed.** `AdjustDamageForMoveType`
rounds a doubly-resisted hit down to zero damage — its own comment says the case "only occurs if a
move that would do 2 or 3 damage is 0.25x effective" — and zero damage sets `wMoveMissed`, so the
game prints the ordinary miss text for an attack that connected and was merely too weak to
register.

**Full paralysis is 63 in 256, not exactly one in four.** The check is `cp 25 percent`, and the
disassembly's own `percent()` macro truncates a percentage into a comparison byte instead of
rounding it — 25 percent of 256 truncates to 63, not 64.

**A sleeping Pokémon's counter is rolled 1–7 turns**, not the commonly quoted 1–4 or 1–7-inclusive
range some references give with a different convention: `SleepEffect` masks a random byte with
`SLP_MASK` and rerolls a zero, and the source's own comment gives the range as "a random number
between 1 and 7." And the turn the counter reaches zero is not free: both the "still asleep" and
"just woke up" paths fall into the same `.sleepDone` label, so waking up costs a whole turn rather
than skipping straight to an action.

**Raising or lowering a stat stage recomputes the stat from scratch, and recomputing it is not
free of side effects.** `StatModifierUpEffect` (and its downward counterpart) pulls from the
unmodified stat and reapplies any badge boost every time it runs, rather than checking whether one
is already folded in — its own comments say plainly that badge boosts "get reapplied again to
every stat" and that doing this means "paralysis and burn penalties, as well as badge boosts are
ignored." So changing a stage also wipes out a burn or paralysis penalty already baked into that
stat, and repeated stage changes on a badge-covered stat compound the boost instead of applying it
once. A burned Pokémon whose Attack is lowered a stage, for instance, has its burn halving undone
in the same stroke.

**HYPER BEAM skips its forced recharge turn if the hit that used it was also the knockout.**
`HYPER_BEAM_EFFECT` is absent from `AlwaysHappenSideEffects` — the list of effects the engine
applies unconditionally — and the code path that notices a knockout returns before the recharge
flag is ever set.

**COUNTER's priority is not merely low, it is fixed.** `MainInBattleLoop`'s hard-coded turn order
runs it in its own slot *after* the opponent's attack has already resolved that turn, regardless of
either side's Speed — a faster Pokémon using COUNTER still goes second, because there would be
nothing to counter otherwise.

**THRASH, PETAL DANCE and BIDE roll their lock length from the same counter a trapping move
uses** — [`wPlayerNumAttacksLeft`](#s-wPlayerNumAttacksLeft) — and from the same routine shape:
`ThrashPetalDanceEffect` and `BideEffect` each draw one random bit and add two, giving a roll of 2
or 3 (measured over 161 Thrash locks and 281 Bides), so a Thrash lock runs 3-4 attacking turns and
a Bide runs the same length to its release. `TrappingEffect`'s own comment states "3/8 chance for
2 and 3 attacks, and 1/8 chance for 4 and 5 attacks," measured close to that shape over 371 locks
(39%, 34%, 13%, 14%).

**The toxic-tick counter is shared with LEECH SEED rather than gated to poison.** The
disassembly's own comment says the ticks are counted "even if the damage is not poison (hence the
Leech Seed glitch)" — so a Pokémon that is both badly poisoned and seeded bumps the same counter
from both effects each turn, and has both drains multiplied by it.

### What the battle screen shows, and what it doesn't

A player looking at the battle screen sees both Pokémon's **names** — the nickname, not the
species, since an unrenamed Pokémon's nickname defaults to its species name and only the nickname
is ever drawn — and **levels**, both **HP bars**, the player's own **exact HP** as a fraction, and,
on the FIGHT menu, the highlighted move's **type and PP**. It never shows the opponent's stats, its
exact HP, or its move list.

The wild and trainer battle screens are the same picture. Traced frame by frame over a matched
pair — one save, one challenger, only the wild-or-trainer line different — **no cell differs
across all 1,200 paired frames**, and the fewest differing on any single frame is **two, the level
digits**. The only tell is the opening banner: `Wild <NAME> appeared!` is on screen for **9
frames**, `<NAME> wants to fight!` for **11** — about a sixth of a second, with nothing
distinguishing the two screens before or after it.

Throwing a ball at a trainer's Pokémon costs nothing but the turn. `ItemUseBall` branches to
`ThrowBallAtTrainerMon`, prints its own refusal text, and returns without touching the bag —
measured over 64 throws in synthesized trainer battles: **64 blocked, 64 balls kept, 0 spent.**

### Catching

The game hands over its first Poké Balls rather than selling them. `wEventFlags` bit 4 of byte 4
(`$D74B`) is `EVENT_GOT_POKEBALLS_FROM_OAK` — the flag immediately before `EVENT_GOT_POKEDEX` — and
the script that sets it also gives five `POKE_BALL`s, with Oak explaining that the balls are how
you actually catch a Pokémon rather than just see one. Buying one instead costs ¥200 — Viridian's
mart, the first the player can reach, stocks it (see [the bag](bag.md#the-first-poké-ball)).

`ItemUsePokeBall` (`engine/items/item_effects.asm`) is not the pass/fail test a modern formula
assumes. Three things make it different:

- **A Great or Ultra Ball's check is a resampling loop, not a threshold you can fail.** A random
  byte over the ball's cutoff (200 for Great, 150 for Ultra and Safari) jumps back and redraws, so
  the ball's real effect is to bias the draw toward low values rather than reject high ones —
  reading it as a pass/fail hurdle, the natural reading for a modern formula, understates every
  Great and Ultra Ball's odds.
- **A status bonus is subtracted from the roll and can catch outright.** Sleep or freeze is worth
  25, any other status 12; if the random byte comes in under the bonus, the Pokémon is caught with
  no further check at all.
- **The shake animation is decided after the outcome, not before it.** Everything that produces the
  one-, two- or three-shake animation runs only once capture or failure is already settled — three
  shakes and a break is not a near miss, because the miss happened earlier in the routine.

Disassembled and turned into a closed form, the routine agrees with the retail cartridge: sweeping
HP, status and ball type across **251 throws**, the observed catch rate was **61.0%** against a
closed-form prediction of **59.8%** — `z = +0.36` against a 3.09% standard error. The full
disassembly, the verbatim script text, and the per-bucket odds table are on
[catching](catching.md), which this section only summarises.

<!-- atlas:begin (table) — generated by tools/render.py from the atlas data; edit the data, not the table -->

**359 entries** · 159 distinct addresses · **121 with a written description** · 147 repeated slots folded onto slot 1.

| address | bytes | symbol | ev | what it is |
|---|---:|---|:--:|---|
| `$CC2D` | 1 | <a id="s-wBattleAndStartSavedMenuItem"></a>`wBattleAndStartSavedMenuItem` | RL |  |
| `$CC2F` | 1 | <a id="s-wPlayerMonNumber"></a>`wPlayerMonNumber` | RL | Which party slot is currently out. |
| `$CC5B` | 1 | <a id="s-wBoostExpByExpAll"></a>`wBoostExpByExpAll`<br><a id="s-wAnimationType"></a>`wAnimationType` <a id="s-wDexRatingNumMonsSeen"></a>`wDexRatingNumMonsSeen` <a id="s-wElevatorWarpMaps"></a>`wElevatorWarpMaps` <a id="s-wFilteredBagItems"></a>`wFilteredBagItems` <a id="s-wHallOfFame"></a>`wHallOfFame` <a id="s-wMonPartySpritesSavedOAM"></a>`wMonPartySpritesSavedOAM` <a id="s-wNPCMovementDirections"></a>`wNPCMovementDirections` <a id="s-wOaksAideRewardItemName"></a>`wOaksAideRewardItemName` <a id="s-wSlotMachineSevenAndBarModeChance"></a>`wSlotMachineSevenAndBarModeChance` <a id="s-wTrainerCardBlkPacket"></a>`wTrainerCardBlkPacket` <a id="s-wUnusedFlag"></a>`wUnusedFlag` <a id="s-wVermilionDockTileMapBuffer"></a>`wVermilionDockTileMapBuffer` | RL | **`wBoostExpByExpAll`** — One of a dozen unrelated one-off scratch uses sharing this WRAM byte across different screens and routines — also, among others in this atlas, `wFilteredBagItems` when a bag menu is open; which one is live depends entirely on what is currently executing. **`wFilteredBagItems`** — A filtered view of the bag — only the items a particular menu will accept, rebuilt each time it opens. |
| `$CC79` | 30 | <a id="s-wAnimPalette"></a>`wAnimPalette` | RL |  |
| `$CCD3` | 1 | <a id="s-wCanEvolveFlags"></a>`wCanEvolveFlags`<br><a id="s-wAddedToParty"></a>`wAddedToParty` <a id="s-wMiscBattleData"></a>`wMiscBattleData` <a id="s-wParentMenuItem"></a>`wParentMenuItem` <a id="s-wSimulatedJoypadStatesEnd"></a>`wSimulatedJoypadStatesEnd` | RL | **`wCanEvolveFlags`** — Shares this byte with `wSimulatedJoypadStatesEnd` and other unrelated one-off scratch uses at this address; which is live depends on what is currently executing. **`wSimulatedJoypadStatesEnd`** — One past the canned button sequence. |
| `$CCD4` | 1 | <a id="s-wForceEvolution"></a>`wForceEvolution` | RL |  |
| `$CCD5` | 2 | <a id="s-wAILayer2Encouragement"></a>`wAILayer2Encouragement` | RL |  |
| `$CCD7` | 1 | <a id="s-wPlayerSubstituteHP"></a>`wPlayerSubstituteHP` | RL | The substitute's remaining hit points on your side. A whole byte, so a substitute is a quantity and not a two-state flag. |
| `$CCD8` | 1 | <a id="s-wEnemySubstituteHP"></a>`wEnemySubstituteHP` | RL | The same for the opponent's substitute. |
| `$CCDB` | 1 | <a id="s-wMoveMenuType"></a>`wMoveMenuType` | RL |  |
| `$CCDC` | 1 | <a id="s-wPlayerSelectedMove"></a>`wPlayerSelectedMove` | RL | The move you chose this turn. |
| `$CCDD` | 1 | <a id="s-wEnemySelectedMove"></a>`wEnemySelectedMove` | RL | The move the opponent chose this turn. |
| `$CCDF` | 3 | <a id="s-wAICount"></a>`wAICount` | RL | How many AI passes are left this turn. |
| `$CCE3` | 2 | <a id="s-wLastSwitchInEnemyMonHP"></a>`wLastSwitchInEnemyMonHP` | RL |  |
| `$CCE5` | 3 | <a id="s-wTotalPayDayMoney"></a>`wTotalPayDayMoney` | RL |  |
| `$CCE8` | 1 | <a id="s-wSafariEscapeFactor"></a>`wSafariEscapeFactor` | RL | How likely the Safari Zone opponent is to flee. |
| `$CCE9` | 2 | <a id="s-wSafariBaitFactor"></a>`wSafariBaitFactor` | RL | How long bait keeps it interested. |
| `$CCEB` | 2 | <a id="s-wTransformedEnemyMonOriginalDVs"></a>`wTransformedEnemyMonOriginalDVs` | RL | The opponent's real DVs, kept while Transform has overwritten them. |
| `$CCED` | 1 | <a id="s-wMonIsDisobedient"></a>`wMonIsDisobedient` | RL | Set when a traded Pokemon above your badge level refuses an order. |
| `$CCEE` | 1 | <a id="s-wPlayerDisabledMoveNumber"></a>`wPlayerDisabledMoveNumber` | RL |  |
| `$CCEF` | 1 | <a id="s-wEnemyDisabledMoveNumber"></a>`wEnemyDisabledMoveNumber` | RL |  |
| `$CCF0` | 1 | <a id="s-wInHandlePlayerMonFainted"></a>`wInHandlePlayerMonFainted` | RL |  |
| `$CCF1` | 1 | <a id="s-wPlayerUsedMove"></a>`wPlayerUsedMove` | RL |  |
| `$CCF2` | 1 | <a id="s-wEnemyUsedMove"></a>`wEnemyUsedMove` | RL |  |
| `$CCF3` | 1 | <a id="s-wEnemyMonMinimized"></a>`wEnemyMonMinimized` | RL | The same for the opponent. |
| `$CCF4` | 1 | <a id="s-wMoveDidntMiss"></a>`wMoveDidntMiss` | RL |  |
| `$CCF6` | 1 | <a id="s-wLowHealthAlarmDisabled"></a>`wLowHealthAlarmDisabled` | RL |  |
| `$CCF7` | 14 | <a id="s-wPlayerMonMinimized"></a>`wPlayerMonMinimized` | RL | Set once your active Pokemon has used MINIMIZE. **Cosmetic in Generation 1**: it only changes how the sprite is drawn. Nothing doubles a stomping move's damage against it until a later generation. |
| `$CD0F` | 1 | <a id="s-wPlayerMonUnmodifiedLevel"></a>`wPlayerMonUnmodifiedLevel`<br><a id="s-wInGameTradeGiveMonSpecies"></a>`wInGameTradeGiveMonSpecies` <a id="s-wMiscBattleDataEnd"></a>`wMiscBattleDataEnd` <a id="s-wVermilionDockTileMapBufferEnd"></a>`wVermilionDockTileMapBufferEnd` | RL |  |
| `$CD10` | 2 | <a id="s-wPlayerMonUnmodifiedMaxHP"></a>`wPlayerMonUnmodifiedMaxHP`<br><a id="s-wInGameTradeTextPointerTablePointer"></a>`wInGameTradeTextPointerTablePointer` | RL | Your maximum HP before any stat stage was applied. The stages are applied to a copy, so the original has to be kept. |
| `$CD12` | 1 | <a id="s-wPlayerMonUnmodifiedAttack"></a>`wPlayerMonUnmodifiedAttack`<br><a id="s-wInGameTradeTextPointerTableIndex"></a>`wInGameTradeTextPointerTableIndex` | RL | Attack for your side before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD14` | 2 | <a id="s-wPlayerMonUnmodifiedDefense"></a>`wPlayerMonUnmodifiedDefense` | RL | Defense for your side before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD16` | 2 | <a id="s-wPlayerMonUnmodifiedSpeed"></a>`wPlayerMonUnmodifiedSpeed` | L | Speed for your side before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD18` | 2 | <a id="s-wPlayerMonUnmodifiedSpecial"></a>`wPlayerMonUnmodifiedSpecial` | RL | Special for your side before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD1A` | 1 | <a id="s-wPlayerMonAttackMod"></a>`wPlayerMonAttackMod`<br><a id="s-wPlayerMonStatMods"></a>`wPlayerMonStatMods` | RL | **`wPlayerMonAttackMod`** — Attack stage for your side, **biased by 7**: `1` to `13` is `-6` to `+6`, and a freshly zeroed block reads as `-6` on everything until `InitBattleVariables` writes the sevens. **`wPlayerMonStatMods`** — In-battle stat stages for your side. **7 is neutral, not 0**: the block is zeroed work RAM until `InitBattleVariables` writes the sevens, so a sample taken on the first frames of a battle reads zero everywhere and looks like -6 on everything. |
| `$CD1B` | 1 | <a id="s-wPlayerMonDefenseMod"></a>`wPlayerMonDefenseMod` | L | Defense stage for your side, **biased by 7**: `1` to `13` is `-6` to `+6`, and a freshly zeroed block reads as `-6` on everything until `InitBattleVariables` writes the sevens. |
| `$CD1C` | 1 | <a id="s-wPlayerMonSpeedMod"></a>`wPlayerMonSpeedMod` | RL | Speed stage for your side, **biased by 7**: `1` to `13` is `-6` to `+6`, and a freshly zeroed block reads as `-6` on everything until `InitBattleVariables` writes the sevens. |
| `$CD1D` | 1 | <a id="s-wPlayerMonSpecialMod"></a>`wPlayerMonSpecialMod` | RL | Special stage for your side, **biased by 7**: `1` to `13` is `-6` to `+6`, and a freshly zeroed block reads as `-6` on everything until `InitBattleVariables` writes the sevens. |
| `$CD1E` | 1 | <a id="s-wPlayerMonAccuracyMod"></a>`wPlayerMonAccuracyMod`<br><a id="s-wInGameTradeReceiveMonName"></a>`wInGameTradeReceiveMonName` | R | Accuracy stage for your side, biased by 7. **There is no accuracy value anywhere** — only this stage: `MoveHitTest` scales the move table's accuracy byte by it and by the inverse of the defender's evasion stage, so the number in the move table stops being true the first time a SAND-ATTACK lands. Measured on the cartridge: TACKLE's 242/255 becomes 159/255 after one stage down and 121/255 after two. |
| `$CD1F` | 3 | <a id="s-wPlayerMonEvasionMod"></a>`wPlayerMonEvasionMod` | RL | Evasion stage for your side, biased by 7, and the other half of `MoveHitTest`'s scaling. |
| `$CD22` | 1 | <a id="s-wPlayerMonStatModsEnd"></a>`wPlayerMonStatModsEnd` | RL | One past your side's six stat stages. |
| `$CD23` | 1 | <a id="s-wEnemyMonUnmodifiedLevel"></a>`wEnemyMonUnmodifiedLevel` | RL |  |
| `$CD24` | 2 | <a id="s-wEnemyMonUnmodifiedMaxHP"></a>`wEnemyMonUnmodifiedMaxHP` | RL | The opponent's maximum HP before stat stages. |
| `$CD26` | 2 | <a id="s-wEnemyMonUnmodifiedAttack"></a>`wEnemyMonUnmodifiedAttack` | RL | Attack for the opponent before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD28` | 1 | <a id="s-wEnemyMonUnmodifiedDefense"></a>`wEnemyMonUnmodifiedDefense` | L | Defense for the opponent before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD2A` | 2 | <a id="s-wEnemyMonUnmodifiedSpeed"></a>`wEnemyMonUnmodifiedSpeed` | RL | Speed for the opponent before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD2C` | 1 | <a id="s-wEnemyMonUnmodifiedSpecial"></a>`wEnemyMonUnmodifiedSpecial` | L | Special for the opponent before any stat stage was applied. The battle struct's copy is the **modified** value — the one the damage formula uses — and the stages are integer ratios applied with truncation, so the original cannot be recovered by dividing back out. It is kept here instead. |
| `$CD2D` | 1 | <a id="s-wEngagedTrainerClass"></a>`wEngagedTrainerClass` | RL |  |
| `$CD2E` | 1 | <a id="s-wEngagedTrainerSet"></a>`wEngagedTrainerSet`<br><a id="s-wEnemyMonAttackMod"></a>`wEnemyMonAttackMod` <a id="s-wEnemyMonStatMods"></a>`wEnemyMonStatMods` | RL | **`wEngagedTrainerSet`** — Shares this address with `wEnemyMonAttackMod`/`wEnemyMonStatMods` — see those entries. **`wEnemyMonAttackMod`** — Attack stage for the opponent, same bias. **`wEnemyMonStatMods`** — In-battle stat stages for the opponent, same 1-13 range with 7 neutral. |
| `$CD2F` | 1 | <a id="s-wEnemyMonDefenseMod"></a>`wEnemyMonDefenseMod` | L | Defense stage for the opponent, same bias. |
| `$CD30` | 1 | <a id="s-wEnemyMonSpeedMod"></a>`wEnemyMonSpeedMod` | L | Speed stage for the opponent, same bias. |
| `$CD31` | 1 | <a id="s-wEnemyMonSpecialMod"></a>`wEnemyMonSpecialMod` | RL | Special stage for the opponent, same bias. |
| `$CD32` | 1 | <a id="s-wEnemyMonAccuracyMod"></a>`wEnemyMonAccuracyMod` | RL | Accuracy stage for the opponent, same bias. |
| `$CD33` | 1 | <a id="s-wEnemyMonEvasionMod"></a>`wEnemyMonEvasionMod` | RL | Evasion stage for the opponent, same bias. |
| `$CD36` | 1 | <a id="s-wEnemyMonStatModsEnd"></a>`wEnemyMonStatModsEnd` | L | One past the opponent's six. |
| `$CD47` | 1 | <a id="s-wBattleTransitionSpiralDirection"></a>`wBattleTransitionSpiralDirection` | RL |  |
| `$CD6D` | 4 | <a id="s-wBattleMenuCurrentPP"></a>`wBattleMenuCurrentPP`<br><a id="s-wEvoDataBuffer"></a>`wEvoDataBuffer` <a id="s-wMoveData"></a>`wMoveData` <a id="s-wNameBuffer"></a>`wNameBuffer` <a id="s-wPayDayMoney"></a>`wPayDayMoney` | RL |  |
| `$CEEB` | 1 | <a id="s-wHPBarOldHP"></a>`wHPBarOldHP`<br><a id="s-wAlphabetCase"></a>`wAlphabetCase` <a id="s-wEvoMonTileOffset"></a>`wEvoMonTileOffset` | RL |  |
| `$CEED` | 2 | <a id="s-wHPBarNewHP"></a>`wHPBarNewHP`<br><a id="s-wNamingScreenLetter"></a>`wNamingScreenLetter` | RL |  |
| `$CEEF` | 1 | <a id="s-wHPBarDelta"></a>`wHPBarDelta` | RL |  |
| `$CEF0` | 13 | <a id="s-wHPBarTempHP"></a>`wHPBarTempHP` | RL |  |
| `$CEFD` | 8 | <a id="s-wHPBarHPDifference"></a>`wHPBarHPDifference` | RL |  |
| `$CF05` | 1 | <a id="s-wAIItem"></a>`wAIItem` | RL |  |
| `$CF07` | 1 | <a id="s-wAnimSoundID"></a>`wAnimSoundID` | RL |  |
| `$CF0B` | 1 | <a id="s-wBattleResult"></a>`wBattleResult` | RL | How the battle ended: won, lost, or ran. |
| `$CF4B` | 2 | <a id="s-wExpAmountGained"></a>`wExpAmountGained`<br><a id="s-wStringBuffer"></a>`wStringBuffer` | RL | **`wExpAmountGained`** — Shares this address with `wStringBuffer` — see that entry. **`wStringBuffer`** — Scratch for a string being assembled before it is placed. |
| `$CFD8` | 1 | <a id="s-wEnemyMonSpecies2"></a>`wEnemyMonSpecies2` | RL | The species the battle initialiser branches on, written before `wEnemyMon` is filled. |
| `$CFD9` | 1 | <a id="s-wBattleMonSpecies2"></a>`wBattleMonSpecies2` | RL | Your active Pokemon's species, kept outside the structure. |
| `$CFDA` | 11 | <a id="s-wEnemyMonNick"></a>`wEnemyMonNick` | RL | The nickname shown above the opponent. |
| `$CFE5` | 1 | <a id="s-wEnemyMonSpecies"></a>`wEnemyMonSpecies`<br><a id="s-wEnemyMon"></a>`wEnemyMon` | RL | **`wEnemyMonSpecies`** — Species **internal index** — not the Pokedex number. `PokedexOrder` converts. **`wEnemyMon`** — The opponent's active Pokemon, the same 29-byte `battle_struct`. |
| `$CFE6` | 2 | <a id="s-wEnemyMonHP"></a>`wEnemyMonHP` | RL | Current HP, **big-endian**. |
| `$CFE8` | 1 | <a id="s-wEnemyMonBoxLevel"></a>`wEnemyMonBoxLevel`<br><a id="s-wEnemyMonPartyPos"></a>`wEnemyMonPartyPos` | RL | **`wEnemyMonBoxLevel`** — The level as stored. Kept in step with `Level` for a party Pokemon; for a stored one it is the only level there is. **`wEnemyMonPartyPos`** — Which party slot this battler came from. Occupies the byte a party structure uses for `BoxLevel`. |
| `$CFE9` | 1 | <a id="s-wEnemyMonStatus"></a>`wEnemyMonStatus` | RL | Sleep, poison, burn, freeze, paralysis, as one byte of flags and a sleep counter. |
| `$CFEA` | 1 | <a id="s-wEnemyMonType1"></a>`wEnemyMonType1`<br><a id="s-wEnemyMonType"></a>`wEnemyMonType` | RL | **`wEnemyMonType1`** — First type. **`wEnemyMonType`** — The opponent's two type bytes as a pair, `Type1` then `Type2`. *(×2, stride 1)* |
| `$CFEB` | 1 | <a id="s-wEnemyMonType2"></a>`wEnemyMonType2` | RL | Second type; equal to `Type1` for a single-typed Pokemon. |
| `$CFEC` | 1 | <a id="s-wEnemyMonCatchRate"></a>`wEnemyMonCatchRate` | L | The species' base catch rate, copied in at capture. Held item in later generations; here it is the ball maths. |
| `$CFED` | 4 | <a id="s-wEnemyMonMoves"></a>`wEnemyMonMoves` | RL | Four move ids, 0 for an empty slot. |
| `$CFF1` | 2 | <a id="s-wEnemyMonDVs"></a>`wEnemyMonDVs` | RL | Two bytes, four nybbles: attack, defense, speed, special. **The HP DV is not stored** — it is assembled from bit 0 of the other four, most significant first. These sixteen bits are the whole of what makes one Pokemon differ from another of its species and level. |
| `$CFF3` | 1 | <a id="s-wEnemyMonLevel"></a>`wEnemyMonLevel` | RL | The level the stats below were computed at. |
| `$CFF4` | 2 | <a id="s-wEnemyMonMaxHP"></a>`wEnemyMonMaxHP`<br><a id="s-wEnemyMonStats"></a>`wEnemyMonStats` | RL | **`wEnemyMonMaxHP`** — Computed maximum HP, big-endian. **`wEnemyMonStats`** — Label for the five computed stats as one run. |
| `$CFF6` | 2 | <a id="s-wEnemyMonAttack"></a>`wEnemyMonAttack` | RL | Computed Attack, big-endian. |
| `$CFF8` | 2 | <a id="s-wEnemyMonDefense"></a>`wEnemyMonDefense` | RL | Computed Defense, big-endian. |
| `$CFFA` | 2 | <a id="s-wEnemyMonSpeed"></a>`wEnemyMonSpeed` | RL | Computed Speed, big-endian. |
| `$CFFC` | 2 | <a id="s-wEnemyMonSpecial"></a>`wEnemyMonSpecial` | RL | Computed Special, big-endian. One stat here; Gen 2 splits it in two. |
| `$CFFE` | 4 | <a id="s-wEnemyMonPP"></a>`wEnemyMonPP` | RL | Four bytes: current PP in the low 6 bits, PP Ups applied in the top 2. |
| `$D002` | 5 | <a id="s-wEnemyMonBaseStats"></a>`wEnemyMonBaseStats` | RL |  |
| `$D007` | 1 | <a id="s-wEnemyMonActualCatchRate"></a>`wEnemyMonActualCatchRate` | RL | The catch rate the ball routine actually reads — **not** the species' base rate, because Safari bait and rocks move it. |
| `$D008` | 1 | <a id="s-wEnemyMonBaseExp"></a>`wEnemyMonBaseExp` | R | The opponent's species' base experience yield, copied in when the battle starts so the reward can be computed without re-reading the species table afterward. Measured across a real playthrough (walk, a forced battle, menus): changed value, during battle-trigger; values seen: [204]. |
| `$D009` | 11 | <a id="s-wBattleMonNick"></a>`wBattleMonNick` | RL | The nickname shown above your side of the battle. |
| `$D014` | 1 | <a id="s-wBattleMonSpecies"></a>`wBattleMonSpecies`<br><a id="s-wBattleMon"></a>`wBattleMon` | RL | **`wBattleMonSpecies`** — Species **internal index** — not the Pokedex number. `PokedexOrder` converts. **`wBattleMon`** — Your active Pokemon as the battle engine sees it: a 29-byte `battle_struct`, **not** the 44-byte party shape. Original trainer, experience and stat experience are gone, so the level and the five stats sit at different offsets — confusing the two is the classic way to produce a level 6,400 Pokemon. The level here is also what a critical hit doubles: Generation 1 substitutes twice the level into the damage formula, so the worst critical outcome is above the best ordinary one. |
| `$D015` | 2 | <a id="s-wBattleMonHP"></a>`wBattleMonHP` | RL | Current HP, **big-endian**. |
| `$D017` | 1 | <a id="s-wBattleMonBoxLevel"></a>`wBattleMonBoxLevel`<br><a id="s-wBattleMonPartyPos"></a>`wBattleMonPartyPos` | L | **`wBattleMonBoxLevel`** — The level as stored. Kept in step with `Level` for a party Pokemon; for a stored one it is the only level there is. **`wBattleMonPartyPos`** — Which party slot this battler came from. Occupies the byte a party structure uses for `BoxLevel`. |
| `$D018` | 1 | <a id="s-wBattleMonStatus"></a>`wBattleMonStatus` | RL | Sleep, poison, burn, freeze, paralysis, as one byte of flags and a sleep counter. |
| `$D019` | 1 | <a id="s-wBattleMonType1"></a>`wBattleMonType1`<br><a id="s-wBattleMonType"></a>`wBattleMonType` | RL | **`wBattleMonType1`** — First type. **`wBattleMonType`** — Your active Pokemon's two type bytes as a pair. *(×2, stride 1)* |
| `$D01A` | 1 | <a id="s-wBattleMonType2"></a>`wBattleMonType2` | RL | Second type; equal to `Type1` for a single-typed Pokemon. |
| `$D01B` | 1 | <a id="s-wBattleMonCatchRate"></a>`wBattleMonCatchRate` | L | The species' base catch rate, copied in at capture. Held item in later generations; here it is the ball maths. |
| `$D01C` | 4 | <a id="s-wBattleMonMoves"></a>`wBattleMonMoves` | RL | Four move ids, 0 for an empty slot. |
| `$D020` | 2 | <a id="s-wBattleMonDVs"></a>`wBattleMonDVs` | RL | Two bytes, four nybbles: attack, defense, speed, special. **The HP DV is not stored** — it is assembled from bit 0 of the other four, most significant first. These sixteen bits are the whole of what makes one Pokemon differ from another of its species and level. |
| `$D022` | 1 | <a id="s-wBattleMonLevel"></a>`wBattleMonLevel` | RL | The level the stats below were computed at. |
| `$D023` | 2 | <a id="s-wBattleMonMaxHP"></a>`wBattleMonMaxHP`<br><a id="s-wBattleMonStats"></a>`wBattleMonStats` | RL | **`wBattleMonMaxHP`** — Computed maximum HP, big-endian. **`wBattleMonStats`** — Label for the five computed stats as one run. |
| `$D025` | 2 | <a id="s-wBattleMonAttack"></a>`wBattleMonAttack` | RL | Computed Attack, big-endian. |
| `$D027` | 2 | <a id="s-wBattleMonDefense"></a>`wBattleMonDefense` | RL | Computed Defense, big-endian. |
| `$D029` | 2 | <a id="s-wBattleMonSpeed"></a>`wBattleMonSpeed` | RL | Computed Speed, big-endian. |
| `$D02B` | 2 | <a id="s-wBattleMonSpecial"></a>`wBattleMonSpecial` | RL | Computed Special, big-endian. One stat here; Gen 2 splits it in two. |
| `$D02D` | 4 | <a id="s-wBattleMonPP"></a>`wBattleMonPP` | RLI | Four bytes: current PP in the low 6 bits, PP Ups applied in the top 2. |
| `$D057` | 1 | <a id="s-wIsInBattle"></a>`wIsInBattle` | RL | What kind of battle is running: 0 none, 1 wild, 2 trainer — and **`$FF` means the player blacked out**, which is neither. A catch-all arm that treats anything non-zero as a trainer battle will show a battle panel in the overworld full of stale data. |
| `$D059` | 1 | <a id="s-wCurOpponent"></a>`wCurOpponent` | RL | **The byte that starts a battle.** The overworld loop polls it every frame; non-zero means fight. Below 200 it is a species (a wild battle at `wCurEnemyLevel`), at or above 200 it is a trainer class plus that offset. This is what the game's own debug menu writes. |
| `$D05A` | 1 | <a id="s-wBattleType"></a>`wBattleType` | RL | 0 normal, 1 the old man's tutorial, 2 Safari Zone. The tutorial is a real wild battle with real rolled DVs, but its Pokemon can never be added to the party. |
| `$D05B` | 1 | <a id="s-wDamageMultipliers"></a>`wDamageMultipliers` | RL |  |
| `$D05C` | 1 | <a id="s-wGymLeaderNo"></a>`wGymLeaderNo`<br><a id="s-wLoneAttackNo"></a>`wLoneAttackNo` | RL |  |
| `$D05D` | 1 | <a id="s-wTrainerNo"></a>`wTrainerNo` | RL | Which roster of the trainer class to build, once `wCurOpponent` has named the class. |
| `$D05E` | 1 | <a id="s-wCriticalHitOrOHKO"></a>`wCriticalHitOrOHKO` | RL | Whether the damage calculation produced a critical hit or a one-hit knockout. |
| `$D05F` | 1 | <a id="s-wMoveMissed"></a>`wMoveMissed` | RL | Set when the accuracy check failed. Gen 1 gives even a 100%-accuracy move a 1-in-256 chance of landing here. |
| `$D062` | 1 | <a id="s-wPlayerBattleStatus1"></a>`wPlayerBattleStatus1` | RL | Volatile battle state for your side: confusion, trapping, flinching and the rest, one bit each. *(×3, stride 1)* |
| `$D063` | 1 | <a id="s-wPlayerBattleStatus2"></a>`wPlayerBattleStatus2` | RL | The second volatile byte for your side. It carries `GETTING_PUMPED`, the FOCUS ENERGY bit, with no counter beside it — and in Generation 1 that bit **quarters** the critical-hit rate instead of raising it, because `CriticalHitTest` shifts the wrong way. Measured for a base-Speed-140 Electrode: 0.2604 of 960 attacks crit without it, 0.0612 of 588 with. |
| `$D064` | 1 | <a id="s-wPlayerBattleStatus3"></a>`wPlayerBattleStatus3` | RL | The third volatile byte for your side. Bit 1 is LIGHT SCREEN and bit 2 is REFLECT, and in Generation 1 **both belong to the Pokemon, not to the side**: `SendOutMon` zeroes this byte, so a screen does not survive a switch. Measured driving a real voluntary switch: `$04` with REFLECT up, `$00` immediately afterwards, with the opponent's copy at `$00` throughout. |
| `$D067` | 1 | <a id="s-wEnemyBattleStatus1"></a>`wEnemyBattleStatus1` | RL | The same volatile state for the opponent. *(×3, stride 1)* |
| `$D068` | 1 | <a id="s-wEnemyBattleStatus2"></a>`wEnemyBattleStatus2` | RL | The opponent's copy of the second volatile byte. |
| `$D069` | 1 | <a id="s-wEnemyBattleStatus3"></a>`wEnemyBattleStatus3` | RL | The opponent's copy of the third volatile byte, screens included. |
| `$D06A` | 1 | <a id="s-wPlayerNumAttacksLeft"></a>`wPlayerNumAttacksLeft` | RL | Turns left on a multi-turn move. **A trapping move that misses leaves this standing**: `TrappingEffect` runs before the hit test and `MoveHitTest`'s miss path clears the status flag without clearing the counter, so the next roll looks exactly like a decrement of the old one. Key a measurement on the flag, not on this byte. |
| `$D06D` | 2 | <a id="s-wPlayerDisabledMove"></a>`wPlayerDisabledMove` | RL | Which of your moves DISABLE has locked, and for how long, packed into one byte: the move slot 1-4 in the high nybble and the countdown in the low one. **Measured over 136 applications**: the countdown is drawn uniformly from 1 to 8, and the move menu refuses the slot for as long as the byte is non-zero. |
| `$D06F` | 1 | <a id="s-wEnemyNumAttacksLeft"></a>`wEnemyNumAttacksLeft` | RL | Turns left on the opponent's multi-turn move, and it carries the same trap as your side's copy: a trapping move that misses clears the flag and leaves this standing. |
| `$D072` | 2 | <a id="s-wEnemyDisabledMove"></a>`wEnemyDisabledMove` | RL | The same for the opponent. |
| `$D078` | 1 | <a id="s-wEscapedFromBattle"></a>`wEscapedFromBattle`<br><a id="s-wBattleStatusDataEnd"></a>`wBattleStatusDataEnd` | RL | Set when a run attempt succeeded. |
| `$D07C` | 1 | <a id="s-wAnimationID"></a>`wAnimationID`<br><a id="s-wDefaultMap"></a>`wDefaultMap` <a id="s-wMenuItemOffset"></a>`wMenuItemOffset` | RL |  |
| `$D081` | 1 | <a id="s-wBaseCoordX"></a>`wBaseCoordX` | RL |  |
| `$D082` | 1 | <a id="s-wBaseCoordY"></a>`wBaseCoordY` | RL |  |
| `$D083` | 1 | <a id="s-wLowHealthAlarm"></a>`wLowHealthAlarm` | RL | The state of the low-HP beep, which is its own channel and outlives the battle music. |
| `$D084` | 1 | <a id="s-wFBTileCounter"></a>`wFBTileCounter` | RL | How many tiles the current battle-transition animation has drawn so far. Measured across a real playthrough (walk, a forced battle, menus): written from 3 places, chiefly bank 30 `$4016`, during battle, intro; 14 distinct values seen, 0-16. |
| `$D09C` | 2 | <a id="s-wFBDestAddr"></a>`wFBDestAddr` | RL |  |
| `$D09E` | 1 | <a id="s-wFBMode"></a>`wFBMode` | RL |  |
| `$D0D7` | 4 | <a id="s-wDamage"></a>`wDamage` | RL | Damage for the current hit, big-endian. |
| `$D120` | 1 | <a id="s-wNumRunAttempts"></a>`wNumRunAttempts` | RL | How many times you have tried to run this battle; each attempt makes the next more likely. |
| `$D127` | 1 | <a id="s-wCurEnemyLevel"></a>`wCurEnemyLevel` | RL | The level a wild opponent is generated at. |
| `$D156` | 1 | <a id="s-wEvoStoneItemID"></a>`wEvoStoneItemID` | RL |  |
| `$D61F` | 1 | <a id="s-wSafariZoneGateCurScript"></a>`wSafariZoneGateCurScript` | RL |  |
| `$D70D` | 2 | <a id="s-wSafariSteps"></a>`wSafariSteps` | RL | Steps left in the current Safari Zone run. |
| `$D713` | 1 | <a id="s-wEnemyMonOrTrainerClass"></a>`wEnemyMonOrTrainerClass` | RL |  |
| `$D89C` | 1 | <a id="s-wEnemyPartyCount"></a>`wEnemyPartyCount` | RL | How many Pokemon the opposing trainer has. |
| `$D89D` | 7 | <a id="s-wEnemyPartySpecies"></a>`wEnemyPartySpecies` | RL | The opposing team's species list, `$FF`-terminated, mirroring `wPartySpecies`. |
| `$D8A4` | 1 | <a id="s-wEnemyMon1Species"></a>`wEnemyMon1Species`<br><a id="s-wEnemyMon1"></a>`wEnemyMon1` <a id="s-wEnemyMons"></a>`wEnemyMons` <a id="s-wWaterRate"></a>`wWaterRate` | RL | **`wEnemyMon1Species`** — Species **internal index** — not the Pokedex number. **`wEnemyMon1`** — The opposing trainer's first Pokemon, a whole 44-byte `party_struct` — the *party* shape, not the 29-byte battle shape. Shares its storage with the map's wild encounter tables, so it only means anything once `wIsInBattle` says a trainer battle is running. **`wEnemyMons`** — The opposing team as six 44-byte `party_struct`s — the *party* shape, not the battle shape. Shares storage with the map's wild encounter tables. **`wWaterRate`** — How often surfing rolls an encounter. *(×6, stride 44)* |
| `$D8A5` | 2 | <a id="s-wEnemyMon1HP"></a>`wEnemyMon1HP`<br><a id="s-wWaterMons"></a>`wWaterMons` | RL | **`wEnemyMon1HP`** — Current HP, **big-endian**. **`wWaterMons`** — The map's wild encounter table for water. *(×6, stride 44)* |
| `$D8A7` | 1 | <a id="s-wEnemyMon1BoxLevel"></a>`wEnemyMon1BoxLevel` | L | The level as stored, kept in step with `Level`. *(×6, stride 44)* |
| `$D8A8` | 1 | <a id="s-wEnemyMon1Status"></a>`wEnemyMon1Status` | RL | Sleep, poison, burn, freeze, paralysis, as one byte of flags and a sleep counter. *(×6, stride 44)* |
| `$D8A9` | 1 | <a id="s-wEnemyMon1Type1"></a>`wEnemyMon1Type1`<br><a id="s-wEnemyMon1Type"></a>`wEnemyMon1Type` | L | **`wEnemyMon1Type1`** — First type. **`wEnemyMon1Type`** — The two type bytes as a pair. *(×6, stride 44)* |
| `$D8AB` | 1 | <a id="s-wEnemyMon1CatchRate"></a>`wEnemyMon1CatchRate` | L | The species' base catch rate, carried in the structure. Fixed for a trainer's Pokemon, which is one of the things that makes a trainer battle different from a wild one. *(×6, stride 44)* |
| `$D8AC` | 4 | <a id="s-wEnemyMon1Moves"></a>`wEnemyMon1Moves` | RL | Four move ids, 0 for an empty slot. *(×6, stride 44)* |
| `$D8B0` | 2 | <a id="s-wEnemyMon1OTID"></a>`wEnemyMon1OTID` | L | The original trainer's ID, **big-endian**. *(×6, stride 44)* |
| `$D8B2` | 3 | <a id="s-wEnemyMon1Exp"></a>`wEnemyMon1Exp` | L | Experience, **three bytes big-endian**. *(×6, stride 44)* |
| `$D8B5` | 2 | <a id="s-wEnemyMon1HPExp"></a>`wEnemyMon1HPExp` | L | Stat experience for HP, big-endian. *(×6, stride 44)* |
| `$D8B7` | 2 | <a id="s-wEnemyMon1AttackExp"></a>`wEnemyMon1AttackExp` | L | Stat experience for Attack, big-endian. *(×6, stride 44)* |
| `$D8B9` | 2 | <a id="s-wEnemyMon1DefenseExp"></a>`wEnemyMon1DefenseExp` | L | Stat experience for Defense, big-endian. *(×6, stride 44)* |
| `$D8BB` | 2 | <a id="s-wEnemyMon1SpeedExp"></a>`wEnemyMon1SpeedExp` | L | Stat experience for Speed, big-endian. *(×6, stride 44)* |
| `$D8BD` | 2 | <a id="s-wEnemyMon1SpecialExp"></a>`wEnemyMon1SpecialExp` | L | Stat experience for Special, big-endian. *(×6, stride 44)* |
| `$D8BF` | 2 | <a id="s-wEnemyMon1DVs"></a>`wEnemyMon1DVs` | L | Two bytes, four nybbles: attack, defense, speed, special. **A trainer's Pokemon has fixed DVs** — `LoadEnemyMonData` writes the same two bytes into every one of them — so anything that draws a conclusion from DVs has to exclude trainer battles or it will report the same answer forever. *(×6, stride 44)* |
| `$D8C1` | 4 | <a id="s-wEnemyMon1PP"></a>`wEnemyMon1PP` | RL | Four bytes: current PP in the low 6 bits, PP Ups applied in the top 2. *(×6, stride 44)* |
| `$D8C5` | 1 | <a id="s-wEnemyMon1Level"></a>`wEnemyMon1Level` | RL | The level the stats below were computed at. *(×6, stride 44)* |
| `$D8C6` | 2 | <a id="s-wEnemyMon1MaxHP"></a>`wEnemyMon1MaxHP`<br><a id="s-wEnemyMon1Stats"></a>`wEnemyMon1Stats` | RL | **`wEnemyMon1MaxHP`** — Computed maximum HP, big-endian. **`wEnemyMon1Stats`** — The five computed stats, in order, big-endian. *(×6, stride 44)* |
| `$D8C8` | 2 | <a id="s-wEnemyMon1Attack"></a>`wEnemyMon1Attack` | L | Computed Attack, big-endian. *(×6, stride 44)* |
| `$D8CA` | 2 | <a id="s-wEnemyMon1Defense"></a>`wEnemyMon1Defense` | L | Computed Defense, big-endian. *(×6, stride 44)* |
| `$D8CC` | 2 | <a id="s-wEnemyMon1Speed"></a>`wEnemyMon1Speed` | L | Computed Speed, big-endian. *(×6, stride 44)* |
| `$D8CE` | 2 | <a id="s-wEnemyMon1Special"></a>`wEnemyMon1Special` | L | Computed Special, big-endian. *(×6, stride 44)* |
| `$D9AC` | 11 | <a id="s-wEnemyMon1OT"></a>`wEnemyMon1OT`<br><a id="s-wEnemyMonOT"></a>`wEnemyMonOT` | RL | **`wEnemyMon1OT`** — Slot 1's original-trainer name, 11 bytes. **`wEnemyMonOT`** — The opposing team's original-trainer names. *(×6, stride 11)* |
| `$D9EE` | 11 | <a id="s-wEnemyMon1Nick"></a>`wEnemyMon1Nick`<br><a id="s-wEnemyMonNicks"></a>`wEnemyMonNicks` | RL | **`wEnemyMon1Nick`** — Slot 1's nickname, 11 bytes. **`wEnemyMonNicks`** — The opposing team's nicknames. *(×6, stride 11)* |
| `$DA46` | 1 | <a id="s-wSafariZoneGameOver"></a>`wSafariZoneGameOver` | RL |  |

<!-- atlas:end (table) -->
