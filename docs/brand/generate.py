#!/usr/bin/env python3
"""Generate the AtlasGB brand SVGs.

Every letterform is hand-drawn here as a stroked skeleton path — **no font is
embedded, subset or traced**, so there is no third-party licence in these files.
This is the same approach, the same palette and the same panel as TerminalGB's
`docs/brand/generate.py`, because AtlasGB is a sibling project and the two marks
should read as a family; the *motif* is AtlasGB's own.

Run from anywhere:  python3 docs/brand/generate.py
It rewrites `atlasgb-logo.svg` and `atlasgb-icon.svg` deterministically.  Edit
this file, never the SVGs by hand.
"""

import os

OUT = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# Palette — TerminalGB's, unchanged.  A sibling that recoloured would read as a
# different project rather than a companion one.
BG = "#0d1117"         # near-black panel (matches GitHub dark, works on light)
EDGE = "#30363d"       # faint panel border so the card reads on pure black too
FG = "#f0f3f6"         # wordmark white
GREY = "#8b949e"       # tagline grey
DMG_LIGHT = "#9bbc0f"  # DMG LCD lit shade — the single accent

TAGLINE = "A MEMORY MAP WITH EVIDENCE"

# ---------------------------------------------------------------------------
# Stroke-skeleton capital letters.  Cap height 100, stroke 26 (half-stroke 13);
# every endpoint is inset 13 so round caps land on the ink edge.  The value is
# (advance width, list of path data).
#
# A B E G I L M N O R T U Y are the shapes TerminalGB already draws, unchanged
# so the two wordmarks are visibly the same alphabet.  C D H P S V W are new
# here and drawn to the same rules: same cap height, same inset, same stroke.
S = 13
GLYPHS = {
    'T': (72, ["M13 13 H59", "M36 13 V87"]),
    'E': (56, ["M43 13 H13 V87 H43", "M13 50 H36"]),
    'R': (68, ["M13 87 V13 H36 A19 19 0 0 1 36 51 H13", "M37 54 L54 87"]),
    'M': (86, ["M13 87 V13 L43 57 L73 13 V87"]),
    'I': (26, ["M13 13 V87"]),
    'N': (64, ["M13 87 V13 L51 87 V13"]),
    'A': (76, ["M11 87 L38 15 L65 87", "M23 62 H53"]),
    'L': (54, ["M13 13 V87 H41"]),
    'G': (70, ["M57 13 H13 V87 H57 V56 H42"]),
    'B': (66, ["M13 13 V87", "M13 13 H35 A18 18 0 0 1 35 49 H13",
               "M13 49 H37 A19 19 0 0 1 37 87 H13"]),
    'O': (64, ["M32 13 A19 37 0 1 0 32 87 A19 37 0 1 0 32 13"]),
    'U': (64, ["M13 13 V68 A19 19 0 0 0 51 68 V13"]),
    'Y': (64, ["M13 13 L32 47 L51 13", "M32 47 V87"]),
    # --- new for AtlasGB, same construction ---
    # S: two wide half-turns with *flat* terminals.  Curling the terminals back
    # into the bowls — the textbook skeleton S — closes the counters to a
    # pinhole at this stroke weight; cutting them flat keeps the letter open at
    # the same weight as B's bowls, which is the alphabet's reference.
    'S': (72, ["M57 13 H36 A23 18.5 0 0 0 36 50 A23 18.5 0 0 1 36 87 H15"]),
    # C: the same ellipse as O, opened between the two 60-degree terminals.
    'C': (60, ["M42 18 A19 37 0 1 0 42 82"]),
    'D': (68, ["M13 13 V87", "M13 13 H31 A24 37 0 0 1 31 87 H13"]),
    'P': (62, ["M13 87 V13", "M13 13 H32 A17 18 0 0 1 32 49 H13"]),
    'V': (64, ["M13 13 L32 87 L51 13"]),
    'W': (96, ["M13 13 L33 87 L48 34 L63 87 L83 13"]),
    'H': (66, ["M13 13 V87", "M53 13 V87", "M13 50 H53"]),
    ' ': (30, []),
}
TRACK = 8  # tight letter spacing


def word_width(text, track=TRACK, widths=None):
    w = 0
    for i, ch in enumerate(text):
        w += (widths.get(ch) if widths and ch in widths else GLYPHS[ch][0])
        if i < len(text) - 1:
            w += track
    return w


def draw_word(text, x, y, scale, color, track=TRACK):
    """SVG for `text` with the letter grid's top-left at (x, y)."""
    parts = []
    cx = 0.0
    for ch in text:
        w, paths = GLYPHS[ch]
        for d in paths:
            parts.append(
                f'<path transform="translate({x + cx * scale:.1f} {y:.1f}) '
                f'scale({scale:.4f})" d="{d}" fill="none" stroke="{color}" '
                f'stroke-width="26" stroke-linecap="round" '
                f'stroke-linejoin="round"/>')
        cx += w + track
    return "\n".join(parts)


def svg(width, height, body, comment):
    return (f"<!-- {comment}\n"
            "     Hand-authored for AtlasGB (CC BY-SA 4.0). No font embedded, "
            "subset or traced;\n     letterforms are original stroked paths. "
            "Regenerate with docs/brand/generate.py -->\n"
            f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} '
            f'{height}" width="{width}" height="{height}" role="img">\n'
            f"{body}\n</svg>\n")


def panel(w, h, rx=24):
    return (f'<rect x="1" y="1" width="{w - 2}" height="{h - 2}" rx="{rx}" '
            f'fill="{BG}" stroke="{EDGE}" stroke-width="2"/>')


def write(name, content):
    path = os.path.join(OUT, name)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as handle:
        handle.write(content)
    print(f"wrote {name} ({len(content)} bytes)")


# ---------------------------------------------------------------------------
# THE MARK.
#
# TerminalGB's mark is a solid block cursor standing where a letter would be,
# closed by an accent-colour full stop.  AtlasGB keeps the panel, the alphabet,
# the palette and the full stop — that is the family — and brings its own motif:
# **the region bar**.  Four segments under the wordmark, one for each RAM region
# the atlas walks, contiguous with no holes, the last one short because high RAM
# really is 127 bytes against work RAM's 8,192.  It is the shape of the claim
# the project makes: the whole space, accounted for, nothing missing.
#
# Nothing here resembles any Nintendo mark, logotype or trade dress; the whole
# vocabulary is a bar chart and a full stop.

# Video RAM, cartridge RAM, work RAM, high RAM.  The last is deliberately a
# stub — the segments say "four regions, one of them tiny", not "to scale".
SEGMENTS = (3, 3, 3, 1)
SEG_GAP = 12


def region_bar(x, y, width, height, gap=SEG_GAP, colour=DMG_LIGHT):
    units = sum(SEGMENTS)
    inner = width - gap * (len(SEGMENTS) - 1)
    parts, cx = [], x
    for n in SEGMENTS:
        w = inner * n / units
        parts.append(f'<rect x="{cx:.1f}" y="{y:.1f}" width="{w:.1f}" '
                     f'height="{height}" rx="{height / 2:.1f}" fill="{colour}"/>')
        cx += w + gap
    return "\n".join(parts)


def logo():
    W, H = 1200, 380
    text = "ATLASGB"
    DOT_R = 16          # the full stop, bottom-aligned with the letter ink
    scale = 1.42
    total = (word_width(text) + TRACK + 2 * DOT_R) * scale
    x0 = (W - total) / 2
    y0 = 84
    parts = [panel(W, H)]
    parts.append(draw_word(text, x0, y0, scale, FG))
    cx = word_width(text) + TRACK
    parts.append(
        f'<circle cx="{x0 + (cx + DOT_R) * scale:.1f}" '
        f'cy="{y0 + (100 - DOT_R) * scale:.1f}" r="{DOT_R * scale:.1f}" '
        f'fill="{DMG_LIGHT}"/>')
    parts.append(region_bar(x0, 262, total, 14))
    tw = word_width(TAGLINE, track=14) * 0.30
    parts.append(draw_word(TAGLINE, (W - tw) / 2, 306, 0.30, GREY, track=14))
    write("atlasgb-logo.svg",
          svg(W, H, "\n".join(parts),
              "AtlasGB logo — the wordmark, the full stop, the region bar"))


def icon():
    """The region bar alone, stacked, at avatar and favicon size.

    Every coordinate is a multiple of 8 inside the 128 box, so at a 16 px
    favicon each unit lands on a whole device pixel and the four rows stay four
    rows instead of blurring into one.
    """
    IW = 128
    body = [panel(IW, IW, rx=28)]
    for i, n in enumerate(SEGMENTS):
        w = 80 if n == 3 else 32
        y = 24 + i * 24
        body.append(f'<rect x="24" y="{y}" width="{w}" height="16" rx="8" '
                    f'fill="{DMG_LIGHT}"/>')
    write("atlasgb-icon.svg",
          svg(IW, IW, "\n".join(body),
              "AtlasGB icon — the region bar, stacked"))


if __name__ == "__main__":
    logo()
    icon()
