#!/usr/bin/env python3
"""Generate the notebook atlas and item at native pixel resolution.

Usage: python3 tools/gen-progress.py
"""
from pathlib import Path
import struct
import zlib


def color(rgb, alpha=255):
    return (*bytes.fromhex(rgb), alpha)


# Copied from gen-hud.py to keep the atlas in the same palette.
OUTLINE = color("0A0B0E")
STEEL_DARK = color("25282B")
STEEL_LOW = color("454A50")
STEEL_MID = color("747C86")
STEEL_HIGH = color("D8DDE4")
BRASS_LOW, BRASS_MID, BRASS_HIGH = map(color, ("756039", "B59654", "F0DC9C"))
COPPER_LOW, COPPER_MID, COPPER_HIGH = map(color, ("794C38", "B77658", "E1A17D"))
WELL = color("16191F", 0xF0)
TRANSPARENT = (0, 0, 0, 0)
PANEL_BODY = color("181A20", 0xE0)


def canvas(width, height):
    return [[TRANSPARENT] * width for _ in range(height)]


def rect(pixels, x, y, width, height, shade):
    for row in pixels[y:y + height]:
        row[x:x + width] = [shade] * width


def art(pixels, x, y, rows, palette):
    for dy, row in enumerate(rows):
        for dx, symbol in enumerate(row):
            if symbol != ".":
                pixels[y + dy][x + dx] = palette[symbol]


def node(pixels, x, light, mid, shade):
    """Draw a clipped 22×22 frame with a 16×16 well at (3, 3)."""
    rect(pixels, x + 2, 0, 18, 22, OUTLINE)
    rect(pixels, x, 2, 22, 18, OUTLINE)
    rect(pixels, x + 2, 1, 18, 20, shade)
    rect(pixels, x + 1, 2, 20, 18, shade)
    rect(pixels, x + 2, 1, 18, 1, light)
    rect(pixels, x + 1, 2, 1, 18, light)
    rect(pixels, x + 2, 2, 18, 18, mid)
    rect(pixels, x + 3, 3, 16, 16, WELL)


def tab(pixels, x, selected):
    """Draw a 48×14 tab with a panel-colored join when selected."""
    rect(pixels, x + 1, 40, 46, 14, OUTLINE)
    rect(pixels, x, 41, 48, 13, OUTLINE)
    rect(pixels, x + 1, 41, 46, 12, STEEL_LOW)
    rect(pixels, x + 2, 42, 44, 10, STEEL_DARK)
    rect(pixels, x + 2, 41, 44, 1, STEEL_MID)
    if selected:
        rect(pixels, x + 1, 42, 46, 12, STEEL_LOW)
        rect(pixels, x + 2, 42, 44, 12, PANEL_BODY)
        rect(pixels, x + 2, 41, 44, 1, STEEL_HIGH)
        rect(pixels, x + 19, 41, 10, 1, BRASS_HIGH)


# Center these 12×12 drawings in the 16×16 icon cells.
ICONS = (
    (  # Reach: a long arrow leaving a fixed post.
        "............",
        "........h...",
        "........mh..",
        ".h......lmh.",
        ".h.......lmh",
        ".h.hhhhhhhhh",
        ".m.mmmmmmmmm",
        ".m.......mhl",
        ".m......mhl.",
        "........ml..",
        "........l...",
        "............",
    ),
    (  # Acceleration: two rising chevrons.
        ".....hh.....",
        "....hhmh....",
        "...hh..mh...",
        "..hh....mh..",
        ".hm......mh.",
        ".m........m.",
        ".....hh.....",
        "....hhmh....",
        "...hh..mh...",
        "..hh....mh..",
        ".hm......mh.",
        ".m........m.",
    ),
    (  # Coin jump: a short vertical arrow over a foreshortened coin.
        ".....hh.....",
        "....hhmh....",
        "...hhhhmh...",
        "..hmmhhmmh..",
        ".....hm.....",
        ".....hm.....",
        ".....mm.....",
        "............",
        "..lbbbbbbl..",
        ".lbbbbbbssl.",
        ".lsbbbbsskl.",
        "..lkkkkkkl..",
    ),
    (  # Economy: one drop held at the neck of a broad vial.
        "....hhhh....",
        "....mmhm....",
        ".....hm.....",
        "...hmc.mm...",
        "..h..cp..m..",
        ".h..ccp.m.m.",
        ".h...pq...m.",
        ".h......m.m.",
        ".h........m.",
        ".m......m.m.",
        "..m......m..",
        "...mmmmmm...",
    ),
    (  # Capacity: a broad, filled vial with gauge marks on the glass.
        "....hhhh....",
        "....mmhm....",
        ".....hm.....",
        "...hm..mm...",
        "..h......m..",
        ".hc....h..m.",
        ".hccccccppm.",
        ".hcpppphppm.",
        ".hcppppppqm.",
        ".mcpppphpqm.",
        "..mqqqqqqm..",
        "...mmmmmm...",
    ),
    (  # Anchor principle: the lighter body moves toward the heavier body.
        "............",
        "............",
        "........hhh.",
        "....h...hmmm",
        ".....h..hmmm",
        "hh.hhhh.hmmm",
        "mm.mmmh.hmmm",
        ".....h..hmmm",
        "....h...mmmm",
        "............",
        "............",
        "............",
    ),
    (  # Hint: a folded sheet with two short lines of writing.
        "..hhhhhh....",
        "..hhhhhmh...",
        "..hhhhhmhh..",
        "..hhhhmmmm..",
        "..hhhhhhhm..",
        "..hlll.mhm..",
        "..hhhhhhhm..",
        "..hlmhhhhm..",
        "..hhhhhhhm..",
        "..hhhhhhmm..",
        "..mmmmmmmm..",
        "............",
    ),
)


# P8: the general nodes past the first row, at y = 64 (one 16-pixel cell each, in catalog
# order after capacity).
ICONS_P8 = (
    (  # Flare cost: a flame burning low, its tip clipped by a saving notch.
        "............",
        ".....c......",
        ".....cc.....",
        "....cpc..hhh",
        "....cpcc.mmm",
        "...cpbpc....",
        "...cpbbpc...",
        "..cpbbbbpc..",
        "..cpbbbbpc..",
        "..cqpbbpqc..",
        "...cqqqqc...",
        "....mmmm....",
    ),
    (  # Dosing: one full drop and a small plus beside it.
        ".....h......",
        ".....h......",
        "....hmh.....",
        "....hmh..b..",
        "...hcpmh.b..",
        "...hcppmbbbb",
        "..hcpppqm.b.",
        "..hcpppqm.b.",
        "..hcppqqm...",
        "..hmcqqmm...",
        "...hmmmm....",
        "....mmm.....",
    ),
    (  # Simultaneous burning: two flames side by side.
        "............",
        "..c......c..",
        "..cc.....cc.",
        ".cpc....cpc.",
        ".cpcc...cpcc",
        "cpbpc..cpbpc",
        "cpbbc..cpbbc",
        "cpbbc..cpbbc",
        ".cqqc...cqqc",
        "..mm.....mm.",
        "............",
        "............",
    ),
    (  # Transition: a flame and an arrow bending down: the flare eases off.
        "............",
        "...c...hhh..",
        "...cc....h..",
        "..cpc....h..",
        "..cpcc...h..",
        ".cpbpc...h..",
        ".cpbbc.hhhhh",
        ".cpbbc..hhh.",
        "..cqqc...h..",
        "...mm.......",
        "............",
        "............",
    ),
    (  # Far sight: an open eye with rays reaching out.
        "............",
        "h....hh....h",
        ".h.hhhhhh.h.",
        "..hm....mh..",
        ".hm..bb..mh.",
        "hm..bkkb..mh",
        "hm..bkkb..mh",
        ".hm..bb..mh.",
        "..hm....mh..",
        ".h.hhhhhh.h.",
        "h....hh....h",
        "............",
    ),
    (  # Clarity: an eye with a spark of light over it.
        "........b...",
        ".......bsb..",
        "........b...",
        "...hhhhh....",
        "..hm...mh...",
        ".hm.bb..mh..",
        "hm.bkkb..mh.",
        ".hm.bb..mh..",
        "..hm...mh...",
        "...hhhhh....",
        "............",
        "............",
    ),
    (  # Hearing: a source and three widening sound arcs.
        "............",
        "..........m.",
        ".......m...m",
        "....h...m..m",
        ".....h..m..m",
        "bb...h..m..m",
        "bb...h..m..m",
        ".....h..m..m",
        "....h...m..m",
        ".......m...m",
        "..........m.",
        "............",
    ),
    (  # Keen ears: the arcs with a plus.
        ".b..........",
        "bbb.......m.",
        ".b.....m...m",
        "....h...m..m",
        ".....h..m..m",
        "bb...h..m..m",
        "bb...h..m..m",
        ".....h..m..m",
        "....h...m..m",
        ".......m...m",
        "..........m.",
        "............",
    ),
)

# P8, the third row (y = 80): hearing's memory, pewter's nodes, Reflexes and the bridge.
ICONS_P8_ROW3 = (
    (  # Echo: an hourglass beside two sound arcs.
        "............",
        "............",
        "bbbb.....m..",
        "b..b..h...m.",
        ".bb....h..m.",
        ".ss....h..m.",
        ".bb....h..m.",
        "bssb...h..m.",
        "bbbb..h...m.",
        ".........m..",
        "............",
        "............",
    ),
    (  # Strength: a pewter gauntlet clenched over a brass cuff.
        "............",
        "...hhhhhh...",
        "..hmhmhmhh..",
        "..hmhmhmhm..",
        "..hmmmmmmm..",
        ".hhmmmmmmm..",
        ".hmhmmmmml..",
        ".hmmmmmmml..",
        "..hmmmmml...",
        "...mmmmll...",
        "...kkkkk....",
        "...sssss....",
    ),
    (  # Endurance: a pewter shield with a brass cross.
        "............",
        ".hhhhhhhhhh.",
        ".hmmmbbmmml.",
        ".hmmmbbmmml.",
        ".hmbbbbbbml.",
        ".hmbbbbbbml.",
        ".hmmmbbmmml.",
        "..hmmbbmml..",
        "..hmmbbmml..",
        "...hmmmml...",
        "....hmml....",
        ".....ll.....",
    ),
    (  # Pewter leap: an arrow rising off the ground.
        ".....hh.....",
        "....hhhh....",
        "...hhmmhh...",
        "..h.hmmh.h..",
        ".....mm.....",
        ".....mm.....",
        "....m..m....",
        "...m....m...",
        "..m......m..",
        "............",
        "..hhhhhhhh..",
        "...m....m...",
    ),
    (  # Spring: two chevrons up.
        "............",
        ".....hh.....",
        "....hmmh....",
        "...hm..mh...",
        "............",
        ".....hh.....",
        "....hmmh....",
        "...hm..mh...",
        "............",
        "............",
        "..mmmmmmmm..",
        "...m....m...",
    ),
    (  # Cushion: an arrow falling onto a soft brass pillow.
        ".....hm.....",
        ".....hm.....",
        ".....hm.....",
        ".....hm.....",
        "...hhhmmm...",
        "....hmmm....",
        ".....hm.....",
        "............",
        "..bbbbbbbb..",
        ".bssssssssb.",
        "..ssssssss..",
        "............",
    ),
    (  # Reflexes: a brass bolt with motion lines.
        "........bbb.",
        ".......bbs..",
        "hh....bbs...",
        ".....bbs....",
        "....bbbbbbb.",
        "hhh.....bbs.",
        ".......bbs..",
        "......bbs...",
        "hh...bbs....",
        "....bbs.....",
        "...bs.......",
        "............",
    ),
    (  # Steel body: a pewter figure over steel's upward arrow.
        ".....hh.....",
        ".....hh.....",
        "...hhmmhh...",
        "..hmmmmmmh..",
        "..hmmmmmmh..",
        "...hmmmmh...",
        "....m..m....",
        "....m..m....",
        "............",
        ".....bb.....",
        "....bbbb....",
        ".....ss.....",
    ),
)


def draw_atlas():
    pixels = canvas(128, 96)

    locked_rim = color("363B42")
    node(pixels, 0, STEEL_LOW, locked_rim, locked_rim)
    node(pixels, 22, STEEL_MID, STEEL_LOW, STEEL_DARK)
    node(pixels, 44, BRASS_HIGH, BRASS_MID, BRASS_LOW)
    node(pixels, 66, color("DDE7F3"), STEEL_HIGH, STEEL_MID)
    rect(pixels, 30, 1, 4, 1, BRASS_HIGH)
    rect(pixels, 30, 2, 1, 1, BRASS_LOW)
    # Small nicks break up the long metal edges.
    pixels[1][37] = STEEL_LOW
    pixels[1][57] = BRASS_MID
    pixels[15][45] = BRASS_MID

    icon_palette = {
        "h": STEEL_HIGH, "m": STEEL_MID, "l": STEEL_LOW,
        "b": BRASS_HIGH, "s": BRASS_MID, "k": BRASS_LOW,
        "c": COPPER_HIGH, "p": COPPER_MID, "q": COPPER_LOW,
    }
    for index, rows in enumerate(ICONS):
        art(pixels, 16 * index + 2, 26, rows, icon_palette)
    for index, rows in enumerate(ICONS_P8):
        art(pixels, 16 * index + 2, 66, rows, icon_palette)
    for index, rows in enumerate(ICONS_P8_ROW3):
        art(pixels, 16 * index + 2, 82, rows, icon_palette)

    rect(pixels, 112, 24, 4, 4, STEEL_LOW)
    rect(pixels, 113, 25, 2, 2, OUTLINE)
    rect(pixels, 116, 24, 4, 4, BRASS_LOW)
    rect(pixels, 117, 25, 2, 2, BRASS_MID)
    rect(pixels, 116, 24, 4, 1, BRASS_HIGH)

    tab(pixels, 0, selected=False)
    tab(pixels, 48, selected=True)

    art(pixels, 96, 40, (
        ".ooooooo.",
        "osssssssl",
        "osdddhdsl",
        "osddhhdsl",
        "oshdhddsl",
        "oshhhddsl",
        "osdhdddsl",
        "osssssssl",
        ".lllllll.",
    ), {"o": OUTLINE, "s": STEEL_LOW, "l": STEEL_DARK, "d": WELL, "h": BRASS_HIGH})
    art(pixels, 105, 40, (
        ".ooooooo.",
        "omssssssl",
        "os.....sl",
        "os.....sl",
        "os.....sl",
        "os.....sl",
        "os.....sl",
        "osssssssl",
        ".lllllll.",
    ), {"o": OUTLINE, "m": STEEL_MID, "s": STEEL_LOW, "l": STEEL_DARK})
    art(pixels, 114, 40, (
        "...o...",
        "..oho..",
        ".ohhmo.",
        "ohhmllo",
        ".omllo.",
        "..olo..",
        "...o...",
    ), {"o": OUTLINE, "h": BRASS_HIGH, "m": BRASS_MID, "l": BRASS_LOW})

    return pixels


def draw_notebook():
    pixels = canvas(16, 16)
    art(pixels, 2, 1, (
        "...oooooooo.",
        "..ohsbbbbbo.",
        ".ohsbbbbbao.",
        "ohsbbabbbao.",
        "ossbbbbbaao.",
        "osbbbbbagggo",
        "osbbabbbgkgo",
        "osbbbbbagglo",
        "osbbaaabaeo.",
        "osbbaaaaefo.",
        "osaaaaaefeo.",
        ".ooooopeeo..",
        "..opeeeoo...",
        "...oooo.....",
    ), {
        "o": OUTLINE, "h": STEEL_HIGH, "s": STEEL_MID,
        "b": color("674A3E"), "a": color("50372F"),
        "g": BRASS_MID, "k": BRASS_HIGH, "l": BRASS_LOW,
        "e": color("C6B79C"), "f": color("928872"), "p": color("E4D9C0"),
    })
    return pixels


def png(rows):
    raw = b"".join(b"\0" + b"".join(map(bytes, row)) for row in rows)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    header = struct.pack(">IIBBBBB", len(rows[0]), len(rows), 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


if __name__ == "__main__":
    textures = Path(__file__).resolve().parents[1] / "mod/assets/mistbrn/textures"
    for name, rows in (("gui/progress.png", draw_atlas()), ("item/notebook.png", draw_notebook())):
        (textures / name).write_bytes(png(rows))
        print(name)
