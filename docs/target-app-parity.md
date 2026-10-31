# ScoreCraft vs enScore: target-app parity

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-31 · **Change:** patch (ANNO-18 added, breadth 38%, 40 of 124 rows; earlier: M4 part 1, with M3 below it: breadth 39% measured, 40 of 123 rows, ANNO-17 added; alpha ~200–460 h; earlier: M1 foundation done: breadth 14% measured, alpha ~220–490 h; earlier: M0 skeleton done, remaining hours lowered; earlier: 4 rows added from the Microsoft Store description; estimates lowered for PDFCraft reuse) · **Target:** enScore 3.7 (2026)

The authoritative answer to "how close is ScoreCraft to enScore, and how much work is left". The
one-page summary is [`ROADMAP.md`](../ROADMAP.md); the ranked work list is [`gaps.md`](gaps.md);
the feature catalog is [`feature-map.md`](feature-map.md); the measured checklists will be
[`scorecard.md`](scorecard.md) (`cargo xtask scorecard`) and [`parity-checklist.md`](parity-checklist.md)
(`cargo xtask parity`). Detail by dimension: [UI](ui-parity.md), [annotation](annotation-parity.md),
[file formats](file-format-parity.md), [hardware](hardware-parity.md),
[practice and audio](practice-parity.md), [sync](sync-parity.md),
[localization](localization-parity.md).

## Headline (2026-10-31, engine, library store, the M3 reader window; no pen in the app yet)

| Number | Value | Kind |
|---|---|---|
| **Feature breadth** (does each enScore feature exist?) | **38%** weighted (40 of 124 feature-map rows live, all `partial`: engine commands with tests; markup has no UI yet) | measured by `cargo xtask parity` |
| **Ready for real work** (could a musician replace enScore with it on stage?) | **0%** | estimated, weights below |
| **Practising musician** (reads, marks, turns with a pedal, uses setlists) | 0% | estimated, [method](#practising-musician) |
| **Essentials user** (open a PDF, turn pages, mark it up) | 0% | estimated, [method](#essentials-user) |
| **Stage** | **pre-alpha** | the [alpha gate](roadmap.md#alpha-gate) does not pass |
| Remaining to **alpha** | **~200–460 agent-hours** | estimated, uncalibrated: P0 rows (200–460 h, after PDFCraft reuse); M0 (2026-10-28), M2 and M1 (2026-10-31) are done; M4 part 1 (the ink engine) took 20–30 h off; M3's own reduction is recorded with M3 |
| Remaining to **beta** | **~650–1,340 agent-hours** | estimated, uncalibrated: alpha + P1 rows (350–680 h) + stability and performance (100–200 h) |
| Remaining to **full parity** | **~883–1,796 agent-hours** | estimated, uncalibrated: beta + P2 rows (233–456 h, with the ruler); P3 (ours only) adds 80–170 h |

"Agent-hours" are wall-clock hours for one agent working sequentially, including tests, as in
PhotoCraft's docs. They are **uncalibrated** until this repository has merged-PR history
([below](#remaining-effort-and-calibration)). About 75% of the work parallelizes: most feature
rows are independent command modules; `pdf` rendering, the library store and sync are the serial
parts.

## Target measured

- **enScore 3.7** on Apple platforms (3.7.0 per a public app directory, 3.5.9 per the App Store's
  version history), **3.5.15** on Android, Microsoft Store build **2026.10.21.305** on Windows,
  measured on 2026-10-28.
- **From public sources only** ([clean-room](clean-room.md#2-allowed-sources)): the store
  listings (including the full Microsoft Store description, S8), release notes and partner pages summarised in
  [`observations/public-listings-2026-10-28.md`](observations/public-listings-2026-10-28.md). Unlike
  PhotoCraft's measurement of Photoshop, **no installed package was inspected**, so items enScore
  ships without describing publicly are missing from the denominator until someone observes them
  as a user. Rows tagged `inferred` in the feature map are capabilities any reader needs (undo,
  zoom) that the listings imply but do not name.
- enscore.app was not reachable from our build environment on 2026-10-28; reading it is the first
  re-measure task.

## Feature breadth: 38% (weighted, 2026-10-31)

| Component | Weight | Ours / enScore | % | Kind |
|---|---:|---|---:|---|
| Reading, layout and page turning (READ, TURN) | 25% | 14 / 23 | 61% | measured |
| Annotation (ANNO) | 20% | 9 / 18 | 50% | measured |
| Library, page editing, navigation (LIB, PAGE, NAV) | 15% | 12 / 28 | 43% | measured |
| Setlists (LIST) | 10% | 4 / 7 | 57% | measured |
| Import, export, scanning, IMSLP, forScore (IMP, EXP) | 10% | 0 / 12 | 0% | measured |
| Practice, audio, MIDI (PRAC, PLAY, MIDI) | 10% | 0 / 23 | 0% | measured |
| Sync and backup (SYNC) | 5% | 0 / 8 | 0% | measured |
| Platforms and languages (PLAT) | 5% | 1 / 5 | 20% | measured |
| **Weighted** | 100% | | **38%** | |

Measured on the M4 part 1 branch, which sits on the M3 reader (READ, TURN and PLAT rows come
from M3). Annotation rows are `partial`: the engine draws, erases and stores ink with tests, but
no pen draws in the app until M4 part 4.

Weights reflect what a performing musician relies on most; they are set here once and only moved by
RFC.

## Ready for real work: 0% (weighted)

| Dimension | Weight | % ready | Remaining (agent-h) | Evidence | Detail |
|---|---:|---:|---:|---|---|
| Features in depth | 25% | 0% | 840–1,690 | no commands | [feature-map](feature-map.md) |
| Page-turn reliability and latency | 15% | 0% | 60–120 | no renderer; budgets defined in [scorecard](scorecard.md#performance) | [ui-parity](ui-parity.md) |
| Ink feel (latency, pressure, smoothing) | 10% | 0% | 30–70 | ink engine with pressure curve and corner-keeping smoothing (M4 part 1); no pen in the app to judge on a device | [annotation-parity](annotation-parity.md) |
| PDF fidelity (renders what other readers render) | 15% | 0% | 20–40 | no adapter yet; engine reused from PDFCraft | [file-format-parity](file-format-parity.md) |
| Stability (never crash, never lose markings) | 10% | 0% | 60–120 | no code | [gaps G3](gaps.md#g3-stability-and-data-safety) |
| Hardware (pedals, MIDI, stylus, e-ink) | 10% | 0% | 50–100 | none | [hardware-parity](hardware-parity.md) |
| Platforms (desktop, Android, iOS, web) | 10% | 0% | 80–200 | none | [gaps G9](gaps.md#g9-mobile-platforms) |
| Localization | 5% | 0% | 40–80 | English only planned at alpha | [localization-parity](localization-parity.md) |

## Practising musician

Weekly areas for a typical gigging or studying musician: open and read PDFs (READ-1..5), pedal turns
(TURN-4), pen and highlighter (ANNO-1..3, 6, 8), library search (LIB-3..5), setlists (LIST-1..4),
metronome (PRAC-1), export with ink (EXP-2). Ready % = the share of those rows `done`, discounted
by 25% for any open stability gap in the area. Today 0%.

## Essentials user

Open a PDF, turn pages by tap and keyboard, pen and eraser, save and reopen with markings intact.
Ready % = share of READ-1, READ-2, TURN-1, TURN-3, ANNO-1, ANNO-6, ANNO-13, ANNO-15 `done`. Today 0%.

## Feature areas

| Area | % ready | Hours | Notes |
|---|---:|---:|---|
| PDF engine (parse, render, write) | 0% | 20–40 | reused from PDFCraft (decision D1); the adapter and turn caching are ours |
| Reader layout and turns | 0% | 40–80 | half-page and alternating turns are the signature feature |
| Ink engine and markup tools | 0% | 80–150 | latency and pressure feel need real devices |
| Library store, search, metadata | 0% | 60–120 | must scale to 10,000 scores |
| Setlists, shortcuts, snips | 0% | 40–80 | Pro features in enScore, free here |
| Import / export / print | 0% | 40–80 | |
| forScore `.4sc` import | 0% | 30–60 | black-box format, two-team rule |
| Scanning (camera, eSCL) | 0% | 36–64 | |
| IMSLP | 0% | 12–24 | public API, terms |
| Audio playback and practice tools | 0% | 60–120 | time-stretch, pitch-shift, tuner |
| MIDI | 0% | 20–40 | |
| Sync and backup | 0% | 120–240 | offline-first CRDT plus three backends |
| Platforms and shells | 0% | 80–200 | iOS is the least mature egui target |
| Localization | 0% | 40–80 | |

## Remaining effort and calibration

**Calibration.** There is no history yet. The per-row estimates in the feature map use PhotoCraft's
published calibration (2–9 agent-hours per checklist item of tool or UI depth, 8–20 per format or
subsystem, from its first 1,004 merged PRs) as a prior. After about 50 merged PRs here, replace
the prior with this repository's own numbers (median PR hours by size, as PhotoCraft's
calibration table does) and re-estimate every row.

**Totals.** Sum of feature-map estimates after PDFCraft reuse: P0 250–530, P1 360–700, P2 230–450, P3 80–170
(agent-hours). Infrastructure not in the rows: M0 skeleton and xtask 60–100, stability and
performance hardening before beta 100–200.

## How to re-measure

1. `cargo xtask parity` and `cargo xtask scorecard` for the measured rows (once they exist).
2. Re-read every public source in the observation note; add rows for new capabilities (new
   release notes), with their source, and record the enScore version.
3. Observation passes by contributors who use enScore: fill the open questions in
   [`observations/public-listings-2026-10-28.md`](observations/public-listings-2026-10-28.md#open-questions-need-observation-or-the-developer-site).
4. Re-grade the estimated rows from open issues by area and corpus floors. Move a number only with
   new evidence, and add a revision-history row.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-31 | patch | ANNO-18 ruler angle added from the owner's answers (124 rows): breadth 39% → 38% |
| 2026-10-31 | minor | M4 part 1 (on M3): breadth 14% → 39% (40 of 123 rows; ANNO-17 Ruler added from the owner's observation); alpha ~220–490 → ~200–460 h, beta ~670–1,370 → ~650–1,340 h, full ~900–1,820 → ~883–1,796 h; ink feel 40–80 → 30–70 h |
| 2026-10-31 | minor | M1 done (and M2 earlier the same day): breadth 0 → 14% (17 of 122 rows live, engine only); alpha ~250–530 → ~220–490 h, beta ~710–1,430 → ~670–1,370 h, full ~940–1,880 → ~900–1,820 h |
| 2026-10-28 | minor | M0 skeleton done (PRs #5–#8): alpha ~310–630 → ~250–530 h, beta ~770–1,530 → ~710–1,430 h, full ~1,000–1,980 → ~940–1,880 h |
| 2026-10-28 | minor | Microsoft Store description (S8): 118 → 122 rows; beta ~760–1,520 → ~770–1,530 h, full ~980–1,960 → ~1,000–1,980 h; alpha unchanged |
| 2026-10-28 | minor | PDFCraft reuse: alpha ~350–720 → ~310–630 h, beta ~830–1,650 → ~760–1,520 h, full ~1,070–2,120 → ~980–1,960 h |
| 2026-10-28 | major | First assessment: 0 of 118 rows, pre-alpha, estimates from the feature map |
