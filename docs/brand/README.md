# AtlasGB brand

**The mark is [`atlasgb-logo.svg`](atlasgb-logo.svg), and its square icon lockup is
[`atlasgb-icon.svg`](atlasgb-icon.svg). These two files are the only ones to ship —
anywhere.**

![The AtlasGB logo](preview/logo.png)

## The composition

AtlasGB is a sibling of [TerminalGB](https://github.com/Alchemy86/TerminalGB), so it
keeps everything that makes that mark recognisable — the near-black panel with its faint
border, the same heavy geometric alphabet, the same tight tracking, the DMG LCD lit shade
(`#9bbc0f`) as the single accent, the accent full stop closing the word, and the grey
wide-tracked tagline underneath.

What is AtlasGB's own is the **region bar**: four segments under the wordmark, one for
each RAM region the atlas walks — video, cartridge, work and high RAM — contiguous, with
no holes, and the last one short because high RAM really is 127 bytes against work RAM's
8,192. It is the shape of the claim the project makes: *the whole space, accounted for,
nothing missing.* The segments say "four regions, one of them tiny"; they are not drawn
to scale and the repository does not claim they are.

The icon is that bar alone, stacked. Two marks that both had to survive 16 px needed to
stay apart at 16 px too: TerminalGB's is one tall block and a dot, AtlasGB's is four rows
with a short one at the bottom, and neither can be mistaken for the other
([`preview/icon-16.png`](preview/icon-16.png) is the real favicon test, not a resized
illustration).

## Licence and provenance

Everything here is **hand-authored**: the letterforms are original stroked skeleton paths
drawn in [`generate.py`](generate.py); **no font is embedded, subset or traced**, so
there is no third-party licence in any of these files. Thirteen of the letters are the
shapes TerminalGB's own generator draws, carried over unchanged so the two wordmarks are
visibly the same alphabet — that is our work in both places. Seven (`C D H P S V W`) are
new here and drawn to the same rules.

No Nintendo artwork, logotype or trade dress is used or imitated; the whole vocabulary is
a bar chart and a full stop.

## Regenerating

**Edit the generator, never the SVGs by hand**, and re-render the previews in the same
commit:

```bash
python3 docs/brand/generate.py
cd docs/brand
magick -background none atlasgb-logo.svg preview/logo.png
magick -background none atlasgb-icon.svg preview/icon-128.png
magick -background none atlasgb-icon.svg -resize 16x16 preview/icon-16.png
```

Every SVG paints its own panel, so it survives GitHub light mode, dark mode and a pure
black page — nothing is theme-conditional, so there is no `prefers-color-scheme` trap.

## Files

- [`atlasgb-logo.svg`](atlasgb-logo.svg) — **the** wordmark lockup, 1200×380
- [`atlasgb-icon.svg`](atlasgb-icon.svg) — **the** square icon lockup, 128×128
- [`preview/logo.png`](preview/logo.png), [`preview/icon-128.png`](preview/icon-128.png),
  [`preview/icon-16.png`](preview/icon-16.png) — rendered previews at hero, avatar and
  favicon size
- [`generate.py`](generate.py) — the only source of truth; edit it, never the SVGs
