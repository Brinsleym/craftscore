# ScoreCraft roadmap

**Stage: pre-alpha** · next: alpha, the [alpha gate](docs/roadmap.md#alpha-gate) (~220–490 h away)

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-31 · **Change:** patch (owner's answers on markup, 124 rows, breadth 38%; earlier: M4 part 1: the ink engine and markup commands, breadth 39%; earlier: M1 foundation and M2 PDF reading done; earlier: owner's priorities after the M0 review; earlier: M0 skeleton done; earlier: 4 features added from the Microsoft Store description; PDF backend reuses PDFCraft) · **Target:** enScore 3.7 (2026)

ScoreCraft's one-page summary of how close it is to enScore 3.7 (measured from public listings on
2026-10-28). Detail: [target-app parity](docs/target-app-parity.md) (methodology, weights,
evidence), [gaps](docs/gaps.md) (the ranked work list), [roadmap](docs/roadmap.md) (milestones,
current focus), [feature map](docs/feature-map.md) (all 124 features), [scorecard](docs/scorecard.md)
and [parity checklist](docs/parity-checklist.md) (generated once the engine exists),
[architecture](docs/architecture.md), [clean-room methodology](docs/clean-room.md).

Hours are agent wall-clock hours for one agent working sequentially, **uncalibrated** until this
repository has merged-PR history. About 75% of the work parallelizes.

## Headline numbers

| | Value | Kind |
|---|---|---|
| **Feature breadth** | **38%** | measured: 40 of 124 feature-map rows live (with M3's reader); markup in the engine only |
| **Ready for real work** | **0%** | estimated |
| Practising musician | 0% | estimated ([method](docs/target-app-parity.md#practising-musician)) |
| Essentials user | 0% | estimated ([method](docs/target-app-parity.md#essentials-user)) |
| Remaining to **alpha** | **~200–460 h** | estimated |
| Remaining to **beta** | **~650–1,340 h** | estimated |
| Remaining to **full parity** | **~883–1,796 h** | estimated |

**Why pre-alpha.** The project has its process, clean-room rules, architecture and feature map, and
the M0 skeleton, PDF reading over PDFCraft (M2) and the foundation (M1): the document model,
undo history and a crash-safe on-disk library, driven by about 35 engine commands (import,
folders, metadata, page edits, bookmarks, setlists, undo, verify). There is no UI yet, so
nothing is ready for real work; next is the reader app with page turns and the pedal (M3).

## By dimension

| Dimension | % ready | Hours to full | Doc |
|---|---:|---:|---|
| Features (depth) | 0% | 840–1,690 | [feature-map](docs/feature-map.md) |
| Page-turn reliability and latency | 0% | 60–120 | [ui-parity](docs/ui-parity.md) |
| Ink feel | 0% | 30–70 | [annotation-parity](docs/annotation-parity.md) |
| PDF fidelity (engine reused from [PDFCraft](https://github.com/storytold/pdfcraft)) | 0% | 20–40 | [file-format-parity](docs/file-format-parity.md) |
| Stability | 0% | 50–100 | [gaps G3](docs/gaps.md#g3-stability-and-data-safety) |
| Hardware (pedals, MIDI, stylus, e-ink) | 0% | 50–100 | [hardware-parity](docs/hardware-parity.md) |
| Platforms | 0% | 80–200 | [gaps G9](docs/gaps.md#g9-mobile-platforms) |
| Sync | 0% | 130–250 | [sync-parity](docs/sync-parity.md) |
| Practice and audio | 0% | 60–120 | [practice-parity](docs/practice-parity.md) |
| Localization | 0% | 40–80 | [localization-parity](docs/localization-parity.md) |

## Languages

enScore lists fifteen languages. ScoreCraft ships English first; the rest follow at M11
([localization-parity](docs/localization-parity.md)).

## Upcoming

| Rank | Milestone | Estimate |
|---|---|---:|
| 1 | M3 reader app, with page turns and the pedal | 66–122 h |
| 2 | M4 ink (part 1, the engine, done) | 60–120 h |
| 3 | M5 library: search, sort, thumbnails, the library UI | 40–80 h |

Detail: [docs/roadmap.md](docs/roadmap.md).

## Progress log

| Date | Entry |
|---|---|
| 2026-10-31 | M4 part 1: [ink spec](docs/spec/ink.md) from the owner's toolbar observation; `scorecraft-ink`, CPU ink with a multiply highlighter, palm filter, 16 markup commands (pen, highlighter, whiteout, erasers, swatches and palette, Draw with Touch, touch lock, ruler); ANNO-17 Ruler added; parity 40 of 123; then the owner's answers (chisel highlighter, highlighter colours, palm grace, screen ruler, text and stamps specified), 124 rows |
| 2026-10-31 | M1 done: `geom`, `raster`, `doc`, `ops` and `format` v1, about 35 library, page, list and undo commands; a save crashed at any write reopens intact; parity 17 of 122 |
| 2026-10-31 | M2 done: PDF reading over PDFCraft 0.15.0, page cache and pre-rendering, `score.open` and `view.render`; render gaps tracked in PDFCraft |
| 2026-10-28 | M0 review acted on: Rust 1.95 (PDFCraft's) with a pinned toolchain, one egui on craft-ui checked in CI, parity counts only implemented rows, CLI keeps partial results; owner's answers recorded (Windows tablet first, pedal in M3, Google Drive sync, Swift pen host on iPad) |
| 2026-10-28 | M0 done: generated scorecard (`cargo xtask scorecard`), testkit synthetic and hostile PDFs |
| 2026-10-28 | M0: engine command registry and `Session::dispatch`, generated parity checklist (measured 0 of 122), `panic_hunt` in CI, CLI `commands` and `run` |
| 2026-10-28 | M0 started: Cargo workspace with all 29 crates and 6 apps as stubs, never-crash lints on every crate, `cargo xtask layers` and `cargo xtask wasm` green |
| 2026-10-28 | Feature map reconciled with the Microsoft Store description: 4 features added (per-setlist MIDI, recording at score open, two-way folder mirror, Apple Vision), Score Snips corrected |
| 2026-10-28 | PDF backend to reuse PDFCraft's crates (decision D1); 13 features marked as reuse |
| 2026-10-28 | Documentation and process: clean-room rules, feature map (118 rows), architecture, parity docs, RFC process |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-31 | patch | Owner's answers on markup: ANNO-18 ruler angle added (124 rows), breadth 39% → 38% |
| 2026-10-31 | minor | M4 part 1 (on M3): breadth 14% → 39% (40 of 123 rows); alpha ~220–490 → ~200–460 h, beta and full parity lower; ink feel 40–80 → 30–70 h |
| 2026-10-31 | minor | M1 and M2 done: breadth 0 → 14%, alpha ~250–530 → ~220–490 h, beta and full parity 40 h lower, stability 60–120 → 50–100 h |
| 2026-10-28 | minor | Owner's priorities: pedal mapping moves into M3; headline numbers unchanged |
| 2026-10-28 | minor | M0 done: remaining estimates drop by the M0 skeleton (60–100 h): alpha ~250–530 h, beta ~710–1,430 h, full ~940–1,880 h; upcoming starts at M2 |
| 2026-10-28 | minor | M0 started (workspace skeleton); numbers unchanged |
| 2026-10-28 | minor | Microsoft Store description: 122 rows; beta and full estimates up slightly |
| 2026-10-28 | minor | PDFCraft reuse: estimates lowered |
| 2026-10-28 | major | First version, per [`docs/standards/progress-docs.md`](docs/standards/progress-docs.md) |
