# Licence, and what is and is not ours to license

[← AtlasGB](../README.md) · [provenance](provenance.md)

**This repository is licensed in two parts, and the boundary is a path.**

| path | licence | why |
|---|---|---|
| `atlases/**` — every atlas's TSV, JSON, evidence record, README and chapter pages | **[CC BY-SA 4.0](../LICENSE-CC-BY-SA)** | the verification, the tiers, the completeness accounting and the descriptions are ours; credit them and share improvements back |
| `docs/**.md` — the shared prose | **[CC BY-SA 4.0](../LICENSE-CC-BY-SA)** | same reason: written from scratch |
| `docs/brand/**` — the mark and its SVGs | **[CC BY-SA 4.0](../LICENSE-CC-BY-SA)** | original letterforms, no font embedded or traced |
| `README.md`, `AGENTS.md` and the root prose | **[CC BY-SA 4.0](../LICENSE-CC-BY-SA)** | prose |
| `tools/**` — extract, render, export, validate, checklinks, `fetch-atlas.sh` | **[MIT](../LICENSE)** | other projects should be able to lift this without a second thought |
| `schema/atlas.schema.json` | **[MIT](../LICENSE)** | so anybody can produce a **compatible** atlas — that interoperability is the point of [adding an atlas](adding-an-atlas.md) |
| `Makefile`, `.github/**` | **[MIT](../LICENSE)** | build and CI |

In one line: **credit AtlasGB and share your improvements to the atlas content
alike; use the tooling and the schema however you like.**

This page is the honest longer answer, because a licence banner alone would overclaim.

---

## The moat is the verification, not the licence

Start here, because it is the thing that is actually true.

Nobody is stopped from writing down `$D05A` by a licence, and nothing on this page tries
to stop them. What nobody else can hand you is the **evidence**: `rom` because the address
appears as an instruction operand in the cartridge image, `live` because the byte was
watched changing while a cycle-accurate emulator ran the cartridge under a fixed script,
`inv` because a named invariant proved the symbol means what the atlas says it means. That
takes an emulator, a cartridge and a harness, and it has to be re-run whenever the symbol
set moves. See [verification.md](verification.md).

Share-alike does not create that advantage. It just makes sure that when somebody improves
on it, **the improvement comes back** rather than disappearing into a closed product with
the evidence quietly stripped off it.

## The addresses are facts

An address, a structure layout and a field offset are **facts about a cartridge**. Facts
are not subject to copyright, in this jurisdiction or the ones this is most likely to be
read in. `wPartyCount` is at `$D163` whether or not anybody writes it down, and writing it
down does not create a monopoly on it.

So the CC BY-SA grant over each atlas's `atlas.tsv` should be read as covering **the
compilation** — this particular selection, arrangement, evidence and prose — rather than
as a claim that the numbers were ours to give away. Use the numbers. You do not need our
permission and you never did, and you can re-derive every one of them from
[`pret/pokered`](https://github.com/pret/pokered) yourself. **That is fine.** It is worth
saying out loud, because a share-alike banner over a table of facts is exactly the kind of
thing that quietly grows into a claim nobody ever meant to make.

## The symbol names are the community's

`wPartyCount`, `hRandomAdd`, `sBoxMon1` and the two thousand others are
[`pret/pokered`](https://github.com/pret/pokered)'s work, built up over many years by many
people. **We did not invent them and we do not claim them.** They are reproduced verbatim
because using a name to refer to a thing is exactly what a name is for, and because a map
that renamed everything would be useless to anybody reading a disassembly.

That is the other half of why the licence here is share-alike and not
permission-required. A permission gate would claim the half that **is not ours** — the
addresses are facts, the names are the community's, and asking somebody's permission to
use them would be asking on behalf of people who never appointed us.

`pret/pokered` is distributed under its own terms; consult that repository. Our position
is narrow and deliberate: **we take addresses, symbol names and structure layouts, and
nothing else.** In particular —

**pokered's prose and comments are not ours to copy.** Every description in
every `atlas.tsv` and every paragraph on every page here is written from scratch. Where a
page states a fact that came from the disassembly, it cites the repository, the file and
the symbol rather than quoting.

Nothing from pokered is vendored here and nothing is fetched at run time: you point
[`tools/extract.py`](../tools/extract.py) at your own checkout, or you do not run it.

## What is genuinely ours, and what share-alike protects

This is the list CC BY-SA is over. It is deliberately the same list as before — the
licence changed, the honesty about what it covers did not.

- **The verification** — the harness design, the four evidence tiers, the invariants, and
  the loop that keeps the tiers from going stale ([verification.md](verification.md)).
- **The completeness accounting** — the `gap` and `free` rows, and the claim that work RAM
  and high RAM are 100% accounted for.
- **The chapter rules'** editorial judgement — which subsystem a symbol belongs to, and the
  chapter structure that falls out of it. (The *code* expressing them,
  [`tools/chapters.py`](../tools/chapters.py), is MIT like the rest of `tools/`; the
  grouping it encodes is part of the atlas.)
- **Every word of prose** in `desc` and on these pages.
- **The brand** in [`docs/brand/`](brand/) — whose letterforms are original stroked paths
  with no font embedded, subset or traced, so there is no third-party licence in those
  files either. (Its generator, [`docs/brand/generate.py`](brand/generate.py), is code and
  is MIT; the marks it produces are content.)

**The tooling in [`tools/`](../tools/) stays MIT, without qualification.** The generator,
the fetch script, the schema tooling and the CI are code other projects should be able to
use freely, and gating them would discourage exactly the adoption this project wants. If
`tools/fetch-atlas.sh` is useful to you, copy it — that is what it is for.

**The schema stays freely usable for the same reason.** A second atlas that is compatible
with this one is worth more than an exclusive first one, and a schema you need permission
to implement is a schema nobody implements.

## What is nobody's, and is not here

**No commercial ROM data, in any form.** No sprites, no text, no tables, no music, no
fragment of any of them. The cartridge's own data tables appear in the
[rom-data](../atlases/pokemon-rb/docs/rom-data.md) chapter as *addresses*; the numbers at those addresses are read
out of the player's own cartridge at run time and are never written down here.

**No boot-ROM content, in any form.**

**No ROM is distributed and none is fetched.** Verifying this atlas against a cartridge
requires you to supply your own, legally.

---

## What actually changed, and what did not

This section exists because "we relicensed" is the kind of sentence that is usually doing
more work than it has any right to.

**Every copy already published under MIT stays MIT.** A grant already made cannot be
withdrawn. That includes the complete Pokémon Red/Blue atlas already vendored into the
public [TerminalGB](https://github.com/Alchemy86/TerminalGB) repository at
`third_party/atlasgb/pokemon-rb/atlas.tsv`: that snapshot was taken under MIT and remains
under MIT, in perpetuity, and so does every other copy anybody took before this change.
CC BY-SA governs **versions published from this point on** — including the next snapshot
anybody pins. This is not a clean break and it would be dishonest to present it as one.

**Facts still cannot be owned.** Share-alike protects the compilation, the descriptions
and the evidence — not the fact that `$D05A` is `wBattleType`. Anyone can re-derive the
addresses from the public disassembly without touching this repository at all, and nothing
here asks them not to.

**Nothing about the data changed.** No row was rewritten, no tier moved, no anchor moved.
The seven unevidenced entries are still marked as unevidenced. A licence change is a
change to the terms, not to the file.

**The tooling did not become less permissive.** It was MIT and it stays MIT.

---

## Citing it

Share-alike makes this **more** important, not less: attribution is now a condition of the
licence for the atlas content, not just a courtesy. The minimum that satisfies it:

> AtlasGB — every address in Pokémon Red and Blue, with the evidence for it.
> <https://github.com/Alchemy86/AtlasGB> — CC BY-SA 4.0

Name the atlas you used, and the tag or commit if you can: an address is a fact about *one
cartridge*, and a credit that does not say which one is not much of a credit. If you
publish a modified or extended atlas, say so and release it under the same terms — that is
the share-alike half.

[`CITATION.cff`](../CITATION.cff) is the machine-readable form; GitHub renders a "Cite this
repository" button from it. [consuming.md](consuming.md#what-you-are-agreeing-to) states
the same thing from the integrator's side.

---

## Trade marks

Pokémon, Game Boy and Nintendo are trade marks of their respective owners. This project is
not affiliated with, endorsed by or connected to any of them. Naming a game a map
describes is nominative use — the standard practice for documentation of this kind — and
no artwork, logotype or trade dress of theirs is used or imitated anywhere in this
repository.
