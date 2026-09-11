#!/usr/bin/env python3
"""Generate a single-page PDF fixture that uses a PDF 1.5+ cross-reference
*stream* (/Type /XRef) instead of a classic xref table + trailer keyword.

Every other fixture in this suite uses the classic (non-stream)
cross-reference table; this one isolates the alternate xref-parsing path
that modern PDF producers commonly emit. All content/font/info objects stay
as plain, directly-referenced, uncompressed objects -- only the
cross-reference mechanism changes.

No third-party libraries. The xref stream itself is stored uncompressed
(no /Filter) for readability; the PDF spec allows this.
"""
import sys


def obj_bytes(num, body):
    return b"%d 0 obj\n" % num + body + b"\nendobj\n"


def build():
    objects = {
        1: b"<< /Type /Catalog /Pages 2 0 R >>",
        2: b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        3: (
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            b"/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
        ),
        5: b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        6: (
            b"<< /Title (Inkbind Xref Stream Fixture) "
            b"/Producer (inkbind gen_xref_stream_pdf.py) /Creator (inkbind) >>"
        ),
    }
    stream = b"BT /F1 24 Tf 72 720 Td (Xref Stream) Tj ET\n"
    objects[4] = b"<< /Length %d >>\nstream\n" % len(stream) + stream + b"endstream"

    out = bytearray(b"%PDF-1.5\n%\xe2\xe3\xcf\xd3\n")
    offsets = {}
    for num in range(1, 7):
        offsets[num] = len(out)
        out += obj_bytes(num, objects[num])

    xref_obj_num = 7
    xref_offset = len(out)
    size = xref_obj_num + 1  # object numbers 0..7

    # W = [1, 4, 2]: 1-byte type, 4-byte offset/next-free, 2-byte generation.
    rows = bytearray()
    rows += bytes([0]) + (0).to_bytes(4, "big") + (0).to_bytes(2, "big")  # obj 0: free list head
    for num in range(1, 7):
        rows += bytes([1]) + offsets[num].to_bytes(4, "big") + (0).to_bytes(2, "big")
    rows += bytes([1]) + xref_offset.to_bytes(4, "big") + (0).to_bytes(2, "big")  # obj 7: this stream

    xref_dict = b"<< /Type /XRef /Size %d /W [1 4 2] /Root 1 0 R /Info 6 0 R /Length %d >>" % (
        size,
        len(rows),
    )
    out += b"%d 0 obj\n" % xref_obj_num
    out += xref_dict
    out += b"\nstream\n"
    out += bytes(rows)
    out += b"\nendstream\nendobj\n"
    out += b"startxref\n%d\n%%%%EOF\n" % xref_offset
    return bytes(out)


if __name__ == "__main__":
    dest = sys.argv[1] if len(sys.argv) > 1 else "xref_stream.pdf"
    data = build()
    with open(dest, "wb") as f:
        f.write(data)
    print("wrote %s (%d bytes)" % (dest, len(data)))
