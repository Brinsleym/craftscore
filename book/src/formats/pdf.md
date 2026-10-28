# PDF

`scorecraft-pdf` is a standalone crate (no workspace dependencies) implemented from ISO 32000-1
and ISO 32000-2. It must handle what real score PDFs contain: scanned pages with JBIG2, CCITT G4,
DCT and JPX images; engraved pages with embedded Type 1, TrueType, CFF and Type 3 fonts; damaged
cross-reference tables; incremental updates; outlines; text layers for search. It writes new PDFs
for export (ink flattened or as ISO 32000 annotations), manuscript paper and practice reports.

The choice of rasteriser is open decision D1 ([`docs/decisions.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/decisions.md)).
