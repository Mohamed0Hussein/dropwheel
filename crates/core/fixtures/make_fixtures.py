"""Regenerates the test fixtures in this folder. Needs Pillow:

    python make_fixtures.py

Every image is a few kilobytes: 64x48 pixels with four coloured quadrants, so tests can
tell which way up a decoded image is (top-left red, top-right green, bottom-left blue,
bottom-right white).
"""

import io
import pathlib
import struct
import zlib

from PIL import Image, PngImagePlugin

HERE = pathlib.Path(__file__).resolve().parent
W, H = 64, 48
QUADRANTS = [(220, 30, 30), (30, 200, 60), (40, 60, 220), (250, 250, 250)]


def quadrants(alpha=False):
    img = Image.new("RGBA" if alpha else "RGB", (W, H))
    px = img.load()
    for y in range(H):
        for x in range(W):
            q = (1 if x >= W // 2 else 0) + (2 if y >= H // 2 else 0)
            r, g, b = QUADRANTS[q]
            # A little texture so lossy encoders have something to do.
            n = (x * 7 + y * 13) % 9
            colour = (min(255, r + n), min(255, g + n), min(255, b + n))
            px[x, y] = colour + ((x * 255 // (W - 1),) if alpha else ())
    return img


def exif(orientation=1):
    e = Image.Exif()
    e[0x010F] = "Dropwheel Test"  # Make
    e[0x0110] = "Fixture 1"  # Model
    e[0x0112] = orientation  # Orientation
    e[0x0132] = "2026:10:19 12:00:00"  # DateTime
    e[0x013B] = "Test Author"  # Artist
    gps = e.get_ifd(0x8825)
    gps[1] = "N"
    gps[2] = (30.0, 2.0, 40.0)
    gps[3] = "E"
    gps[4] = (31.0, 14.0, 9.0)
    return e


XMP = (
    b'<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF '
    b'xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/></x:xmpmeta>'
)


def png_chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def main():
    rgb = quadrants()
    rgba = quadrants(alpha=True)

    # Stored sideways (48x64) with Orientation 6, as phones do; shown upright it is 64x48.
    rgb.transpose(Image.Transpose.ROTATE_90).save(
        HERE / "photo.jpg", quality=90, exif=exif(orientation=6), comment=b"fixture comment", xmp=XMP
    )

    info = PngImagePlugin.PngInfo()
    info.add_text("Author", "Test Author")
    info.add_text("Description", "تجربة", zip=False)
    rgba.save(HERE / "alpha.png", pnginfo=info, exif=exif())

    rgba.save(HERE / "alpha.webp", quality=80, exif=exif(), xmp=XMP)
    rgb.save(HERE / "photo.bmp")
    rgb.save(HERE / "photo.tiff", tiffinfo=exif())
    rgb.save(HERE / "photo.gif")
    rgba.resize((32, 32)).save(HERE / "icon.ico", sizes=[(32, 32)])

    # Broken files: none of these may crash anything.
    jpeg = (HERE / "photo.jpg").read_bytes()
    (HERE / "truncated.jpg").write_bytes(jpeg[: len(jpeg) // 2])
    png = (HERE / "alpha.png").read_bytes()
    (HERE / "truncated.png").write_bytes(png[: len(png) // 2])
    (HERE / "garbage.webp").write_bytes(b"RIFF\x40\x00\x00\x00WEBPVP8 " + bytes(range(64)))
    (HERE / "garbage.gif").write_bytes(b"GIF89a" + b"\xff" * 40)
    # A valid PNG that claims to be 60000 x 60000 pixels.
    ihdr = struct.pack(">IIBBBBB", 60000, 60000, 8, 6, 0, 0, 0)
    idat = zlib.compress(b"\x00" * 64)
    (HERE / "bomb.png").write_bytes(
        b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", ihdr) + png_chunk(b"IDAT", idat) + png_chunk(b"IEND", b"")
    )
    (HERE / "empty.jpg").write_bytes(b"")


if __name__ == "__main__":
    main()
