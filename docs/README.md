# AtlasGB documentation

[← AtlasGB](../README.md) · [the data](../data/atlas.tsv)

## Start here

| page | what it is for |
|---|---|
| **[the schema](schema.md)** | every column of `atlas.tsv`, its units and its allowed values. Enough to write a parser without opening the tools |
| **[consuming it](consuming.md)** | fetching, vendoring, pinning, and the anti-drift gate worth copying |
| **[verification](verification.md)** | the four evidence tiers, the invariants, and the loop that keeps them true |
| **[provenance](provenance.md)** | where the data came from, what is ours and what is the community's |
| **[licence](licensing.md)** | MIT, and the honest longer answer about facts, names and prose |
| **[the brand](brand/README.md)** | the mark, and how it is generated |

## The map

| page | what is in it |
|---|---|
| [by address](by-address.md) | the whole map in address order, one anchor per address |
| [by name](by-name.md) | the same map indexed by symbol name |
| [the structures](structures.md) | `party_struct`, `box_struct`, `battle_struct` and the rest, field by field |

### The chapters

Grouped by subject rather than by linker section — the rules are in
[`tools/chapters.py`](../tools/chapters.py).

| | | |
|---|---|---|
| [player](player.md) | [party](party.md) | [storage](storage.md) |
| [battle](battle.md) | [bag](bag.md) | [pokedex](pokedex.md) |
| [events](events.md) | [overworld](overworld.md) | [sprites](sprites.md) |
| [screen](screen.md) | [graphics](graphics.md) | [audio](audio.md) |
| [link](link.md) | [rng](rng.md) | [save](save.md) |
| [system](system.md) | [scratch](scratch.md) | [rom-data](rom-data.md) |

Every chapter page is a table generated from `data/atlas.tsv` plus hand-written prose
around it. Edit the data, not the table; the prose outside the `<!-- atlas:… -->` markers
is preserved by [`tools/render.py`](../tools/render.py).
