#!/usr/bin/env python3
"""Generate the original 160×64 HUD atlas at native GUI resolution.

Usage: tools/gen-hud.py [output.png]
"""
import struct
import sys
import zlib

WIDTH, HEIGHT = 160, 64
TRANSPARENT = (0, 0, 0, 0)
pixels = [[TRANSPARENT] * WIDTH for _ in range(HEIGHT)]


def color(rgb, alpha=255):
    return (*bytes.fromhex(rgb), alpha)


def rect(x, y, width, height, shade):
    for row in range(y, y + height):
        pixels[row][x:x + width] = [shade] * width


def art(x, y, rows, palette):
    for dy, row in enumerate(rows):
        for dx, symbol in enumerate(row):
            if symbol != ".":
                pixels[y + dy][x + dx] = palette[symbol]


OUTLINE = color("0A0B0E")
STEEL_DARK = color("25282B")
STEEL_LOW = color("454A50")
STEEL_MID = color("747C86")
STEEL_HIGH = color("D8DDE4")
BRASS_LOW, BRASS_MID, BRASS_HIGH = map(color, ("756039", "B59654", "F0DC9C"))
COPPER_LOW, COPPER_MID, COPPER_HIGH = map(color, ("794C38", "B77658", "E1A17D"))
WELL = color("16191F", 0xF0)
TRACK = color("101217")
TRACK_FLOOR = color("242830")


def bar_frame(x, y, plate_high, plate_mid, plate_low):
    """Draw a clipped 64×8 frame around a 60×4 track at (2, 2)."""
    rect(x + 1, y, 62, 8, OUTLINE)
    rect(x, y + 1, 64, 6, OUTLINE)
    rect(x + 1, y + 1, 62, 1, STEEL_HIGH)
    rect(x + 1, y + 6, 62, 1, STEEL_DARK)
    rect(x + 2, y + 2, 60, 3, TRACK)
    rect(x + 2, y + 5, 60, 1, TRACK_FLOOR)
    for end in (x + 1, x + 62):
        art(end, y + 2, ("h", "m", "m", "l"), {
            "h": plate_high, "m": plate_mid, "l": plate_low,
        })
    for nick in (9, 54):
        pixels[y + 1][x + nick] = STEEL_MID


def fill(y, ramp, seam_rows):
    """Draw a 60×4 reserve strip that can be cropped to any width from the left."""
    for row, shade in enumerate(ramp):
        rect(16, y + row, 60, 1, shade)
    for x in range(16 + 9, 16 + 60, 10):
        for row in seam_rows:
            pixels[y + row][x] = ramp[row + 1]


METALS = (
    ("D0D2D4", "929497", "55585C"),  # Iron
    ("B9CDDD", "778C9F", "435466"),  # Steel
    ("F0F1F0", "C2C6C8", "82898D"),  # Tin
    ("BBB5AF", "827A76", "514B49"),  # Pewter
    ("CBDDD8", "8DA8A0", "4E6761"),  # Zinc
    ("F0DC9C", "B59654", "756039"),  # Brass
    ("E1A17D", "B77658", "794C38"),  # Copper
    ("D8B488", "A27646", "5F4128"),  # Bronze
)


def tile(x, y, size, high, mid, low):
    rect(x, y, size, size, OUTLINE)
    rect(x + 1, y + 1, size - 2, size - 2, mid)
    rect(x + 1, y + 1, size - 2, 1, high)
    rect(x + 1, y + 1, 1, size - 2, high)
    rect(x + 1, y + size - 2, size - 2, 1, low)
    rect(x + size - 2, y + 1, 1, size - 2, low)
    if size == 8:
        pixels[y + 4][x + 3] = high
        pixels[y + 3][x + 4] = high


def slot(x, rim, bevel, shadow, plate, *, worn=False):
    """Draw a 24×24 wheel slot with a two-pixel rim and 5×5 corner plates."""
    y = 32
    rect(x, y, 24, 24, OUTLINE)
    rect(x + 1, y + 1, 22, 22, shadow)
    rect(x + 1, y + 1, 22, 1, rim)
    rect(x + 1, y + 1, 1, 22, rim)
    rect(x + 2, y + 2, 19, 1, bevel)
    rect(x + 2, y + 2, 1, 19, bevel)
    rect(x + 3, y + 3, 18, 18, OUTLINE)
    rect(x + 4, y + 4, 16, 16, WELL)

    for dx, dy in ((0, 0), (19, 0), (0, 19), (19, 19)):
        cx, cy = x + dx, y + dy
        rect(cx, cy, 5, 5, OUTLINE)
        rect(cx + 1, cy + 1, 3, 3, plate)
        rect(cx + 1, cy + 1, 3, 1, rim)
        rect(cx + 1, cy + 1, 1, 3, rim)
        pixels[cy + 3][cx + 3] = shadow
        pixels[cy + 2][cx + 2] = OUTLINE

    if worn:
        pixels[y + 1][x + 8] = bevel
        pixels[y + 21][x + 16] = bevel


def ring(x, y):
    """Draw a centered 24×24 octagon with an open center."""
    bright = (OUTLINE, STEEL_HIGH, STEEL_MID, OUTLINE)
    neutral = (OUTLINE, STEEL_MID, STEEL_LOW, OUTLINE)
    dark = (OUTLINE, STEEL_LOW, STEEL_DARK, OUTLINE)
    for dy in range(24):
        for dx in range(24):
            edge_x = min(dx, 23 - dx)
            edge_y = min(dy, 23 - dy)
            # Inset the straight and diagonal edges together.
            diagonal_depth = edge_x + edge_y - 8
            depth = min(edge_x, edge_y, diagonal_depth)
            if not 0 <= depth < len(bright):
                continue
            if diagonal_depth == depth and (dx < 12) != (dy < 12):
                ramp = neutral
            else:
                ramp = bright if dx + dy <= 23 else dark
            pixels[y + dy][x + dx] = ramp[depth]

    for dx, dy in ((11, 1), (21, 11), (11, 21), (1, 11)):
        art(x + dx, y + dy, ("hm", "ml"), {
            "h": BRASS_HIGH, "m": BRASS_MID, "l": BRASS_LOW,
        })


def draw_atlas():
    # Bar frames, fills and endpoint marker. The coordinates match mod/src/hud.rs.
    bar_frame(16, 0, BRASS_HIGH, BRASS_MID, BRASS_LOW)
    bar_frame(96, 8, COPPER_HIGH, COPPER_MID, COPPER_LOW)
    # Burning reads as warm yellow, flared as orange: both clearly apart from the steel "off".
    fill(8, tuple(map(color, ("FFF0A8", "F2C744", "C9961E", "7A5814"))), (1, 2))
    fill(12, tuple(map(color, ("FFD49A", "F28A34", "C2561A", "6E2E14"))), (0, 1, 2))
    fill(16, (color("A8AEB7"), color("848B95"), color("656B74"), STEEL_LOW), (2,))

    art(80, 0, (
        "..o..",
        ".oho.",
        "ohsmo",
        "osmlo",
        ".olo.",
        "..o..",
    ), {"o": OUTLINE, "h": color("EEF0F4"), "s": STEEL_HIGH, "m": STEEL_MID, "l": STEEL_LOW})

    # Lock and flames. The flame silhouettes remain distinct in grayscale.
    art(88, 0, (
        ".ooooo.",
        ".oshso.",
        ".o...o.",
        "ooooooo",
        "obbbbbo",
        "ommomlo",
        "ollollo",
        "ooooooo",
    ), {"o": OUTLINE, "h": STEEL_HIGH, "s": STEEL_MID,
        "b": BRASS_HIGH, "m": BRASS_MID, "l": BRASS_LOW})

    art(96, 0, (
        "...o...",
        "..omo..",
        ".oemmo.",
        "oemmmeo",
        "oemcmeo",
        "oemcmeo",
        "oeemeeo",
        ".ooooo.",
    ), {"o": OUTLINE, "e": color("9A6A1C"), "m": color("F2C744"), "c": color("FFF0A8")})
    art(104, 0, (
        "..o....",
        ".omo.o.",
        ".oemo.o",
        "oemmmeo",
        "omccmmo",
        "omccmeo",
        ".oemmo.",
        "..ooo..",
    ), {"o": OUTLINE, "e": color("9A3A18"), "m": color("F28A34"), "c": color("FFE2B0")})

    # Small reserve tiles and big wheel tiles in Metal::ALL order.
    for index, palette in enumerate(METALS):
        shades = tuple(map(color, palette))
        tile(8 * index, 24, 6, *shades)
        tile(8 * index, 56, 8, *shades)

    slot(0, STEEL_MID, STEEL_LOW, STEEL_DARK, STEEL_LOW, worn=True)
    slot(24, BRASS_HIGH, BRASS_MID, BRASS_LOW, BRASS_MID, worn=True)
    rect(24 + 5, 32 + 4, 14, 1, color("C8A560"))
    pixels[32 + 21][24 + 12] = OUTLINE
    art(24 + 10, 32 + 20, ("..h..", "hh.hh", "..l.."), {
        "h": BRASS_HIGH, "l": BRASS_MID,
    })

    slot(48, STEEL_HIGH, STEEL_MID, STEEL_LOW, STEEL_MID)
    hover_glow = color("DDE7F3")
    # Step around the corner plates and keep the glow outside the tile and lock footprints.
    for side in (2, 21):
        rect(48 + side, 32 + 5, 1, 14, hover_glow)
    for edge, step in ((3, 5), (22, 18)):
        rect(48 + 5, 32 + edge, 14, 1, hover_glow)
        for corner in (5, 18):
            rect(48 + corner, 32 + min(edge, step), 1, abs(edge - step) + 1, hover_glow)
        rect(48 + 2, 32 + step, 3, 1, hover_glow)
        rect(48 + 19, 32 + step, 3, 1, hover_glow)

    ring(72, 32)


def png(rows):
    raw = b"".join(b"\0" + b"".join(map(bytes, row)) for row in rows)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    header = struct.pack(">IIBBBBB", WIDTH, HEIGHT, 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


if __name__ == "__main__":
    draw_atlas()
    output = sys.argv[1] if len(sys.argv) > 1 else "mod/assets/mistbrn/textures/gui/hud.png"
    with open(output, "wb") as atlas:
        atlas.write(png(pixels))
    print(output)
