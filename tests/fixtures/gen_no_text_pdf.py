#!/usr/bin/env python3
"""Generate a single-page PDF with an empty content stream (no text at all).

No third-party libraries. Emits a PDF 1.7 file with:
  * catalog -> pages -> one page (Letter, 612x792)
  * a zero-length content stream (no BT/ET, no Tj) so the page draws nothing
  * an empty /Font resource dict, kept for shape-parity with the other
    fixtures even though no font is ever selected
  * an /Info dictionary with /Title and /Producer
  * a classic (non-stream) cross-reference table + trailer

The xref offsets are computed from the actual serialized bytes, so the file
is valid without manual offset bookkeeping. Mirrors gen_minimal_pdf.py's
approach.
"""
import sys


def build():
    objects = []
    objects.append(b"<< /Type /Catalog /Pages 2 0 R >>")
    objects.append(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    objects.append(
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
        b"/Resources << /Font << >> >> /Contents 4 0 R >>"
    )
    stream = b""
    objects.append(b"<< /Length %d >>\nstream\n" % len(stream) + stream + b"endstream")
    objects.append(
        b"<< /Title (Inkbind No-Text Fixture) /Producer (inkbind gen_no_text_pdf.py) "
        b"/Creator (inkbind) >>"
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
    out += b"trailer\n<< /Size %d /Root 1 0 R /Info 5 0 R >>\n" % n
    out += b"startxref\n%d\n%%%%EOF\n" % xref_pos
    return bytes(out)


if __name__ == "__main__":
    dest = sys.argv[1] if len(sys.argv) > 1 else "no_text.pdf"
    data = build()
    with open(dest, "wb") as f:
        f.write(data)
    print("wrote %s (%d bytes)" % (dest, len(data)))
