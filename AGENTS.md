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

**`by-name.md` carries `#s-<symbol>` too, and that is not redundant.** An outside project
linking into an index by name will guess the index page, because an index is the page you can
guess — this atlas's own [`discoveries.md`](atlases/pokemon-rb/docs/discoveries.md) links
sixteen symbols to `by-name.md#s-<symbol>` for exactly that reason. Before that page moved here
from TerminalGB, every one of those links pointed in from outside the repository, and until the
anchors existed each one silently landed at the top of the page. A fragment that resolves
nowhere fails quietly; nothing could have caught it from either side. The same rule is why
`name_index_block` links **aliases** to their `#s-` fragment as well:
they have carried anchors on the chapter pages for as long as they have been listed, and
sending `wPartyMons` to the top of a 105-row page is a worse answer than the row it asked for.

### An entry may carry a description; a symbol name is not a source

`desc` is the column that says what an address is *for*, and it is written from a source: what
the disassembly says the symbol is (cited by symbol name, never by bare address), what
TerminalGB measured on the retail cartridge, or a structure this atlas already documents whose
sibling field or slot 1 is described here. **A symbol name on its own is not a source** — the
atlas already shows the symbol name, and a plausible sentence derived from it is exactly the
thing this project exists as a reaction to. That is why ninety `w<Map>CurScript` bytes are
blank and one is not: only Oak's laboratory has been driven and watched.

The reasoning behind a finding does **not** go in the cell. `desc` is exported verbatim into
`atlas.json`, vendored into other repositories and read by tools that never asked for a
hyperlink, so a URL there ages badly in places nobody is looking. The fact goes on the entry;
the story goes in the chapter page's prose, in a `### Findings behind these bytes` table
linking to `atlases/pokemon-rb/docs/discoveries.md#<slug>`. Those slugs are this atlas's own
published contract — read the page for them, never invent one. The page used to live in
TerminalGB, and while it did, `make check` only resolved that link as external and could not
catch a rotted fragment; now that it lives here, `tools/checklinks.py` resolves the fragment on
every push and a rotted one turns CI red.

`tools/render.py` folds `instance` rows onto slot 1, **except where the row has a description**.
`wPlayerBattleStatus1`..`3` are consecutive one-byte symbols and so read as a repeated
structure to the extractor, but they hold different bits; a written description is the signal
that a slot means something of its own, and folding it away would hide the thing that was
written down.

### Game-data content lives here, not in TerminalGB

AtlasGB documents and discovers game data; TerminalGB documents its own emulator. Before the
migration recorded here, that line was blurry: `atlases/pokemon-rb/docs/discoveries.md` — the
reasoning behind two dozen of this atlas's findings — lived in TerminalGB's repository and every
chapter page linked out to it. It now lives here, as a hand-written page with no `atlas.tsv`
behind it (`tools/render.py` never touches it; its headings are load-bearing anchors other
projects and this atlas's own chapter pages link into by slug, so **never reword or remove one**
— add new entries, correct a wrong one in place, exactly as the page's own top section says).

**One entry did not move.** "A cartridge can tell there is no sound chip" was about TerminalGB's
own emulator leaving its APU uninitialised, not about the cartridge, and stayed in TerminalGB —
proof that a page can move almost whole and still leave behind the one entry that was never
about the game.

**A second, wider pass moved everything else that was game knowledge.** The first pass above
duplicated facts into chapter-page prose while leaving the TerminalGB originals whole — which is
exactly the "two copies to drift" problem the project exists to avoid, applied to itself. The
second pass went back through every remaining TerminalGB Gen 1 page and Gen 2's whole `docs/gen2/`
area, applied the same game-versus-emulator line, and this time **split** rather than duplicated
wherever a page mixed both: the game-fact half became new AtlasGB content — either folded into an
existing chapter's prose (`battle.md`, `overworld.md`, `party.md`, `rom-data.md`, `screen.md`,
`sprites.md` all grew this way) or a new hand-written page beside `discoveries.md`
(`sharp-edges.md`, `paper-claims.md`, `catching.md`, `cerulean-gym.md`) — while the
emulator/harness/agent-tooling half was left in place in TerminalGB, because nothing here can
safely delete or rewrite a file in another repository. `battle-engine.md`, `battling.md`,
`learning-moves.md`, `level-up-drill.md`, `reading-the-screen.md`, `memory-map.md`,
`link-from-outside.md`, `leaving-the-cable-club.md`, `docs/link-trade.md`,
`authoring-an-area.md`, `battle-control.md` and `agent-play.md` were all read in full for this;
`agent-play.md`, `battle-control.md` and `authoring-an-area.md` turned out to be mixed rather
than pure tooling as their names suggest — each gave up a specific extracted fact or section
before the rest stayed put. `gen1/atlas/README.md` and `gen1/README.md` were confirmed to be
pure TerminalGB mechanism/index pages with no game content at all, and stayed untouched.
See `atlas-gamedata-migrate-wide-t7-handover.md` in firstmate's state directory for the
page-by-page reasoning and the exhaustive link-fix table for TerminalGB's side.

**Generation 2 (Pokémon Gold/Silver) has real game-data prose now, without a data pipeline —
see "A pending atlas has no `meta.json`" below.** `tools/validate.py` still requires every
*discovered* atlas to have a working `data/atlas.tsv`, and there is still no Gold extractor, no
built `pret/pokegold` pipeline and no verification run. What changed is recognising that
`discoveries.md`'s own pattern — hand-written prose, no `atlas.tsv`, never touched by
`tools/render.py` — does not require an atlas to be discovered at all. `atlases/pokemon-gs/`
holds Gold and Silver's memory map, save file, screen-reading and Time Capsule facts as prose,
with no `meta.json`, so `tools/atlases.py` never finds it and `make check` never asks it for
data it does not have. A real, evidenced `pokemon-gs` atlas — extractor, `atlas.tsv`,
verification run — is still exactly the work [adding an atlas](docs/adding-an-atlas.md)
describes, and this prose does not substitute for it; it only means the citable facts TerminalGB
had already written down are not stuck waiting for that work to happen first.

### A pending atlas has no `meta.json`

`atlases/pokemon-gs/` exists and holds real prose, but `tools/atlases.py` discovers atlases
solely by the presence of `meta.json` (see "The data is namespaced by game" above), and this
directory deliberately has none. That is not an oversight — it is what keeps `validate`,
`evidence`, `render --check` and `export --check` from ever being asked about a `data/atlas.tsv`
that does not exist and should not be faked. `tools/checklinks.py` is the one tool that is *not*
gated this way: it walks every Markdown file in the repository regardless, so a pending atlas's
links and anchors are checked exactly like a published one's. **Do not add a `meta.json` to
`atlases/pokemon-gs/` without also doing the real work `docs/adding-an-atlas.md` describes** — an
extractor, a real `atlas.tsv`, a landed verification run — or the moment discovery turns on,
`validate.py` fails looking for a data file that was never built.

### Two licences, and the boundary is a path

**Content is CC BY-SA 4.0; code is MIT.** `atlases/**`, `docs/**.md`, `docs/brand/**` and
the root prose are content (`LICENSE-CC-BY-SA`); `tools/**`, `schema/**`, the `Makefile`
and `.github/**` are code (`LICENSE`). The split is not cosmetic: the schema and the
tooling are permissive *on purpose*, because somebody producing a compatible atlas
elsewhere is worth more than an exclusive one here, and a schema you need permission to
implement is a schema nobody implements. Do not "tidy" a file across the boundary without
moving its licence with it.

The reasoning lives in `docs/licensing.md` and it is the page to amend, never to replace:
its argument that **the addresses are facts and the symbol names are the community's** is
why the licence is share-alike rather than permission-required. A licence that gated the
addresses would be claiming the half that is not ours.

Two facts that are easy to state wrongly and are stated plainly in several places, so keep
them consistent if you touch any of them:

- **MIT already granted cannot be revoked.** Every copy published before the relicence
  stays MIT — including the atlas vendored into public TerminalGB at
  `third_party/atlasgb/pokemon-rb/atlas.tsv`. It is a **re-pin** that brings CC BY-SA in,
  not the passage of time.
- **Share-alike reaches the compilation, the evidence and the prose — not the fact that
  `$D05A` is `wBattleType`.** Anyone may re-derive every address from the public
  disassembly. Saying so is the point; overclaiming is what `docs/licensing.md` exists to
  avoid.

One known trade-off, taken deliberately: `LICENSE` opens with the path table rather than
with the bare MIT text, so GitHub's licence detector will likely show "Other" in the
sidebar instead of "MIT". That is the honest outcome — a sidebar reading "MIT" over a
repository whose data is share-alike is worse than one reading nothing — and the two
README badges carry the real answer. Not verified against a live render; if it matters,
check it on github.com rather than assuming either way.

Three generated things carry the licence and will silently go stale if the terms move:
the two README badges in `tools/render.py:badges_block`, the `license` / `license_url` /
`attribution` keys in `tools/export.py:meta` (they exist because `atlas.json` gets copied
away from the LICENSE files), and the SVG header comment in `docs/brand/generate.py`.

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
| `LICENSE` | MIT, over `tools/`, `schema/`, the `Makefile` and CI. Opens with the path table |
| `LICENSE-CC-BY-SA` | CC BY-SA 4.0, over the atlas content and the prose. Verbatim CC text under a short scope note |
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

### Re-running the verification yourself: `cargo test --release` can fail spuriously

`testharness/gen1atlas.rs` in TerminalGB is what produces an evidence report — see
[docs/verification.md](docs/verification.md#running-it-yourself) for the full command.
**`cargo test --release --test gen1atlas` intermittently fails there with `the crate X
requires panic strategy 'abort'`.** This is a known, documented build-cache collision in
TerminalGB itself (`docs/pitfalls/builds.md`, `[profile.release]` sets `panic = 'abort'`
but a test target needs to unwind) — not a real error, and not anything wrong with the
atlas or this repository. `cargo test --profile quick --test gen1atlas` sidesteps it
outright (that profile sets `panic = "unwind"` and skips LTO) and produces identical
pass/fail/tier results; only wall-clock time differs. Confirmed 2026-08-25: two runs under
`--profile quick`, byte-identical, both agreeing with the previously-published run.

### A fault in the game and a fault in this atlas's own data are different homes

[`atlases/pokemon-rb/docs/discoveries.md`](atlases/pokemon-rb/docs/discoveries.md) is
where the *cartridge* surprising us goes. [`docs/data-issues.md`](docs/data-issues.md) is
where *this atlas being wrong* goes — an address, a role, a structure size or an evidence
tier that a verification run shows disagreeing with the cartridge. The distinction is not
cosmetic: conflating them would mean a reader cannot tell "the game does something
strange" from "we published something wrong and fixed it," and those carry opposite
implications for whether the *rest* of the atlas can be trusted. Every future verification
run — a clean one or one that finds a disagreement — gets logged in `data-issues.md`,
including a clean one, because a verification pass that found nothing wrong is itself
worth recording as of when it ran.

### Descriptions can be generated, not only found — see `tools/gen1-observe/`

When the well of already-written-down evidence (a sibling entry sharing an address, a
comment in TerminalGB's own plugin source) runs dry — it very nearly did, after round two's
seventeen new descriptions — the next place to look is not the symbol name. It is the
cartridge itself: [`tools/gen1-observe/`](tools/gen1-observe/) is a standalone Rust crate
(a path dependency on TerminalGB's published library, nothing written into that repository)
that steps a real playthrough and records, per address, which code writes it, when, and to
what values — see [`docs/observation.md`](docs/observation.md) for the results and
[the tool's own README](tools/gen1-observe/README.md) for the method. It found and worked
around a real trap worth knowing generally: **a ROM entry's `len` column is "distance to the
next atlas entry in the same bank," which is exhaustively accurate for WRAM/HRAM but can be
wildly inflated for a sparsely-covered ROM bank** — `ItemNames`' recorded length is 13,598
bytes because nothing else in its bank is named yet, not because the table is that large, and
an unguarded range match against it credited it with writing dozens of unrelated bytes before
a small-window cap caught it. Never use a ROM entry's `len` for anything beyond the
completeness invariant it was built for.

Two more traps the same tool found on its second round, both standing rules for anyone
extending it, not one-off fixes:

- **`ROM0` (`$0000`-`$3FFF`) is always mapped, regardless of the bank register.** Grouping its
  writers by `(bank, pc)` instead of by `pc` alone splits one shared routine into several
  apparently-different ones, by the accident of whatever bank happened to be switched in when
  each write happened — a 39-address generic byte-copy utility at `$00B6` was nearly credited
  as evidence for a dozen unrelated relationships this way. For any PC below `$4000`, group
  and look it up by PC alone.
- **A script's own `debug_write()` setup calls can be misattributed to whatever instruction
  happens to run first in the next traced frame.** The tool forces states directly (a battle,
  a stocked bag) the same way TerminalGB's own debug menu does, which is legitimate, but the
  *detection* of that write is a diff against a shadow map taken once at the start of the run
  — so the forced write's "before" value is stale by the time it is compared, and it can look
  like the game's own code wrote two addresses in one instruction when really this tool wrote
  them itself, outside any frame, well before either was ever diffed. Any address the script
  directly `debug_write()`s must be excluded from writer-grouping and co-occurrence analysis
  entirely, not merely have its first event distrusted.

Grouping addresses by their **writing routine** — not by naive frame- or instruction-level
co-occurrence, which mostly surfaces things that already change every frame regardless of
each other — is what actually produced measured linkage: a sound-engine channel-init routine
writing nine per-channel fields together, two shadow-OAM slots never touched independently
regardless of which of several routines is doing the touching, a stat-stage reset routine
distinct from the one an existing description already credited. See
[`docs/observation.md`](docs/observation.md) for the full, current list — that page, not this
one, is where new rounds' findings accumulate.

### The `related` column names symbols, and only a shared writer earns a listing

`related` is the eleventh column, "ours" alongside `verify` and `desc`: a comma-separated
list of other symbols this one is measured to be read or written alongside, the same shape
as `verify` and likewise turned into a JSON list on export. `tools/validate.py` fails if a
`related` cell names a symbol that does not exist elsewhere in the atlas, so a rename cannot
leave it silently pointing at nothing.

Its first backfill (31 measured writer-groups, 54 addresses, from
[`docs/observation.md`](docs/observation.md#the-31-groups-in-full)) populated only **six** of
those groups — the audio channel-state fields, the shadow-OAM sprite-pair fields (both the
live pair and the backup/restore pair), the battle stat-stage fields, and the two party
experience-split flags — because those are the ones that round's own prose actually examined
and stood behind. **Two groups from the same table were deliberately left out**: a 13-member
`(ROM0, $36E3)` group and a 4-member `(ROM0, $1E7E)` group, neither of which the prose ever
discusses individually, and the `(ROM0, $0F39)` group, which the prose names explicitly and
then declines — "plausibly one generic 'copy N bytes' utility reused across three unrelated
features... not treated as evidence that these three features are related in any way beyond
sharing a low-level tool." A shared writer is *measured*, but "measured" and "examined" are
not the same thing, and this column only wants the second. Extending the backfill from the
raw group table means examining the group first, the same way the channel-state and
shadow-OAM groups were, not copying rows across because they happen to already be counted.

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
