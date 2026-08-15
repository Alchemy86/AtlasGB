# AtlasGB.
#
#   make check      everything CI runs. Do this before you push.
#   make docs       regenerate the pages and the README's tables from the data
#   make data       regenerate the JSON exports from the data
#   make brand      regenerate the logo and icon (needs ImageMagick for previews)
#   make extract    refresh the DERIVED columns from a built pokered checkout
#
# `data/atlas.tsv` is the single source of truth. Everything else is generated
# from it, so a number in the prose cannot drift away from the map it describes
# — and `make check` is red until every generated file agrees with the data.
#
# The `verify` column is NOT editable by hand: it is written by landing a
# verification run (tools/apply-evidence.py) and CI checks the digest. See
# docs/verification.md.

PYTHON ?= python3

.PHONY: all check docs data brand extract links validate evidence clean help

all: docs data

help:
	@sed -n '3,9p' $(MAKEFILE_LIST) | sed 's/^# \{0,1\}//'

# Everything that runs in CI, in the order that gives the most useful first
# failure: is the data sound, does the published evidence still name a real
# run, do the generated files match it, does every link resolve.
check: validate evidence
	$(PYTHON) tools/render.py --check
	$(PYTHON) tools/export.py --check
	$(PYTHON) tools/checklinks.py

# The structural checks that need nothing but the file itself. The ones that
# need a cartridge live in the emulator that publishes evidence back here.
validate:
	$(PYTHON) tools/validate.py

# Is the verify column still the one the last landed verification run produced?
evidence:
	$(PYTHON) tools/apply-evidence.py --check

# Rewrite every generated block in docs/*.md and in README.md. Prose outside
# the <!-- atlas:… --> markers is preserved.
docs:
	$(PYTHON) tools/render.py

# Rewrite data/atlas.json and data/atlas.min.json.
data:
	$(PYTHON) tools/export.py

links:
	$(PYTHON) tools/checklinks.py --external

# The mark. Regenerating the previews needs ImageMagick; the SVGs do not.
brand:
	$(PYTHON) docs/brand/generate.py
	@command -v magick >/dev/null 2>&1 || { \
	  echo "note: ImageMagick not found — SVGs written, previews left alone"; exit 0; }
	cd docs/brand && magick -background none atlasgb-logo.svg preview/logo.png
	cd docs/brand && magick -background none atlasgb-icon.svg preview/icon-128.png
	cd docs/brand && magick -background none atlasgb-icon.svg -resize 16x16 preview/icon-16.png

# Refresh the eight DERIVED columns from a built pret/pokered checkout. Needs
# RGBDS 1.0.3 and a build whose ROM matches pokered's own roms.sha1; nothing is
# vendored and nothing is fetched. See docs/provenance.md.
#
#   make extract POKERED=~/src/pokered
extract:
	@test -n "$(POKERED)" || { echo "set POKERED=/path/to/a/built/pokered"; exit 2; }
	$(PYTHON) tools/extract.py --pokered "$(POKERED)" --write
	$(MAKE) docs data

clean:
	find . -name '__pycache__' -type d -prune -exec rm -rf {} +
