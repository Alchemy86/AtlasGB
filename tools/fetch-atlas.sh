#!/bin/sh
# Vendor a pinned snapshot of one AtlasGB atlas into your own repository.
#
# This script is meant to be COPIED into a consuming project. It needs nothing
# but `curl` and `sha256sum` (or `shasum -a 256`), so vendoring the atlas adds
# no dependency to your build.
#
# AtlasGB publishes one atlas per cartridge under `atlases/<id>/`, so every
# invocation names one. `--atlas` defaults to pokemon-rb, which is the only one
# published today; pinning a second cartridge is a flag, not a restructuring.
#
#   tools/fetch-atlas.sh --ref v1.0.0 --dest third_party/atlasgb
#       fetch that ref, write the file and a lock recording atlas/ref/commit/sha256.
#
#   tools/fetch-atlas.sh --ref v1.0.0 --atlas pokemon-rb --dest third_party/atlasgb
#       the same, said explicitly. Worth doing once there is more than one.
#
#   tools/fetch-atlas.sh --ref 91b5d18 --file data/atlas.tsv --dest third_party/atlasgb
#       override the path inside the repository. Needed only for a ref from
#       BEFORE the atlases were namespaced by game, when the Red/Blue data sat
#       at data/atlas.tsv.
#
#   tools/fetch-atlas.sh --verify --dest third_party/atlasgb
#       OFFLINE. Recompute the sha256 and compare it to the lock. This is the
#       check that matters day to day: it catches somebody hand-editing the
#       vendored copy instead of fixing it upstream. Cheap enough to run inside
#       a test suite, not only in CI.
#
#   tools/fetch-atlas.sh --check-upstream --dest third_party/atlasgb
#       Network. Report whether upstream's default branch has moved past the
#       pinned commit. ADVISORY: exits 0 either way, because "upstream released
#       something" is news, not a build failure.
#
# Why vendored rather than a submodule or a build-time download: a submodule
# breaks shallow CI checkouts and source tarballs and makes a one-file
# dependency cost a clone; a build-time download puts the network on the path
# of your test suite. A committed snapshot shows up in the diff when it moves,
# which is the reviewable option. See docs/consuming.md.
#
# Licence: THIS SCRIPT is MIT, like everything in AtlasGB's tools/ — copy it,
# change it, ship it. The atlas content it fetches is CC BY-SA 4.0: credit
# AtlasGB, and share a modified or extended atlas under the same terms. A
# snapshot pinned before that relicence is MIT and stays MIT; it is a re-pin
# that brings the new terms in. See docs/licensing.md.

set -eu

REPO="Alchemy86/AtlasGB"
ATLAS="pokemon-rb"
FILE=""
NAME="atlas.tsv"
LOCK="atlas.lock"

REF=""
DEST="third_party/atlasgb"
MODE="fetch"

usage() {
	sed -n '2,38p' "$0" | sed 's/^# \{0,1\}//'
	exit "${1:-0}"
}

while [ $# -gt 0 ]; do
	case "$1" in
	--ref) REF="$2"; shift 2 ;;
	--atlas) ATLAS="$2"; shift 2 ;;
	--file) FILE="$2"; shift 2 ;;
	--dest) DEST="$2"; shift 2 ;;
	--verify) MODE="verify"; shift ;;
	--check-upstream) MODE="upstream"; shift ;;
	-h | --help) usage 0 ;;
	*) echo "unknown argument: $1" >&2; usage 2 ;;
	esac
done

# The path inside the repository, unless --file overrode it.
[ -n "$FILE" ] || FILE="atlases/$ATLAS/data/atlas.tsv"

sha256() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | cut -d' ' -f1
	else
		shasum -a 256 "$1" | cut -d' ' -f1
	fi
}

lock_get() {
	# The lock is `key=value` per line: no JSON parser required, on purpose.
	sed -n "s/^$1=//p" "$DEST/$LOCK" 2>/dev/null || true
}

case "$MODE" in
verify)
	[ -f "$DEST/$NAME" ] || { echo "FAIL: $DEST/$NAME is missing" >&2; exit 1; }
	[ -f "$DEST/$LOCK" ] || { echo "FAIL: $DEST/$LOCK is missing" >&2; exit 1; }
	want="$(lock_get sha256)"
	got="$(sha256 "$DEST/$NAME")"
	if [ "$want" != "$got" ]; then
		echo "FAIL: $DEST/$NAME does not match $DEST/$LOCK" >&2
		echo "  locked $want" >&2
		echo "  actual $got" >&2
		echo "  The vendored copy is not the one that was pinned. Do not edit it" >&2
		echo "  here — fix it upstream at https://github.com/$REPO and re-pin." >&2
		exit 1
	fi
	echo "ok: $DEST/$NAME matches the lock ($(lock_get atlas), $(lock_get ref), ${got%"${got#??????????}"}…)"
	;;

upstream)
	pinned="$(lock_get commit)"
	head="$(curl -fsSL "https://api.github.com/repos/$REPO/commits/HEAD" |
		sed -n 's/^  "sha": "\([0-9a-f]*\)",*$/\1/p' | head -1)"
	if [ -z "$head" ]; then
		echo "note: could not reach github.com; nothing checked"
		exit 0
	fi
	if [ "$pinned" = "$head" ]; then
		echo "ok: pinned to the current head of $REPO ($pinned)"
	else
		echo "note: $REPO has moved on."
		echo "  pinned  $pinned  ($(lock_get ref), fetched $(lock_get fetched))"
		echo "  current $head"
		echo "  Re-pin with: $0 --ref <tag> --atlas $(lock_get atlas) --dest $DEST"
		echo "  If the evidence tiers changed, that is a real update: they are"
		echo "  the reason to track this file rather than freeze it."
	fi
	;;

fetch)
	[ -n "$REF" ] || { echo "give --ref (a tag, a branch or a commit)" >&2; exit 2; }
	mkdir -p "$DEST"
	tmp="$(mktemp)"
	trap 'rm -f "$tmp"' EXIT
	url="https://raw.githubusercontent.com/$REPO/$REF/$FILE"
	echo "fetching $url"
	curl -fsSL "$url" -o "$tmp"
	# Resolve the ref to the commit it actually named, so the lock records what
	# was fetched rather than a moving label.
	commit="$(curl -fsSL "https://api.github.com/repos/$REPO/commits/$REF" |
		sed -n 's/^  "sha": "\([0-9a-f]*\)",*$/\1/p' | head -1)"
	[ -n "$commit" ] || commit="(unresolved)"
	head -1 "$tmp" | grep -q '^region	bank	addr' || {
		echo "FAIL: what came back is not an atlas (no header row)" >&2
		exit 1
	}
	mv "$tmp" "$DEST/$NAME"
	trap - EXIT
	cat >"$DEST/$LOCK" <<EOF
# Pinned AtlasGB snapshot. Written by tools/fetch-atlas.sh — do not hand-edit,
# and do not hand-edit $NAME either: fix it upstream and re-pin.
#
# Terms: AtlasGB's atlas content is CC BY-SA 4.0 — credit AtlasGB, and share a
# modified or extended atlas under the same terms. This script cannot tell you
# which terms *this* snapshot came under, because a ref from before the
# relicence is MIT and stays MIT: check the LICENSE files at the commit below.
# Either way the tooling — including this script — is MIT and free of that
# condition. See docs/licensing.md.
repo=https://github.com/$REPO
attribution=AtlasGB — https://github.com/$REPO — atlas $ATLAS @ $REF
atlas=$ATLAS
file=$FILE
ref=$REF
commit=$commit
sha256=$(sha256 "$DEST/$NAME")
fetched=$(date -u +%Y-%m-%d)
EOF
	echo "wrote $DEST/$NAME and $DEST/$LOCK"
	echo "  atlas  $ATLAS"
	echo "  file   $FILE"
	echo "  ref    $REF"
	echo "  commit $commit"
	echo "  sha256 $(sha256 "$DEST/$NAME")"
	;;
esac
