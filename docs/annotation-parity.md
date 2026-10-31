# Annotation parity: ink, tools and layers

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-31 · **Change:** minor (M4 part 1: stroke engine, highlighter, whiteout, erasers, pen only, touch lock and ruler partial in the engine; earlier: first version, all rows missing) · **Target:** enScore 3.7 (2026)

How ScoreCraft's markup compares with enScore 3.7 (PhotoCraft's brush-parity checklist, for a score
reader). Status and evidence as in [`ui-parity.md`](ui-parity.md).

**Summary (estimated):** about 15% ready (2026-10-31): the stroke engine, the highlighter, whiteout, both
erasers, pressure, palm rejection logic and the ruler work in the engine and the CPU renderer with
tests ([spec](spec/ink.md)); nothing is drawn by a pen in the app yet (M4 part 4), so no ink-feel
row can be judged. Text, stamps, layers commands, selection, shapes and export are still to do.

## Ink feel

| Id | Target | Rows | Status | Evidence |
|---|---|---|---|---|
| ANN-1 | Pen-down to first visible ink ≤ 1 frame at 120 Hz on supported tablets (predicted points, replaced by real ones) | ANNO-1 | missing | |
| ANN-2 | Pressure maps to width with a configurable curve; no jumps at stroke start | ANNO-2 | partial | `PressureCurve` (floor 0.25, γ 0.6) and smoothed start pressure, tested in `scorecraft-ink` (INK-4); not yet on a device |
| ANN-3 | Stroke smoothing that keeps sharp corners of handwritten fingerings | ANNO-1 | partial | Corner-aware smoothing and Catmull–Rom subdivision; a jittered 90° corner survives within 1 pt (INK-15) |
| ANN-4 | Strokes stay crisp at every zoom (vector, not bitmap) | ANNO-1 | partial | Stored as page-space samples, drawn per render scale from the outline; cached per page and scale |
| ANN-5 | Palm rejection while the pen draws; pen only by default with Draw with Touch; touch lock | ANNO-14 | partial | `PalmFilter` in `scorecraft-input` (INK-20..23), `input.pen.only` and `input.touch.lock` in the engine (INK-17, -18, -24); the shell doesn't route pointers through it yet |

## Tools

| Id | Target | Rows | Status |
|---|---|---|---|
| ANN-10 | Pen with colours, widths and presets; eight changeable swatches and a fixed 30-colour palette; width per tool | ANNO-1, ANNO-4 | partial (swatches, palette and widths in the engine; presets missing) |
| ANN-11 | Highlighter (translucent, multiply-like over notation, stronger option) | ANNO-3 | partial (multiply with no build-up within a stroke, INK-5, -6) |
| ANN-12 | Whiteout (opaque page-coloured ink, then write over) | ANNO-5 | partial (filter paper colour, INK-7, -28) |
| ANN-13 | Stroke eraser | ANNO-6 | partial (INK-8) |
| ANN-14 | Pixel eraser splitting strokes | ANNO-7 | partial (INK-9) |
| ANN-15 | Text notes: font size, colour, move, edit | ANNO-8 | missing |
| ANN-16 | Stamps from a SMuFL font: fingerings, dynamics, bowings, breath marks, custom stamps from a selection | ANNO-9 | missing |
| ANN-17 | Rectangle select: move, copy, resize, recolour, delete; starts snips and shortcuts | ANNO-12 | missing |
| ANN-18 | Shapes and straight lines | ANNO-16 | missing |
| ANN-19 | Stylus button / double-tap tool switching | ANNO-11 | missing |
| ANN-20 | Ruler, its button hidden unless a setting shows it | ANNO-17 | partial (ruling along the edge, INK-27; no UI) |

## Layers and history

| Id | Target | Rows | Status |
|---|---|---|---|
| ANN-30 | Layers: new, rename, reorder, show/hide, lock; hidden layers excluded from performance view and optionally from export | ANNO-10, EXP-2 | missing |
| ANN-31 | Undo/redo per score, one step per stroke | ANNO-13 | partial (one step per stroke and per eraser gesture, INK-2, -8, -9) |
| ANN-32 | Markings never alter the PDF; survive rotate, crop, reorder and page-range changes | ANNO-15 | partial (page space, rotate and reorder tested, INK-3, -12; crop isn't applied by the reader yet) |
| ANN-33 | Markings imported from `.4sc` and from PDF annotations land in the right layers | IMP-5, IMP-7 | missing |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-31 | minor | M4 part 1: ANN-2..5, -10..14, -31, -32 partial; ANN-20 ruler added; ANN-17 becomes rectangle select (owner's observation) |
| 2026-10-28 | major | First version |
