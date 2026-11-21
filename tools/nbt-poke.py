#!/usr/bin/env python3
"""Test-only: replace one root key of a gzip NBT file (a Minecraft player .dat) with an Int.

Usage: tools/nbt-poke.py FILE KEY VALUE

Used by scenario S11 to put a non-compound `NeoForgeData` into a player file while the game is
closed. Keeps every other tag byte-for-byte (strings are kept as raw modified-UTF-8 bytes). Writes
to FILE.tmp and renames it over FILE. Standard library only, so it runs on every OS with Python 3.
"""
import gzip
import os
import struct
import sys

END, BYTE, SHORT, INT, LONG, FLOAT, DOUBLE, BYTES, STRING, LIST, COMPOUND, INTS, LONGS = range(13)


class Reader:
    def __init__(self, data):
        self.data, self.pos = data, 0

    def take(self, n):
        chunk = self.data[self.pos:self.pos + n]
        if len(chunk) != n:
            raise ValueError("truncated NBT")
        self.pos += n
        return chunk

    def unpack(self, fmt):
        return struct.unpack(">" + fmt, self.take(struct.calcsize(">" + fmt)))[0]

    def name(self):
        return self.take(self.unpack("H"))

    def payload(self, kind):
        if kind == BYTE:
            return self.unpack("b")
        if kind == SHORT:
            return self.unpack("h")
        if kind == INT:
            return self.unpack("i")
        if kind == LONG:
            return self.unpack("q")
        if kind == FLOAT:
            return self.take(4)
        if kind == DOUBLE:
            return self.take(8)
        if kind == BYTES:
            return self.take(self.unpack("i"))
        if kind == STRING:
            return self.name()
        if kind == LIST:
            inner, count = self.unpack("b"), self.unpack("i")
            return (inner, [self.payload(inner) for _ in range(count)])
        if kind == COMPOUND:
            entries = []
            while True:
                tag = self.unpack("b")
                if tag == END:
                    return entries
                key = self.name()
                entries.append((tag, key, self.payload(tag)))
        if kind == INTS:
            return [self.unpack("i") for _ in range(self.unpack("i"))]
        if kind == LONGS:
            return [self.unpack("q") for _ in range(self.unpack("i"))]
        raise ValueError(f"unknown tag {kind}")


def write(kind, value, out):
    if kind == BYTE:
        out += struct.pack(">b", value)
    elif kind == SHORT:
        out += struct.pack(">h", value)
    elif kind == INT:
        out += struct.pack(">i", value)
    elif kind == LONG:
        out += struct.pack(">q", value)
    elif kind in (FLOAT, DOUBLE):
        out += value
    elif kind == BYTES:
        out += struct.pack(">i", len(value)) + value
    elif kind == STRING:
        out += struct.pack(">H", len(value)) + value
    elif kind == LIST:
        inner, items = value
        out += struct.pack(">bi", inner, len(items))
        for item in items:
            write(inner, item, out)
    elif kind == COMPOUND:
        for tag, key, payload in value:
            out += struct.pack(">bH", tag, len(key)) + key
            write(tag, payload, out)
        out += b"\x00"
    elif kind == INTS:
        out += struct.pack(">i", len(value)) + b"".join(struct.pack(">i", v) for v in value)
    elif kind == LONGS:
        out += struct.pack(">i", len(value)) + b"".join(struct.pack(">q", v) for v in value)
    else:
        raise ValueError(f"unknown tag {kind}")


def main():
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    path, key, value = sys.argv[1], sys.argv[2].encode(), int(sys.argv[3])
    if not -(2**31) <= value < 2**31:
        sys.exit("VALUE must fit a signed 32-bit Int")
    with gzip.open(path, "rb") as f:
        reader = Reader(f.read())
    root_kind, root_name = reader.unpack("b"), reader.name()
    if root_kind != COMPOUND:
        sys.exit("root is not a compound")
    root = reader.payload(COMPOUND)
    if reader.pos != len(reader.data):
        sys.exit(f"{len(reader.data) - reader.pos} trailing bytes after the root compound")
    before = [entry for entry in root if entry[1] == key]
    root = [entry for entry in root if entry[1] != key] + [(INT, key, value)]
    out = bytearray(struct.pack(">bH", COMPOUND, len(root_name)) + root_name)
    write(COMPOUND, root, out)
    tmp = path + ".tmp"
    with gzip.open(tmp, "wb") as f:
        f.write(bytes(out))
    os.replace(tmp, path)
    old = before[0][0] if before else "absent"
    print(f"nbt-poke: {key.decode()} was tag {old}, now Int {value} in {path}")


if __name__ == "__main__":
    main()
