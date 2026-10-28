# UI parity: reading, turning, gestures and feel

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, all rows missing) · **Target:** enScore 3.7 (2026)

How ScoreCraft's direct manipulation compares with enScore 3.7: layouts, turns, gestures, zoom,
library browsing and panels. Ink has its own checklist in [`annotation-parity.md`](annotation-parity.md);
pedals, MIDI and styluses are in [`hardware-parity.md`](hardware-parity.md). The design system is
[`ui-design.md`](ui-design.md).

Status: **done** (behaves as specified in `docs/spec/`, with a test), **partial** (exists, differs
or untested), **missing**. Evidence is a file under `crates/ui-egui/src/` unless another path is
given, a scorecard id, or an issue.

**Summary (estimated, weights = how often the gesture is used on a stand):** 0% ready.

## Reader

| Id | Target | Rows | Status | Evidence |
|---|---|---|---|---|
| UI-1 | Turn forward/back by tap zones with no visible delay (turn from cache ≤ 16 ms to first frame) | TURN-1, READ-1 | missing | |
| UI-2 | Swipe turns follow the finger and settle; a short flick turns | TURN-2 | missing | |
| UI-3 | Half-page turn: top/bottom halves update in the observed order, split position remembered per score | READ-5 | missing | |
| UI-4 | Alternating turns for awkward breaks | READ-6 | missing | |
| UI-5 | Single, two-page and full-screen layouts; spread pairing correct for scores with a cover page | READ-2..4 | missing | |
| UI-6 | Pinch zoom and pan within a page; double-tap to fit; landscape page zoom | READ-12, READ-15 | missing | |
| UI-7 | Toolbar shows on a tap in the top band and hides on the next turn | READ-4 | missing | |
| UI-8 | Bookmarks and outline jump list | LIB-11, NAV-5 | missing | |
| UI-9 | Go to page, first/last page | TURN-8 | missing | |
| UI-10 | Display filters (dark, sepia with intensity, inverted) with legible ink | READ-7 | missing | |
| UI-11 | Score Shortcut hot spots are visible in edit mode, invisible but tappable in performance | NAV-1 | missing | |
| UI-12 | Setlist position indicator and next-piece preview | LIST-4 | missing | |

## Library

| Id | Target | Rows | Status | Evidence |
|---|---|---|---|---|
| UI-20 | Thumbnail grid and list views, multi-select, drag to folder or list | LIB-14, LIB-3 | missing | |
| UI-21 | Search field with field filters (composer:, key:, tag:) and instant results | LIB-5 | missing | |
| UI-22 | Metadata editor with bulk edit | LIB-4 | missing | |
| UI-23 | Page editor: thumbnails to reorder, rotate, hide, delete, crop, straighten | PAGE-1..5 | missing | |
| UI-24 | Recently Deleted with restore and empty | LIB-10 | missing | |

## Keyboard shortcuts

| Id | Target | Status |
|---|---|---|
| UI-30 | Every reader action has a default shortcut and appears in the command palette | missing |
| UI-31 | Shortcuts are editable and exportable | missing |

## Windows and panels

| Id | Target | Rows | Status |
|---|---|---|---|
| UI-40 | Two scores side by side | READ-10 | missing |
| UI-41 | Multiple windows | READ-11 | missing |
| UI-42 | E-ink mode | PLAT-2 | missing |
| UI-43 | Light and dark app themes | PLAT-4 | missing |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
