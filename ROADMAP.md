# ScoreCraft roadmap

**Stage: pre-alpha** · next: alpha, the [alpha gate](docs/roadmap.md#alpha-gate) (~310–630 h away)

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** minor (4 features added from the Microsoft Store description; earlier: PDF backend reuses PDFCraft; first version) · **Target:** enScore 3.7 (2026)

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
| Remaining to **alpha** | **~310–630 h** | estimated |
| Remaining to **beta** | **~770–1,530 h** | estimated |
| Remaining to **full parity** | **~1,000–1,980 h** | estimated |

**Why pre-alpha.** The project has its process, clean-room rules, architecture and feature map, and
no code. The first milestone is the workspace skeleton (M0).

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
| 1 | M0 skeleton and CI | 60–100 h |
| 2 | M2 PDF integration: adapter over PDFCraft's crates | 20–40 h |
| 3 | M1 foundation | 40–80 h |
| 4 | M3 reader app | 60–110 h |
| 5 | M4 ink | 80–150 h |

Detail: [docs/roadmap.md](docs/roadmap.md).

## Progress log

| Date | Entry |
|---|---|
| 2026-10-28 | Feature map reconciled with the Microsoft Store description: 4 features added (per-setlist MIDI, recording at score open, two-way folder mirror, Apple Vision), Score Snips corrected |
| 2026-10-28 | PDF backend to reuse PDFCraft's crates (decision D1); 13 features marked as reuse |
| 2026-10-28 | Documentation and process: clean-room rules, feature map (118 rows), architecture, parity docs, RFC process |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | minor | Microsoft Store description: 122 rows; beta and full estimates up slightly |
| 2026-10-28 | minor | PDFCraft reuse: estimates lowered |
| 2026-10-28 | major | First version, per [`docs/standards/progress-docs.md`](docs/standards/progress-docs.md) |
