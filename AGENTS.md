# Project agent memory

Durable, project-intrinsic notes for anyone — human or agent — working in AtlasGB.
Build, test and architecture facts that should travel with the code.

## What this repository is

A **published dataset**, not an application — and a **project**, not one file. One atlas
per Game Boy cartridge, under `atlases/<id>/`. Each atlas's `atlas.tsv` is the single
source of truth for that cartridge; every page, index, table and JSON file is generated
from it. People browse this on github.com far more often than they clone it, so the
browsing experience is part of the deliverable.

One atlas is published today: `atlases/pokemon-rb/` (Pokémon Red and Blue). The layout is
what it is so that a second cartridge is a new sibling directory rather than a merge into a
global file — see `docs/adding-an-atlas.md`.

`make check` is what CI runs. Run it before you push. Every target acts on all atlases;
`ATLAS=<id>` narrows `docs` and `data`, and deliberately does **not** narrow `check`.

## Sharp edges

### The data is namespaced by game; the contracts are not

`atlases/<id>/` owns everything specific to one cartridge: `meta.json` (which game),
`data/` (TSV, JSON, evidence record) and `docs/` (chapters and indexes). Everything that
makes the data trustworthy is shared and written once: `schema/atlas.schema.json`,
`tools/`, `docs/schema.md`, `docs/consuming.md`, `docs/verification.md`, the brand.

**An atlas that needed its own copy of a shared thing is a sign the shared thing is
wrong** — fix the shared thing. `tools/atlases.py` discovers atlases by the presence of
`meta.json`, so there is no list to add to: a list is a thing people forget, and the
failure mode is an atlas that quietly stops being validated.

`tools/extract.py` is the exception and is *deliberately* atlas-specific: a different
cartridge means a different disassembly and a different map file, so it gets its own
extractor beside that one.

### An atlas's `atlas.tsv` must start with its header row, with no comment lines

GitHub's CSV/TSV viewer has no notion of a comment line. The file used to open with six
`#` provenance lines, and the viewer read them as one-column rows and **refused to render
the file at all** — losing the sortable, searchable table view that is most of what makes
this data browsable without cloning. `tools/validate.py` checks for both properties on
every push, and `tools/extract.py` never writes a preamble. Provenance lives in
`docs/provenance.md`.

### The `verify` column is not editable by hand

Evidence tiers are produced elsewhere — proving an address against a running cartridge
needs an emulator — and arrive as a report landed by `tools/apply-evidence.py`.
Each atlas's `data/evidence.json` records the run's provenance **and a SHA-256 digest of
the verify column it produced**; `--check` runs in CI, over every atlas. So:

- editing a tier by hand turns CI red;
- adding a symbol to the atlas without re-running the verification turns CI red, because
  the new symbol changes the digest. That is intended: the atlas grew, so the last run no
  longer covers it.

The digest is over `symbol → verify` only, so writing a description or rewording a chapter
does not invalidate a run.

### Never change the `<!-- atlas:begin (…) -->` marker wording casually

`tools/render.py` finds each generated block by an exact string, and the string is
deliberately **path-free** — it names no atlas and no file, so namespacing the data by game
did not force a third rewrite of every marker. It used to fall back to rebuilding the page
from a stub when the marker was missing, which **silently replaced twenty-one pages of
hand-written prose with a four-line placeholder** the moment the marker text was edited. `splice()` now raises `MarkerLost` for a page that already exists; the
stub is only used to create a page that is not there yet. If you do change the wording, you
must rewrite every existing marker in the same commit.

### Anchors are a public URL surface

`atlases/<id>/docs/<group>.md#s-<symbol>` and `.../by-address.md#a-<addr>` are meant to be
linked to from outside, so they are derived from the data and survive regeneration. The
fragments survived the namespacing move — only the directory in front of them changed, and
`data/README.md` is the page that says so. There are ~2,400 of them and they have rotted
before (370 dead at once, when aliases were linked before they were given anchors). `tools/checklinks.py` checks every link and fragment on every push.

`tools/checklinks.py`'s slug function must **not** collapse runs of whitespace: GitHub
turns each space into its own dash, so a heading containing an em dash — which is stripped
— produces two consecutive dashes (`## I — an invariant covers it` is
`#i--an-invariant-covers-it`).

GitHub also **lowercases** the `id` it renders from inline HTML and prefixes it:
`<a id="s-wPartyCount">` becomes `id="user-content-s-wpartycount"`. Fragments resolve
case-insensitively, so `#s-wPartyCount` is the form to publish and it works — verified in a
browser, not assumed. The consequence is that two symbols differing only in case would
collide on one anchor, so `tools/validate.py` fails if one ever appears.

## Layout

| path | what it is |
|---|---|
| `atlases/<id>/meta.json` | which cartridge this atlas is about; every page title comes from it |
| `atlases/<id>/data/atlas.tsv` | the source of truth; everything else is derived |
| `atlases/<id>/data/atlas.json`, `atlas.min.json` | generated by `tools/export.py`, checked in CI |
| `atlases/<id>/data/evidence.json` | which verification run its tiers came from, plus the digest |
| `atlases/<id>/README.md` | that atlas's front page: evidence, coverage, worked example, chapters |
| `atlases/<id>/docs/*.md` | its generated pages; prose outside the markers is preserved |
| `schema/atlas.schema.json` | JSON Schema for every `atlas.json`; validated by `tools/validate.py` |
| `docs/*.md` | the shared contracts: schema, consuming, verification, adding an atlas |
| `docs/brand/generate.py` | the only source of truth for the SVGs — never edit the SVGs |
| `data/README.md` | a signpost for the pre-namespacing paths; there is no data here any more |
| `tools/atlases.py` | discovers the atlases; every other tool takes `--atlas` from it |
| `tools/extract.py` | rewrites the eight derived columns of `pokemon-rb` from a built `pret/pokered` |
| `tools/render.py` | rewrites every generated block, including both READMEs' |
| `tools/fetch-atlas.sh` | for *consumers* to copy: pin, vendor and verify a snapshot (`--atlas`, `--file`) |

## Regeneration order

`tools/extract.py --write` → land a verification report (`tools/apply-evidence.py
report.json --atlas <id>`) → `make docs data`. The extractor says so when the symbol set
changed, because that is exactly when the evidence needs re-running.

The evidence report's `atlas` key is **optional** on purpose: reports written before the
namespacing still land. Do not make it required without coordinating with the producer
(TerminalGB) — the digest contract is what lets a consumer reproduce a published run byte
for byte, and it must not move underneath them.

## Things that must stay true

- **The seven unevidenced entries stay marked as unevidenced.** That honesty is the
  product. `tools/validate.py` prints them by name on every run so they cannot quietly
  disappear.
- **No commercial ROM data and no boot-ROM content, in any form.** The cartridge's tables
  appear as addresses only; the numbers at them are read from the player's own cartridge at
  run time.
- **pokered's prose and comments are not ours to copy.** Every description is written from
  scratch; facts are cited by repository, file and symbol. The same applies to whatever
  disassembly a future atlas derives from.
- **Every published number names its cartridge.** The badges, the page titles, the JSON
  `meta` block and the citation file are all generated or checked, because a count with no
  game attached reads as a claim about the whole platform. `tools/validate.py` fails if an
  atlas's `meta.json` is missing `title`, `games` or `summary`.
- **The storage count and the including-ROM-tables count are different numbers.** For
  `pokemon-rb`, invariants cover 36 storage entries and 39 including the three ROM-table
  labels. `data/evidence.json` carries both, under `evidence` and
  `evidence_including_rom_tables`, and the atlas README states both so neither can be
  quoted as the other.
