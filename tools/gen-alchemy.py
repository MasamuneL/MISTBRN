#!/usr/bin/env python3
"""Generate the P3c alchemy textures (items and blocks) at native 16x16 resolution.

Provisional art: reviewed in Claude/Codex rounds before it is kept (asset rule).
Usage: python3 tools/gen-alchemy.py
"""
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parent.parent / "mod/assets/mistbrn/textures"


def color(rgb, alpha=255):
    return (*bytes.fromhex(rgb), alpha)


TRANSPARENT = (0, 0, 0, 0)

# The vial outline of vial_empty.png; "~" marks the liquid, "*" a highlight in it.
VIAL = [
    "................",
    "......AAAA......",
    ".....ABBBCA.....",
    ".....ADCCEA.....",
    "....FGEEEEHF....",
    "....FGIIIHJF....",
    ".....FJHHJF.....",
    ".....FJIHJF.....",
    "....FGH..HJF....",
    "...FGH~*~~HJF...",
    "...FG~~~~~~JF...",
    "...FI~~~~~~JF...",
    "...FH~~~~~~JF...",
    "....FHIIIHJF....",
    ".....FFFFFF.....",
    "................",
]
VIAL_PALETTE = {
    "A": color("3A2218"), "B": color("E7A760"), "C": color("C17C42"), "D": color("A05B30"),
    "E": color("753B20"), "F": color("242E3A"), "G": color("EBF9FF"), "H": color("97B1C7", 0xB0),
    "I": color("BBD5E7", 0xE7), "J": color("4F6074"),
}


def vial(liquid):
    """`liquid(x, y, highlight)` gives the colour of each liquid pixel."""
    pixels = []
    for y, row in enumerate(VIAL):
        line = []
        for x, symbol in enumerate(row):
            if symbol == ".":
                line.append(TRANSPARENT)
            elif symbol in "~*":
                line.append(liquid(x, y, symbol == "*"))
            else:
                line.append(VIAL_PALETTE[symbol])
        pixels.append(line)
    return pixels


def water(x, y, highlight):
    if highlight:
        return color("CFEFFF")
    return color("3F7FD0") if y >= 11 else color("5C9BE6")


def brine(x, y, highlight):
    if highlight or (x + 2 * y) % 5 == 0:
        return color("F4F6F2")
    return color("A9C4CF") if y >= 11 else color("C3D8DE")


# Iron, tin, zinc and copper bands: the four base metals in one vial.
BANDS = [color("C8C8C4"), color("D9DCDF"), color("9AA5AE"), color("B77658")]


def basic(x, y, highlight):
    if highlight:
        return color("F3F8FA")
    return BANDS[(y - 9) % 4]


def grid(rows, palette):
    return [[palette.get(ch, TRANSPARENT) for ch in row] for row in rows]


SALT = grid([
    "................",
    "................",
    "................",
    "................",
    "........W.......",
    ".......WLW......",
    "......WLSLW.....",
    "....W.LSSSL.W...",
    "...WLWLSSSLWLW..",
    "..WLSLWLSLWLSLW.",
    ".WLSSSLWLWLSSSLW",
    ".OLSSSSLOLSSSSLO",
    "..OOLSSLOOLSSLO.",
    "....OOOO..OOOO..",
    "................",
    "................",
], {"W": color("FFFFFF"), "L": color("E6E4DC"), "S": color("C9C6BA"), "O": color("8E8A7E")})


def block(fn):
    return [[fn(x, y) for x in range(16)] for y in range(16)]


def salt_rock(x, y):
    # Pale halite with grey veins and bright crystal faces.
    h = (x * 7 + y * 13 + (x * y) % 5) % 11
    if (x + y) % 7 == 0 or (x - y) % 9 == 0:
        return color("A7A398")
    if h in (0, 1):
        return color("FBFAF5")
    if h in (2, 3, 4):
        return color("E3DFD3")
    return color("CFCABC")


def alembic_side(x, y):
    # Copper bands at top and bottom, a glass window in the middle.
    if y in (0, 15) or x in (0, 15):
        return color("4A2E22")
    if y <= 2 or y >= 13:
        return color("E1A17D") if y in (1, 13) else color("B77658")
    if 3 <= x <= 12 and 4 <= y <= 11:
        if x in (3, 12) or y in (4, 11):
            return color("242E3A")
        return color("BBD5E7") if (x + y) % 6 else color("EBF9FF")
    return color("794C38")


def alembic_top(x, y):
    if x in (0, 15) or y in (0, 15):
        return color("4A2E22")
    r = abs(x - 7.5) + abs(y - 7.5)
    if r < 3:
        return color("242E3A")
    if r < 4:
        return color("E1A17D")
    return color("B77658") if (x + y) % 2 else color("A8694C")


def write(path, pixels):
    height, width = len(pixels), len(pixels[0])
    raw = b"".join(b"\0" + bytes(c for px in row for c in px) for row in pixels)

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
    png += chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def main():
    write(ROOT / "item/vial_water.png", vial(water))
    write(ROOT / "item/vial_brine.png", vial(brine))
    write(ROOT / "item/vial_basic_metals.png", vial(basic))
    write(ROOT / "item/salt.png", SALT)
    write(ROOT / "block/salt_rock.png", block(salt_rock))
    write(ROOT / "block/alembic_side.png", block(alembic_side))
    write(ROOT / "block/alembic_top.png", block(alembic_top))


if __name__ == "__main__":
    main()
