# ScoreCraft architecture

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version: design plan before any code) · **Target:** enScore 3.7 (2026)

## Current state (2026-10-28)

Nothing is built yet. This page is the **design plan**: the workspace, the layering rules that
keep the engine independent of the UI, the document model and how rendering, input, sync and
automation fit together. Where code later differs, add a "Current state" summary here (as
PhotoCraft does) and let it win over the plan below.

---

## 1. Goals and principles

**Goals**

- A native, portable, fully open-source sheet music reader with enScore's feature surface
  ([`feature-map.md`](feature-map.md)), written only in Rust.
- One codebase on macOS, Windows, Linux, Android, iOS/iPadOS and the web (WebAssembly).
- Instant, dependable page turns on a music stand. A reader that hesitates or crashes mid-concert
  has failed, whatever else it does.
- Markings that never damage the user's PDF and never get lost (offline-first, sync optional).
- Every feature drivable by agents and scripts.

**Principles** (carried over from the Crafting Apps framework)

1. **Engine-first, headless-first.** Every feature is reachable without a GUI: from tests, the CLI
   and MCP. The GUI is one client of the engine.
2. **Everything is a command.** Each user-visible action has a stable `CommandId`
   (`view.turn.next`, `annotate.stroke`, `list.entry.add`) with typed, serializable parameters.
   Tap zones, pedals, MIDI, menus, the command palette, the CLI, the control channel and MCP all
   dispatch the same commands. Ids come from [`feature-map.md`](feature-map.md).
3. **Data describes the UI; toolkits draw it.** Tools produce overlay primitives as data; dialogs
   come from parameter schemas. A new toolkit implements one renderer for each.
4. **The source PDF is immutable.** Imported files are stored content-addressed and never
   rewritten. Page edits, crops, rotations and markings are data on top. Export writes a new file.
5. **Immutable snapshots.** Large payloads (page bitmaps, PDF bytes, stroke lists) are `Arc`-shared
   so a library or score snapshot is cheap; undo, background rendering, autosave and sync read
   snapshots without locks.
6. **Determinism.** The same score renders the same on CPU and GPU (within tolerance), across
   thread counts and between screen and export.
7. **Pure Rust by default.** C/C++ is allowed only behind Cargo features at the I/O edge
   (for example a platform MIDI or camera binding), and only in the isolated `devices` crate.
8. **Clean-room.** See [`clean-room.md`](clean-room.md). Specs come from ISO 32000, MIDI 1.0,
   HID, eSCL, SMuFL, papers and observed behaviour.
9. **Never crash.** See [`standards/never-crash.md`](standards/never-crash.md). A hostile PDF, a
   corrupt library, an odd pedal or a full disk produces an error the user can act on.

### 1.1 Rules we adopt from day one

Each would force a rewrite later if baked in as an assumption.

| Property | Rule | Where |
|---|---|---|
| **Page geometry** | Annotations are stored in the **unrotated source page's PDF user space** (points, origin bottom-left as in ISO 32000 §8.3), never in screen pixels. Rotation, crop, margins and layout are transforms applied after. | `doc`, `layout` |
| **Page identity** | A score is a list of `PageRef { source, page_index }`, not "the PDF's pages". Reorder, hide, delete, page ranges, snips and setlists are all edits of references. | `doc` |
| **Input** | A pointer sample carries position, pressure, tilt, twist, timestamp and device kind (pen, touch, mouse, eraser end) from the start, even where a platform reports only position. | `input`, `ink` |
| **Colour** | Ink colours are sRGB with alpha; display filters (dark, sepia, inverted) apply to the *page*, and ink keeps a separate, legible mapping per filter. | `render` |
| **Size** | Libraries of 10,000+ scores and scores of 1,000+ pages are normal. Nothing loads a whole library or a whole score into memory; lists are paged and indexed. | `format`, `search` |
| **Time** | Practice logs and sync clocks go through `chrono` and hybrid logical clocks; nothing hand-rolls calendar maths. | `practice`, `sync` |
| **UI toolkit** | The core never depends on the toolkit (§4). | layering |
| **Platforms** | L0–L6 compile to `wasm32-unknown-unknown` and to Android/iOS targets; filesystem, camera, MIDI and power APIs sit behind platform-service traits. | all |

The test: **adding a new page source (images, MusicXML engraving), a new input device or a new
sync backend touches a leaf crate, not the foundation.**

---

## 2. Workspace layout

```text
scorecraft/
├─ Cargo.toml                  # [workspace], shared deps, lints, profiles
├─ crates/
│  │  ── L0 foundation (standalone, wasm-safe) ──
│  ├─ geom/        scorecraft-geom      points, rects, affine transforms, page boxes, bezier
│  ├─ raster/      scorecraft-raster    page bitmaps, tiles, mip pyramids, damage regions
│  ├─ pdf/         scorecraft-pdf       PDF read / render / write; own format-level model; depends on nothing in the workspace (publishable)
│  ├─ codecs/      scorecraft-codecs    raster decode/encode for imported images (PNG, JPEG, TIFF, HEIC, WebP)
│  ├─ midi/        scorecraft-midi      MIDI 1.0 messages, parser, learn-mode matching; no I/O (standalone)
│  ├─ fsc/         scorecraft-fsc       forScore .4sc reader, from the black-box spec only (standalone, read only)
│  │  ── L1 document ──
│  ├─ doc/         scorecraft-doc       library model: scores, page refs, annotation layers, setlists, bookmarks, metadata (pure data + serde)
│  │  ── L2 algorithms ──
│  ├─ ops/         scorecraft-ops       reversible operations, transactions, undo/redo
│  ├─ ink/         scorecraft-ink       stroke engine: smoothing, pressure curves, highlighter, whiteout, erasers, hit testing, stamps
│  ├─ layout/      scorecraft-layout    page layout: single, two-page, half-page and alternating turns, zoom, crop, snips
│  ├─ text/        scorecraft-text      text notes and stamp glyphs (shaping via a pure-Rust stack, SMuFL fonts)
│  ├─ search/      scorecraft-search    metadata and full-text index (incremental, on-disk)
│  ├─ algo/        scorecraft-algo      imaging for scans: edge detection, perspective, deskew, clean-up, Enhance filter
│  ├─ audio/       scorecraft-audio     decode, time-stretch, pitch-shift, loop; metronome synthesis
│  ├─ practice/    scorecraft-practice  metronome scheduling, tuner pitch detection, practice log, reports
│  │  ── L3 rendering and storage ──
│  ├─ render/      scorecraft-render    CPU page compositor (the oracle): PDF raster + filters + ink + snips
│  ├─ gpu/         scorecraft-gpu       wgpu compositor for pages and live ink
│  ├─ format/      scorecraft-format    on-disk library: content-addressed blobs, score records, atomic saves, verify/repair
│  │  ── L4 I/O and services ──
│  ├─ io/          scorecraft-io        import/export orchestration: PDF, images, folders, .4sc, setlist bundles, print spooling
│  ├─ sync/        scorecraft-sync      sync protocol, CRDT merge, backends (folder, WebDAV, ScoreCraft server), backup
│  ├─ metadata/    scorecraft-metadata  title/composer detection: PDF info, text layer, OCR, optional public catalogues
│  ├─ imslp/       scorecraft-imslp     IMSLP client over its public MediaWiki API
│  ├─ scan/        scorecraft-scan      camera capture pipeline and eSCL network scanners
│  ├─ input/       scorecraft-input     pedal key maps, MIDI learn, pen shortcuts, palm rejection → commands
│  ├─ devices/     scorecraft-devices   platform MIDI, audio I/O, camera, keep-awake; the one crate where FFI may appear (feature-gated)
│  │  ── L5 the façade every frontend talks to ──
│  ├─ engine/      scorecraft-engine    Session: open library, command registry + dispatch, jobs, events, view-models, preferences
│  │  ── L6 frontends ──
│  ├─ ui-egui/     scorecraft-ui-egui   egui shell: library, reader, tools, dialogs (thin: all actions go through the engine)
│  ├─ automation/  scorecraft-automation MCP server and control-channel handlers
│  └─ testkit/     scorecraft-testkit   synthetic PDFs, stroke generators, fixtures
├─ apps/
│  ├─ scorecraft/              desktop app (eframe/wgpu), TCP control server
│  ├─ scorecraft-cli/          headless CLI (import / info / export / run / batch / commands / mcp)
│  ├─ scorecraft-web/          the same app in the browser (trunk + wasm-bindgen)
│  ├─ scorecraft-android/      Android shell (eframe android-activity)
│  ├─ scorecraft-ios/          iOS / iPadOS shell (winit + wgpu)
│  └─ scorecraft-sync-server/  self-hostable sync and backup server
└─ xtask/                      cargo xtask layers | wasm | ci | stats | corpus | test-corpus | parity | perf | scorecard
```

## 3. Dependency layering (enforced)

```text
L6  ui-egui  automation
L5  engine
L4  io  sync  metadata  imslp  scan  input  devices
L3  render  gpu  format
L2  ops  ink  layout  text  search  algo  audio  practice
L1  doc
L0  geom  raster  pdf  codecs  midi  fsc
```

- A crate may depend only on **lower** layers. `cargo xtask layers` checks every `Cargo.toml`
  against the table in `xtask/src/layers.rs`; a new crate must be registered there.
- `pdf`, `midi` and `fsc` depend on **nothing** in the workspace, so they can be published and
  fuzzed alone.
- Nothing below `ui-egui` may use egui, eframe, winit or a file dialog crate.
- `devices` is the only crate that may contain FFI, behind Cargo features, with
  `unsafe_code = "deny"` and a `SAFETY:` comment on every block; every other crate forbids
  `unsafe` (PhotoCraft's `tablet` exception, generalised to one crate).
- `testkit` is a dev-dependency only.

## 4. The UI-swap seam (the most important interface)

### 4.1 `Session`: commands in, events out

```rust
pub struct Session { /* library, open readers, history, jobs, prefs */ }

impl Session {
    pub fn dispatch(&mut self, id: &str, params: serde_json::Value) -> Result<serde_json::Value, EngineError>;
    pub fn events(&self) -> impl Iterator<Item = Event>;     // ScoreChanged, PageTurned, SyncStatus, JobProgress…
    pub fn view_model(&self, reader: ReaderId) -> ReaderView; // what to draw, as data
}
```

The UI never mutates the library directly. It dispatches commands and draws view-models.

### 4.2 `CommandRegistry`

Each command is a `CommandSpec`: `id`, `label`, menu path, default shortcut, a params doc string
(`{"layer":id?,"tool":"pen|highlighter"="pen"}`, what agents read through `commands`),
an `enabled` predicate and `run`. `run` validates params, returns a JSON result and records
**exactly one** history step (strokes and drags coalesce). Commands live in area modules
(`crates/engine/src/<area>_cmds.rs` exposing `specs()`), registered in `commands.rs`.

`feature_catalog.rs` holds the feature-map rows (id, title, command ids); `cargo xtask parity`
checks which commands are registered and live and writes [`parity-checklist.md`](parity-checklist.md).

### 4.3 Tools and gestures

Tools (pen, highlighter, whiteout, erasers, text, stamp, lasso, snip, shortcut) are state
machines: pointer samples in, ops and overlay primitives out. Page-turn gestures are a tool too,
so the same zone logic serves touch, mouse and the control channel's synthetic pointer events.

### 4.4 Reader view

`layout` turns (score, page refs, mode, viewport, turn state) into a list of placed page regions:
whole pages, half pages during a half-page turn, snips. `render`/`gpu` draw that list. Turn
animations are interpolations of two layouts, so e-ink mode simply skips them.

---

## 5. Document model (`scorecraft-doc`)

Pure data, `serde`, no I/O.

```text
Library
├─ folders: tree of Folder { id, name, children }
├─ scores:  ScoreId → Score
├─ lists:   ListId → Setlist { name, entries: [Entry { score, page_range?, tempo?, note? }] }
├─ trash:   Recently Deleted (ScoreId, deleted_at)
└─ settings (library-level, synced)

Score
├─ id: ScoreId (UUID v7)            source: BlobRef (sha256 of the imported PDF / image set)
├─ metadata: title, composer, key, tags, custom fields, notes, detection provenance
├─ pages: [PageRef { source_page, rotation, hidden, crop, margins }]   page_range?
├─ layers: [AnnotationLayer { id, name, visible, locked, order }]
├─ annotations: PageKey → [Annotation]
│     Stroke { tool: Pen|Highlighter|Whiteout, colour, width, points: [InkPoint { x, y, pressure, tilt, twist, t }] }
│     Text { rect, text, style }     Stamp { glyph (SMuFL codepoint), rect, colour }
│     Snip { from: (page, rect), to: (page, rect) }   Shortcut { at: (page, rect), target: (page, rect?) }
├─ bookmarks: [Bookmark { name, page }]
├─ audio: Track? { blob, turn_recording: [(time, page)] , loop?, speed, transpose }
├─ midi_on_open: { program?, bank?, song? }
└─ practice notes
```

Every annotation has an id, a layer, a creator device and a hybrid-logical-clock stamp so sync can
merge without conflicts (§9). Erasing a part of a stroke replaces it with its remaining pieces
(new ids) and tombstones the original.

## 6. Operations and history (`scorecraft-ops`)

Reversible ops on a score or the library, grouped into transactions; one command = one undo step.
History is per score and per library, bounded in memory, and survives a reopen for the current
session. Sync applies remote ops as their own transactions, never merged into the user's undo
stack.

## 7. Rendering

- **PDF.** `scorecraft-pdf` parses lazily (xref streams, object streams, incremental updates,
  repaired xref for damaged files) and rasterises pages through a pure-Rust pipeline. Which
  rasteriser we build on (our own on top of `vello_cpu`/`kurbo`, or an existing permissively
  licensed pure-Rust renderer) is open decision D1 in [`decisions.md`](decisions.md).
- **Caching.** Rendered tiles are cached per (page, zoom bucket, filter). The current, next and
  previous pages (and the next setlist piece's first page) are pre-rendered in background jobs so a
  turn is a cache hit. Budgets live in `perf/budgets.toml` ([`scorecard.md`](scorecard.md#performance)).
- **Two compositors.** `render` is the CPU oracle (export, print, tests, wasm fallback); `gpu` draws
  pages and live ink with wgpu (Metal, Vulkan, DX12, GLES, WebGPU). They are tested against each
  other within 1/255.
- **Ink.** Live strokes are drawn on the GPU as they arrive (predicted points for latency, replaced
  by real ones); committed strokes are tessellated once and cached per page.
- **Filters.** Dark, sepia and inverted are per-page colour transforms with ink colours remapped
  for legibility; Enhance is an `algo` pass on scanned (image-only) pages, cached.

## 8. Library storage (`scorecraft-format`)

```text
<Library>.scorecraft/
├─ manifest.json            version, library id, device id
├─ blobs/sha256/ab/cdef…    imported PDFs, images, audio (immutable, content-addressed)
├─ scores/<score-id>.json   score record (metadata, page refs, layers, bookmarks)
├─ ink/<score-id>/<page>.bin annotation op log per page (append-only, compacted)
├─ lists/<list-id>.json
├─ index/                   search index (rebuildable)
└─ journal/                 crash-safe write-ahead log
```

- Saves are atomic (write a temp file, fsync, rename), per record, never a whole-library rewrite.
- `library.verify` checks every blob hash and record and rebuilds the index (LIB-12).
- The format is documented in [`file-format-parity.md`](file-format-parity.md#scorecraft-library-format)
  so other tools can read it. Adding a `doc` field must add it here with `#[serde(default)]`; the
  `format` crate fails to compile when a `doc` struct gains a field it does not map (PhotoCraft's
  rule for `.pcraft`).

## 9. Sync

- **Offline-first.** The device's library is the source of truth; sync exchanges ops and blobs.
- **Merge.** Annotations are an add-wins set with tombstones; metadata fields are last-writer-wins
  registers keyed by hybrid logical clock; list entry order uses a sequence CRDT. Two devices
  marking the same page offline both keep their strokes.
- **Backends** behind one trait: a shared folder the OS already syncs (iCloud Drive, OneDrive,
  Dropbox, Syncthing), WebDAV, and `scorecraft-sync-server` (self-hostable). No account is ever
  required. Optional end-to-end encryption of ops and blobs.
- **Backup** is a snapshot of the library directory into a single archive, local or to a backend.

## 10. Input

- **Pedals.** Bluetooth page-turn pedals present as keyboards. `input` maps key events (arrows,
  Page Up/Down, Space, Enter, custom) to `view.turn.*`, with a learn mode and per-pedal profiles.
- **MIDI.** `midi` parses messages; `devices` opens ports (CoreMIDI, Windows MIDI, ALSA, Android
  MIDI, Web MIDI); `input` maps learnt messages to commands. Output sends Program Change, Bank
  Select and Song Select on score open.
- **Pen.** Pressure, tilt and eraser end come from winit where it reports them and from platform
  hooks in `devices` where it doesn't (as PhotoCraft's `tablet` crate does on macOS and Windows).
- **Palm rejection.** Pen-only mode ignores touch on the page while a pen is in range.

## 11. Engine runtime model

One `Session` per process, background jobs (render, import, OCR, sync, scan) on a rayon pool with
cancellation, events to the shell. Preferences are typed, serde, versioned, and every preference
must change behaviour (the scorecard counts "settings that do nothing").

## 12. Automation and extensibility

- **CLI** (`scorecraft-cli`): `import`, `info`, `export`, `run --cmd <id> --params <json>`,
  `batch`, `commands`, `mcp`.
- **Control channel**: authenticated, loopback-only JSON lines (`scorecraft --control`) to read
  UI state, send pointer and key events, and take offscreen screenshots
  ([`control-protocol.md`](control-protocol.md)).
- **MCP**: the same commands as tools, headless or bridged to the running app.

## 13. Testing and quality

- Unit tests in every crate; integration tests per engine area; `panic_hunt` fuzzes every
  command with adversarial params.
- `pdf` and `fsc` get round-trip, synthetic-generator, malformed-input and `cargo fuzz` targets.
- Real-file corpora (public-domain PDFs from IMSLP and Mutopia, the PDF Association's test
  suites, our own `.4sc` samples) live in a separate corpus repository, pinned and
  sha256-verified ([`development.md`](development.md#test-corpora)).
- GPU vs CPU render parity; offscreen UI snapshots for visual checks; performance budgets on a
  fixed runner.

## 14. Open decisions

Tracked with dates in [`decisions.md`](decisions.md): D1 PDF rasteriser, D2 on-disk record
encoding (JSON vs CBOR), D3 iOS shell, D4 OCR model for title detection, D5 sync server protocol
(HTTP + JSON ops vs. a binary protocol), D6 product name.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version: design plan adapted from PhotoCraft's architecture for a score reader |
