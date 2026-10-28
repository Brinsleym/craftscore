# Contributing

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version) · **Target:** enScore 3.7 (2026)

Thanks for helping. Start with [`AGENTS.md`](../AGENTS.md) (it applies to humans too), then
[`clean-room.md`](clean-room.md). Project-wide decisions and changes to this process go through
[RFCs](../rfcs/0000-decision-process.md).

## Rules

- **Language:** Rust only. No JavaScript or TypeScript. On the web, `wasm-bindgen` generates a
  small loader; never hand-write JS.
- **Licence:** contributions are MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`). By
  opening a PR you agree your contribution is licensed that way.
- **Clean-room:** follow [`clean-room.md`](clean-room.md). Do not copy code, assets, sounds, icons,
  fonts or text from enScore, forScore or any proprietary software, and never inspect their
  packages or traffic. Match behaviour from public information and observation as a user. Name
  your sources in the PR. Only use permissively licensed assets, keep their licence file next to
  the asset, and list every asset in [`ATTRIBUTION.md`](../ATTRIBUTION.md).
- **Specs before code:** a feature without a behavioural spec in [`spec/`](spec/README.md) gets one
  first, written from [observation notes](observations/README.md) and public standards.
- **Never crash:** non-test code must not panic and must not use `unsafe` (outside
  `scorecraft-devices`). Errors go through `Result` and `?`; input-derived indices and sizes are
  checked; every crash fix ships with a regression test. See [`standards/never-crash.md`](standards/never-crash.md).
- **Commands, not handlers:** new features are engine commands with tests, and the UI calls them
  (checklist below).
- **Layering:** `cargo xtask layers` must pass. Register new crates in `xtask/src/layers.rs`.
- **Tests:** required for every change. Format code needs round-trip and malformed-input tests.
  Rendering code is tested CPU against GPU. If you touch `pdf`, `fsc`, `io`, `render`, `gpu` or
  `format`, also run `cargo xtask test-corpus`. Never commit corpus files.
- **Style:** `cargo fmt`, and `cargo clippy -- -D warnings`. Match surrounding code. Comments
  explain *why*.
- **UI:** use `theme::Tokens` and `widgets::*`. Verify visually (offscreen `snapshot` example or the
  control channel) and attach before/after screenshots, rendered on public-domain scores, to PRs.
- **Commits:** small, focused, with a clear subject line. Prefix with the feature-map id when there
  is one: `READ-5: half-page turns in single-page layout`.
- **Progress docs:** when a change moves a number or closes a gap, update the parity doc, gap entry
  and feature-map row in the same PR, with the date and a revision-history row
  ([`standards/progress-docs.md`](standards/progress-docs.md)).

## Adding a command

1. **Find the id.** Look up the feature in [`feature-map.md`](feature-map.md) and use its command id.
   If the feature is missing, add a row first (with its public source). Commands without an enScore
   equivalent use a descriptive id in the same style and are tagged `ours`.
2. **Spec.** Make sure `docs/spec/<area>.md` covers it, with acceptance tests.
3. **Algorithm** goes in the lowest crate that fits (`ink`, `layout`, `algo`, `audio`, `practice`,
   `pdf`), with unit tests. It is deterministic (seeded randomness only) and works on any page
   size.
4. **Command** goes in an engine module (`crates/engine/src/<area>_cmds.rs`) exposing `specs()`,
   registered with `v.extend(...)` in `commands.rs`. Fill in:
   - `id`, `label`, `menu` path and default `shortcut`,
   - a params doc string such as `{"mode":"single|twoPage"="single"}` (what agents read through
     `commands` / `command_list`),
   - an `enabled` predicate (greys the menu item out),
   - `run`, which returns a JSON result and records exactly **one** history step (use `coalesce`
     for strokes and drags).
5. **Respect the context:** the active layer and its lock, hidden pages, page ranges, the current
   setlist entry, rotation and crop.
6. **Tests** in the module: behaviour, undo/redo, disabled states, bad params (→ `Err`, never a
   panic), rotated and cropped pages.
7. **UI** (if any): wire the control, tap zone or dialog to the command id; dialogs come from the
   params schema.
8. Run `cargo xtask parity` and commit the updated [`parity-checklist.md`](parity-checklist.md);
   flip the feature-map row's status.

## Adding an asset

1. Check the licence is MIT, Apache-2.0, BSD, ISC, zlib, CC0, CC BY or SIL OFL.
2. Put the licence file next to the asset (`assets/fonts/OFL-Bravura.txt`).
3. Add an `ATTRIBUTION.md` row (path, title, author, source, licence) in the same change.
4. Metronome sounds, stamps and icons are original or permissively licensed, never sampled from
   another app.

## Pull requests

Use the [PR template](../.github/pull_request_template.md). A PR is ready when CI is green, the
clean-room checklist is ticked, docs and parity rows are updated, and a maintainer other than the
author approves.
