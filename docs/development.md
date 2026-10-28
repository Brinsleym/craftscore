# Development guide

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, written ahead of milestone M0) · **Target:** enScore 3.7 (2026)

How to build, test, run and drive ScoreCraft. Commands marked _(M0)_ arrive with the workspace
skeleton ([`roadmap.md`](roadmap.md#milestones)); until then this page is the contract M0 must meet.

## Prerequisites

- Rust stable (edition 2024; the minimum version is pinned in `Cargo.toml` `rust-version`) with
  `rustfmt` and `clippy`.
- For the web build: `rustup target add wasm32-unknown-unknown`, `cargo install trunk`.
- For Android: the Android SDK and NDK, `cargo install cargo-apk` (or `xbuild`), and
  `rustup target add aarch64-linux-android`.
- For iOS: Xcode, `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`.
- Linux desktop: the usual winit/wgpu system libraries (Wayland/X11, Vulkan or GL drivers), ALSA
  headers only if you enable the `alsa-midi` feature of `scorecraft-devices`.
- Optional: `cargo install cargo-nextest cargo-fuzz mdbook`.

## Build and run _(M0)_

```sh
cargo run --release -p scorecraft -- score.pdf      # the desktop app
cargo run -p scorecraft-cli -- --help               # the headless CLI
cargo test --workspace                              # the test suite
cargo xtask ci                                      # what CI runs on Linux
(cd apps/scorecraft-web && trunk serve)             # the browser build
```

### Repository tasks

| Task | What it checks |
|---|---|
| `cargo xtask ci` | what the Linux CI jobs run: `fmt --check`, clippy with `-D warnings`, `test --workspace` (plus doctests), `panic_hunt`, `layers`, `parity --check`, `scorecard --check`, `wasm` |
| `cargo xtask layers` | every package is registered in `xtask/src/layers.rs`, depends only on lower layers, uses the workspace lints and carries the never-crash `#![deny(...)]` at its root; PDFCraft and UI-toolkit allow-lists |
| `cargo xtask parity` | regenerates `crates/engine/src/feature_catalog.rs` and [`parity-checklist.md`](parity-checklist.md) from the feature map and the registry; fails below `FLOOR`; `--check` fails if either is stale |
| `cargo xtask scorecard` | regenerates [`scorecard.md`](scorecard.md) from `perf/budgets.toml`, `scorecard/*.toml`, the parity docs' status columns and measured counts; `--check` fails if stale |
| `cargo xtask wasm` | every L0–L6 crate checks for `wasm32-unknown-unknown` |

Synthetic test PDFs come from `scorecraft_testkit::pdf` (`PdfBuilder` for valid scores of any
size, page count and rotation; `pdf::hostile::all()` for malformed files). To look at one in
another reader: `cargo run -p scorecraft-testkit --example write_sample -- out.pdf`.

### Faster test loops

- `cargo test -p <crate> --lib --tests` skips examples and doctests.
- `cargo test -p <crate> --test <file>` runs one integration-test file.
- `cargo nextest run` runs test binaries in parallel (`.config/nextest.toml`).
- Use your own target dir per agent: `CARGO_TARGET_DIR=target/agent-<name>`.

## Fonts

Fonts are never committed ([`standards/fonts.md`](standards/fonts.md)). To build with the music
symbol and UI fonts:

```sh
git clone <fonts repository> ../scorecraft-fonts
SCORECRAFT_FONTS_DIR="$PWD/../scorecraft-fonts" cargo run --release -p scorecraft
```

Without it, ScoreCraft uses system fonts and a built-in fallback for stamps.

## Driving the app programmatically

Every action is a command, so you can drive ScoreCraft without touching the UI:

```sh
# Headless: import, tag, export with ink
scorecraft-cli import ./scans --library ~/Scores.scorecraft --detect-metadata
scorecraft-cli run ~/Scores.scorecraft \
  --cmd score.metadata.set --params '{"score":"Bach BWV 1007","composer":"J. S. Bach","key":"G major"}' \
  --cmd score.export.pdf   --params '{"score":"Bach BWV 1007","layers":["Performance"],"out":"bwv1007.pdf"}'

# Every command and its params
scorecraft-cli commands --json

# Let an agent drive it over MCP (headless, or bridged to the running app)
scorecraft-cli mcp
```

The desktop app's control channel (`scorecraft --control`) reads UI state, sends pointer, key,
pedal and MIDI events, and takes offscreen screenshots ([`control-protocol.md`](control-protocol.md)).

## Offscreen UI snapshots (no window)

```sh
cargo run -p scorecraft-ui-egui --example snapshot -- --score corpus/public-domain/bach-1007.pdf \
  --layout twoPage --filter sepia --out /tmp/reader.png
```

Look at the PNG before you submit a UI change; attach before/after images to the PR.

## Environment variables

| Variable | Effect |
|---|---|
| `SCORECRAFT_FONTS_DIR` | Build input: fonts checkout (see Fonts) |
| `SCORECRAFT_LOG` | Log filter (`info`, `scorecraft_pdf=debug`) |
| `SCORECRAFT_LIBRARY` | Library path override for tests and the CLI |
| `SCORECRAFT_NO_NETWORK=1` | Refuse every network call (IMSLP, catalogues, sync); CI sets it |
| `SCORECRAFT_NO_PRINT=1` | The engine refuses to spool print jobs; CI sets it |
| `SCORECRAFT_CONTROL_TOKEN` | Fixed control-channel token for local scripting (never in CI logs) |

## Logs

Native builds log to the platform log directory (`~/Library/Logs/ScoreCraft`,
`%LOCALAPPDATA%\ScoreCraft\logs`, `$XDG_STATE_HOME/scorecraft`). Logs never contain score
contents, file names outside the library, or account tokens.

## Testing strategy

| Kind | Where | What |
|---|---|---|
| Unit | every crate | algorithms, model invariants |
| Engine integration | `crates/engine/tests/it/` | each command: behaviour, undo/redo, disabled states, bad params |
| `panic_hunt` | `crates/engine/tests/panic_hunt.rs` (`--ignored`) | every command with adversarial params and states |
| Format | `crates/pdf`, `crates/fsc`, `crates/format` | round trip, synthetic generators, malformed input, fuzz |
| Render parity | `crates/gpu/tests/` | GPU vs CPU within 1/255 on synthetic and corpus pages |
| Layout goldens | `crates/layout/tests/` | half-page and alternating turns, spreads, crops, snips |
| UI snapshots | `crates/ui-egui/examples/snapshot.rs` | offscreen renders compared with goldens |
| Sync | `crates/sync/tests/` | randomised multi-device op histories converge (property tests) |
| Corpus | `--features corpus` | real public-domain PDFs and `.4sc` samples |
| Performance | `cargo xtask perf` | budgets in `perf/budgets.toml` |

Every crash fix ships with a regression test that panicked before the fix.

## Test corpora

Real-file corpora live in `corpus/` (gitignored, **never committed**), fetched at pinned commits and
sha256-verified by `cargo xtask corpus --all`. Pins are in `xtask/src/corpus_pins.rs`.

| Corpus | Contents | Licence rule |
|---|---|---|
| `scorecraft-corpus` (our repository, to be created) | Public-domain score PDFs (IMSLP files marked public domain worldwide, Mutopia), scanned and engraved; synthetic hostile PDFs; `.4sc` samples made by the analyst | Only files we may redistribute |
| PDF Association test suites | Conformance and edge-case PDFs | As published |
| Synthetic | Generated in `testkit` at test time | Ours |

The corpus tests are opt-in (cargo feature `corpus`): plain `cargo test` skips them, and with the
feature on a missing corpus fails with "run `cargo xtask corpus --all`". `cargo xtask test-corpus`
fetches and runs them. CI always runs them, cached by pin.

## Scorecard and performance budgets

`cargo xtask scorecard` regenerates [`scorecard.md`](scorecard.md) from `scorecard/*.toml`,
`perf/budgets.toml`, `perf/baseline.json`, corpus floors and a preferences audit. CI runs
`cargo xtask scorecard --check`. `cargo xtask perf --quick` runs the budget scenarios locally;
`--update-baseline` publishes a full run from the fixed runner.

## Web build

`apps/scorecraft-web` builds with trunk. File access goes through the browser's file APIs and
OPFS for the library; Web MIDI and the Screen Wake Lock API back MIDI and keep-awake where the
browser offers them.

## Mobile builds

`apps/scorecraft-android` and `apps/scorecraft-ios` are thin shells around the same engine and UI.
Platform services (files, share sheet, camera, keep-awake, MIDI, pen) are implemented in
`scorecraft-devices` behind the platform-service traits. Test on a real tablet before claiming a
mobile feature done: emulators do not report pen pressure or pedal timing faithfully.

## Dev log

Append a terse entry to `log/devlog.md` (gitignored) when you finish a task: what landed, numbers,
what is still open. It is how the next agent picks up after an interrupted session.
