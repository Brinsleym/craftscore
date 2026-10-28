# File-format parity

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** minor (PDF rows reuse PDFCraft's crates; earlier: major, first version) · **Target:** enScore 3.7 (2026)

Which files ScoreCraft reads and writes, against what enScore's public listings say it handles.
Status: **done** (with round-trip or corpus tests), **partial**, **missing**. Directions: R read,
W write.

**Summary (estimated):** 0% ready.

## enScore's formats

| Format | enScore | ScoreCraft target | Crate | Spec | Rows | Status |
|---|---|---|---|---|---|---|
| PDF | R, W (export with ink, print, reports) | R, W | `pdf` ♻ pdfcraft-cos, -render, -create, -organize | ISO 32000-1 / 32000-2 | READ-1, EXP-2, PRAC-9 | missing |
| PDF annotations (Ink, Highlight, FreeText, Square/Circle) | export only (inferred) | R, W | `pdf` ♻ pdfcraft-annot | IMP-7, EXP-2 | missing |
| Images (JPEG, PNG, HEIC, TIFF) as pages | R (Photos, camera) | R | `codecs`; ♻ pdfcraft-create (PNG, JPEG, TIFF) | format specs | IMP-3 | missing |
| forScore `.4sc` | R | R only ([why](gaps.md#not-clean-room-feasible-so-far)) | `fsc` | black-box, [two-team](clean-room.md#5-interchange-formats-without-a-public-spec) | IMP-5 | missing |
| enScore setlist / library share file | R, W (container not described publicly) | R, W our own bundle; enScore's own file read once observed | `io` | ours below | LIST-5 | missing |
| Audio recordings | R (attach) | R: WAV, FLAC, MP3, AAC/M4A, Ogg/Opus | `audio` | format specs | PLAY-1 | missing |

## Formats enScore doesn't have (ours only)

| Format | Direction | Why | Rows |
|---|---|---|---|
| ScoreCraft library (directory) | R, W | open, documented, sync-friendly | all |
| ScoreCraft setlist bundle `.scoreset` | R, W | share a setlist with its PDFs and markings | LIST-5 |
| MusicXML / MIDI rendered to pages | R | P3, needs an engraver | IMP-8 |
| Plain CSV/JSON metadata export | W | spreadsheets, migrations | ours |

## ScoreCraft library format

A directory, `<Name>.scorecraft/`, readable without ScoreCraft. Version 1 layout is in
[`architecture.md` §8](architecture.md#8-library-storage-scorecraft-format). Rules:

- `manifest.json`: `{"format":"scorecraft-library","version":1,"library":"<uuid>"}`.
- `blobs/sha256/<2 hex>/<62 hex>`: imported files, byte-for-byte, never modified.
- `scores/<uuid>.json`: one record per score (`doc::Score` serialised), every field
  `#[serde(default)]` so older readers skip unknown fields and newer readers fill missing ones.
- `ink/<uuid>/<page-key>.bin`: annotation ops, length-prefixed records, compacted on idle.
- Writes are atomic per file; a crash leaves either the old or the new record.
- Breaking changes bump `version` and ship a migration plus a reader for the previous version.

## ScoreCraft setlist bundle

`.scoreset` is a zip (stored, not deflated, for PDFs) containing `setlist.json` (order, page
ranges, tempos), `scores/<uuid>.json`, the referenced `blobs/` and `ink/`. Import de-duplicates
blobs by hash and asks before overwriting markings.

## Scanning

| Source | Output | Crate | Rows | Status |
|---|---|---|---|---|
| Camera | multi-page PDF with straightened, cropped, cleaned pages; originals kept | `scan`, `algo` | IMP-1 | missing |
| eSCL network scanner | PDF from flatbed or feeder | `scan` | IMP-2 | missing |

## PDF fidelity targets

The PDF engine is PDFCraft's ([architecture §15](architecture.md#15-reusing-pdfcraft)). These
targets are checked on our score corpus through the `scorecraft-pdf` adapter; failures are
reported and fixed upstream in PDFCraft, then pinned.

| Id | Target | Status |
|---|---|---|
| PDF-1 | ≥ 99% of the pinned public-domain score corpus opens; none crash | missing |
| PDF-2 | ≥ 95% render within tolerance of a reference rasteriser at 150 and 300 dpi | missing |
| PDF-3 | Scanned scores: JBIG2, CCITT G4, DCT, JPX images decode | missing |
| PDF-4 | Damaged files: broken xref and truncated streams are repaired or the bad page shows a placeholder | missing |
| PDF-5 | Exported PDFs pass a validator (veraPDF or qpdf --check) and open in three other readers | missing |
| PDF-6 | Encrypted PDFs with an empty user password open; others ask for the password | missing |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | minor | PDF rows reuse PDFCraft's crates |
| 2026-10-28 | major | First version |
