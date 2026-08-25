# Data issues — where a fault in our own record is logged

[← AtlasGB](../README.md) · [verification](verification.md) · [provenance](provenance.md)

**A fault in Pokémon and a fault in our record of Pokémon are different animals, and this
project keeps them in different places.** [Discoveries](../atlases/pokemon-rb/docs/discoveries.md)
is the home for the first kind: something the *cartridge* does that surprised us, or a
belief about the *game* — ours or somebody else's — that turned out wrong. This page is the
home for the second kind: an entry in `atlas.tsv` — an address, a role, a structure size, an
evidence tier — that was itself incorrect, caught by checking the atlas against the
cartridge rather than by playing the game.

**If you are not sure which one you have, ask this:** would the finding still be true if
this atlas had never been written? A soft-lock in Oak's Lab would — it is a fact about the
cartridge. `wPartyCount`'s address being off by one would not — it is a fact about *this
document*, and it belongs here, not in discoveries.md.

## How an entry gets corrected

The mechanism already exists and is described in full in [verification.md](verification.md)
— this page does not duplicate it, only points at it and keeps the readable history
`data/evidence.json` cannot carry on its own:

1. A [verification run](verification.md#confirmed-against-the-cartridge-independently-on-2026-08-25)
   compares every entry's `verify` tokens, freshly computed against a running cartridge,
   against what is currently published. **Any entry where they disagree is a data issue by
   definition** — the run says so directly (`verification.md`'s "Baseline" section, or the
   harness's own stdout: `"N entries disagree with the pinned atlas"`).
2. It is never corrected quietly. The disagreement is landed as an evidence report
   ([`tools/apply-evidence.py`](../tools/apply-evidence.py)), which rewrites only the
   `verify` column, never `desc` or any derived column — and it is logged here, by hand,
   with what moved and why, so a reader does not have to diff `evidence.json` against git
   history to find out the atlas was ever wrong about something.
3. If the disagreement turns out to be **our reading harness's fault, not the atlas's** —
   the emulator watching the wrong address, a script that never reached the byte it
   claimed to, a stale save — that is *also* logged here, because "the atlas was right and
   our check was wrong" is exactly as much a fact about this project's own reliability as
   the reverse, and hiding it would be the same dishonesty the unevidenced-entries policy
   exists to avoid.

A correction to `desc`, `region`, `addr`, `role` or any of the eight derived columns that is
**not** caught by a verification run — a transcription slip, a symbol misread from the map
file — is still a data issue and still belongs in this log, even though it lands through a
plain pull request rather than through `apply-evidence.py`. The mechanism differs; the
category does not.

## The log

Newest first. An entry that found nothing wrong is still worth recording — a verification
run that turns up zero disagreements is evidence the atlas is currently accurate, not a
non-event, and its absence from this log would look identical to nobody having checked.

### 2026-08-25 — independent re-verification, zero disagreements

A full re-run of [`testharness/gen1atlas.rs`](https://github.com/Alchemy86/TerminalGB/blob/main/testharness/gen1atlas.rs)
against the captain's own retail Pokémon Blue cartridge and save (commit
`5f61061f0565178152b93f47a0bd8d70cc3c0a15` of TerminalGB), run twice for determinism.
**0 of 2,898 entries disagreed with the pinned atlas.** Every invariant that had a save to
check against passed; none were skipped. Full detail: [verification.md](verification.md#confirmed-against-the-cartridge-independently-on-2026-08-25).

This also re-confirms the 2026-08-15 run that first produced these tiers — two independent
boots of the cartridge, eleven days apart, agree byte for byte on all 2,898 claims.

---

*No entry above this line has ever needed correcting.* When one does, it goes at the top,
in the same shape: the date, what moved, which side (the atlas or the check) was wrong and
how that was told apart, and a link to the run or the commit that fixed it.
