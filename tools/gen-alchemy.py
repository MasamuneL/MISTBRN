#!/usr/bin/env python3
"""Generate the P3c alchemy textures (items and blocks) at native 16x16 resolution.

Usage: python3 tools/gen-alchemy.py
"""
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parent.parent / "mod/assets/mistbrn/textures"


def color(rgb, alpha=255):
    return (*bytes.fromhex(rgb), alpha)


TRANSPARENT = (0, 0, 0, 0)

# Shared glass and cork from vial_empty. Underscores mark the liquid cavity.
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
    "...FGH____HJF...",
    "...FG______JF...",
    "...FI______JF...",
    "...FH______JF...",
    "....FHIIIHJF....",
    ".....FFFFFF.....",
    "................",
]
VIAL_PALETTE = {
    "A": color("3A2218"), "B": color("E7A760"), "C": color("C17C42"),
    "D": color("A05B30"), "E": color("753B20"), "F": color("242E3A"),
    "G": color("EBF9FF"), "H": color("97B1C7", 0xB0),
    "I": color("BBD5E7", 0xE7), "J": color("4F6074"),
}


def grid(rows, palette):
    assert len(rows) == 16 and all(len(row) == 16 for row in rows)
    colors = {".": TRANSPARENT, **palette}
    return [[colors[symbol] for symbol in row] for row in rows]


def vial(fill, palette):
    rows = VIAL.copy()
    for y, liquid in enumerate(fill, start=9):
        row = rows[y]
        start = row.index("_")
        assert len(liquid) == row.count("_")
        rows[y] = row[:start] + liquid + row[start + len(liquid):]
    return grid(rows, {**VIAL_PALETTE, **palette})


WATER = vial([
    "lsll",
    "lmmmmm",
    "lmmmmd",
    "mmdddd",
], {"s": color("CFEFFF"), "l": color("8ABDE4"),
    "m": color("578CBE"), "d": color("426992")})

BRINE = vial([
    "wwww",
    "wpwwpw",
    "wwswww",
    "spwsws",
], {"p": color("FCFEF7"), "w": color("D8E6E8"), "s": color("A8BFC5")})

# Bright tin and zinc above a two-pixel ochre copper band: unlike vial_iron's cold grey.
BASIC_METALS = vial([
    "twwt",
    "zzwzzs",
    "oOoooO",
    "ooOooi",
], {"w": color("FAFCFF"), "t": color("E0E5EB"), "z": color("A8B4BF"),
    "s": color("7D8894"), "i": color("343B46"), "o": color("B8862F"), "O": color("E0B458")})

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
    "..DDLSSDOOLSSDO.",
    "....DDDDDDDDDD..",
    "................",
    "................",
], {"W": color("FFFFFF"), "L": color("E6E4DC"), "S": color("C9C6BA"),
    "O": color("A59F92"), "D": color("777369")})

# Cubic faces embedded in a pale matrix, with short, stepped fractures.
# The upper-right cluster continues across both edges of the tile.
SALT_ROCK = grid([
    "lsmmllmmlllslhhl",
    "msmhhhlmmllsslll",
    "lmlhhhhhsmmllmll",
    "lmlhllllsmlhllml",
    "lmhhllllsmlhslml",
    "mmlhllllssmllmll",
    "mslhlllssmlllmms",
    "smmllssmmllhhlds",
    "mllmmmlmllhhhhsm",
    "llhhlmlmmhllllsm",
    "lhhhlsmllhllllsm",
    "hhllssmmlhllsslh",
    "lllsmllmsllssmll",
    "lslmllssdmmlmmll",
    "mllmmllmsmlslmhm",
    "lsmmllmmlllslhhl",
], {"h": color("F9F7F0"), "l": color("EAE6DB"), "m": color("DCD8CC"),
    "s": color("C7C4BD"), "d": color("AAA8A5")})

COPPER = {
    "d": color("573B31"), "s": color("794C38"),
    "m": color("A8694C"), "l": color("B77658"), "h": color("E1A17D"),
}

# A flared still with a cap and a glass condenser tube routed down its side.
ALEMBIC_SIDE = grid([
    "mmmmmmmmmmmmmmmm",
    "dddddshhldsddddd",
    "sssssmlhhllsssss",
    "ssssdhlmmdllssss",
    "sssdhllmmmsldsss",
    "ssdhhlllmmmsldss",
    "sdhhllllmddlddss",
    "sdhllllmdabggdss",
    "sdhlllmmdbgagdss",
    "sdhlllmsdbgagdss",
    "sdllllmsdbgagdss",
    "sdmllmmssdbgydss",
    "ssdmmmmmssddddss",
    "ssddddddddddssss",
    "llllllllllllllll",
    "mmmmmmmmmmmmmmmm",
], {**COPPER, "a": color("E2F1F7"), "b": color("BBD5E7"),
    "g": color("6F8A9F"), "y": color("354957")})

# Concentric hammered shoulders and the capped neck seen from above.
ALEMBIC_TOP = grid([
    "mmmmmmmmmmmmmmmm",
    "mmmmllllllllmmmm",
    "mmmllhhhhhhllmmm",
    "mmllhlllllmmllmm",
    "mllhllhhhmmssllm",
    "mlhllhmmsdmmsslm",
    "mlhlhmlhlsdmsslm",
    "mlhlhmlmlsdmsslm",
    "mlhlhmlmsddmsslm",
    "mlhlhmsdddsmsslm",
    "mllhlhsddsmssllm",
    "mmllhlmmmsssllmm",
    "mmmllhsssslllmmm",
    "mmmmllllllllmmmm",
    "mmmmmmmmmmmmmmmm",
    "mmmmmmmmmmmmmmmm",
], COPPER)


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
    write(ROOT / "item/vial_water.png", WATER)
    write(ROOT / "item/vial_brine.png", BRINE)
    write(ROOT / "item/vial_basic_metals.png", BASIC_METALS)
    write(ROOT / "item/salt.png", SALT)
    write(ROOT / "block/salt_rock.png", SALT_ROCK)
    write(ROOT / "block/alembic_side.png", ALEMBIC_SIDE)
    write(ROOT / "block/alembic_top.png", ALEMBIC_TOP)


if __name__ == "__main__":
    main()
