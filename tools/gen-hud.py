#!/usr/bin/env python3
"""Generate the HUD's sprite sheet (mod/assets/mistbrn/textures/gui/hud.png, 160x64).

Original pixel art for this mod, a worn-metal look kept at the game's own pixel size: bevelled
steel and brass frames (1 px light top-left, shadow bottom-right, reinforced corners, a few
worn pixels), bars with a dark inner channel, end rivets and faint divisions, three-tone
flames, bevelled metal tiles, a lock and an aged steel ring with four brass notches.
Muted golds and oranges so the HUD does not compete with items. Standard library only.
Every region's position must match the constants in mod/src/hud.rs.

Usage: tools/gen-hud.py [output.png]
"""
import struct
import sys
import zlib

W, H = 160, 64
px = [[(0, 0, 0, 0)] * W for _ in range(H)]


def hexc(value, alpha=255):
    value = value.lstrip("#")
    return (int(value[0:2], 16), int(value[2:4], 16), int(value[4:6], 16), alpha)


def put(x, y, color):
    if 0 <= x < W and 0 <= y < H:
        px[y][x] = color


def rect(x, y, w, h, color):
    for j in range(y, y + h):
        for i in range(x, x + w):
            put(i, j, color)


def art(x0, y0, rows, palette):
    for j, row in enumerate(rows):
        for i, ch in enumerate(row):
            if ch != ".":
                put(x0 + i, y0 + j, palette[ch])


# Palette: background, shadow, steel, highlight, brass, copper (muted).
OUTLINE = hexc("0a0b0e")
SHADOW = hexc("1b1d23")
STEEL_LO = hexc("3a3e47")
STEEL = hexc("5c626d")
STEEL_MID = hexc("7d838e")
STEEL_HI = hexc("b4b9c2")
BRASS_LO = hexc("6b5428")
BRASS = hexc("a88a4e")
BRASS_HI = hexc("d9c38c")
COPPER_LO = hexc("6a3a22")
COPPER = hexc("a8643a")
COPPER_HI = hexc("d99a6a")
WINDOW = hexc("101217", 0xDC)
TRACK = hexc("0c0d11")
TRACK_HI = hexc("191b21")

# --- Bar frames, 64x8, inner track 60x4 at (2,2): FRAME at 16,0 (cold), FRAME_LIT at 96,8 ----
def bar_frame(fx, fy, cap_hi, cap, cap_lo):
    rect(fx + 1, fy, 62, 8, STEEL)
    rect(fx + 1, fy, 62, 1, OUTLINE); rect(fx + 1, fy + 7, 62, 1, OUTLINE)
    rect(fx, fy + 1, 1, 6, OUTLINE); rect(fx + 63, fy + 1, 1, 6, OUTLINE)
    rect(fx + 1, fy + 1, 62, 1, STEEL_HI)          # light on the top edge
    rect(fx + 1, fy + 6, 62, 1, STEEL_LO)          # shadow on the bottom edge
    rect(fx + 2, fy + 2, 60, 4, TRACK)             # dark inner channel
    rect(fx + 2, fy + 2, 60, 1, OUTLINE)           # channel's own top shadow
    rect(fx + 2, fy + 5, 60, 1, TRACK_HI)
    for ex in (fx + 1, fx + 62):                   # end plates with a rivet each
        put(ex, fy + 2, cap_hi); put(ex, fy + 3, cap); put(ex, fy + 4, cap); put(ex, fy + 5, cap_lo)
    put(fx + 9, fy + 1, STEEL_MID); put(fx + 41, fy + 6, STEEL)   # worn pixels
    put(fx + 54, fy + 1, STEEL_MID)


bar_frame(16, 0, BRASS_HI, BRASS, BRASS_LO)
bar_frame(96, 8, COPPER_HI, hexc("d0803c"), COPPER_LO)   # burning: warm accents

# --- Fills, 60x4: highlight, light, base, shadow; a faint division every 10 px --------------
FILLS = {
    8: ("f0dfa8", "d6b467", "aa8640", "6e5426"),   # burning: muted brass gold
    12: ("f2c49a", "d98a4e", "b05a2c", "6e3418"),  # flared: muted hot copper
    16: ("c2c6ce", "979ca6", "737883", "4a4e57"),  # off: steel
}
for y, shades in FILLS.items():
    for row, shade in enumerate(shades):
        rect(16, y + row, 60, 1, hexc(shade))
    for i in range(9, 60, 10):  # divisions, darker than the base, not across the highlight
        put(16 + i, y + 1, hexc(shades[2])); put(16 + i, y + 2, hexc(shades[3]))

# --- Marker on the fill's end, 5x6 (GEM at 80,0) ------------------------------------------
art(80, 0, [
    "..o..",
    ".oHo.",
    "oHSMo",
    "oSMLo",
    ".oLo.",
    "..o..",
], {"o": OUTLINE, "H": hexc("eef0f4"), "S": STEEL_HI, "M": STEEL_MID, "L": STEEL})

# --- Lock 7x8 (LOCK at 88,0) ---------------------------------------------------------------
art(88, 0, [
    ".ooooo.",
    ".oHSSo.",
    ".o...o.",
    "ooooooo",
    "oBBBBBo",
    "oBBoBLo",
    "oLLoLLo",
    "ooooooo",
], {"o": OUTLINE, "H": STEEL_HI, "S": STEEL, "B": BRASS, "L": BRASS_LO})

# --- Flames 7x8, three tones with a light core (FLAME at 96,0; FLAME_FLARED at 104,0) ------
FLAME = [
    "...o...",
    "..oEo..",
    ".oEMEo.",
    ".oMCMo.",
    "oEMCCMo",
    "oEMCCEo",
    ".oEMEo.",
    "..ooo..",
]
art(96, 0, FLAME, {"o": OUTLINE, "E": hexc("8a5a22"), "M": hexc("c99a4a"), "C": hexc("f2e0a8")})
art(104, 0, FLAME, {"o": OUTLINE, "E": hexc("7a2e16"), "M": hexc("cf6a32"), "C": hexc("f6d6a0")})

# --- Metal tiles, bevelled squares in Metal::ALL order: 6x6 at y 24, 8x8 at y 56 --------------
METALS = [
    ("c9ccd2", "8e939c", "565b64"),  # iron
    ("b9c8da", "7f93ab", "46566b"),  # steel
    ("eceef0", "c4c7cb", "8a8e94"),  # tin
    ("b3b5ba", "85888f", "54575e"),  # pewter
    ("d3dee0", "9fb1b5", "64767b"),  # zinc
    ("e3cf8f", "b8994e", "75602c"),  # brass
    ("e2aa82", "b46a40", "70402a"),  # copper
    ("d9b88a", "a07a46", "62482a"),  # bronze
]


def tile(x0, y0, size, hi, mid, lo):
    rect(x0, y0, size, size, OUTLINE)
    rect(x0 + 1, y0 + 1, size - 2, size - 2, mid)
    rect(x0 + 1, y0 + 1, size - 2, 1, hi); rect(x0 + 1, y0 + 1, 1, size - 2, hi)
    rect(x0 + 1, y0 + size - 2, size - 2, 1, lo); rect(x0 + size - 2, y0 + 1, 1, size - 2, lo)
    if size >= 8:  # a diagonal glint on the larger tiles
        put(x0 + 3, y0 + 4, hi); put(x0 + 4, y0 + 3, hi)


for index, (hi, mid, lo) in enumerate(METALS):
    tile(8 * index, 24, 6, hexc(hi), hexc(mid), hexc(lo))
    tile(8 * index, 56, 8, hexc(hi), hexc(mid), hexc(lo))

# --- Wheel slots 24x24 at y 32: SLOT (steel, dim) x 0, SLOT_CURRENT (brass) x 24, ----------
# --- SLOT_HOVERED (bright steel) x 48 ------------------------------------------------------
MARK = ["..o..", ".oBo.", "oB.Bo", ".oBo.", "..o.."]   # the mod's own engraved mark (a rhombus)


def slot(x0, hi, mid, lo, corner, corner_lo, mark=None, glow=None):
    y0 = 32
    rect(x0 + 1, y0 + 1, 22, 22, WINDOW)
    rect(x0 + 1, y0, 22, 1, OUTLINE); rect(x0 + 1, y0 + 23, 22, 1, OUTLINE)
    rect(x0, y0 + 1, 1, 22, OUTLINE); rect(x0 + 23, y0 + 1, 1, 22, OUTLINE)
    # Bevelled rim: light top-left, shadow bottom-right, body between.
    rect(x0 + 1, y0 + 1, 22, 1, hi); rect(x0 + 1, y0 + 1, 1, 22, hi)
    rect(x0 + 2, y0 + 2, 20, 1, mid); rect(x0 + 2, y0 + 2, 1, 20, mid)
    rect(x0 + 2, y0 + 21, 20, 1, lo); rect(x0 + 21, y0 + 2, 1, 20, lo)
    rect(x0 + 1, y0 + 22, 22, 1, SHADOW); rect(x0 + 22, y0 + 1, 1, 22, SHADOW)
    rect(x0 + 3, y0 + 3, 18, 1, OUTLINE); rect(x0 + 3, y0 + 3, 1, 18, OUTLINE)  # inner groove
    rect(x0 + 3, y0 + 20, 18, 1, OUTLINE); rect(x0 + 20, y0 + 3, 1, 18, OUTLINE)
    # Reinforced corners: a 5x5 plate on each corner, bevelled, with a rivet.
    for px0, py0 in ((x0, y0), (x0 + 19, y0), (x0, y0 + 19), (x0 + 19, y0 + 19)):
        rect(px0, py0, 5, 5, OUTLINE)
        rect(px0 + 1, py0 + 1, 3, 3, corner)
        put(px0 + 1, py0 + 1, hi); put(px0 + 3, py0 + 3, corner_lo)
        put(px0 + 2, py0 + 2, OUTLINE)
    put(x0 + 8, y0 + 1, mid); put(x0 + 22, y0 + 15, lo)  # worn pixels
    if glow:
        rect(x0 + 4, y0 + 4, 16, 1, glow); rect(x0 + 4, y0 + 4, 1, 16, glow)
        rect(x0 + 4, y0 + 19, 16, 1, glow); rect(x0 + 19, y0 + 4, 1, 16, glow)
    if mark:
        art(x0 + 10, y0 + 19, MARK, {"o": OUTLINE, "B": mark})


slot(0, hexc("8d929b"), STEEL, STEEL_LO, STEEL_MID, STEEL_LO)                     # dim
slot(24, BRASS_HI, BRASS, BRASS_LO, BRASS_HI, BRASS_LO, mark=BRASS_HI)              # current
slot(48, hexc("e6e9ee"), STEEL_HI, STEEL, hexc("f4f5f7"), STEEL_MID,
     mark=STEEL_HI, glow=hexc("dfe6f0", 0x70))                                      # hovered

# --- Centre ring 24x24 (RING at 72,32): aged steel octagon, four brass notches ---------------
RING = [
    "........oooooooo........",
    "......ooHHHSSHHHoo......",
    ".....oHSooobboooSHo.....",
    "....oHSo...bb...oSLo....",
    "...oHSo..........oSLo...",
    "..oHSo............oSLo..",
    ".oHSo..............oSLo.",
    "oHSo................oSLo",
    "oHSo................oSLo",
    "oHSo................oSLo",
    "oHo..................oLo",
    "obbb................bbbo",
    "obbb................bbbo",
    "oHo..................oLo",
    "oHSo................oSLo",
    "oSSo................oSLo",
    ".oSSo..............oSLo.",
    "..oSSo............oSLo..",
    "...oSSo..........oSLo...",
    "....oSSo...bb...oSLo....",
    ".....oSSooobboooSLo.....",
    "......ooLLLSSLLLoo......",
    "........oooooooo........",
    "........................",
]
art(72, 32, RING, {"o": OUTLINE, "H": STEEL_HI, "S": STEEL, "L": STEEL_LO, "b": BRASS})
put(72 + 5, 32 + 6, STEEL_MID); put(72 + 18, 32 + 16, STEEL_MID)  # uneven relief


def png(rows):
    raw = b"".join(b"\0" + b"".join(bytes(c) for c in row) for row in rows)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


out = sys.argv[1] if len(sys.argv) > 1 else "mod/assets/mistbrn/textures/gui/hud.png"
with open(out, "wb") as f:
    f.write(png(px))
print(out)
