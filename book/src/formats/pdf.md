# PDF

`scorecraft-pdf` is a thin adapter over [PDFCraft](https://github.com/storytold/pdfcraft)'s PDF
crates (`pdfcraft-cos`, `-model`, `-render`, `-annot`, `-organize`, `-create`; MIT OR Apache-2.0),
pinned to one revision. Through them it must handle what real score PDFs contain: scanned pages with JBIG2, CCITT G4,
DCT and JPX images; engraved pages with embedded Type 1, TrueType, CFF and Type 3 fonts; damaged
cross-reference tables; incremental updates; outlines; text layers for search. It writes new PDFs
for export (ink flattened or as ISO 32000 annotations), manuscript paper and practice reports.

Why and how: [`docs/architecture.md` §15](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/architecture.md#15-reusing-pdfcraft) and
decision D1 in [`docs/decisions.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/decisions.md).
