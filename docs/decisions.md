# Implementation decisions

Dated records of choices that shape the code, and the open questions waiting for one. Decisions
that affect the process or several areas go through an [RFC](../rfcs/0000-decision-process.md);
this file records the outcome. Newest first.

## Open

| Id | Question | Options | Needed by |
|---|---|---|---|
| D2 | Record encoding on disk | JSON (readable, diffable) vs CBOR (smaller, faster) for score records; ink op logs are binary either way | M1 |
| D4 | OCR for title/composer detection | `pdfcraft-ocr` (ocrs, on-device; leading option, model licences to confirm before shipping) vs PDF metadata and text layer only | M9 |
| D5 | Device-to-device sync protocol | Wire format and pairing for Wi-Fi peers and the Google Drive relay (architecture §9): JSON ops vs a compact binary encoding; how optional sign-in links devices | M8 |

## 2026-10-28: Owner's answers to the M0 review

Brinsley answered the [M0 review](https://claude.ai/artifact/H2skGhp9UjwUSVmdJ6sMts)'s questions on
2026-10-28. Each is a decision:

- **Primary device: a Windows tablet.** Pen and touch on Windows are the first platform to get
  right (architecture §10, §17). Desktops follow, then Android tablets and iPad.
- **Web build: nice-to-have, medium priority** (not low). Every L0–L6 crate keeps checking for
  wasm32, and the browser reader ships after the desktop one.
- **D3 decided: a thin native Swift host on iPad for the pen.** It owns the Apple Pencil (pressure,
  tilt, hover, predicted touches) and hands samples to the Rust ink engine; no Swift UI logic.
- **D6 decided: the name is ScoreCraft.** ScoreCraft is independent of storytold but canonical with
  the Craft suite apps: same framework, conventions and look.
- **Rust 1.95, PDFCraft's version** (toolchain decision below). Take PDFCraft's constraints only
  where ScoreCraft needs them, with **no bloat** from PDFCraft features a score reader doesn't use.
- **ScoreCraft is a different app from PDFCraft.** It shares the framework and the implementation
  of what the two have in common. It has its own UI and does not use `craft-update`. A build holds
  exactly one egui: ScoreCraft's, built on `craft-ui` and distinct from PDFCraft's UI (architecture
  §15, §16; `cargo xtask layers` enforces it).
- **Sync has two parts** (architecture §9): (1) sync the whole library (files and folders), app
  configuration and annotations to the user's **Google Drive**; (2) sync between devices and
  platforms, with optional sign-in, over **Wi-Fi** and through Google Drive upload and download.
  **Download everything, offline first**: nobody waits on a slow network in a concert hall.
  Google Drive first; OneDrive and Dropbox later.
- **forScore `.4sc` import with the two-team split: yes**; the owner has sample files.
- **No tiers and no required account: confirmed.**
- **Priority after the PDF engine: page turns and the pedal.**
- **Everything is controllable by AI**: every part of the app, not only the engine's commands
  (shell state, dialogs, settings, sync) goes through commands an agent can call.

## 2026-10-28: Workspace toolchain and lint baseline (M0)

**Decision.** Edition 2024, resolver 3. `rust-version = "1.95"`, matching PDFCraft's and
craft-libs' `rust-version` (raised from 1.85 on 2026-10-28 at the owner's request: PDFCraft's crates
don't build on older Rust). The MSRV CI job checks it. The toolchain everyone builds with is pinned
in `rust-toolchain.toml` (1.99.1 on 2026-10-28), so a new stable release or clippy lint can't turn
`main` red on its own; bump it in its own PR, and raise `rust-version` only when PDFCraft or
craft-libs do. Never-crash lints live once in `[workspace.lints]` (`unsafe_code = "forbid"`; clippy `unwrap_used`,
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
user already has (Google Drive first) or devices on the same Wi-Fi, and signing in to link devices
is optional. Reason: the Crafting Apps are free and open source, and
an account requirement would contradict offline-first.

## 2026-10-28: Shared standards copied in

The Crafting Apps' shared standards live in a private repository. ScoreCraft keeps its own copies,
in its own words, in [`standards/`](standards/README.md), so outside contributors can read every
rule they are held to.
