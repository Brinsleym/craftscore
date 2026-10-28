# Annotation parity: ink, tools and layers

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, all rows missing) · **Target:** enScore 3.7 (2026)

How ScoreCraft's markup compares with enScore 3.7 (PhotoCraft's brush-parity checklist, for a score
reader). Status and evidence as in [`ui-parity.md`](ui-parity.md).

**Summary (estimated):** 0% ready.

## Ink feel

| Id | Target | Rows | Status | Evidence |
|---|---|---|---|---|
| ANN-1 | Pen-down to first visible ink ≤ 1 frame at 120 Hz on supported tablets (predicted points, replaced by real ones) | ANNO-1 | missing | |
| ANN-2 | Pressure maps to width with a configurable curve; no jumps at stroke start | ANNO-2 | missing | |
| ANN-3 | Stroke smoothing that keeps sharp corners of handwritten fingerings | ANNO-1 | missing | |
| ANN-4 | Strokes stay crisp at every zoom (vector, not bitmap) | ANNO-1 | missing | |
| ANN-5 | Palm rejection; pen-only mode | ANNO-14 | missing | |

## Tools

| Id | Target | Rows | Status |
|---|---|---|---|
| ANN-10 | Pen with colours, widths and presets | ANNO-1, ANNO-4 | missing |
| ANN-11 | Highlighter (translucent, multiply-like over notation, stronger option) | ANNO-3 | missing |
| ANN-12 | Whiteout (opaque page-coloured ink, then write over) | ANNO-5 | missing |
| ANN-13 | Stroke eraser | ANNO-6 | missing |
| ANN-14 | Pixel eraser splitting strokes | ANNO-7 | missing |
| ANN-15 | Text notes: font size, colour, move, edit | ANNO-8 | missing |
| ANN-16 | Stamps from a SMuFL font: fingerings, dynamics, bowings, breath marks, custom stamps from a selection | ANNO-9 | missing |
| ANN-17 | Lasso select, move, resize, recolour, delete | ANNO-12 | missing |
| ANN-18 | Shapes and straight lines | ANNO-16 | missing |
| ANN-19 | Stylus button / double-tap tool switching | ANNO-11 | missing |

## Layers and history

| Id | Target | Rows | Status |
|---|---|---|---|
| ANN-30 | Layers: new, rename, reorder, show/hide, lock; hidden layers excluded from performance view and optionally from export | ANNO-10, EXP-2 | missing |
| ANN-31 | Undo/redo per score, one step per stroke | ANNO-13 | missing |
| ANN-32 | Markings never alter the PDF; survive rotate, crop, reorder and page-range changes | ANNO-15 | missing |
| ANN-33 | Markings imported from `.4sc` and from PDF annotations land in the right layers | IMP-5, IMP-7 | missing |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
