# Licence, and what is and is not ours to license

[← AtlasGB](../README.md) · [provenance](provenance.md)

**Everything in this repository is [MIT](../LICENSE).** One permissive licence over the
whole thing — the tooling, the data file, the pages, the brand — because the point of this
repository is to be adopted, and a licence somebody has to think about is a licence
somebody routes around.

This page is the honest longer answer, because "MIT" alone would overclaim.

---

## The addresses are facts

An address, a structure layout and a field offset are **facts about a cartridge**. Facts
are not subject to copyright, in this jurisdiction or the ones this is most likely to be
read in. `wPartyCount` is at `$D163` whether or not anybody writes it down, and writing it
down does not create a monopoly on it.

So the MIT grant over each atlas's `atlas.tsv` should be read as *a promise not to make trouble*
rather than as a claim that the numbers were ours to give away. Use them. You do not need
our permission and you never did.

## The symbol names are the community's

`wPartyCount`, `hRandomAdd`, `sBoxMon1` and the two thousand others are
[`pret/pokered`](https://github.com/pret/pokered)'s work, built up over many years by many
people. **We did not invent them and we do not claim them.** They are reproduced verbatim
because using a name to refer to a thing is exactly what a name is for, and because a map
that renamed everything would be useless to anybody reading a disassembly.

`pret/pokered` is distributed under its own terms; consult that repository. Our position
is narrow and deliberate: **we take addresses, symbol names and structure layouts, and
nothing else.** In particular —

**pokered's prose and comments are not ours to copy.** Every description in
every `atlas.tsv` and every paragraph on every page here is written from scratch. Where a
page states a fact that came from the disassembly, it cites the repository, the file and
the symbol rather than quoting.

Nothing from pokered is vendored here and nothing is fetched at run time: you point
[`tools/extract.py`](../tools/extract.py) at your own checkout, or you do not run it.

## What is genuinely ours, and MIT-licensed without qualification

- **The verification** — the harness design, the four evidence tiers, the invariants, and
  the loop that keeps the tiers from going stale ([verification.md](verification.md)).
- **The completeness accounting** — the `gap` and `free` rows, and the claim that work RAM
  and high RAM are 100% accounted for.
- **The chapter rules** in [`tools/chapters.py`](../tools/chapters.py), which are
  editorial judgement.
- **Every word of prose** in `desc` and on these pages.
- **The tooling** in `tools/`, and **the brand** in [`docs/brand/`](brand/) — whose
  letterforms are original stroked paths with no font embedded, subset or traced, so there
  is no third-party licence in those files either.

## What is nobody's, and is not here

**No commercial ROM data, in any form.** No sprites, no text, no tables, no music, no
fragment of any of them. The cartridge's own data tables appear in the
[rom-data](../atlases/pokemon-rb/docs/rom-data.md) chapter as *addresses*; the numbers at those addresses are read
out of the player's own cartridge at run time and are never written down here.

**No boot-ROM content, in any form.**

**No ROM is distributed and none is fetched.** Verifying this atlas against a cartridge
requires you to supply your own, legally.

---

## Citing it

Not required. Appreciated, and useful to the next person:

> AtlasGB — every address in Pokémon Red and Blue, with the evidence for it.
> <https://github.com/Alchemy86/AtlasGB>

[`CITATION.cff`](../CITATION.cff) is the machine-readable form; GitHub renders a "Cite this
repository" button from it.

---

## Trade marks

Pokémon, Game Boy and Nintendo are trade marks of their respective owners. This project is
not affiliated with, endorsed by or connected to any of them. Naming a game a map
describes is nominative use — the standard practice for documentation of this kind — and
no artwork, logotype or trade dress of theirs is used or imitated anywhere in this
repository.
