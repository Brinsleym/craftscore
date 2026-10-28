# Feature map

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, 118 rows from public listings) · **Target:** enScore 3.7 (2026)

Every user-visible enScore capability we know of, one row each, with the ScoreCraft command that
will implement it. This is ScoreCraft's equivalent of PhotoCraft's Photoshop menu catalog: it is
the **denominator** of the breadth number in [`target-app-parity.md`](target-app-parity.md), the
source of `crates/engine/src/feature_catalog.rs` once the engine exists, and what
`cargo xtask parity` will measure into [`parity-checklist.md`](parity-checklist.md).

Rows come from public sources only ([clean-room](clean-room.md)); the per-row source tags (S1–S6)
are listed in [`observations/public-listings-2026-10-28.md`](observations/public-listings-2026-10-28.md).
Add a row when a new enScore release or an observation note shows a new capability; never delete a
row, mark it `dropped` with a reason.

## Columns

- **Id**: stable. Prefix = area. Used in specs, scorecard rows, commits (`READ-5: half-page turns`).
- **Tier**: what enScore charges for it. `Free`, `Pro` (one-time), `Sync` (subscription), `Acct`
  (needs an account). **ScoreCraft has no tiers:** every feature is free and works without an
  account. The column exists so parity reports can compare like with like.
- **Command**: the engine command id(s), `area.noun.verb` in lowerCamelCase segments, like
  PhotoCraft's (`view.turn.next`). Pure view state that never touches a document may be a shell
  command (`ui.*`), as in PhotoCraft's `UI_COMMANDS`.
- **Crate**: where the algorithm lives (see [`architecture.md`](architecture.md#2-workspace-layout)).
- **P**: priority. `P0` = needed for the [alpha gate](roadmap.md#alpha-gate), `P1` = needed for
  beta, `P2` = full parity, `P3` = ours only or optional.
- **Est**: agent-hours including tests, uncalibrated until we have merged-PR history
  ([calibration](target-app-parity.md#remaining-effort-and-calibration)).
- **Status**: `missing` · `partial` · `done` (behaves as specified, with a test) · `dropped`.

## Summary

| Area | Prefix | Rows | P0 | Done | Partial | Missing |
|---|---|---:|---:|---:|---:|---:|
| Reading and display | READ | 15 | 8 | 0 | 0 | 15 |
| Page turning inputs | TURN | 8 | 4 | 0 | 0 | 8 |
| Annotation | ANNO | 16 | 8 | 0 | 0 | 16 |
| Library and organisation | LIB | 15 | 6 | 0 | 0 | 15 |
| Page editing | PAGE | 8 | 4 | 0 | 0 | 8 |
| In-score navigation | NAV | 5 | 1 | 0 | 0 | 5 |
| Setlists | LIST | 7 | 3 | 0 | 0 | 7 |
| Audio playback | PLAY | 6 | 0 | 0 | 0 | 6 |
| MIDI | MIDI | 4 | 0 | 0 | 0 | 4 |
| Practice tools | PRAC | 11 | 1 | 0 | 0 | 11 |
| Import and scanning | IMP | 8 | 1 | 0 | 0 | 8 |
| Export and print | EXP | 4 | 1 | 0 | 0 | 4 |
| Sync and backup | SYNC | 7 | 0 | 0 | 0 | 7 |
| Platforms, UI and languages | PLAT | 4 | 2 | 0 | 0 | 4 |
| **Total** | | **118** | **39** | **0** | **0** | **118** |

Sum of the estimates: **~990–1,990 agent-hours** for every row (P0 alone: see [`roadmap.md`](roadmap.md#alpha-gate)).

## READ: reading and display

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| READ-1 | Open a PDF score and render its pages crisply at any zoom, fast enough to turn without a visible delay | Free | S1 | `score.open`, `view.render` | pdf, render | P0 | 40–80 | missing |
| READ-2 | Single-page layout | Free | S1, S4 | `view.layout.single` | layout | P0 | 4–8 | missing |
| READ-3 | Two-page spread layout, with correct first-page placement for spreads | Free | S1 | `view.layout.twoPage` | layout | P0 | 6–12 | missing |
| READ-4 | Full-screen (chrome hidden) performance mode | Free | S1, S4 | `ui.view.fullScreen` | ui-egui | P0 | 3–6 | missing |
| READ-5 | Half-page turns: the next system appears before the full turn | Free | S1, S2, S3 | `view.turn.halfPage` (mode) | layout | P0 | 8–16 | missing |
| READ-6 | Alternating page turns for awkward page breaks | Free | S4 | `view.turn.alternating` (mode) | layout | P1 | 4–8 | missing |
| READ-7 | Display filters: light, dark, sepia (with intensity), inverted | Free | S1 | `view.filter.set` | render | P1 | 6–10 | missing |
| READ-8 | Enhance filter for scanned pages (contrast / clean-up) | Free | S1 | `view.filter.enhance` | algo | P2 | 8–16 | missing |
| READ-9 | Keep the screen awake while a score or setlist is open | Free | S1, S4 | `ui.power.keepAwake` | app shell | P0 | 2–4 | missing |
| READ-10 | Two scores side by side | Free | S1, S3 | `view.split.open` | engine, ui-egui | P2 | 8–16 | missing |
| READ-11 | Multiple windows, each with its own score | Free | S1 | `ui.window.new` | app shell | P2 | 8–16 | missing |
| READ-12 | Landscape Page Zoom (fit width in landscape, scroll within a page) | Free | S1 | `view.zoom.landscapeFit` | layout | P1 | 4–8 | missing |
| READ-13 | Non-destructive crop and margin adjustment per page | Free | S1 | `page.crop.set`, `page.margins.set` | doc, layout | P1 | 6–12 | missing |
| READ-14 | Robust with damaged pages and very large scores (hundreds of pages, huge scans) | Free | S1 | n/a (quality bar) | pdf, render | P0 | 10–20 | missing |
| READ-15 | Pinch / scroll zoom and pan within a page | Free | inferred | `view.zoom.set`, `view.pan` | layout, ui-egui | P0 | 4–8 | missing |

## TURN: page turning inputs

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| TURN-1 | Tap zones turn forward and back | Free | S1, S4 | `view.turn.next`, `view.turn.previous` | ui-egui | P0 | 3–6 | missing |
| TURN-2 | Swipe to turn | Free | S1, S4 | `view.turn.next`, `view.turn.previous` | ui-egui | P0 | 3–6 | missing |
| TURN-3 | Keyboard arrow keys (and Page Up / Down, Space) | Free | S4 | `view.turn.next`, `view.turn.previous` | ui-egui | P0 | 1–2 | missing |
| TURN-4 | Bluetooth foot pedals (keyboard-emulating HID pedals, configurable key mapping) | Free | S1, S4 | `input.pedal.map` | input | P0 | 6–12 | missing |
| TURN-5 | MIDI pedals and controllers turn pages | Free | S1 | `input.midi.mapTurn` | midi, input | P2 | 6–12 | missing |
| TURN-6 | Record page turns during audio playback; replay them automatically | Free | S1 | `play.turns.record`, `play.turns.follow` | engine, audio | P2 | 8–16 | missing |
| TURN-7 | Turn across score boundaries inside a setlist (last page → next piece) | Pro | S1 | `list.turn.next` | engine | P1 | 3–6 | missing |
| TURN-8 | Jump to first / last page, go to page number | Free | inferred | `view.page.goto` | engine | P1 | 1–2 | missing |

## ANNO: annotation

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| ANNO-1 | Pen ink with stylus, finger ("draw with touch") or mouse, low latency | Free | S1, S2, S4 | `annotate.tool.pen`, `annotate.stroke` | ink | P0 | 16–30 | missing |
| ANNO-2 | Pressure-sensitive strokes (Apple Pencil, Windows Ink, Android stylus, Wacom) | Free | S1, S2 | `annotate.stroke` (pressure) | ink, input | P0 | 8–16 | missing |
| ANNO-3 | Highlighter (translucent, stronger option) | Free | S1 | `annotate.tool.highlighter` | ink | P0 | 4–8 | missing |
| ANNO-4 | Custom pen colours and widths, saved presets | Free | S1 | `annotate.pen.preset.set` | ink, engine | P1 | 4–8 | missing |
| ANNO-5 | Whiteout: cover printed marks, then write over them | Free | S1, S2 | `annotate.tool.whiteout` | ink | P1 | 4–8 | missing |
| ANNO-6 | Stroke eraser (removes whole strokes) | Free | S1 | `annotate.tool.strokeEraser` | ink | P0 | 4–8 | missing |
| ANNO-7 | Pixel eraser (erases part of a stroke) | Free | S1 | `annotate.tool.pixelEraser` | ink | P1 | 8–16 | missing |
| ANNO-8 | Typed text annotations, movable and editable | Free | S1, S4 | `annotate.text.add`, `annotate.text.edit` | doc, text | P0 | 8–14 | missing |
| ANNO-9 | Stamps: fingerings, dynamics, bowings and other music symbols (from an OFL SMuFL font) | Free | S1 | `annotate.stamp.place` | ink, text | P1 | 10–20 | missing |
| ANNO-10 | Annotation layers: create, rename, show/hide individually (hide for performance) | Pro | S1, S3, S4 | `annotate.layer.new`, `annotate.layer.setVisible` | doc | P1 | 8–14 | missing |
| ANNO-11 | Pen shortcuts (stylus double-tap / barrel button switches tool) | Free | S1 | `input.pen.shortcut.map` | input | P2 | 4–8 | missing |
| ANNO-12 | Select, move, resize and recolour existing annotations (lasso) | Free | inferred | `annotate.select`, `annotate.transform` | ink, engine | P1 | 10–18 | missing |
| ANNO-13 | Undo / redo of every markup step | Free | inferred | `edit.undo`, `edit.redo` | ops | P0 | 6–10 | missing |
| ANNO-14 | Palm rejection and "pen only" drawing mode | Free | inferred | `input.pen.only` | input | P0 | 4–8 | missing |
| ANNO-15 | Annotations never alter the source PDF; stored separately per page | Free | S1 | n/a (model rule) | doc | P0 | 2–4 | missing |
| ANNO-16 | Shapes and lines (straight line, circle) snapped from strokes | Free | inferred | `annotate.tool.shape` | ink | P2 | 6–10 | missing |

## LIB: library and organisation

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| LIB-1 | Import PDFs from a file picker, open-with / share sheet, drag and drop | Free | S1, S4 | `library.import.files` | io | P0 | 6–12 | missing |
| LIB-2 | Import a whole folder, with folder-level bulk add | Free | S1, S3 | `library.import.folder` | io | P1 | 4–8 | missing |
| LIB-3 | Folders | Free | S1 | `library.folder.new`, `library.score.move` | doc | P0 | 4–8 | missing |
| LIB-4 | Metadata fields: title, composer, tags, key | Free | S1 | `score.metadata.set` | doc | P0 | 4–8 | missing |
| LIB-5 | Search across title, composer, tags, key and folders | Free | S1 | `library.search` | search | P0 | 6–12 | missing |
| LIB-6 | Full-text search inside PDFs and practice notes | Free | S1 | `library.search.fullText` | search, pdf | P1 | 12–24 | missing |
| LIB-7 | Automatic title and composer detection from generic scan names | Acct | S1, S2 | `score.metadata.detect` | metadata | P2 | 16–30 | missing |
| LIB-8 | Detection for a whole folder of files at once | Acct | S1 | `library.metadata.detectAll` | metadata | P2 | 4–8 | missing |
| LIB-9 | Sort by title, composer, date added, recently opened | Free | S1 | `library.sort.set` | engine | P0 | 2–4 | missing |
| LIB-10 | Recently Deleted folder with restore | Free | S1, S3 | `library.trash.restore`, `library.trash.empty` | doc | P1 | 4–8 | missing |
| LIB-11 | Bookmarks: name a page, list and jump | Free | S1, S4 | `score.bookmark.add`, `view.bookmark.goto` | doc | P0 | 3–6 | missing |
| LIB-12 | Verify & Repair Library (check files, rebuild index) | Free | S1 | `library.verify` | format | P1 | 6–12 | missing |
| LIB-13 | Unlimited scores; library opens fast with thousands of PDFs | Free | S4 | n/a (performance budget) | format, search | P1 | 8–16 | missing |
| LIB-14 | Thumbnails / cover grid and list views | Free | inferred | `ui.library.view` | ui-egui, render | P1 | 6–12 | missing |
| LIB-15 | Recently opened list | Free | inferred | `library.recent.list` | engine | P1 | 2–4 | missing |

## PAGE: page editing

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| PAGE-1 | Reorder pages | Free | S1, S4 | `page.move` | doc | P0 | 3–6 | missing |
| PAGE-2 | Rotate pages | Free | S1, S4 | `page.rotate` | doc | P0 | 2–4 | missing |
| PAGE-3 | Hide pages (kept, not shown) | Free | S1, S4 | `page.setHidden` | doc | P0 | 2–4 | missing |
| PAGE-4 | Delete pages | Free | S1, S4 | `page.delete` | doc | P0 | 2–4 | missing |
| PAGE-5 | Crop and straighten scanned pages without altering the file | Free | S1 | `page.straighten`, `page.crop.set` | algo, doc | P1 | 8–16 | missing |
| PAGE-6 | Blank manuscript paper: single staff, grand staff or tab, any page size | Free | S1 | `score.newManuscript` | render, pdf | P1 | 8–14 | missing |
| PAGE-7 | Score Snips: copy bars from one place and show them elsewhere (for example the end of the previous page after a turn) | Pro | S1, S3, S4 | `page.snip.create`, `page.snip.place` | doc, render | P1 | 12–24 | missing |
| PAGE-8 | Page Range: limit a score to a range of its PDF's pages | Free | S1 | `score.pageRange.set` | doc | P1 | 3–6 | missing |

## NAV: in-score navigation

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| NAV-1 | Score Shortcuts: tappable link on a repeat / D.S. / coda sign that jumps to its target | Pro | S1, S4 | `nav.shortcut.add`, `nav.shortcut.follow` | doc, engine | P1 | 10–18 | missing |
| NAV-2 | Create a shortcut from the viewer menu | Pro | S1 | `nav.shortcut.add` | ui-egui | P1 | 2–4 | missing |
| NAV-3 | Return from a jump (back to where the shortcut was followed) | Pro | inferred | `nav.back` | engine | P1 | 2–4 | missing |
| NAV-4 | Page-turn order follows shortcuts during a performance (repeat-aware turning) | Pro | S1, S4 | `nav.follow.mode` | engine | P2 | 8–16 | missing |
| NAV-5 | Table of contents from PDF outline | Free | inferred | `view.outline.goto` | pdf | P0 | 3–6 | missing |

## LIST: setlists

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| LIST-1 | Setlists ("Lists") for practice, lessons and concerts: create, reorder, rename | Pro | S1, S4 | `list.new`, `list.entry.add`, `list.entry.move` | doc | P0 | 6–12 | missing |
| LIST-2 | Page range per list entry | Pro | S1 | `list.entry.pageRange.set` | doc | P1 | 3–6 | missing |
| LIST-3 | Tempo per list entry (feeds the metronome) | Pro | S1 | `list.entry.tempo.set` | doc, practice | P1 | 2–4 | missing |
| LIST-4 | Pieces flow into one another; screen stays awake until the end | Pro | S1, S2 | `list.play` | engine | P0 | 4–8 | missing |
| LIST-5 | Share a whole setlist as one file (PDFs, order, page ranges, markings) and import it | Pro | S1, S3 | `list.export.bundle`, `list.import.bundle` | io, format | P1 | 10–18 | missing |
| LIST-6 | The same score appears in several lists without duplication | Pro | inferred | n/a (model rule) | doc | P0 | 1–2 | missing |
| LIST-7 | Placeholders / breaks in a list (intermission, spoken item) | Pro | inferred | `list.entry.addPlaceholder` | doc | P3 | 2–4 | missing |

## PLAY: audio playback

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| PLAY-1 | Attach an audio recording to a score | Free | S1 | `play.track.attach` | doc, audio | P2 | 6–12 | missing |
| PLAY-2 | Play at any speed without changing pitch (time-stretch) | Free | S1 | `play.speed.set` | audio | P2 | 12–24 | missing |
| PLAY-3 | Loop a passage (A–B) | Free | S1 | `play.loop.set` | audio | P2 | 4–8 | missing |
| PLAY-4 | Transpose playback by semitones (pitch-shift) | Free | S1 | `play.transpose.set` | audio | P2 | 10–20 | missing |
| PLAY-5 | Record page turns during playback (see TURN-6) | Free | S1 | `play.turns.record` | engine | P2 | 2–4 | missing |
| PLAY-6 | Transport controls while reading (play, pause, scrub) | Free | S1 | `play.toggle`, `play.seek` | ui-egui, audio | P2 | 4–8 | missing |

## MIDI

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| MIDI-1 | Turn pages with MIDI pedals or controllers (learn mode) | Free | S1 | `input.midi.mapTurn` | midi | P2 | 6–12 | missing |
| MIDI-2 | Send Program Change and Bank Select when a score opens | Free | S1 | `score.midi.onOpen.set` | midi | P2 | 4–8 | missing |
| MIDI-3 | Send Song Select when a score opens | Free | S1 | `score.midi.onOpen.set` | midi | P2 | 1–2 | missing |
| MIDI-4 | Choose MIDI input/output devices (USB, Bluetooth LE MIDI, network) | Free | S1 | `input.midi.devices` | midi | P2 | 8–16 | missing |

## PRAC: practice tools

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| PRAC-1 | Metronome with tap tempo | Free | S1, S4 | `practice.metronome.toggle`, `practice.metronome.tap` | practice | P0 | 6–12 | missing |
| PRAC-2 | Metronome subdivisions and beats per bar, accent | Free | S1 | `practice.metronome.set` | practice | P1 | 3–6 | missing |
| PRAC-3 | Nine metronome sounds (our own recordings or synthesis) | Free | S1 | `practice.metronome.sound.set` | practice | P2 | 4–8 | missing |
| PRAC-4 | Tuner (pitch detection from the microphone, reference pitch) | Free | S1 | `practice.tuner.toggle` | practice | P2 | 12–24 | missing |
| PRAC-5 | Practice log: minutes per day, streaks, pieces practised and when | Free | S1 | `practice.log.*` | practice, doc | P2 | 8–16 | missing |
| PRAC-6 | Daily practice goals | Free | S1 | `practice.goal.set` | practice | P2 | 2–4 | missing |
| PRAC-7 | Pause and resume logging | Free | S1 | `practice.log.pause` | practice | P2 | 1–2 | missing |
| PRAC-8 | Practice notes inside scores | Free | S1 | `score.note.add` | doc | P2 | 3–6 | missing |
| PRAC-9 | Shareable PDF practice report | Free | S1 | `practice.report.export` | practice, pdf | P2 | 6–10 | missing |
| PRAC-10 | Practice log backup and restore | Free | S1 | `practice.log.export`, `practice.log.import` | practice, format | P2 | 2–4 | missing |
| PRAC-11 | Visual metronome pulse (silent performance cue) | Free | inferred | `practice.metronome.visual` | ui-egui | P3 | 2–4 | missing |

## IMP: import and scanning

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| IMP-1 | Camera scan with automatic edge detection, perspective straightening, crop and clean-up into a PDF | Free | S1, S2 | `scan.camera` | scan | P2 | 24–40 | missing |
| IMP-2 | Wi-Fi flatbed and feeder scanners (eSCL / AirScan) | Free | S1 | `scan.network` | scan | P2 | 12–24 | missing |
| IMP-3 | Import photos / images (PNG, JPEG, HEIC, TIFF) as pages | Free | S1 | `library.import.images` | io, codecs | P1 | 6–10 | missing |
| IMP-4 | IMSLP search and import of public-domain scores | Pro | S1, S3, S4 | `imslp.search`, `imslp.import` | imslp | P2 | 12–24 | missing |
| IMP-5 | Import a forScore library from a `.4sc` file: pages, ink, stamps, layers | Free | S1, S4 | `library.import.forscore` | fsc, io | P1 | 30–60 | missing |
| IMP-6 | Import report: what was imported, skipped, or could not be read | Free | inferred | n/a (result of import) | io | P0 | 2–4 | missing |
| IMP-7 | Import PDF annotations made in other PDF apps (ISO 32000 ink, highlight, free-text annots) | Free | ours | `library.import.files` | pdf, io | P1 | 10–18 | missing |
| IMP-8 | Import MusicXML / MIDI rendered to pages | n/a | ours | `library.import.musicxml` | engrave | P3 | 80–160 | missing |

## EXP: export and print

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| EXP-1 | Print a score with or without annotations | Free | S1, S4 | `score.print` | pdf, app shell | P1 | 8–16 | missing |
| EXP-2 | Export a PDF with ink flattened, or as real PDF annotations, choosing which layers | Free | S1, S4 | `score.export.pdf` | pdf | P0 | 10–18 | missing |
| EXP-3 | Export a setlist as a single combined PDF | Pro | inferred | `list.export.pdf` | pdf | P2 | 3–6 | missing |
| EXP-4 | Export a page as an image | Free | inferred | `page.export.image` | render, codecs | P2 | 2–4 | missing |

## SYNC: sync and backup

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| SYNC-1 | Sync scores, markings, lists and bookmarks across devices | Sync | S1, S3, S4 | `sync.now`, `sync.status` | sync | P1 | 60–120 | missing |
| SYNC-2 | Cloud backup of the whole library | Sync | S1, S4 | `backup.create`, `backup.restore` | sync, format | P1 | 12–24 | missing |
| SYNC-3 | Offline-first: the device is the source of truth; the server holds a copy | Sync | S1 | n/a (design rule) | sync | P1 | n/a | missing |
| SYNC-4 | Account deletion removes server copies, keeps local ones | Sync | S1 | `sync.account.delete` | sync | P2 | 2–4 | missing |
| SYNC-5 | Fast first download on a new device; background downloads | Sync | S1 | n/a (performance) | sync | P2 | 8–16 | missing |
| SYNC-6 | Conflict-free merging of concurrent markings on two devices | Sync | inferred | n/a (CRDT) | sync | P1 | 20–40 | missing |
| SYNC-7 | Sync through a folder the user already syncs (iCloud Drive, OneDrive, Dropbox, Syncthing) or WebDAV, no account | n/a | ours (S3 review) | `sync.backend.set` | sync | P1 | 16–30 | missing |

## PLAT: platforms, UI and languages

| Id | Feature | Tier | Source | Command | Crate | P | Est | Status |
|---|---|---|---|---|---|---|---:|---|
| PLAT-1 | Same app on desktop (macOS, Windows, Linux), tablet and phone (iPadOS, iOS, Android), Chromebook, web | Free | S1–S3 | n/a | apps | P0 (desktop), P1 (Android, web), P2 (iOS) | 80–200 | missing |
| PLAT-2 | E-ink / e-reader mode (no animation, high contrast, partial refresh friendly) | Free | S3, S6 | `ui.display.eInk` | ui-egui | P2 | 8–16 | missing |
| PLAT-3 | Twelve or more interface languages (see [`localization-parity.md`](localization-parity.md)) | Free | S1 | n/a | ui-egui | P1 | 40–80 | missing |
| PLAT-4 | Light and dark app themes, independent of the page filter | Free | S4 | `ui.theme.set` | ui-egui | P0 | 4–8 | missing |

## Ours only (not in enScore)

Rows tagged `ours` above (IMP-7, IMP-8, SYNC-7) and these capabilities come from the Crafting Apps
framework rather than from enScore. They are counted in the scorecard, not in parity:

- **Everything is a command**, drivable from the CLI, a JSON control channel and an MCP server
  ([`control-protocol.md`](control-protocol.md)).
- **Headless batch**: `scorecraft-cli` imports, tags, exports and checks libraries without a UI.
- **Open library format** on disk ([`file-format-parity.md`](file-format-parity.md#scorecraft-library-format)),
  readable without ScoreCraft.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version: 118 rows from public listings (S1–S6) |
