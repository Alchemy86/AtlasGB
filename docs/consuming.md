# Using an atlas in your own project

[← AtlasGB](../README.md) · [the schema](schema.md) · [verification](verification.md) ·
[adding an atlas](adding-an-atlas.md)

This repository is meant to be **consumed**. You are writing an emulator, a save editor, a
randomiser, a mod tool, a debugger, a bot — and you want the addresses without adopting
anybody's build system. Nothing here depends on any other project, and using it does not
require you to run any of the tooling in `tools/`.

**Consume one atlas at a time.** AtlasGB publishes one atlas per cartridge, under
`atlases/<id>/`; the examples below use `pokemon-rb`, which is the only one published
today. Every path here has the atlas id in it on purpose — an address is a fact about *a
cartridge*, and code that forgets which one is the bug this layout exists to make hard.

---

## Just the data

Everything is a plain file. Fetch what you want:

```bash
BASE=https://raw.githubusercontent.com/Alchemy86/AtlasGB/main/atlases/pokemon-rb/data

# the source of truth, tab separated (see docs/schema.md for the columns)
curl -fsSLO $BASE/atlas.tsv

# the same rows as typed JSON, or minified for the network
curl -fsSLO $BASE/atlas.json
curl -fsSLO $BASE/atlas.min.json
```

Pin a tag rather than `main` if you want a stable answer:
`.../AtlasGB/<tag>/atlases/pokemon-rb/data/atlas.tsv`.

The JSON's own `meta` block names the cartridge it is about — `atlas`, `title` and `games`
— so a file that has been copied out of its directory can still say what it is. Read it
rather than assuming.

> **These paths changed** when the repository was namespaced by game; the Red/Blue data
> used to sit at `data/atlas.tsv`. The rows did not change, so an existing pinned snapshot
> is still correct — only a refresh needs the new URL. See
> [`../data/README.md`](../data/README.md).

Five lines of Python is a complete reader:

```python
import csv

with open("atlas.tsv", encoding="utf-8", newline="") as f:
    atlas = {r["symbol"]: r for r in csv.DictReader(f, delimiter="\t")}

print(atlas["wPartyCount"]["addr"])       # $D163
print(atlas["wPartyCount"]["verify"])     # rom,live,inv
```

And in JavaScript, from the JSON:

```js
const { entries } = await (await fetch("atlas.min.json")).json();
const byAddr = new Map(entries.map((e) => [e.addr_int, e]));
byAddr.get(0xd163).symbol;                // "wPartyCount"
```

**Read [the schema](schema.md) before you write the parser.** In particular:
`(region, bank, addr)` is the identifying triple and not `addr` alone; `role` tells you
which rows are canonical; `verify` is the evidence and an empty one means *not observed*,
never *not used*.

---

## Vendoring it, if you build against it

If your build or your tests read the atlas, **vendor a snapshot and pin it.** Not a
submodule, and not a fetch at build time:

- a **submodule** breaks shallow CI checkouts and source tarballs, and makes a
  one-file dependency cost a clone;
- a **build-time fetch** puts the network on the path of your test suite, which is fatal
  if — as in the emulator that verifies this atlas — the structural checks are supposed to
  run on every plain `cargo test`;
- a **vendored snapshot** is one committed file that is right there in the diff when it
  changes, which is the reviewable option.

The one thing a vendored snapshot needs that people usually forget is a **lock and a
freshness check**, because a vendored file is exactly the file somebody will eventually
hand-edit instead of fixing upstream.

[`tools/fetch-atlas.sh`](../tools/fetch-atlas.sh) does the whole thing, and needs nothing
but `curl` and `sha256sum` — copy it into your repository:

```bash
# pin the current release into third_party/atlasgb/
tools/fetch-atlas.sh --ref v1.0.0 --atlas pokemon-rb --dest third_party/atlasgb

# later: is the vendored copy still the file we pinned?
tools/fetch-atlas.sh --verify --dest third_party/atlasgb

# and: has upstream moved on? (network; advisory, not a failure)
tools/fetch-atlas.sh --check-upstream --dest third_party/atlasgb
```

`--atlas` defaults to `pokemon-rb` and names the directory under `atlases/`. If you vendor
more than one, give each its own `--dest` — `third_party/atlasgb/pokemon-rb/` — so the
locks do not collide and a reader can see which cartridge a file is about from its path.
`--file` overrides the path inside the repository, which is needed only for a ref from
before the atlases were namespaced.

It writes two files:

```
third_party/atlasgb/atlas.tsv        the snapshot
third_party/atlasgb/atlas.lock       atlas, ref, commit, sha256, fetched-on
```

`--verify` is offline and cheap — run it in your test suite, not just in CI. It is what
catches the hand-edit.

---

## The anti-drift gate

This is the check worth copying even if you take nothing else.

**Every cartridge address hard-coded anywhere in your source must appear in that
cartridge's atlas.** In the emulator this project came out of, `wCurMap` was written out in
nine separate files before the gate existed and nothing could tell you whether they agreed.
The gate is a text scan over a named list of files, looking for constants of the shape a
Game Boy address is written in, and it costs nothing:

```rust
// Only *named* constants of the right width: an inline literal may be a length,
// a mask or a species id, and a scan that guessed would cry wolf.
const NAMED: &str = "const NAME: u16 = 0xXXXX;";
```

For every such constant, require that its value is the `addr` of some atlas entry in a
storage region. A new consumer joins the gate by being named in the file list — which is
the point: adding a file is a one-line change, and forgetting to is what the gate exists
to make visible.

It settles arguments, too. `battle_struct` is **29** bytes; two files disagreed (28 and
29), and the atlas plus `wTrainerClass` sitting immediately after `wBattleMonPP + 4`
proves it.

---

## Publishing verification back

If you have an emulator and a cartridge, you can do the thing this repository cannot do
for itself: **prove entries and publish the tiers back.** That is not a nice-to-have — an
atlas whose evidence never gets re-run becomes a transcription again, slowly.

The report format and the loop are in [verification.md](verification.md). The short
version: emit one JSON object naming the atlas and mapping every symbol to its evidence
tokens, open a pull request that runs `tools/apply-evidence.py report.json --atlas <id>`,
and the run's provenance — your repository, your commit, the cartridge's SHA-1, the script
— is recorded beside the data.

Your harness already computes the tokens if it does any of this work; emitting them is a
few lines. In Rust, roughly:

```rust
// `fresh` is what this run observed: symbol -> the tokens it earned.
// Write it under an env var so an ordinary test run does not produce a file.
if let Ok(path) = std::env::var("ATLAS_EVIDENCE_OUT") {
    let mut out = String::from(
        "{\n \"schema\": \"atlasgb-evidence/1\",\n \"atlas\": \"pokemon-rb\",\n");
    out += &format!(
        " \"produced_by\": {{\"repo\": \"{repo}\", \"commit\": \"{commit}\", \
          \"harness\": \"{harness}\"}},\n \
         \"cartridge\": {{\"title\": \"{title}\", \"sha1\": \"{sha1}\"}},\n \
         \"script\": {{\"name\": \"{script}\", \"frames\": {frames}}},\n \
         \"run\": {{\"date\": \"{date}\"}},\n \"verify\": {{\n"
    );
    // EVERY symbol in the atlas, including the ones that earned nothing —
    // an omission would be read as a downgrade, and apply-evidence.py
    // refuses a partial report rather than guessing.
    let rows: Vec<String> = entries.iter().map(|e| {
        let toks: Vec<String> = fresh[&e.symbol].iter()
            .map(|t| format!("\"{t}\"")).collect();
        format!("  \"{}\": [{}]", e.symbol, toks.join(", "))
    }).collect();
    out += &rows.join(",\n");
    out += "\n }\n}\n";
    std::fs::write(path, out).expect("write the evidence report");
}
```

Guard it behind an environment variable rather than writing on every run: the report is a
deliberate publication, and a file that appears whenever the tests run is a file somebody
commits by accident.

**The highest-value contribution available right now** is a second live script. The
current one plays the opening and then walks and opens menus, so it never reaches a battle
or a PC; a script that starts from a save, fights something and opens the storage system
would move several hundred entries from "no evidence" to observed. The machinery exists.
Only the script is missing.

---

## What you should not do

- **Do not hand-edit the `verify` column.** It is written by landing a verification report
  and CI checks the digest; a hand-edited tier is a claim with nothing behind it, which is
  the failure mode this whole project is a reaction to.
- **Do not treat an unmarked entry as unused.** It means nobody has observed it yet.
- **Do not use one atlas's addresses for another cartridge.** In particular, not for
  Pokémon Yellow. Its work RAM shifted, so an address from the Red/Blue atlas is *wrong*
  there rather than approximately right, and the failure mode is a write landing in the
  middle of somebody else's data and passing its own checksum. A cartridge with no atlas here has no atlas
  here; see [adding one](adding-an-atlas.md).
- **Do not ship game data.** The addresses are facts and are free to use; the cartridge's
  contents are not ours or yours to distribute. Locate the tables in the player's own
  cartridge at run time, as [rom-data](../atlases/pokemon-rb/docs/rom-data.md) describes.
