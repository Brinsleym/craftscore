# Implementation decisions

Dated records of choices that shape the code, and the open questions waiting for one. Decisions
that affect the process or several areas go through an [RFC](../rfcs/0000-decision-process.md);
this file records the outcome. Newest first.

## Open

| Id | Question | Options | Needed by |
|---|---|---|---|
| D2 | Record encoding on disk | JSON (readable, diffable) vs CBOR (smaller, faster) for score records; ink op logs are binary either way | M1 |
| D3 | iOS shell | winit + wgpu + egui directly vs a thin Swift host around a Rust static library (no Swift UI logic) | M7 |
| D4 | OCR for title/composer detection | `pdfcraft-ocr` (ocrs, on-device; leading option, model licences to confirm before shipping) vs PDF metadata and text layer only | M9 |
| D5 | Sync server protocol | HTTP + JSON ops vs a compact binary protocol over WebSocket | M8 |
| D6 | Product name | "ScoreCraft" (working name, following the `{Function}Craft` convention) or another name; must not include "enScore" | before the first release |

## 2026-10-28: Workspace toolchain and lint baseline (M0)

**Decision.** Edition 2024, `rust-version = "1.85"` (the first release with edition 2024), resolver
3. Never-crash lints live once in `[workspace.lints]` (`unsafe_code = "forbid"`; clippy `unwrap_used`,
`expect_used`, `panic`, `unimplemented`, `todo`, `unreachable` deny, `indexing_slicing` warn, which
CI's `-D warnings` makes an error) and every crate root repeats the `#![deny(...)]`. Cargo cannot
mix `[lints] workspace = true` with a per-crate override, so `scorecraft-devices` copies the table
with `unsafe_code = "deny"`; `cargo xtask layers` fails if the copy drifts. Reason: one place to
tighten the rules, and a checker that catches a crate that opts out.

## 2026-10-28: D1 reuse PDFCraft's PDF crates

**Decision.** ScoreCraft does not write its own PDF engine. `scorecraft-pdf` is an adapter over
[storytold/pdfcraft](https://github.com/storytold/pdfcraft)'s crates (`geom`, `filters`, `crypt`,
`cos`, `model`, `content`, `fonts`, `render`, `annot`, `organize`, `create`), with `pdfcraft-print`
in `io` and `pdfcraft-ocr` in `metadata` and `scan`. Requested by the project owner on 2026-10-28
("reuse as much of the PDF related content from pdfcraft as possible in the backend").

**Licence check.** PDFCraft is MIT OR Apache-2.0 (its `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`
and every crate's `license.workspace`), identical to ScoreCraft's, so it is compatible for
dependency, modification and redistribution. We carry its copyright notice and its third-party
notices (hayro, Foxit standard-14 fonts and Adobe CMaps under BSD-3-Clause, lopdf, the CC0 ICC
profile) in `NOTICE`. Its ArtCraft brand files are excluded.

**How.** Git dependencies pinned to one PDFCraft revision, plus the same `[patch.crates-io]`
entries for its patched hayro family, because the crates are not on crates.io and Cargo patches
apply only at the workspace root. Vendoring is the fallback, per crate, if we need a patch PDFCraft
won't take. Detail: [`architecture.md` §15](architecture.md#15-reusing-pdfcraft).

**Effect.** M2 drops from 80–160 h of new engine code to 20–40 h of integration; 13 feature-map
rows are marked ♻ and their estimates fall from ~156–292 h to ~69–134 h.

**Risks.** PDFCraft is pre-1.0 (0.14.0) and its APIs move: the adapter hides them, and revision
bumps are separate PRs with corpus runs. Its renderer is labelled "bootstrap: hayro" and may change;
that change happens behind `pdfcraft-render` and our adapter.

## 2026-10-28: Stricter clean-room than PhotoCraft

PhotoCraft measured Photoshop partly from its installed bundle. ScoreCraft does not open any
installed enScore or forScore package at all, and formats without a public spec need a two-team
split. Reason: a score reader's value is mostly behaviour, which public listings and use as a user
describe well enough, and the extra distance removes any doubt about provenance. See
[`clean-room.md`](clean-room.md).

## 2026-10-28: No tiers, no required account

Every enScore Pro and Sync capability is free in ScoreCraft and works offline; sync uses storage the
user already has or a self-hosted server. Reason: the Crafting Apps are free and open source, and
an account requirement would contradict offline-first.

## 2026-10-28: Shared standards copied in

The Crafting Apps' shared standards live in a private repository. ScoreCraft keeps its own copies,
in its own words, in [`standards/`](standards/README.md), so outside contributors can read every
rule they are held to.
