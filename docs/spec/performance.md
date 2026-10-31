# Performance: setlists on stage, shortcuts, snips and the metronome

> **Last reviewed:** 2026-10-31 · **Last updated:** 2026-10-31 · **Change:** patch (M6 part 2: metronome; earlier: first version) · **Target:** enScore 3.7 (2026)

What a musician needs on stage, beyond reading one score: playing a setlist from piece to piece,
jumping at repeats and codas, seeing the bars after a turn before it, and a metronome. Written
for milestone M6, which closes the [alpha gate](../roadmap.md#alpha-gate). Setlists as data
(create, reorder, page ranges, tempos, placeholders) are M1's (LIST-1..3, LIST-7).

## Sources

- Public listings S1, S2, S3, S4 and S8:
  [../observations/public-listings-2026-10-28.md](../observations/public-listings-2026-10-28.md)
  ("Lists", "Score Shortcuts", "Score Snips", "Practice"). They name lists whose pieces flow into
  one another with the screen kept awake, tappable shortcuts on repeat and coda signs, snips of
  the bars after a turn pasted before it, and a metronome with tap tempo. How each behaves in
  detail is ScoreCraft's choice here, listed under [Open questions](#open-questions).

## Requirements

### LIST-4 Playing a list

- Playing a list opens its first entry's score (or the entry the player starts from) at the
  first page of its page range. Turning past the last page opens the next entry at its first
  page; turning back from the first page opens the previous entry at its last page. Pedals,
  keys, taps and swipes all turn this way.
- Breaks (placeholders, LIST-7) are passed over by turns; the turn that passes one reports its
  label so the reader can show it briefly. Entries whose score is in the trash or gone are passed
  over too.
- While a list plays, each entry shows only its page range (LIST-2); stopping shows the whole
  score again. Opening another score by hand stops the list.
- The screen stays awake from the first entry to the end (READ-9 applies throughout, as a score
  is always open).
- The reader can show where the player is: the list's name, entry *n* of *m*, the score's title,
  its tempo (LIST-3) and the next score's title.

### LIST-6 One score, many lists

- A list entry refers to a library score; it never copies it. The same score may be in any
  number of lists, and more than once in one. Editing the score (metadata, pages, markings)
  shows in every list; deleting a list never deletes a score.

### IMP-6 Import report

- Every import reports what it did: how many scores were imported, how many files were already
  in the library (the same bytes) and skipped, how many were not PDFs, and each file that
  couldn't be read with the reason (the first five named, the rest counted).

### PRAC-1 Metronome with tap tempo

- A metronome at 20–300 beats per minute, in 2, 3, 4 or 6, with an accent on the first beat, as
  a click and a light pulse along the top edge of the reader (brighter, in the accent colour, on
  the first beat of the bar, fading within the first 40% of the beat). The pulse is in the app
  (M6 part 2); the click waits on audio output (`scorecraft-audio`), which the app doesn't have
  yet. Changing the tempo while it runs keeps the beat count going. Tapping the tempo button four times sets
  the tempo from the average of the last three intervals; taps more than 2 s apart start again.
  A playing list's entry tempo sets it when the entry opens.

### NAV-1 Score Shortcuts (M6 part 3)

- A shortcut is a rectangle on a page (drawn with the rectangle tool, ANNO-12) that jumps to a
  target page, and optionally a part of it, when tapped. It is drawn as a faint outline in markup
  mode and invisible otherwise; tapping it in the reader jumps instead of turning.

### PAGE-7 Score Snips (M6 part 3)

- A snip copies a rectangle of one page and shows it at a place on another, typically the first
  bars of the next page at the foot of this one, so the turn can wait. It is drawn into the page
  like ink, on its own layer, and can be moved, resized and deleted.

## Acceptance tests

| Id | Given | When | Then |
|---|---|---|---|
| PERF-1 | A list: score A (2 pages), a break "Interval", score B (3 pages) | it is played and the pedal pressed on A's last page, then the player turns back | B opens at page 1 past the interval; back opens A at page 2 and reports "Interval"; past B's end nothing turns |
| PERF-2 | An entry with pages 2–3 of a 5-page score | the list is played, then stopped | the reader shows 2 pages, then all 5 |
| PERF-3 | One score twice in each of two lists | the score is renamed and a list deleted | both lists show the new title; the library still has one score |
| PERF-4 | An empty list; a list with one entry | the first is played; the second from entry 5; then a score opened by hand | refused; refused; the list stops |
| PERF-6 | 120 BPM in 3, started at t = 1 s | the beat is asked for at 1.0 s, 1.75 s and 2.5 s | beat 1 (accented, next at 1.5 s); beat 2 half way; beat 1 of the next bar |
| PERF-7 | Taps at 0, 500, 1000, 1500 ms, then 2100, then 5000 | each tap | 120 BPM at the fourth; the fifth averages the last three intervals; after the pause, nothing until four more |
| PERF-8 | The engine's clock | the metronome is set to 120 in 3, started, read 1.1 s later, then tapped every 400 ms | the third beat of the bar; 150 BPM; out-of-range tempos and meters refused |
| PERF-5 | An import of two new PDFs, one already imported, three other files and one broken PDF | it finishes | "Imported 2 scores; 1 already in the library; 3 files not PDFs; 1 couldn't be read (broken.pdf: …)." |

PERF-1 to PERF-4 are engine tests (`crates/engine/src/play_cmds/tests.rs`); PERF-5 is a shell
test (`crates/ui-egui/src/app.rs`); PERF-6 and PERF-7 are `scorecraft-practice` tests, PERF-8 an
engine test (`crates/engine/src/metronome_cmds.rs`). Shortcuts and snips get theirs when their
part lands.

## Open questions

- Whether enScore stops at a placeholder (an interval screen) or passes it; ScoreCraft passes it
  and reports it.
- Whether enScore's list entry ranges hide the other pages entirely or only start and end there.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-31 | patch | M6 part 2: the metronome's pulse and timing, PERF-6..8 |
| 2026-10-31 | major | First version for M6: playing a list, one score in many lists, the import report; the metronome, shortcuts and snips outlined for parts 2 and 3; PERF-1..5 |
