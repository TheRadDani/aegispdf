#!/usr/bin/env python3
"""
Generate minimal valid placeholder icons for CI builds.

tauri::generate_context!() is a proc-macro that reads every icon listed in
tauri.conf.json at *compile time* and panics if any file is missing.  The real
icons are produced by scripts/gen-icons.sh (requires inkscape/imagemagick).

This script uses only Python 3 stdlib so it works on any GitHub Actions runner
without installing extra tools.  The output files are syntactically valid but
contain only a solid-colour square — sufficient for the proc-macro to succeed.

Run from the repository root:
    python3 scripts/gen-placeholder-icons.py
"""

import os
import struct
import zlib


def png_chunk(tag: bytes, data: bytes) -> bytes:
    crc = zlib.crc32(tag + data) & 0xFFFF_FFFF
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)


def make_png(size: int, rgba: tuple = (70, 130, 212, 255)) -> bytes:
    """Minimal solid-colour RGBA PNG (colour type 6 — required by tauri::generate_context!)."""
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)  # 6 = RGBA
    # Each row: filter byte 0x00 + RGBA pixels
    row = bytes([0]) + bytes(rgba) * size
    idat = zlib.compress(row * size, level=9)
    return (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", ihdr)
        + png_chunk(b"IDAT", idat)
        + png_chunk(b"IEND", b"")
    )


def make_ico(sizes: tuple = (16, 32, 48)) -> bytes:
    """Multi-image ICO containing one PNG entry per size."""
    images = [(s, make_png(s)) for s in sizes]
    n = len(images)
    # ICONDIR header: reserved=0, type=1 (ICO), count=n
    header = struct.pack("<HHH", 0, 1, n)
    # Each directory entry is 16 bytes; data starts after header + all entries
    offset = 6 + n * 16
    entries = b""
    blobs = b""
    for size, data in images:
        # width, height, palette_count, reserved, planes, bpp, size, offset
        entries += struct.pack(
            "<BBBBHHII", size, size, 0, 0, 1, 32, len(data), offset
        )
        offset += len(data)
        blobs += data
    return header + entries + blobs


def make_icns(png_32: bytes) -> bytes:
    """Minimal ICNS with a single 32×32 PNG chunk (ic07, macOS 10.7+)."""
    entry = b"ic07" + struct.pack(">I", 8 + len(png_32)) + png_32
    return b"icns" + struct.pack(">I", 8 + len(entry)) + entry


def main() -> None:
    out_dir = os.path.join("src-tauri", "icons")
    os.makedirs(out_dir, exist_ok=True)

    png32 = make_png(32)
    png128 = make_png(128)
    png256 = make_png(256)

    files = {
        "32x32.png": png32,
        "128x128.png": png128,
        "128x128@2x.png": png256,
        "icon.ico": make_ico((16, 32, 48)),
        "icon.icns": make_icns(png32),
    }

    for name, data in files.items():
        path = os.path.join(out_dir, name)
        with open(path, "wb") as f:
            f.write(data)
        print(f"  created {path} ({len(data)} bytes)")

    print(f"Placeholder icons written to {out_dir}/")


if __name__ == "__main__":
    main()
