# A paper's Gen 1 claims, checked — Pokémon Red/Blue

[← Pokémon Red/Blue](../README.md) · [by address](by-address.md) · [by name](by-name.md) · [discoveries](discoveries.md) · [AtlasGB](../../../README.md)

An outside paper's state-space derivation for Generation 1 competitive play asserts a list of
mechanics — how long a status lasts, how many states a volatile has, what a critical hit needs.
Those are claims about the cartridge, and this atlas exists to check claims about the cartridge
against it. Fifteen of them are gathered here: the eleven the paper states as counts or ranges,
plus four more it touches in passing.

**Most of them hold.** Where one does not, the verdict below says exactly how: **Confirmed** (the
cartridge does what the claim says), **Different convention** (the claim's number is right under
one honest reading of "how long", and a different reading gives a different number), or
**Disagreement** (no reading of the claim matches the cartridge). Every verdict is settled by a
`pret/pokered` routine or symbol, never by the claim's own plausibility.

---

## The eleven claims

| # | Claim | Verdict |
|---|---|---|
| 1 | [Toxic residual damage is `⌊N/16⌋ × maxHP` on turn *N*](#1-toxic-residual-damage) | **Disagreement** |
| 2 | [The toxic counter's range is 1–15](#2-the-toxic-counters-range) | **Disagreement** |
| 3 | [Reflect and Light Screen are per-Pokémon volatiles, not field-wide screens](#3-reflect-and-light-screen-are-per-pokémon-volatiles) | **Confirmed** |
| 4 | [Confusion is 5 states: none, or 1–4 turns](#4-confusion-is-5-states) | **Confirmed** *(different convention on the raw counter)* |
| 5 | [Disable is 29 states: off, or one of 4 moves × 1–7 turns](#5-disable-is-29-states) | **Different convention** |
| 6 | [Thrash lock is 1–3 turns = 3 states](#6-thrash-lock-is-3-states) | **Confirmed** *(counter-value convention)* |
| 7 | [Bide is 1–2 turns = 2 states](#7-bide-is-2-states) | **Different convention** |
| 8 | [Partial trapping lock is 1–4 turns = 4 states](#8-partial-trapping-lock-is-4-states) | **Confirmed** |
| 9 | [Focus Energy is a 2-state volatile](#9-focus-energy-is-a-2-state-volatile) | **Confirmed** |
| 10 | [What Focus Energy does](#10-what-focus-energy-does) | **Confirmed** *(and the effect is the famous trap)* |
| 11 | [The Gen 1 critical-hit mechanism](#11-the-gen-1-critical-hit-mechanism) | **Confirmed** |

One consequence follows from rows 6–8: the mutually-exclusive move-lock group (no lock, Thrash,
two-turn/Fly/Dig, must-recharge, Bide, partial trapping) sums to **13** states under the
counter-value reading the claim uses everywhere else, not 12 — because Bide contributes 3 and not
2 (see [claim 7](#7-bide-is-2-states)).

### 1. Toxic residual damage

**Claim.** The residual is `⌊N/16⌋ × maxHP` on the *N*th tick.

**Cartridge fact.** `HandlePoisonBurnLeechSeed_DecreaseOwnHP` (`engine/battle/core.asm`) computes
the operands the other way round and floors the base at 1: damage on the *N*th tick is
`max(1, ⌊maxHP/16⌋) × N`. Already documented on this atlas's
[`wPlayerToxicCounter`](scratch.md#s-wPlayerToxicCounter) entry, which gives the same formula. The
claimed form also can't be right on its own terms — `⌊N/16⌋` is zero for every *N* from 1 to 15,
so it would deal no damage at all until turn 16 and produce one trajectory rather than fifteen.

### 2. The toxic counter's range

**Claim.** 1–15.

**Cartridge fact.** There is no cap in the code: the counter is one byte, incremented and never
compared, per the same [`wPlayerToxicCounter`](scratch.md#s-wPlayerToxicCounter) entry. What
bounds it in play is arithmetic, not a check — a badly-poisoned Pokémon with no healing has lost
its whole HP bar by the sixth tick regardless of what that bar holds, and the entry records the
counter reaching **5** in a plain battle and **10** against a target that heals itself every turn.
15 is a reasonable ceiling for a state-space *estimate* but is not a hardware bound in either
direction: nothing in the code refuses 15, and nothing in ordinary play reaches it.

### 3. Reflect and Light Screen are per-Pokémon volatiles

**Claim.** Reflect and Light Screen belong to the Pokémon, not to the side — unlike later
generations, where they are field-wide and survive a switch.

**Verdict and citation only** — this is a full entry elsewhere and the claim is exactly what
settled it: [a screen does not survive a switch](discoveries.md#a-screen-does-not-survive-a-switch)
in this atlas's discoveries, backed by [`wPlayerBattleStatus3`](battle.md#s-wPlayerBattleStatus3).
The same measurement that confirmed the paper's claim is what caught this project's own battle
engine getting it backwards.

### 4. Confusion is 5 states

**Claim.** None, or 1–4 turns confused — 5 states.

**Cartridge fact.** `ConfusionSideEffectSuccess` rolls `BattleRandom & 3` and adds 2, so the *byte*
holds 2–5 when freshly applied, stepping down by one each of the confused Pokémon's turns; per
[`wPlayerConfusedCounter`](scratch.md#s-wPlayerConfusedCounter) (measured over 97 applications) the
Pokémon acts confused on `roll − 1` of those turns, i.e. 1–4, and is clear on the turn the byte
reaches zero. The state count is right; the raw counter in WRAM is not the number of turns — it's
one higher, because it counts down through zero rather than to it.

### 5. Disable is 29 states

**Claim.** Off, or one of 4 move slots locked for 1–7 turns — `1 + 4 × 7 = 29`.

**Cartridge fact.** `DisableEffect` rolls `BattleRandom & 7` and adds 1; per
[`wPlayerDisabledMove`](battle.md#s-wPlayerDisabledMove) (measured over 136 applications) the
countdown is drawn uniformly from **1 to 8**, not 1 to 7, and the slot is uniform over 1–4. Read as
"turns still blocked *after* the countdown" — the same reading that makes [claim 4](#4-confusion-is-5-states)
right — a roll of 1 leaves zero turns remaining and is indistinguishable from "off", so the
reachable non-off values are 1–7 and the claimed count of 29 follows. It is a consistent reading,
not the raw roll: the disassembly's own comment for the roll is *"1-8 turns disabled"*.

### 6. Thrash lock is 3 states

**Claim.** 1–3 turns locked into Thrash — 3 states.

**Cartridge fact.** `ThrashPetalDanceEffect` rolls one random bit and adds two, giving 2 or 3 —
documented on this atlas's [`wPlayerNumAttacksLeft`](battle.md#s-wPlayerNumAttacksLeft) entry
(measured over 161 locks) as the same routine shape Bide uses. The counter takes the values 1, 2, 3
while the lock is up, so a lock runs 3 or 4 attacking turns and the claimed 3 states matches under
the counter-value reading.

### 7. Bide is 2 states

**Claim.** 1–2 turns storing damage — 2 states.

**Cartridge fact.** `BideEffect` uses the *same byte, the same mask and the same two increments as
Thrash* — the [`wPlayerNumAttacksLeft`](battle.md#s-wPlayerNumAttacksLeft) entry's own wording —
rolling 2 or 3. Since it's the identical counter and roll as [claim 6](#6-thrash-lock-is-3-states),
Bide can't have a different state count from Thrash under any single reading: the counter takes 1,
2, 3, which is **3 states, not 2**. There is a reading that gives 1–2 — turns spent storing,
excluding both the Bide turn itself and the release turn — but it isn't the reading that makes
claims 4, 5, 6 and 8 correct, so it isn't consistent with how the paper counts everywhere else.
This is the one real discrepancy in the group, and it's what the [move-lock total](#the-eleven-claims)
above corrects.

### 8. Partial trapping lock is 4 states

**Claim.** 1–4 turns locked by Wrap or Bind — 4 states.

**Cartridge fact.** `TrappingEffect` rolls `BattleRandom & 3`, re-rolling on 2 or 3, then adds one —
its own comment says "3/8 chance for 2 and 3 attacks, and 1/8 chance for 4 and 5 attacks." Counter
values land 1–4, the claimed 4 states, over 371 measured locks per the shared
[`wPlayerNumAttacksLeft`](battle.md#s-wPlayerNumAttacksLeft) entry. One methodological trap sits
under this number: a trapping move that *misses* clears the "is trapping" flag but not the counter,
so a fresh roll after a miss looks like a decrement of the old one unless the measurement is keyed
on the flag rather than the byte — see
[a missed trapping move leaves its counter standing](discoveries.md#a-missed-trapping-move-leaves-its-counter-standing),
which used exactly this claim to find the trap (73 of those 371 locks, and 39 of 161 Thrash locks,
ended early that way).

### 9. Focus Energy is a 2-state volatile

**Claim.** Focus Energy is on or off, with no counter — 2 states.

**Cartridge fact.** `GETTING_PUMPED` is a single bit of
[`wPlayerBattleStatus2`](battle.md#s-wPlayerBattleStatus2) with no counter beside it. Confirmed
exactly as claimed.

### 10. What Focus Energy does

**Claim.** *(Context in the source, not a numbered claim of its own.)*

**Verdict and citation only** — a full entry already covers it:
[a critical hit is the worst case](discoveries.md#a-critical-hit-is-the-worst-case). The state
count in [claim 9](#9-focus-energy-is-a-2-state-volatile) is right; the semantics are the famous
Generation 1 trap that the same entry records — Focus Energy **quarters** the critical-hit rate
rather than raising it, because `CriticalHitTest` shifts the wrong way.

### 11. The Gen 1 critical-hit mechanism

**Claim.** *(A cross-check in the source, not a numbered claim of its own.)*

**Verdict and citation only** — same entry, [a critical hit is the worst case](discoveries.md#a-critical-hit-is-the-worst-case),
which gives the `CriticalHitTest` arithmetic and the measured rates against
[`wCriticalHitOrOHKO`](battle.md#s-wCriticalHitOrOHKO).

---

## Four more, not in the numbered list

The source appendix touches these in passing rather than as one of its eleven counted claims. All
four are cartridge facts this atlas already carries elsewhere; each gets one line and a link.

* **Substitute carries HP, not a flag.** The paper counts a substitute as 2 states.
  [`wPlayerSubstituteHP`](battle.md#s-wPlayerSubstituteHP) is a whole byte — a quantity, not a
  two-state flag.
* **Bide accumulates its stored damage in 16 bits.** The release deals damage with
  [`wPlayerBideAccumulatedDamage`](scratch.md#s-wPlayerBideAccumulatedDamage), which is a word, not
  a byte.
* **Minimize is real but cosmetic.** [`wPlayerMonMinimized`](battle.md#s-wPlayerMonMinimized) is 2
  states as the paper says, but in Generation 1 it only changes how the sprite is drawn — there is
  no Stomp-style damage doubling against it until a later generation.
* **Sleep is confirmed at 1–7 turns.** Already stated in this atlas's
  [known battle-engine bugs](battle.md#known-battle-engine-bugs): `SleepEffect` masks a random byte
  with `SLP_MASK` (7) and rerolls a zero, matching the claim exactly.

---

## See also

* [Discoveries](discoveries.md) — the faults, quirks and corrected beliefs this atlas's own
  evidence turned up, including the two entries this page leans on:
  [a screen does not survive a switch](discoveries.md#a-screen-does-not-survive-a-switch) and
  [a critical hit is the worst case](discoveries.md#a-critical-hit-is-the-worst-case).
* [The battle](battle.md) — the volatile bytes, the status counters and the known battle-engine
  bugs cited throughout this page.
