# AtlasGB documentation

[← AtlasGB](../README.md) · [the atlases](../atlases/README.md)

**These pages are shared by every atlas in the project.** They are the contracts — how the
data is shaped, how a claim earns its evidence, how to consume it, and what a new cartridge
would take. The maps themselves live with the cartridge they are about, under
[`atlases/`](../atlases/README.md).

## Start here

| page | what it is for |
|---|---|
| **[the schema](schema.md)** | every column of an `atlas.tsv`, its units and its allowed values. Enough to write a parser without opening the tools |
| **[consuming it](consuming.md)** | fetching, vendoring, pinning, and the anti-drift gate worth copying |
| **[verification](verification.md)** | the evidence tiers, the invariants, and the loop that keeps them true |
| **[adding an atlas](adding-an-atlas.md)** | what a second cartridge would cost, and what you would have to hand over |
| **[provenance](provenance.md)** | where the data came from, what is ours and what is the community's |
| **[licence](licensing.md)** | MIT, and the honest longer answer about facts, names and prose |
| **[the brand](brand/README.md)** | the mark, and how it is generated |

## The atlases

One directory per cartridge. Each carries its own data, its own pages and a `meta.json`
saying which game it is about.

| atlas | its map |
|---|---|
| **[Pokémon Red and Blue](../atlases/pokemon-rb/README.md)** | [by address](../atlases/pokemon-rb/docs/by-address.md) · [by name](../atlases/pokemon-rb/docs/by-name.md) · [the structures](../atlases/pokemon-rb/docs/structures.md) · [the chapters](../atlases/pokemon-rb/README.md#the-chapters) |

Every chapter page is a table generated from that atlas's `atlas.tsv` plus hand-written
prose around it. Edit the data, not the table; the prose outside the `<!-- atlas:… -->`
markers is preserved by [`tools/render.py`](../tools/render.py). The subject grouping is
decided by reviewable rules in [`tools/chapters.py`](../tools/chapters.py), not by
hand-typed cells.

> **The chapter pages moved.** They used to sit here, in `docs/`, when one atlas was the
> whole repository; they are now under `atlases/pokemon-rb/docs/` with their filenames and
> their `#s-…` / `#a-…` anchors unchanged. See [`../data/README.md`](../data/README.md) for
> the full before/after list.
