#!/usr/bin/env python3
"""Generate a single-page PDF whose page embeds one small raster image,
stored as a `/FlateDecode`-compressed `/DCTDecode`-free Image XObject, for
inkbind test fixtures.

No third-party libraries. Emits a PDF 1.7 file with:
  * catalog -> pages -> one page (Letter, 612x792)
  * a content stream that paints the image (`/Im0 Do`) — not used by
    inkbind's image extraction, included only so the file is a well-formed,
    renderable PDF
  * a 2x2 pixel, 8-bit DeviceRGB Image XObject, raw pixel data
    zlib-compressed with `/Filter /FlateDecode`
  * an /Info dictionary with /Title and /Producer
  * a classic (non-stream) cross-reference table + trailer

The xref offsets are computed from the actual serialized bytes, so the file
is valid without manual offset bookkeeping.
"""
import sys
import zlib

# 2x2 pixels, 8 bits/component, DeviceRGB, row-major, no padding:
#   (255,0,0) (0,255,0)
#   (0,0,255) (255,255,0)
PIXELS = bytes([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0])


def build():
    compressed = zlib.compress(PIXELS, level=9)

    objects = []
    objects.append(b"<< /Type /Catalog /Pages 2 0 R >>")
    objects.append(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    objects.append(
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>"
    )
    content = b"q 2 0 0 2 0 0 cm /Im0 Do Q\n"
    objects.append(b"<< /Length %d >>\nstream\n" % len(content) + content + b"endstream")
    objects.append(
        b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode "
        b"/Length %d >>\nstream\n" % len(compressed) + compressed + b"\nendstream"
    )
    objects.append(
        b"<< /Title (Inkbind Image Flate Fixture) "
        b"/Producer (inkbind gen_image_flate_pdf.py) /Creator (inkbind) >>"
    )

    out = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
    offsets = []
    for i, body in enumerate(objects, start=1):
        offsets.append(len(out))
        out += b"%d 0 obj\n" % i
        out += body
        out += b"\nendobj\n"

    xref_pos = len(out)
    n = len(objects) + 1
    out += b"xref\n0 %d\n" % n
    out += b"0000000000 65535 f \n"
    for off in offsets:
        out += b"%010d 00000 n \n" % off
    out += b"trailer\n<< /Size %d /Root 1 0 R /Info 6 0 R >>\n" % n
    out += b"startxref\n%d\n%%%%EOF\n" % xref_pos
    return bytes(out)


if __name__ == "__main__":
    dest = sys.argv[1] if len(sys.argv) > 1 else "image_flate.pdf"
    data = build()
    with open(dest, "wb") as f:
        f.write(data)
    print("wrote %s (%d bytes)" % (dest, len(data)))
