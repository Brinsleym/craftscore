# All areas: public listings (2026-10-28)

- **Feature-map ids:** all rows of [`../feature-map.md`](../feature-map.md)
- **Source:** public store listings and partner pages, retrieved 2026-10-28 (links below)
- **Observer:** project documentation pass

This note summarises, in our own words, what enScore's public listings say it does. It is the
source for the first version of the feature map. Nothing here comes from the app itself.

## Sources

| # | Source | What it covers |
|---|---|---|
| S1 | [Apple App Store: enScore: Sheet Music Reader](https://apps.apple.com/us/app/enscore-sheet-music-reader/id6767764210) | Description, in-app purchases and prices, version history to 3.5.9, compatibility, languages, privacy labels |
| S2 | [Microsoft Store: enScore](https://apps.microsoft.com/detail/9nrrg2s6qb8v) | Windows listing, Windows Ink, build 2026.10.21.305 |
| S3 | [Google Play listing via AppBrain](https://www.appbrain.com/app/enscore-sheet-music-viewer/com.jonathanvardouniotis.enscore) | Android 3.5.15, free / Pro / Sync split, Chromebook side-by-side |
| S4 | [AirTurn app page](https://airturn.com/blogs/apps/enscore-sheet-music-viewer) | Page-turn inputs (tap, swipe, arrow keys, Bluetooth pedal), Pro feature list |
| S5 | [MWM app directory](https://mwm.ai/apps/enscore/6767764210) | Version 3.7.0 on Apple platforms, feature summary |
| S6 | [Amazon Appstore listing](https://www.amazon.com/Jonathan-Vardouniotis-enScore-Sheet-Reader/dp/B0HKD8K22P) | Fire / e-reader availability |
| S7 | enscore.app (developer site) | Linked from every listing; not reachable from our build environment on 2026-10-28, to be read in a later pass |

## What the listings say

**Platforms.** iPhone, iPad, Mac, Apple Vision, Windows, Android (phones, tablets, Chromebooks) and
e-readers, described as the same app everywhere. Apple minimums: iOS/iPadOS 16.4, macOS 26.0,
visionOS 1.0 (S1).

**Business model.** Free download with no ads. **Pro** is a one-time purchase (US$9.99 in S1)
unlocking setlists with page ranges and tempos, annotation layers, Score Snips, Score Shortcuts and
in-app IMSLP search. **Sync** is an optional subscription (US$1.99/month or US$14.99/year in S1)
for syncing and cloud backup. Reading, markup, setlists and scanning work with no account and no
network; an account is needed only for Sync and online title/composer detection (S1, S5).

**Reading.** Two-page, single-page and full-screen layouts. Half-page turns that show the next
system early, and alternating page turns for awkward breaks. Light, dark, sepia (with an intensity
setting) and inverted display filters, an Enhance filter for scans. Keeps the screen awake in
performance. Two scores side by side on tablets and desktops, multiple windows on Mac and iPad.
Landscape Page Zoom. Crop and margin adjustments that leave the original file unchanged (S1, S3,
S4).

**Page turning.** Tap, swipe, keyboard arrow keys, Bluetooth foot pedals, MIDI pedals and
controllers. Page turns can be recorded during audio playback so the score turns itself later
(S1, S4).

**Markup.** Pen, finger ("draw with touch"), mouse, Apple Pencil with pressure and pencil
shortcuts, Windows Ink with pressure. Highlighters, custom pen colours, whiteout (cover an existing
mark and write a new one), fingerings, typed text notes, stroke and pixel erasers. Annotation
layers that can be hidden, for example for a performance (Pro) (S1, S2, S4).

**Library.** Unlimited PDFs, imported from a file picker, other apps, the OS share sheet, Files /
File Explorer, Photos or whole folders (with folder-level bulk add). Folders, composer, tags and
key, all searchable, plus sorting. Full-text search inside PDFs and practice notes. Automatic title
and composer detection from generic scan names, including whole folders (online, needs an account).
Bookmarks, a Recently Deleted folder, Verify & Repair Library (S1, S3).

**Page editing.** Reorder, rotate, hide and delete pages. Crop and straighten scans. Blank
manuscript paper (single staff, grand staff, tab) in any page size. Score Snips: copy and paste
bars, for example to show bars from the previous page after a turn (Pro). Page Range per score
(S1, S3, S5).

**Navigation.** Score Shortcuts: tappable links in a score for multi-page repeats and D.S. al Coda,
created from the viewer menu (Pro). Bookmarks (S1, S4).

**Setlists (Pro).** Lists for practice, lessons and concerts; each entry has a page range and a
tempo; pieces flow into one another; a whole setlist can be shared as one file with PDFs, order,
page ranges and markings (S1, S3).

**Audio.** Attach a recording to a score, play at any speed, loop a passage, transpose by semitones
(S1).

**MIDI.** Turn pages from MIDI. Send program change, bank select and Song Select when a score opens
(S1).

**Practice.** Metronome with tap tempo, subdivisions, beats per bar and nine sounds. A tuner.
Practice log of minutes, streaks and which pieces were played when, with daily goals, pausing,
notes inside scores, a shareable PDF report and a backup of the log (S1).

**Import and scanning.** Camera scanning with straightening, cropping and clean-up; Wi-Fi flatbed
and feeder scanners; IMSLP search and import (Pro); forScore migration through a `.4sc` file that
keeps pages, ink, stamps and layers (S1, S4).

**Export.** Print, export PDFs with ink (and layers) included, PDF practice reports, shared
setlist files (S1, S4).

**Sync.** Scores, markings, lists and bookmarks across signed-in devices; cloud backup; background
and faster first downloads on a new device. Scores live on the device and the cloud holds a copy;
deleting the account removes server copies and keeps local ones (S1). A public review asks for
OneDrive sync (S3).

**Languages.** Listings say twelve interface languages; S1 lists English, Dutch, French, German,
Greek, Italian, Japanese, Korean, Polish, Portuguese (incl. Brazilian), Russian, Simplified
Chinese, Spanish, Swedish and Traditional Chinese.

## Open questions (need observation or the developer site)

- Exact tap-zone layout and whether it is configurable.
- Half-page turn split: fixed 50% or per-page adjustable?
- Which pedal key codes are accepted by default (arrow keys, page up/down, space)?
- How Score Snips are placed and turned off.
- Metronome sound set and visual pulse.
- Tuner range, reference pitch setting, temperaments.
- `.4sc` contents beyond "pages, ink, stamps and layers".
- Behaviour on e-ink displays (refresh, animation).
