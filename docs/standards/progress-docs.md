# Progress docs

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version) · **Target:** enScore 3.7 (2026)

Progress docs say where ScoreCraft stands against enScore and what is left. They are:
[`ROADMAP.md`](../../ROADMAP.md), [`target-app-parity.md`](../target-app-parity.md),
[`gaps.md`](../gaps.md), [`roadmap.md`](../roadmap.md), [`feature-map.md`](../feature-map.md),
every `docs/*-parity.md`, and the specs in [`spec/`](../spec/README.md). Generated docs
([`scorecard.md`](../scorecard.md), [`parity-checklist.md`](../parity-checklist.md)) are not edited
by hand.

## The status header

The first line after the title is a quote block:

```markdown
> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** minor (what changed) · **Target:** enScore 3.7 (2026)
```

- **Last reviewed:** the last date someone checked the *whole* doc against `main`.
- **Last updated:** the last date anything in it changed.
- **Change:** `major` (re-measure, restructure, numbers moved) or `minor` (a row, evidence, a
  correction), with a short parenthesis. Keep the newest few changes; older ones live in the
  revision history.
- **Target:** the enScore version measured, and the year.

## Rules

1. **Numbers have a kind.** Every percentage or count is labelled *measured* (from a tool or a
   test, with the command) or *estimated* (with its method). Never present an estimate as
   measured.
2. **Move a number only with new evidence**, and add a revision-history row that says what moved
   and why.
3. **Update in the same PR.** A change that lands a feature, closes a gap or moves a number updates
   the feature-map row, the gap entry, the parity doc and the one-page `ROADMAP.md` together.
4. **Be honest.** Breadth (does it exist?) and readiness (could a musician rely on it?) are
   separate numbers. Wiring a command is not behaving like enScore.
5. **Hours** are agent wall-clock hours for one agent working sequentially, including tests.
   Until the repository has merged-PR history they are labelled *uncalibrated*; after about 50
   merged PRs, calibrate them as PhotoCraft does (median PR hours by feature size).
6. **Revision history** is the last section: `| Date | Change | Summary |`, newest first.
7. **Dates** are absolute (`2026-10-28`), never "yesterday".

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version, from the Crafting Apps' progress-docs standard |
