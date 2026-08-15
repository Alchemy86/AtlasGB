# `data/` moved — the atlases are namespaced by game now

[← AtlasGB](../README.md) · [Pokémon Red/Blue](../atlases/pokemon-rb/)

This directory used to hold the atlas, back when one atlas was the whole repository.
AtlasGB is a project for mapping **Game Boy cartridges**, and it now says so in its layout:
each cartridge's data lives under [`atlases/<id>/`](../atlases/), so adding a second one is
a new sibling directory rather than a merge into a global file.

Nothing here is deleted or rewritten — only relocated.

## Where everything went

| was | is now |
|---|---|
| `data/atlas.tsv` | [`atlases/pokemon-rb/data/atlas.tsv`](../atlases/pokemon-rb/data/atlas.tsv) |
| `data/atlas.json` | [`atlases/pokemon-rb/data/atlas.json`](../atlases/pokemon-rb/data/atlas.json) |
| `data/atlas.min.json` | [`atlases/pokemon-rb/data/atlas.min.json`](../atlases/pokemon-rb/data/atlas.min.json) |
| `data/evidence.json` | [`atlases/pokemon-rb/data/evidence.json`](../atlases/pokemon-rb/data/evidence.json) |
| `data/atlas.schema.json` | [`schema/atlas.schema.json`](../schema/atlas.schema.json) — shared by every atlas, so it is not under one |
| `docs/<chapter>.md` | [`atlases/pokemon-rb/docs/<chapter>.md`](../atlases/pokemon-rb/docs/) — same filenames |
| `docs/by-address.md`, `docs/by-name.md`, `docs/structures.md` | same, under [`atlases/pokemon-rb/docs/`](../atlases/pokemon-rb/docs/) |

**Unmoved**, because they are contracts shared by every atlas rather than facts about one
cartridge: [`docs/schema.md`](../docs/schema.md),
[`docs/consuming.md`](../docs/consuming.md),
[`docs/verification.md`](../docs/verification.md),
[`docs/provenance.md`](../docs/provenance.md),
[`docs/licensing.md`](../docs/licensing.md), [`tools/`](../tools/) and the brand.

## If you pinned the old path

**The bytes did not change.** A directory move relocates a file; it does not rewrite the
rows. So a vendored snapshot taken from `data/atlas.tsv` before the move is still exactly
correct, its sha256 still matches its lock, and no claim made against it is affected. What
changed is the URL a *refresh* fetches.

Re-pin when convenient:

```bash
# tools/fetch-atlas.sh grew --atlas for exactly this
tools/fetch-atlas.sh --ref main --atlas pokemon-rb --dest third_party/atlasgb

# and --file, so a ref from before the move is still fetchable
tools/fetch-atlas.sh --ref 91b5d18 --file data/atlas.tsv --dest third_party/atlasgb
```

Anchors are unchanged: `#s-wPartyCount` and `#a-D163` mean what they always meant, and the
chapter pages kept their filenames. Only the directory in front of them moved.

## Why the old files are not left behind as stubs

A one-line placeholder at `data/atlas.tsv` would be fetched successfully by every consumer
that pinned it, parse as an atlas with zero rows, and fail silently somewhere much later. A
404 is the honest answer to a request for a file that is not there, and this page is the
part that tells you where it went.
