#!/usr/bin/env python3
"""Generate the reserve HUD's sprite sheet (mod/assets/mistbrn/textures/gui/hud.png, 128x64).

Original pixel art for this mod: a bevelled pewter frame with brass studs, framed bars with
gradient fills, metal gems, a lock and two flames. Standard library only (zlib + struct).
The layout constants must match mod/src/client.rs (`SPRITE_*`).

Usage: tools/gen-hud.py [output.png]
"""
import struct
import sys
import zlib

W, H = 128, 64
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


OUTLINE = hexc("0b0c10")
PEWTER_HI = hexc("9aa0ae")
PEWTER = hexc("5e6472")
PEWTER_LO = hexc("2c3039")
FILL = hexc("171a21", 0xE6)
BRASS_HI = hexc("f3dc95")
BRASS = hexc("c79a3e")
BRASS_LO = hexc("7a5a1e")
TRACK = hexc("0e1015")
TRACK_HI = hexc("1d2029")

# --- Panel, 16x16 nine-slice with 5 px corners (SPRITE_PANEL at 0,0) -------------------------
rect(0, 0, 16, 16, FILL)
rect(0, 0, 16, 1, OUTLINE); rect(0, 15, 16, 1, OUTLINE)
rect(0, 0, 1, 16, OUTLINE); rect(15, 0, 1, 16, OUTLINE)
rect(1, 1, 14, 1, PEWTER_HI); rect(1, 1, 1, 14, PEWTER_HI)  # bevel light (top, left)
rect(1, 14, 14, 1, PEWTER_LO); rect(14, 1, 1, 14, PEWTER_LO)  # bevel shadow (bottom, right)
rect(2, 2, 12, 1, PEWTER); rect(2, 2, 1, 12, PEWTER)
rect(2, 13, 12, 1, OUTLINE); rect(13, 2, 1, 12, OUTLINE)
for cx, cy in ((2, 2), (12, 2), (2, 12), (12, 12)):  # brass studs in the corners
    put(cx, cy, BRASS_HI); put(cx + 1, cy, BRASS)
    put(cx, cy + 1, BRASS); put(cx + 1, cy + 1, BRASS_LO)

# --- Bar frame, 64x8, inner track 60x4 at (2,2) (SPRITE_FRAME at 16,0) ---------------------
fx = 16
rect(fx + 1, 0, 62, 8, PEWTER)
rect(fx + 1, 0, 62, 1, OUTLINE); rect(fx + 1, 7, 62, 1, OUTLINE)
rect(fx, 1, 1, 6, OUTLINE); rect(fx + 63, 1, 1, 6, OUTLINE)
rect(fx + 1, 1, 62, 1, PEWTER_HI)
rect(fx + 1, 6, 62, 1, PEWTER_LO)
rect(fx + 2, 2, 60, 4, TRACK)
rect(fx + 2, 2, 60, 1, OUTLINE)  # inner shadow under the top rim
rect(fx + 2, 5, 60, 1, TRACK_HI)
for ex in (fx + 1, fx + 62):  # brass end caps
    put(ex, 2, BRASS_HI); put(ex, 3, BRASS); put(ex, 4, BRASS); put(ex, 5, BRASS_LO)

# --- Fills, 60x4 each: highlight, light, base, shadow (SPRITE_FILL_* at 16,8 / 16,12 / 16,16) -
FILLS = {
    8: ("fff0b8", "f0c860", "c8962e", "8a6418"),   # burning: brass gold
    12: ("ffd2a0", "ff9a40", "e0601c", "9a3410"),  # flared: hot orange
    16: ("c8ccd6", "9aa0ae", "767c8a", "4c515c"),  # off: pewter
}
for y, shades in FILLS.items():
    for row, shade in enumerate(shades):
        rect(fx, y + row, 60, 1, hexc(shade))
    for i in range(3, 60, 6):  # faint sheen ticks so a long bar is not flat
        put(fx + i, y + 1, hexc(shades[0]))

# --- Marker gem 5x6 (SPRITE_GEM at 80,0): sits on the fill's end --------------------------
art(80, 0, [
    "..o..",
    ".oHo.",
    "oHPPo",
    "oPPLo",
    ".oLo.",
    "..o..",
], {"o": OUTLINE, "H": hexc("eef2fa"), "P": PEWTER_HI, "L": PEWTER})

# --- Lock 7x8 (SPRITE_LOCK at 88,0) --------------------------------------------------------
art(88, 0, [
    ".ooooo.",
    ".oHPPo.",
    ".o...o.",
    "ooooooo",
    "oBBBBBo",
    "oBBoBBo",
    "oLLoLLo",
    "ooooooo",
], {"o": OUTLINE, "H": PEWTER_HI, "P": PEWTER, "B": BRASS, "L": BRASS_LO})

# --- Flames 7x8 (SPRITE_FLAME at 96,0 burning, 104,0 flared) -------------------------------
FLAME = [
    "...o...",
    "..oYo..",
    ".oYRYo.",
    ".oRWRo.",
    "oRYWYRo",
    "oRYWYRo",
    ".oRRRo.",
    "..ooo..",
]
art(96, 0, FLAME, {"o": OUTLINE, "Y": hexc("f0c860"), "R": hexc("c8962e"), "W": hexc("fff0b8")})
art(104, 0, FLAME, {"o": OUTLINE, "Y": hexc("ff9a40"), "R": hexc("d0401a"), "W": hexc("fff4d0")})

# --- Metal gems 6x6, Metal::ALL order (SPRITE_METAL at 0,24 + 8 * index) --------------------
METALS = [
    ("c4c8d0", "8a909c", "4e535e"),  # iron
    ("dce6f2", "8fa4bc", "4a5a70"),  # steel
    ("ffffff", "d4d6d8", "8c9096"),  # tin
    ("b8bcc4", "868a92", "50545c"),  # pewter
    ("e4eef0", "a8bcc0", "687c82"),  # zinc
    ("fff0a0", "e0b840", "8a6a18"),  # brass
    ("ffc8a0", "d87838", "8a4418"),  # copper
    ("f0c890", "b07c38", "6a4618"),  # bronze
]
GEM = [
    ".oooo.",
    "oHHLMo",
    "oHLMMo",
    "oLMMSo",
    "oMMSSo",
    ".oooo.",
]
for index, (hi, mid, lo) in enumerate(METALS):
    light = tuple((a + b) // 2 for a, b in zip(hexc(hi)[:3], hexc(mid)[:3])) + (255,)
    art(8 * index, 24, GEM, {"o": OUTLINE, "H": hexc(hi), "L": light, "M": hexc(mid), "S": hexc(lo)})


# --- Wheel slots, 24x24 (SPRITE_SLOT at 0,32 silver, 24,32 brass = current, 48,32 hovered) ----
def slot(x0, hi, mid, lo, glow=None):
    rect(x0 + 1, 33, 22, 22, hexc("10131a", 0xD8))  # window
    rect(x0 + 1, 32, 22, 1, OUTLINE); rect(x0 + 1, 55, 22, 1, OUTLINE)
    rect(x0, 33, 1, 22, OUTLINE); rect(x0 + 23, 33, 1, 22, OUTLINE)
    rect(x0 + 1, 33, 22, 1, hi); rect(x0 + 1, 33, 1, 22, hi)
    rect(x0 + 1, 54, 22, 1, lo); rect(x0 + 22, 33, 1, 22, lo)
    rect(x0 + 2, 34, 20, 1, mid); rect(x0 + 2, 34, 1, 20, mid)
    rect(x0 + 2, 53, 20, 1, mid); rect(x0 + 21, 34, 1, 20, mid)
    rect(x0 + 3, 35, 18, 1, OUTLINE); rect(x0 + 3, 35, 1, 18, OUTLINE)
    rect(x0 + 3, 52, 18, 1, OUTLINE); rect(x0 + 20, 35, 1, 18, OUTLINE)
    for cx, cy in ((x0 + 1, 33), (x0 + 21, 33), (x0 + 1, 53), (x0 + 21, 53)):  # corner knobs
        put(cx, cy, hi); put(cx + 1, cy, mid); put(cx, cy + 1, mid); put(cx + 1, cy + 1, lo)
    if glow:
        rect(x0 + 4, 36, 16, 1, glow); rect(x0 + 4, 36, 1, 16, glow)
        rect(x0 + 4, 51, 16, 1, glow); rect(x0 + 19, 36, 1, 16, glow)


slot(0, hexc("e8ecf4"), hexc("a8aebc"), hexc("5a606c"))
slot(24, BRASS_HI, BRASS, BRASS_LO)
slot(48, hexc("ffffff"), hexc("d8e4f4"), hexc("8a96a8"), glow=hexc("cfe0ff", 0x90))

# --- Centre ring 24x24 (SPRITE_RING at 72,32): an octagon of pewter ---------------------------
RING = [
    "........oooooooo........",
    "......ooHHHHHHHHoo......",
    ".....oHPoooooooooPHo....",
    "....oHPo........oPHo....",
    "...oHPo..........oPHo...",
    "..oHPo............oPHo..",
    ".oHPo..............oPHo.",
    "oHPo................oPHo",
    "oHPo................oPLo",
    "oHPo................oPLo",
    "oHPo................oPLo",
    "oHPo................oPLo",
    "oHPo................oPLo",
    "oHPo................oPLo",
    "oHPo................oPLo",
    "oHPo................oPLo",
    ".oHPo..............oPLo.",
    "..oHPo............oPLo..",
    "...oHPo..........oPLo...",
    "....oHPo........oPLo....",
    ".....oPPoooooooooPLo....",
    "......ooLLLLLLLLoo......",
    "........oooooooo........",
    "........................",
]
art(72, 32, RING, {"o": OUTLINE, "H": PEWTER_HI, "P": PEWTER, "L": PEWTER_LO})

# --- Big metal gems 8x8 for the wheel (SPRITE_BIG_METAL at 0,56 + 8 * index) -----------------
BIG = [
    "..oooo..",
    ".oHHLMo.",
    "oHHLMMMo",
    "oHLMMMSo",
    "oLMMMSSo",
    "oMMMSSSo",
    ".oMSSSo.",
    "..oooo..",
]
for index, (hi, mid, lo) in enumerate(METALS):
    light = tuple((a + b) // 2 for a, b in zip(hexc(hi)[:3], hexc(mid)[:3])) + (255,)
    art(8 * index, 56, BIG, {"o": OUTLINE, "H": hexc(hi), "L": light, "M": hexc(mid), "S": hexc(lo)})


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
