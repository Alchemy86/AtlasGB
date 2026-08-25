# The cable — Pokémon Red/Blue

[← Pokémon Red/Blue](../README.md) · [by address](by-address.md) · [by name](by-name.md) · [the structures](structures.md) · [AtlasGB](../../../README.md)

Two consoles agree by exchanging **nybbles**, and the state of that exchange is this chapter.

`wLinkState` is the byte everything hangs off, and its value is more than a label — three
routines branch on it directly:

| Symbol | Value | Meaning |
|---|---|---|
| `LINK_STATE_NONE` | `$00` | not using the link |
| `LINK_STATE_IN_CABLE_CLUB` | `$01` | in a Cable Club room |
| `LINK_STATE_START_TRADE` | `$02` | pre-trade initialisation |
| `LINK_STATE_START_BATTLE` | `$03` | pre-battle initialisation |
| `LINK_STATE_BATTLING` | `$04` | in a link battle |
| `LINK_STATE_TRADING` | `$32` | in a link trade |

It does **not** cross the cable — both consoles run the identical 644-byte exchange below and
diverge only afterwards, locally, off this one byte. That is exactly how a virtual link partner
that only knew how to trade once walked into a real link battle: nothing on the wire says a
battle is coming.

**Three places read `wLinkState` directly, and each changes behaviour a reader would otherwise
blame on something else.** `ReadTrainer` (`engine/battle/core.asm`) returns immediately when it
is nonzero, so a link battle's enemy party is never built from the cartridge's trainer data —
it is already sitting in `wEnemyMons`, put there by the received party block.
`LoadEnemyMonData` jumps straight to `LoadEnemyMonFromParty` at `LINK_STATE_BATTLING`, for the
same reason. And `BattleRandom` stops reading the hardware divider at `LINK_STATE_BATTLING`,
drawing instead from `wLinkBattleRandomNumberList` — a shared nine-entry list advanced by
`x*5+1` — because two consoles that disagreed about the dice would desync mid-battle.

**A trade moves three blocks, 644 bytes total.** `CableClub_DoBattleOrTradeAgain`
(`engine/link/cable_club.asm`) hands each to `Serial_ExchangeBytes`, preceded by a run of `$FD`
the receiver syncs on:

| Block | Preamble | Data | Total |
|---|---:|---:|---:|
| random-number list | 7 | 10 | 17 |
| party data | 6 | 418 | 424 |
| patch list | 3 | 197 | 200 |

Each block costs **`N + 1` transfers**, because `Serial_ExchangeBytes` keeps re-sending its
first byte until a `$FD` comes back before it starts storing. Off by one anywhere in that count
shifts the party block, and the game accepts whatever arrives without checking it — a trade has
no internal consistency check once the bytes are in. The 418-byte party block is the sender's
WRAM verbatim: trainer name (11), party count (1), species list plus an `$FF` terminator (7),
six 44-byte party-shaped structures (264), six OT names (66), six nicknames (66), and three
trailing bytes of whatever followed in memory.

**The patch list exists because `$FE` cannot travel as data.** `$FE` is the protocol's own
"no data" filler, skipped on receipt, and real Pokémon data contains it constantly — a stat of
254, a one-byte experience value. So before sending, the game walks the 264-byte Pokémon region,
replaces every `$FE` with `$FF`, and records the 1-based offset of each substitution; the
receiver reverses it once the block is in. The list is split at offset 252 into one part for
the Pokémon region and one for the rest, because an offset is a single byte and `$FD` — the
preamble — cannot be one of its values either. Get the substitution or the split wrong and it
fails silently: one byte of a Pokémon is altered, which is how a trade produces impossible
stats. A nickname or OT name containing `$FE` (the in-game digit `8`) is refused outright
rather than corrupted.

**A traded Pokémon's experience is boosted on arrival, unconditionally.** A level 5 Mew sent on
the wire with **125 experience** — the exact total for level 5 on Mew's own growth curve — reads
**135** once it is in the receiving console's party. Nothing in the exchange carries that
difference; Generation 1 applies it to any Pokémon that arrives through a trade, on the
receiving side, regardless of what came over the wire.

**While `wLinkState` is nonzero, the player is standing in a room with no exit tile at all.**
`data/maps/objects/TradeCenter.asm` declares `def_warp_events` with nothing under it, so
`wNumberOfWarps` reads **0** for that map; the Colosseum is the same. Two consequences follow
from that, and both are the cartridge, not a partner behaving badly:

* **Cancelling takes both consoles.** `.cancelMenuItem_APressed` (`engine/link/cable_club.asm`)
  sends `$F` and loops back to waiting for a button unless the nybble that comes back is also
  `$F`. A player whose partner is still browsing presses A on CANCEL forever — the game keeps
  running the whole time, nothing hangs, and it looks exactly like a stuck link.
* **Leaving the trade screen is not leaving the room.** Once both sides cancel, the fall-through
  reaches `ReturnToCableClubRoom`, which reloads the same map and puts the player back on its
  floor — the floor with no door.

The way out the cartridge actually provides is the START menu. `StartMenu_SaveReset`
(`engine/menus/start_sub_menus.asm`) checks `BIT_LINK_CONNECTED` in `wStatusFlags4`, set for the
whole Cable Club visit, and jumps to `Init` — a soft reset — in place of the normal save
routine. That is why START reads RESET where SAVE belongs everywhere else, and it is safe
specifically because the receptionist requires a save on the way in and `TradeCenter_Trade`
calls `predef SavePartyAndDexData` again immediately after every completed trade (pokered's own
comment on that call: `; this allows reset into Pokecenter`). A soft reset from inside the room
always lands on a save that already has whatever was just traded.

`wSerialExchangeNybbleSendData` is a **latch**, not a message: it keeps being sent until
something resets it, and a round is eleven or twelve transfers of the same byte with the last
one winning. That is also why cancelling has to be answered from every phase of the protocol,
not just the one the player pressed A in.

`hSerialConnectionStatus` says which end of the cable this console is — `$01` external clock,
`$02` internal — and the Cable Club attendant seats you accordingly. Pressing A at the wrong
console is a silent no-op that looks exactly like a broken link.

These facts are documented here; TerminalGB implements the receiving end of this protocol as a
virtual link partner — see [driving the link cable from
outside](https://github.com/Alchemy86/TerminalGB/blob/main/docs/gen1/link-from-outside.md).
TerminalGB's own UI proactively warns players about the exit above — see [leaving the Cable
Club](https://github.com/Alchemy86/TerminalGB/blob/main/docs/gen1/leaving-the-cable-club.md).
Joining two whole emulated consoles with a relaying cable, rather than answering the protocol
from software, is the route for anything nobody has written a partner for — a link battle, or
the Time Capsule to Generation 2 — and that is
[two consoles, one wire](https://github.com/Alchemy86/TerminalGB/blob/main/docs/link-cable.md).

<!-- atlas:begin (table) — generated by tools/render.py from the atlas data; edit the data, not the table -->

**26 entries** · 16 distinct addresses · **10 with a written description**.

| address | bytes | symbol | ev | what it is |
|---|---:|---|:--:|---|
| `$C508` | 1 | <a id="s-wSerialPartyMonsPatchList"></a>`wSerialPartyMonsPatchList`<br><a id="s-wShadowOAMBackup"></a>`wShadowOAMBackup` <a id="s-wShadowOAMBackupSprite00"></a>`wShadowOAMBackupSprite00` <a id="s-wShadowOAMBackupSprite00YCoord"></a>`wShadowOAMBackupSprite00YCoord` <a id="s-wSurroundingTiles"></a>`wSurroundingTiles` <a id="s-wTileMapBackup"></a>`wTileMapBackup` | RL | **`wSerialPartyMonsPatchList`** — The third of the three blocks a trade exchanges: the offsets inside the party block where a byte had to be substituted, because the preamble byte `$FD` cannot appear in the data. The list is split at offset 252, which is `$FD - 1`, into one part for the Pokemon region and one for the rest. **`wShadowOAMBackup`** — A copy of the shadow OAM taken before something borrows the sprite hardware, so the overworld can be restored. **`wSurroundingTiles`** — The block of map tiles around the player, gathered so the collision check and the text engine can look at them without re-reading the block map. **`wTileMapBackup`** — A copy of the screen taken before a menu opens, so closing it can restore what was underneath without redrawing the map. |
| `$C5D0` | 280 | <a id="s-wSerialEnemyMonsPatchList"></a>`wSerialEnemyMonsPatchList` | RL | The same list for the party the other console sent. |
| `$CC38` | 2 | <a id="s-wTradeCenterPointerTableIndex"></a>`wTradeCenterPointerTableIndex` | RL | Which stage of the trade the Cable Club is in. |
| `$CC3D` | 1 | <a id="s-wSerialSyncAndExchangeNybbleReceiveData"></a>`wSerialSyncAndExchangeNybbleReceiveData`<br><a id="s-wLinkMenuSelectionReceiveBuffer"></a>`wLinkMenuSelectionReceiveBuffer` <a id="s-wSerialExchangeNybbleTempReceiveData"></a>`wSerialExchangeNybbleTempReceiveData` | RL | **`wSerialSyncAndExchangeNybbleReceiveData`** — The nybble received by the combined sync-and-exchange round. **`wLinkMenuSelectionReceiveBuffer`** — The other console's link-menu selection, in the same encoding. |
| `$CC3E` | 4 | <a id="s-wSerialExchangeNybbleReceiveData"></a>`wSerialExchangeNybbleReceiveData` | RL | The nybble the other console offered. |
| `$CC42` | 5 | <a id="s-wSerialExchangeNybbleSendData"></a>`wSerialExchangeNybbleSendData`<br><a id="s-wLinkMenuSelectionSendBuffer"></a>`wLinkMenuSelectionSendBuffer` | RL | **`wSerialExchangeNybbleSendData`** — The nybble this console is offering in a synchronisation round. It is a **latch**: it keeps being sent until something resets it, which is why cancelling a trade has to be answered from every phase, not just the one it was pressed in. **`wLinkMenuSelectionSendBuffer`** — The link menu's own use of this byte: `$D0` or'ed with the menu item and the button shifted left twice, so `$D4` is A pressed on TRADE CENTER. |
| `$CD4E` | 1 | <a id="s-wTradedEnemyMonOT"></a>`wTradedEnemyMonOT` | RL |  |
| `$CD59` | 2 | <a id="s-wTradedEnemyMonOTID"></a>`wTradedEnemyMonOTID` | RL |  |
| `$CD81` | 360 | <a id="s-wSerialOtherGameboyRandomNumberListBlock"></a>`wSerialOtherGameboyRandomNumberListBlock`<br><a id="s-wTileMapBackup2"></a>`wTileMapBackup2` | RL | A second 360-byte screen backup. Also the largest clean scratch buffer below the save boundary, which is why other tools borrow it. |
| `$D12B` | 1 | <a id="s-wLinkState"></a>`wLinkState` | RL | What the link cable is doing. Non-zero means the console is in the Cable Club, and the value distinguishes trading from battling — which is why a virtual link partner that only knows how to trade can walk into a real battle. |
| `$D887` | 1 | <a id="s-wLinkEnemyTrainerName"></a>`wLinkEnemyTrainerName`<br><a id="s-wGrassRate"></a>`wGrassRate` | RL | **`wLinkEnemyTrainerName`** — The other player's name, once the blocks have been exchanged. Shares storage with the wild-encounter tables, which is safe because the two can never be live at once. **`wGrassRate`** — How often a step in grass rolls an encounter on this map. |
| `$D893` | 9 | <a id="s-wSerialEnemyDataBlock"></a>`wSerialEnemyDataBlock` | RL | The other console's party block as it arrives. |
| `$FFA9` | 1 | <a id="s-hSerialReceivedNewData"></a>`hSerialReceivedNewData` | R |  |
| `$FFAB` | 1 | <a id="s-hSerialIgnoringInitialData"></a>`hSerialIgnoringInitialData` | R | Set at the start of every block exchange. `Serial_ExchangeBytes` re-sends its first byte until a preamble byte comes back and only then starts storing, so a block of N bytes costs **N + 1** transfers and anything speaking this protocol has to lead each block with one extra preamble byte. Getting it off by one shifts the whole party block. |
| `$FFAC` | 1 | <a id="s-hSerialSendData"></a>`hSerialSendData` | R |  |
| `$FFAD` | 1 | <a id="s-hSerialReceiveData"></a>`hSerialReceiveData` | R |  |

<!-- atlas:end (table) -->
