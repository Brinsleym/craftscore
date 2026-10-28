# Where ScoreCraft falls short of enScore

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** minor (G1 becomes integration of PDFCraft's crates, 80–160 → 20–40 h; earlier: major, first version) · **Target:** enScore 3.7 (2026)

Every known shortfall against enScore 3.7, one entry each, ranked by user impact on the way to
alpha and beta. This is the work list agents pick from: take the highest-ranked entry nobody is
working on, and when you close part of one, update its evidence and estimate here, the matching
parity doc and the feature-map rows. Numbers come from [`target-app-parity.md`](target-app-parity.md);
ids like `READ-5` are rows in [`feature-map.md`](feature-map.md); `#n` are GitHub issues.

"Blocks alpha" and "blocks beta" mark what stands between stages ([roadmap](roadmap.md#alpha-gate)).
Hours are agent-hours, one agent working sequentially, including tests, uncalibrated. **Human**
marks what an agent cannot finish alone (real devices, observation of enScore, legal judgement).

## Summary

| # | Gap | Kind | Impact | Estimate | Parity doc |
|---|---|---|---|---:|---|
| G1 | PDF engine: parse, render, write (reusing PDFCraft) | file format | blocks alpha | 20–40 h | [file-format-parity](file-format-parity.md) |
| G2 | Reader layout and page turns | features, UI | blocks alpha | 40–80 h | [ui-parity](ui-parity.md) |
| G3 | Stability and data safety | stability | blocks alpha | 60–120 h | [scorecard](scorecard.md#reliability) |
| G4 | Ink engine and markup tools | features, UI | blocks alpha | 80–150 h + human | [annotation-parity](annotation-parity.md) |
| G5 | Library store, organisation and search | features | blocks alpha | 60–120 h | [file-format-parity](file-format-parity.md#scorecraft-library-format) |
| G6 | Foot pedals and keyboard turning | hardware | blocks alpha | 10–20 h + human | [hardware-parity](hardware-parity.md) |
| G7 | Setlists, Score Shortcuts and Score Snips | features | blocks beta | 40–80 h | [feature-map](feature-map.md#list-setlists) |
| G8 | Import, export, print and forScore migration | file format | blocks beta | 70–140 h + human | [file-format-parity](file-format-parity.md) |
| G9 | Mobile platforms | platforms | blocks beta | 80–200 h + human | [hardware-parity](hardware-parity.md) |
| G10 | Sync and backup | sync | blocks beta | 120–240 h | [sync-parity](sync-parity.md) |
| G11 | MIDI | hardware | medium | 20–40 h + human | [hardware-parity](hardware-parity.md#midi) |
| G12 | Audio playback and practice tools | features | medium | 60–120 h | [practice-parity](practice-parity.md) |
| G13 | Scanning | features | medium | 36–64 h + human | [file-format-parity](file-format-parity.md#scanning) |
| G14 | IMSLP and automatic metadata | features, online | medium | 30–60 h | [feature-map](feature-map.md#lib-library-and-organisation) |
| G15 | Localization | localization | medium | 40–80 h + human | [localization-parity](localization-parity.md) |
| G16 | E-ink, accessibility and multi-window | UI, platforms | low-medium | 30–60 h + human | [ui-parity](ui-parity.md) |
| G17 | Unknowns: behaviour not yet observed | process | medium | human | [observations](observations/README.md) |

## Feature gaps

### G1. PDF engine: parse, render, write

- **Plan (2026-10-28):** reuse PDFCraft's crates ([architecture §15](architecture.md#15-reusing-pdfcraft),
  [decision D1](decisions.md#2026-10-28-d1-reuse-pdfcrafts-pdf-crates)). What is left is ours: the
  `scorecraft-pdf` adapter, tile caching and pre-rendering for turns, the pinned-revision setup with
  hayro patches, and a score corpus run against PDFCraft's renderer.
- **Missing:** the adapter. Through PDFCraft, `scorecraft-pdf` must parse real-world PDFs (xref streams, object
  streams, incremental updates, broken xref repair, encryption with empty user password), render
  vector and scanned pages (Flate, DCT, JBIG2, CCITT G4, JPX images; Type 1, TrueType, CFF and
  Type 3 fonts), read the outline and text layer, and write new PDFs (export with ink, manuscript
  paper, reports).
- **Rows:** READ-1, READ-14, NAV-5, LIB-6, EXP-2, PAGE-6, IMP-7.
- **Decided:** D1, reuse PDFCraft. Scanned-score quality (JBIG2, CCITT at 600 dpi) is checked on our
  corpus; render bugs go upstream to PDFCraft.
- **Impact:** nothing else works without it. Scanned scores (JBIG2/CCITT) are common on IMSLP.

### G2. Reader layout and page turns

- **Missing:** single and two-page layouts, half-page and alternating turns, full screen,
  zoom/pan, landscape page zoom, keep-awake, turn latency budget.
- **Rows:** READ-2..6, READ-9, READ-12, READ-15, TURN-1..3, TURN-8.
- **Open questions:** tap-zone proportions and half-page split behaviour (G17).
- **Impact:** the core experience. A turn must never wait on rendering.

### G4. Ink engine and markup tools

- **Missing:** pen, pressure, highlighter, whiteout, stroke and pixel erasers, text notes, stamps,
  layers, lasso, undo/redo, palm rejection.
- **Rows:** ANNO-1..16.
- **Human:** ink feel must be judged on real tablets with real styluses (Apple Pencil, Surface Pen,
  S Pen, Wacom).
- **Impact:** marking up parts is the second reason musicians use a reader.

### G5. Library store, organisation and search

- **Missing:** the on-disk library, folders, metadata, search, sort, bookmarks, Recently Deleted,
  verify/repair, thumbnails, performance at 10,000 scores.
- **Rows:** LIB-1, LIB-3..5, LIB-9..15.

### G7. Setlists, Score Shortcuts and Score Snips

- **Missing:** setlists with page ranges and tempos, continuous flow, shared setlist bundles;
  tappable repeat/coda jumps; snips of bars shown after a turn.
- **Rows:** LIST-1..7, NAV-1..4, PAGE-7, PAGE-8, TURN-7.
- **Note:** paid (Pro) in enScore; free in ScoreCraft.

### G8. Import, export, print and forScore migration

- **Missing:** file, folder and image import; import report; PDF export with ink as flattened
  content or real annotations; print; forScore `.4sc` import.
- **Rows:** LIB-1, LIB-2, IMP-3, IMP-5..7, EXP-1..4.
- **Human:** the `.4sc` analysis needs someone who owns forScore to make sample files, and a
  different person to implement ([clean-room §5](clean-room.md#5-interchange-formats-without-a-public-spec)).

### G12. Audio playback and practice tools

- **Missing:** recordings attached to scores, time-stretch, pitch-shift, loops, recorded turns;
  metronome with subdivisions and sounds; tuner; practice log, goals, report.
- **Rows:** PLAY-1..6, TURN-6, PRAC-1..11.

### G13. Scanning

- **Missing:** camera capture with edge detection and perspective correction; eSCL network scanners.
- **Rows:** IMP-1, IMP-2, PAGE-5, READ-8.
- **Human:** camera and scanner testing needs devices.

### G14. IMSLP and automatic metadata

- **Missing:** IMSLP search/import through the public API; title and composer detection.
- **Rows:** IMP-4, LIB-7, LIB-8.
- **Note:** enScore's detection uses its own online service. Ours must work on-device first and
  use public catalogues only with the user's consent ([clean-room §7](clean-room.md#7-online-services)).
  Matching enScore's hit rate may need an OCR model (decision D4).

## UI / UX gaps

### G16. E-ink, accessibility and multi-window

- **Missing:** e-ink mode, screen-reader labels, reduced motion, two scores side by side, multiple
  windows.
- **Rows:** PLAT-2, PLAT-4, READ-10, READ-11.

## Stability gaps

### G3. Stability and data safety

- **Missing:** never-crash lints on every crate, `panic_hunt`, fuzz targets for `pdf`, `fsc`,
  `midi`, the library loader and setlist bundles, atomic saves, crash recovery, the last-resort
  panic guard.
- **Impact:** a crash on stage or lost markings ends trust permanently.

## Hardware gaps

### G6. Foot pedals and keyboard turning

- **Missing:** key mapping for HID pedals (AirTurn, PageFlip, Donner and generic), learn mode,
  per-pedal profiles, debounce.
- **Rows:** TURN-3, TURN-4.
- **Human:** test with at least three real pedal models on each platform.

### G11. MIDI

- **Missing:** MIDI input for turns, output of Program Change, Bank Select and Song Select, device
  selection (USB, BLE MIDI, network).
- **Rows:** TURN-5, MIDI-1..4.

## Platform gaps

### G9. Mobile platforms

- **Missing:** Android and iOS/iPadOS shells, share-sheet import, Files integration, pen input,
  keep-awake, app store packaging.
- **Rows:** PLAT-1.
- **Human:** store accounts, signing, device testing. iOS support in egui/winit is the least mature
  target; decision D3.

### G10. Sync and backup

- **Missing:** op-log sync with CRDT merge; folder, WebDAV and self-hosted server backends; backup
  and restore; account-free operation.
- **Rows:** SYNC-1..7.

## Localization gaps

### G15. Localization

- **Missing:** every language but English. enScore lists 15 locales.
- **Rows:** PLAT-3. Detail in [`localization-parity.md`](localization-parity.md).
- **Human:** native-speaker review per language.

## Process gaps

### G17. Unknowns: behaviour not yet observed

- The feature map is built from public listings only. Exact gestures, defaults and edge cases need
  observation notes from contributors who use enScore, and enscore.app needs reading.
- Open questions are listed in [`observations/public-listings-2026-10-28.md`](observations/public-listings-2026-10-28.md#open-questions-need-observation-or-the-developer-site).

## Not clean-room feasible (so far)

| Item | Why | What we do instead |
|---|---|---|
| Talking to enScore's servers (sync, detection) | Undocumented private service; using it would mean traffic analysis | Our own sync and on-device detection |
| Writing `.4sc` files | No way to verify output without the other app's code; black-box writing risks corrupting a user's forScore library | Read only, until someone can verify written files in forScore |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | minor | G1 re-planned as PDFCraft integration (80–160 → 20–40 h) |
| 2026-10-28 | major | First version: G1–G17 from the feature map |
