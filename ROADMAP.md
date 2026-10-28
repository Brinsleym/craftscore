# ScoreCraft roadmap

**Stage: pre-alpha** · next: alpha, the [alpha gate](docs/roadmap.md#alpha-gate) (~250–530 h away)

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** minor (M0 skeleton done; earlier: 4 features added from the Microsoft Store description; PDF backend reuses PDFCraft) · **Target:** enScore 3.7 (2026)

ScoreCraft's one-page summary of how close it is to enScore 3.7 (measured from public listings on
2026-10-28). Detail: [target-app parity](docs/target-app-parity.md) (methodology, weights,
evidence), [gaps](docs/gaps.md) (the ranked work list), [roadmap](docs/roadmap.md) (milestones,
current focus), [feature map](docs/feature-map.md) (all 122 features), [scorecard](docs/scorecard.md)
and [parity checklist](docs/parity-checklist.md) (generated once the engine exists),
[architecture](docs/architecture.md), [clean-room methodology](docs/clean-room.md).

Hours are agent wall-clock hours for one agent working sequentially, **uncalibrated** until this
repository has merged-PR history. About 75% of the work parallelizes.

## Headline numbers

| | Value | Kind |
|---|---|---|
| **Feature breadth** | **0%** | measured: 0 of 122 feature-map rows |
| **Ready for real work** | **0%** | estimated |
| Practising musician | 0% | estimated ([method](docs/target-app-parity.md#practising-musician)) |
| Essentials user | 0% | estimated ([method](docs/target-app-parity.md#essentials-user)) |
| Remaining to **alpha** | **~250–530 h** | estimated |
| Remaining to **beta** | **~710–1,430 h** | estimated |
| Remaining to **full parity** | **~940–1,880 h** | estimated |

**Why pre-alpha.** The project has its process, clean-room rules, architecture and feature map, and
the M0 skeleton: a Cargo workspace with every crate, CI on three OSes plus wasm, the engine's
command registry, and generated parity and scorecard docs. No feature code yet; next is the PDF
adapter over PDFCraft (M2) and the foundation crates (M1).

## By dimension

| Dimension | % ready | Hours to full | Doc |
|---|---:|---:|---|
| Features (depth) | 0% | 840–1,690 | [feature-map](docs/feature-map.md) |
| Page-turn reliability and latency | 0% | 60–120 | [ui-parity](docs/ui-parity.md) |
| Ink feel | 0% | 40–80 | [annotation-parity](docs/annotation-parity.md) |
| PDF fidelity (engine reused from [PDFCraft](https://github.com/storytold/pdfcraft)) | 0% | 20–40 | [file-format-parity](docs/file-format-parity.md) |
| Stability | 0% | 60–120 | [gaps G3](docs/gaps.md#g3-stability-and-data-safety) |
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
| 1 | M2 PDF integration: adapter over PDFCraft's crates | 20–40 h |
| 2 | M1 foundation | 40–80 h |
| 3 | M3 reader app | 60–110 h |
| 4 | M4 ink | 80–150 h |

Detail: [docs/roadmap.md](docs/roadmap.md).

## Progress log

| Date | Entry |
|---|---|
| 2026-10-28 | M0 done: generated scorecard (`cargo xtask scorecard`), testkit synthetic and hostile PDFs |
| 2026-10-28 | M0: engine command registry and `Session::dispatch`, generated parity checklist (measured 0 of 122), `panic_hunt` in CI, CLI `commands` and `run` |
| 2026-10-28 | M0 started: Cargo workspace with all 29 crates and 6 apps as stubs, never-crash lints on every crate, `cargo xtask layers` and `cargo xtask wasm` green |
| 2026-10-28 | Feature map reconciled with the Microsoft Store description: 4 features added (per-setlist MIDI, recording at score open, two-way folder mirror, Apple Vision), Score Snips corrected |
| 2026-10-28 | PDF backend to reuse PDFCraft's crates (decision D1); 13 features marked as reuse |
| 2026-10-28 | Documentation and process: clean-room rules, feature map (118 rows), architecture, parity docs, RFC process |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | minor | M0 done: remaining estimates drop by the M0 skeleton (60–100 h): alpha ~250–530 h, beta ~710–1,430 h, full ~940–1,880 h; upcoming starts at M2 |
| 2026-10-28 | minor | M0 started (workspace skeleton); numbers unchanged |
| 2026-10-28 | minor | Microsoft Store description: 122 rows; beta and full estimates up slightly |
| 2026-10-28 | minor | PDFCraft reuse: estimates lowered |
| 2026-10-28 | major | First version, per [`docs/standards/progress-docs.md`](docs/standards/progress-docs.md) |
