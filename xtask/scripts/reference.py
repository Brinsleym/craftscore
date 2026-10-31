"""Reference renders for the score corpus (docs/development.md, "Test corpora").

Renders the pages of every PDF under a directory (recursively) with PDFium (pypdfium2,
Apache-2.0/BSD) at 150 dpi, and the first three pages at 300 dpi. `--max-pages N` stops after
N pages of each file. With `--hash-ids`, each file is named by the first 16 hex digits of its
SHA-256 instead of its file name, and byte-identical copies are rendered once: use it for a
private library, so no file name leaves it. Each render is turned into 8-bit grey with the ITU-R
BT.601 weights and box-averaged over 4 x 4 blocks, then written as a binary PGM:

    <out>/<id>/p001-150.pgm

The corpus test (`crates/pdf/tests/corpus.rs`) reduces ScoreCraft's renders the same way and
compares the two.

Images are drawn without smoothing (`no_smoothimage`). PDFium smooths every image it scales by
default; ScoreCraft's renderer (hayro, through PDFCraft) follows each image's /Interpolate flag,
which defaults to off (ISO 32000-2, 8.9.5.3). On a scanned score that one difference made the
whole page fail, while with it removed the renders were pixel-identical. Paths and text stay
anti-aliased on both sides.

Pages are drawn at exactly the requested scale onto a bitmap sized as ScoreCraft sizes it (each
side rounded up, ignoring float noise under 1/100 px). `PdfPage.render` rounds 792 pt at 150 dpi
(1650.0000000000002 px) up to 1651 rows and stretches the page to fill them, which moved every
staff line by up to a pixel and failed engraved pages on that alone.

Run through `cargo xtask corpus --references`, which installs nothing:
`pip install pypdfium2 pillow` first.
"""

import argparse
import hashlib
import math
import pathlib
import sys

import pypdfium2 as pdfium
import pypdfium2.raw as pdfium_c
from PIL import Image

BLOCK = 4
FULL_DPI = 150
DETAIL_DPI = 300
DETAIL_PAGES = 3
# Bump the version when the renders change, so `cargo xtask test-corpus` redoes old references.
MARKER = ".complete-v3"


def device_pixels(pt, scale):
    """Pixels covering `pt` points, as pdfcraft-render's `device_pixels` counts them."""
    return max(1, math.ceil(pt * scale - 0.01))


def render(page, scale):
    """The page at exactly `scale`, on white, with annotations, images unsmoothed."""
    w_pt, h_pt = page.get_size()
    w, h = device_pixels(w_pt, scale), device_pixels(h_pt, scale)
    bitmap = pdfium.PdfBitmap.new_native(w, h, pdfium_c.FPDFBitmap_BGRx)
    bitmap.fill_rect((255, 255, 255, 255), 0, 0, w, h)
    # PDFium maps the page (crop box and /Rotate applied) to points, y down, then this matrix.
    matrix = pdfium_c.FS_MATRIX(scale, 0, 0, scale, 0, 0)
    clip = pdfium_c.FS_RECTF(0, 0, w, h)
    flags = pdfium_c.FPDF_ANNOT | pdfium_c.FPDF_RENDER_NO_SMOOTHIMAGE
    pdfium_c.FPDF_RenderPageBitmapWithMatrix(bitmap.raw, page.raw, matrix, clip, flags)
    return bitmap


def reduce(image):
    grey = image.convert("RGB").convert("L")
    return grey.reduce(BLOCK)


def write_pgm(path, image):
    path.parent.mkdir(parents=True, exist_ok=True)
    w, h = image.size
    with open(path, "wb") as f:
        f.write(b"P5\n%d %d\n255\n" % (w, h))
        f.write(image.tobytes())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("src")
    parser.add_argument("out")
    parser.add_argument("--hash-ids", action="store_true")
    parser.add_argument("--max-pages", type=int, default=0)
    args = parser.parse_args()
    src, out = pathlib.Path(args.src), pathlib.Path(args.out)
    files = sorted(p for p in src.rglob("*") if p.is_file() and p.suffix.lower() == ".pdf")
    if not files:
        sys.exit(f"no PDFs under {src}")
    seen = set()
    for pdf_path in files:
        if args.hash_ids:
            doc_id = hashlib.sha256(pdf_path.read_bytes()).hexdigest()[:16]
        else:
            doc_id = pdf_path.stem
        if doc_id in seen:
            continue
        seen.add(doc_id)
        try:
            pdf = pdfium.PdfDocument(str(pdf_path))
        except pdfium.PdfiumError as e:
            print(f"{doc_id}: PDFium cannot open it ({e}); no reference", file=sys.stderr)
            continue
        count = len(pdf) if args.max_pages <= 0 else min(len(pdf), args.max_pages)
        skipped = []
        for index in range(count):
            dpis = [FULL_DPI] + ([DETAIL_DPI] if index < DETAIL_PAGES else [])
            try:
                page = pdf[index]
                for dpi in dpis:
                    bitmap = render(page, dpi / 72)
                    write_pgm(out / doc_id / f"p{index + 1:03}-{dpi}.pgm", reduce(bitmap.to_pil()))
            except pdfium.PdfiumError:
                # PDFium can't load or draw this page: it gets no reference, so the corpus test
                # checks only that ScoreCraft renders it without an error.
                skipped.append(index + 1)
        (out / doc_id).mkdir(parents=True, exist_ok=True)
        (out / doc_id / "pages.txt").write_text(str(len(pdf)))
        note = f"; PDFium failed on pages {skipped}" if skipped else ""
        print(f"{doc_id}: {count} of {len(pdf)} pages{note}")
    # Written last, so an interrupted run is redone instead of half-used.
    (out / MARKER).write_text("ok")


if __name__ == "__main__":
    main()
