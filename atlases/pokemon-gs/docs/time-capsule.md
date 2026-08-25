# The Time Capsule — the only door between the generations

[← Gold/Silver](../README.md) · [AtlasGB](../../../README.md)

The Time Capsule is Gold and Silver's one sanctioned link to Red, Blue and Yellow. Every fact
on this page comes from a `pokegold` disassembly built and checked byte-for-byte against a
retail cartridge, plus one real trade carried out between two emulated consoles running two
built, verified cartridges.

## Gen 2 refuses a Gen 1 partner at the ordinary trade desk

Gold's normal trade receptionist reads a link-mode byte from the partner and, if it names a
Generation 1 console, refuses with *"You can't link to the past."* The Time Capsule
receptionist is the mirror image: she requires that same byte to be **zero**. There is no
"just link them and see" — a Generation 1 console reaching a Generation 2 game has exactly one
legitimate destination, and picking the ordinary desk is a refusal, not a failure to diagnose.

## The wire is Generation 1's protocol, unchanged

In Time Capsule mode, Generation 2 speaks Generation 1's own link protocol rather than a new
one: the same three-block exchange — a random-number preamble, a party block, patch lists — and
the party block itself is built in **Generation 1's format**: six preamble bytes, an 11-byte
player name, the party count, an `$FF`-terminated species list, then 44-byte structures (not
Generation 2's 48-byte one), then original-trainer names and nicknames. Species indices are
converted on the way out and back by dedicated routines.

This single fact is why a cartridge-accurate relay of the physical link cable can carry a real
Generation 1 ↔ Generation 2 trade with **no Time Capsule-specific logic written for it at all**
— the wire itself does not know what generation it is carrying, only Generation 2's own game
code does the translating.

## The in-game prerequisites

Three conditions gate the Time Capsule receptionist, and two of the three read the opposite way
from what their names suggest:

**Bill must have been met — which means a flag named "met Bill" is clear, not set.** A scene at
Ecruteak City's Pokémon Center clears that flag and sets a separate "Time Capsule" flag when
Bill activates it; the receptionist refuses *while the "met Bill" flag is still set*, i.e.
before that scene has played. Ecruteak is the fourth gym town — roughly three badges and
several hours in.

**A day must have passed — which means a daily flag is clear, not set.** The flag Bill's scene
sets is a *daily* flag; the game's own daily-reset check clears every daily flag once a day has
elapsed on the cartridge's real-time clock. Until then, the receptionist says the Time Capsule
is "being adjusted." So "the flag Bill sets, now clear" is the game's way of encoding "and then
a day went by."

**A party the Generation 1 side can represent.** A dedicated compatibility check inspects the
Generation 2 player's own party before the link opens and refuses the whole party — not just
the offending member — for any one of: a Pokémon species introduced in Generation 2, a move
introduced after Generation 1, or a held Mail item. **All three Johto starter species are
Generation 2 species**, so a save that has been played normally very often cannot use the Time
Capsule while carrying its starter, independent of whether the two gates above are open.

**On the Generation 1 side, nothing about the Time Capsule changes the ordinary rules** — the
Pokédex is required to use the Cable Club at all, and the player picks the same TRADE CENTER
option a normal trade uses. The Time Capsule receptionist stands on every Pokémon Center's
second floor, at the same tile, so any Center works once Bill has been met.

## The link window

The receptionist's script opens a wait for the linked partner **before** the save prompt,
answering the link-role handshake as an internally-clocked slave for **767 frames**, about 13
seconds — which is exactly the handshake byte a Generation 1 master is looking for. So the
entire two-console choreography reduces to one gate: the Generation 1 side may not initiate
until the Generation 2 side's player is standing at the desk, because **a Generation 2 console
answers handshake bytes from its overworld serial handler**, not only from the receptionist
scene — an ungated Generation 1 partner completes a handshake against a Gold that is still
walking around, then proceeds alone and desyncs, with no error printed on either side.

## Which seat is yours is a byte, not geometry

The Time Capsule room seats two consoles, at two tiles each restricted to firing only when the
player faces into it from one specific direction — giving two fixed player seats, one facing
right and one facing left. Which one is actually free is decided by
[`hSerialConnectionStatus`](memory-map.md#the-link) — the internal-clock console takes one
seat, the external-clock console the other — not by where either player happens to arrive from.
Guessing from the arrival coordinate instead has been measured to walk a bot straight into the
partner's own avatar, where it stands blocked indefinitely while that avatar's screen reports
waiting for a partner, with nothing anywhere reporting an error.

**The trade-selection screen itself reads direction presses as cursor movement, not player
movement**, once it is open — a fact anything driving both a seat-facing step and a Pokémon
selection needs to respect at the boundary between the two.

## The shiny rule

The reason to trace the Time Capsule at all: settling what a Generation 1 Pokémon's trainer
panel means by "would be shiny if traded to Generation 2." Read directly out of the
disassembly's own shininess check rather than inferred from one trade: a Pokémon is shiny by
this rule when its Defense, Speed and Special DVs are each exactly 10 and its Attack DV has one
particular bit set — eight Attack values out of sixteen, with three DVs fixed, giving
8 ⁄ 65,536 = **1 in 8,192**, the figure the community has long quoted.

Reading the rule out of the cartridge covers all 65,536 possible DV combinations by
construction; one trade only samples one of them — which is why this is the stronger form of
evidence, not merely a second way of stating the same thing a trade would show. Two
consequences follow directly: **a Generation 1 Pokémon's DVs cross the Time Capsule unchanged**,
so the verdict is knowable before any trade happens, and **Generation 1 has no shiny Pokémon at
all**, so the verdict must always be phrased conditionally — "would be shiny if traded", never
"is shiny."

## Quirks and honest gaps

**The item slot that Generation 1 uses for catch rate becomes a held item on arrival.** A
Pokémon crossing from Generation 1 arrives holding an item whose id equals its Generation 1
catch rate — documented Time Capsule behaviour, not a conversion defect, and a direct
consequence of the two party structures placing different concepts at the same offset (see
[the memory map](memory-map.md#the-48-byte-party-structure-as-far-as-this-page-establishes-it)).

**Leaving the Time Capsule room afterward is asymmetric between the two generations, and
neither side is broken.** Generation 2's Time Capsule room has ordinary warps back out;
Generation 1's Trade/Colosseum-style room has none at all, and its player instead has to cancel
out and use the console's own reset-into-menu path — the same mechanism
[Pokémon Red/Blue's own Cable Club findings](../../pokemon-rb/docs/link.md) describe for leaving
any linked room on that side.

**Not established here:** what a Generation 2 game does when handed a Generation 1 party it
cannot represent (only the reverse direction has been checked); Pokémon Yellow as the
Generation 1 side; Pokémon Silver or Crystal as the Generation 2 side; and Mail, eggs, and
whatever else the compatibility check screens for beyond the three results already stated.

## See also

- [the memory map](memory-map.md) — every address named on this page.
- [the save file](the-save-file.md) — how a save is built to reach the Time Capsule desk
  without playing the several hours it normally takes.
- [Pokémon Red/Blue's own link chapter](../../pokemon-rb/docs/link.md) — the Generation 1 half of
  this same protocol, with its own evidence tiers.
