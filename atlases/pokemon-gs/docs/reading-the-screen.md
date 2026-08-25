# Reading the screen — Pokémon Gold and Silver

[← Gold/Silver](../README.md) · [AtlasGB](../../../README.md)

`wTilemap` (`$C3A0`, 20×18) reads back as the text a player can see — the same address, the
same property, and the same charmap base as [Pokémon Red/Blue's own
screen](../../pokemon-rb/docs/screen.md): `$80` is still `A`, `$A0` is still `a`, `$F6`–`$FF` are
still the digits. The font is loaded so that a tile id *is* the character code of the glyph it
draws, and text is written into the tilemap directly — there is no separate text layer in
either generation.

What differs between the two games is narrower, and all three differences here were measured.

## The overworld renders no letters at all

Generation 2's map tiles are ids below `$80`, which the charmap decodes to spaces — so a
decoded overworld screen is entirely blank, with no dialogue, no menu, nothing. That makes
*any* letter appearing on the decoded tilemap a reliable signal that a menu or a dialog box
currently owns the joypad, which is a stronger and more general test than watching for any
particular word: a screen containing text nobody anticipated still trips it, where a list of
expected phrases would not.

## The menu cursor is a tile, not a variable

The cursor is drawn as tile `$ED`, appearing exactly once on an open menu — found by scanning
the decoded tilemap for that byte. **The WRAM variable that looks like it should hold the
cursor's row does not agree with the drawn cursor**: measured reading `2` while the cursor sat
visibly on row 4. Its address is deliberately not stated on [the memory map](memory-map.md) for
that reason. The tilemap itself, not a named variable, is the thing actually shown to the
player and the thing worth reading.

## Dismissing a dialog is inverted from Generation 1

**Generation 2's dialogs are dismissed with A, never B.** B backs out through a continue-style
menu toward the title screen instead of closing the box in front of it. Generation 1 has the
opposite habit — B is its *safe* dismissal, because it closes a menu and can never open one.
The two generations disagree outright, so this is a fact about each game individually, not
something that transfers from one to the other.

## What is shared with Generation 1, unchanged

Two decoding properties carry over directly: message text is padded to a fixed row width and
wraps across rows, so matching a phrase means tolerating runs of whitespace rather than
matching raw; and text types onto the screen character by character, so a screen sampled
mid-message holds only a prefix of the final line.

**Whether Generation 2 packs multiple characters into single tiles the way Generation 1's
charmap does** (`PKMN`, `TRAINER` and similar as one tile each) is not established here.

## See also

- [Pokémon Red/Blue's own screen](../../pokemon-rb/docs/screen.md) — the tilemap-as-text property
  in the cartridge this project's evidence pipeline actually covers.
- [the Time Capsule](time-capsule.md) — the RTC recovery screen this property caught a
  word-list test missing.
