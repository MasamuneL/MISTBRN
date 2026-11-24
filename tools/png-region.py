#!/usr/bin/env python3
"""png-region.py PNG X Y W H: the SHA-256 of a region's RGB pixels (8-bit RGB or RGBA PNG).
png-region.py --self-test: decodes a small PNG built here with every row filter.

Used by the P7b scenarios to tell whether a part of two captures is identical (L4: with
«reduce flashing» the reserve bars do not pulse). Standard library only.
"""
import hashlib
import struct
import sys
import zlib


def decode(data):
    """(width, height, bpp, rows) of an 8-bit RGB/RGBA, non-interlaced PNG."""
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    pos, idat, header = 8, b"", None
    while pos < len(data):
        length, kind = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + length]
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", chunk)
        elif kind == b"IDAT":
            idat += chunk
        pos += 12 + length
    width, height, depth, color, _, _, interlace = header
    if depth != 8 or color not in (2, 6) or interlace:
        raise ValueError("only 8-bit RGB/RGBA, non-interlaced")
    bpp = 3 if color == 2 else 4
    raw, stride = zlib.decompress(idat), width * bpp
    rows, prev, i = [], bytearray(stride), 0
    for _ in range(height):
        kind, line = raw[i], bytearray(raw[i + 1:i + 1 + stride])
        i += 1 + stride
        for x in range(stride):
            a = line[x - bpp] if x >= bpp else 0
            b = prev[x]
            c = prev[x - bpp] if x >= bpp else 0
            if kind == 1:
                line[x] = (line[x] + a) & 255
            elif kind == 2:
                line[x] = (line[x] + b) & 255
            elif kind == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif kind == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pred = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[x] = (line[x] + pred) & 255
        rows.append(line)
        prev = line
    return width, height, bpp, rows


def region_hash(rows, bpp, x, y, w, h):
    digest = hashlib.sha256()
    for row in rows[y:y + h]:
        digest.update(bytes(row[x * bpp:(x + w) * bpp][i] for i in range(w * bpp) if i % bpp < 3))
    return digest.hexdigest()


def self_test():
    # A 2x5 RGB image, one row per filter, encoded with filter 0 except the Sub row, which is
    # stored as differences: decoding must give the same pixels back for every row.
    pixels = [bytes([10 * r + c for c in range(6)]) for r in range(5)]
    raw = b""
    for r, row in enumerate(pixels):
        if r == 1:
            sub = bytes((row[i] - (row[i - 3] if i >= 3 else 0)) & 255 for i in range(6))
            raw += b"\x01" + sub
        else:
            raw += b"\x00" + row
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 2, 5, 8, 2, 0, 0, 0))
           + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))
    width, height, bpp, rows = decode(png)
    assert (width, height, bpp) == (2, 5, 3)
    assert [bytes(r) for r in rows] == pixels
    assert region_hash(rows, bpp, 0, 0, 2, 5) == hashlib.sha256(b"".join(pixels)).hexdigest()
    print("png-region self-test: ok")


def main():
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return
    if len(sys.argv) != 6:
        sys.exit(__doc__)
    width, height, bpp, rows = decode(open(sys.argv[1], "rb").read())
    x, y, w, h = (int(float(v)) for v in sys.argv[2:])
    if x < 0 or y < 0 or w <= 0 or h <= 0 or x + w > width or y + h > height:
        sys.exit(f"region {x},{y},{w},{h} outside {width}x{height}")
    print(region_hash(rows, bpp, x, y, w, h))


if __name__ == "__main__":
    main()
