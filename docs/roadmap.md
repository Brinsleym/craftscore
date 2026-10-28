# Roadmap

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version: alpha gate, milestones M0–M11, current focus) · **Target:** enScore 3.7 (2026)

Forward-looking plan: milestones, the **Current focus** list, and what's next with estimates.
Where we stand today is in [`target-app-parity.md`](target-app-parity.md) and the ranked work list
in [`gaps.md`](gaps.md); the one-page summary is [`ROADMAP.md`](../ROADMAP.md).

Status legend: ✅ done · 🟡 in progress · ⬜ not started. Hours are agent wall-clock hours for one
agent working sequentially, uncalibrated ([calibration](target-app-parity.md#remaining-effort-and-calibration)).

**Stage: pre-alpha** (no code). Alpha needs the gate below: ~350–720 h.

## Alpha gate

The workflows a performing musician runs every week, checked end to end on the main platform
(desktop first, then one tablet), including quitting and reopening with everything intact. Any
**no**, or a **partial** that blocks the workflow, keeps ScoreCraft pre-alpha.

| Core workflow | Works end to end? | Rows | Hours to pass |
|---|---|---|---:|
| Build a library: import PDFs and a folder, set title/composer/tags, find a score by search | no | LIB-1, LIB-3..5, LIB-9, IMP-6 | 40–80 |
| Read: open a score, single and two-page layouts, half-page turns, turn by tap, keys and a Bluetooth pedal, screen stays awake | no | READ-1..5, READ-9, READ-15, TURN-1..4 | 90–180 |
| Mark up: pen with pressure, highlighter, stroke eraser, text note, undo; quit, reopen, markings intact and the PDF untouched | no | ANNO-1..3, ANNO-6, ANNO-8, ANNO-13..15 | 60–110 |
| Perform a setlist: build a list, play it through with the pedal, pieces flow, no crash | no | LIST-1, LIST-4, LIST-6, NAV-5 | 15–30 |
| Edit pages: reorder, rotate, hide, delete; bookmarks | no | PAGE-1..4, LIB-11 | 10–20 |
| Share: export a PDF with markings that opens in other PDF readers | no | EXP-2 | 10–18 |
| Count in: metronome with tap tempo | no | PRAC-1 | 6–12 |

Plus the skeleton (M0, 60–100 h) and the stability floor: never-crash lints on every crate,
`panic_hunt` green, fuzzed `pdf` and library loader.

**Result: does not pass** (2026-10-28, no code).

## Milestones

| M | Status | Name | Where we are |
|---|---|---|---|
| M0 | ⬜ | Skeleton | docs only (this PR) |
| M1 | ⬜ | Foundation | |
| M2 | ⬜ | PDF v1 | |
| M3 | ⬜ | Reader app | |
| M4 | ⬜ | Ink | |
| M5 | ⬜ | Library | |
| M6 | ⬜ | Performance (→ alpha) | |
| M7 | ⬜ | Mobile | |
| M8 | ⬜ | Sync | |
| M9 | ⬜ | Interop (→ beta with M7, M8) | |
| M10 | ⬜ | Practice and MIDI | |
| M11 | ⬜ | Parity | |

## Current focus

Ranked; pick from the top unless an issue is assigned to you.

1. **M0 skeleton** ([G3](gaps.md#g3-stability-and-data-safety)): workspace, every crate stub
   registered in `xtask/src/layers.rs`, lints, `clippy.toml`, `rustfmt.toml`, `xtask` (`layers`,
   `wasm`, `ci`, `parity`, `scorecard`), CI on Linux, macOS, Windows and wasm, testkit with a
   synthetic PDF generator.
2. **Decision D1, the PDF rasteriser** ([G1](gaps.md#g1-pdf-engine-parse-render-write)): evaluate
   candidates against a 200-file public-domain score corpus (render correctness, speed, wasm size,
   licence), then write the decision.
3. **Specs for the alpha rows**: `docs/spec/page-turning.md`, `ink.md`, `library.md`,
   `setlists.md` from the public listings plus observation passes ([G17](gaps.md#g17-unknowns-behaviour-not-yet-observed)).
4. **Create the corpus repository** with pinned public-domain scores and hostile synthetic PDFs.

## What's next, with estimates

| Rank | Milestone | Estimate |
|---|---|---:|
| 1 | M0 skeleton and CI | 60–100 h |
| 2 | M1 foundation (geom, raster, doc, ops, format v1) | 40–80 h |
| 3 | M2 PDF v1 | 80–160 h |
| 4 | M3 reader app (layouts, turns, control channel, desktop + web) | 60–110 h |
| 5 | M4 ink | 80–150 h |
| 6 | M5 library | 60–120 h |
| 7 | M6 performance (setlists, pedals, shortcuts, snips, metronome) | 50–100 h |
| Later | M7 mobile, M8 sync, M9 interop, M10 practice and MIDI, M11 parity | 480–950 h |

## Milestone definitions

Each milestone has a **definition of done (DoD)** and must leave `main` green on all tier-1
platforms (macOS, Windows, Linux) plus a wasm build.

| M | Name | Scope (key items) | DoD / acceptance |
|---|---|---|---|
| **M0** | Skeleton | Workspace, all crate stubs, lints, `xtask` (layers, wasm, ci, parity, scorecard), CI matrix, testkit | `cargo test --workspace` green; `cargo xtask wasm` green; layering check passes; `docs/parity-checklist.md` generated (0 / 118) |
| **M1** | Foundation | geom, raster (tiles, pyramids), doc (full model incl. layers, snips, shortcuts, setlists), ops (history), format v1 (blobs, records, atomic saves) | Property tests on history and the store; crash-during-save test; format fails to compile on an unmapped doc field |
| **M2** | PDF v1 | parse (xref, object streams, repair), render (paths, text, images incl. JBIG2/CCITT/DCT), outline, text layer, writer | ≥ 95% of the pinned score corpus renders within tolerance of the reference; fuzz target; malformed-input tests; 300 dpi page render within budget |
| **M3** | Reader app | egui shell, library-less open, single/two-page, half-page and alternating turns, zoom, full screen, keep-awake, tap/key turns, control channel, CLI `run`, desktop + web | Turn latency within budget from cache; agent drives a turn and screenshots it over the control channel |
| **M4** | Ink | ink engine, pen/highlighter/whiteout/erasers, text notes, stamps, layers, lasso, undo/redo, export with ink | Ink latency budget on desktop; strokes survive rotate/crop/reorder; exported PDF annotations open in two other readers |
| **M5** | Library | import files/folders/images, folders, metadata, search (incl. full text), sort, bookmarks, Recently Deleted, verify/repair, thumbnails | 10,000-score synthetic library opens and searches within budget |
| **M6** | Performance | setlists, page ranges, tempos, pedal mapping, Score Shortcuts, Score Snips, metronome | **Alpha gate passes** |
| **M7** | Mobile | Android and iOS/iPadOS shells, pen pressure, share-sheet import, store packaging | Alpha gate passes on one Android tablet and one iPad |
| **M8** | Sync | op-log sync, CRDT merge, folder/WebDAV/server backends, backup/restore | Randomised two-device histories converge; offline edits on both devices merge without loss |
| **M9** | Interop | forScore `.4sc` import, camera and eSCL scanning, IMSLP, title/composer detection, print | Two-team `.4sc` spec and import of the sample corpus; **beta** (with M7 and M8) |
| **M10** | Practice and MIDI | audio attach, time-stretch, pitch-shift, loops, recorded turns, tuner, practice log, MIDI in/out | Practice workflow end to end; MIDI learn with two controllers |
| **M11** | Parity | localization (15 locales), e-ink mode, multi-window, side by side, remaining P2 rows | Breadth ≥ 95% of the feature map |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
