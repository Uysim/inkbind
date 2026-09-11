#!/usr/bin/env python3
"""Generate a single-page PDF whose page embeds one `/DCTDecode` (JPEG)
Image XObject, for inkbind test fixtures.

No third-party libraries. The image stream's bytes are a placeholder (SOI/EOI
markers wrapping a fixed byte string) rather than a real, decodable JPEG —
encoding a genuine JPEG from scratch needs an encoder, which is out of scope
for a hand-verifiable fixture. The placeholder is enough to test that inkbind
passes `/DCTDecode` stream bytes through untouched (`ImageFormat::Jpeg`)
without attempting to decode JPEG pixel data itself, which it never does.

Otherwise structured like gen_image_flate_pdf.py:
  * catalog -> pages -> one page (Letter, 612x792)
  * a content stream that paints the image (`/Im0 Do`) — not used by
    inkbind's image extraction, included only so the file is a well-formed
    PDF
  * a classic (non-stream) cross-reference table + trailer
"""
import sys

# SOI + APP0-ish marker + payload + EOI. NOT a decodable JPEG — see
# module docstring and tests/fixtures/README.md.
JPEG_PLACEHOLDER = b"\xff\xd8\xff\xe0" + b"NOT A REAL JPEG - INKBIND PLACEHOLDER" + b"\xff\xd9"


def build():
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
        b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 "
        b"/ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode "
        b"/Length %d >>\nstream\n" % len(JPEG_PLACEHOLDER) + JPEG_PLACEHOLDER + b"\nendstream"
    )
    objects.append(
        b"<< /Title (Inkbind Image JPEG Fixture) "
        b"/Producer (inkbind gen_image_jpeg_pdf.py) /Creator (inkbind) >>"
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
    dest = sys.argv[1] if len(sys.argv) > 1 else "image_jpeg.pdf"
    data = build()
    with open(dest, "wb") as f:
        f.write(data)
    print("wrote %s (%d bytes)" % (dest, len(data)))
