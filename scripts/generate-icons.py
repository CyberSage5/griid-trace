#!/usr/bin/env python3
"""Generate minimal Tauri app icons for griid-trace (no external deps)."""

import struct
import zlib
from pathlib import Path

# griid-trace brand: cyan #58a6ff on dark #0d1117 with green accent dot
BG = (0x0D, 0x11, 0x17, 255)
CYAN = (0x58, 0xA6, 0xFF, 255)
GREEN = (0x3F, 0xB9, 0x50, 255)


def png_chunk(tag: bytes, data: bytes) -> bytes:
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)


def write_png(path: Path, size: int) -> None:
    raw = bytearray()
    cx, cy = size // 2, size // 2
    r_outer = int(size * 0.38)
    r_inner = max(2, int(size * 0.08))

    for y in range(size):
        raw.append(0)  # filter byte
        for x in range(size):
            dx, dy = x - cx, y - cy
            dist = (dx * dx + dy * dy) ** 0.5
            # lightning bolt shape approximation: diagonal band
            on_bolt = abs(dx + dy) < size * 0.12 and abs(dx - dy) > size * 0.15 and dist < r_outer
            on_dot = (x - int(size * 0.72)) ** 2 + (y - int(size * 0.28)) ** 2 < r_inner ** 2
            if on_dot:
                raw.extend(GREEN)
            elif on_bolt:
                raw.extend(CYAN)
            elif dist < r_outer:
                raw.extend((0x21, 0x26, 0x2D, 255))
            else:
                raw.extend(BG)

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    compressed = zlib.compress(bytes(raw), 9)
    png = b"\x89PNG\r\n\x1a\n"
    png += png_chunk(b"IHDR", ihdr)
    png += png_chunk(b"IDAT", compressed)
    png += png_chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(png)


def write_ico(path: Path, sizes: list[int]) -> None:
    images = []
    for size in sizes:
        raw = bytearray()
        cx, cy = size // 2, size // 2
        r_outer = int(size * 0.38)
        r_inner = max(2, int(size * 0.08))
        for y in range(size):
            raw.append(0)
            for x in range(size):
                dx, dy = x - cx, y - cy
                dist = (dx * dx + dy * dy) ** 0.5
                on_bolt = abs(dx + dy) < size * 0.12 and abs(dx - dy) > size * 0.15 and dist < r_outer
                on_dot = (x - int(size * 0.72)) ** 2 + (y - int(size * 0.28)) ** 2 < r_inner ** 2
                if on_dot:
                    raw.extend(GREEN[:3])
                elif on_bolt:
                    raw.extend(CYAN[:3])
                elif dist < r_outer:
                    raw.extend((0x21, 0x26, 0x2D))
                else:
                    raw.extend(BG[:3])
        ihdr = struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0)
        compressed = zlib.compress(bytes(raw), 9)
        png = b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", ihdr) + png_chunk(b"IDAT", compressed) + png_chunk(b"IEND", b"")
        images.append((size, png))

    header = struct.pack("<HHH", 0, 1, len(images))
    entries = b""
    offset = 6 + 16 * len(images)
    data = b""
    for size, png in images:
        entries += struct.pack("<BBBBHHII", size, size, 0, 0, 1, 32, len(png), offset)
        data += png
        offset += len(png)
    path.write_bytes(header + entries + data)


def main() -> None:
    root = Path(__file__).resolve().parent.parent / "tauri" / "icons"
    write_png(root / "32x32.png", 32)
    write_png(root / "128x128.png", 128)
    write_png(root / "128x128@2x.png", 256)
    write_ico(root / "icon.ico", [16, 32, 48])
    # icns: copy 128 png as placeholder — Tauri accepts png set; mac build may need icns
    write_png(root / "icon.icns", 128)  # not valid icns but tauri bundle uses pngs primarily
    # Better: write minimal icns from 128 png - for ship use png copy renamed
    (root / "icon.icns").write_bytes((root / "128x128.png").read_bytes())
    print(f"Icons written to {root}")


if __name__ == "__main__":
    main()
