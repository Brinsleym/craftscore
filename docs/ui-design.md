# UI design system

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version) · **Target:** enScore 3.7 (2026)

ScoreCraft's look is its own. We match enScore's *conventions* where a musician's hands expect them
(tap to turn, pen to mark, the score filling the screen), never its visual identity, icons or copy
([clean-room §1](clean-room.md#1-what-we-copy-and-what-we-never-copy)).

## Principles

1. **The score is the interface.** In the reader, chrome hides by default; the page fills the
   screen. Controls appear on a tap in the top band and disappear on the next turn.
2. **Nothing moves under a performer.** No layout shifts, no pop-ups or toasts over the page while
   performing, no update prompts, no sync banners. Errors in performance mode wait in a
   non-blocking indicator until the performer leaves the reader.
3. **Reachable on a stand.** Touch targets ≥ 44 pt (48 dp on Android); primary reader actions within
   thumb reach at the screen edges; every action also has a key and a command id.
4. **Legible at arm's length.** Page filters never reduce contrast of notation below WCAG AA for
   text-equivalent content; ink colours are remapped per filter to stay visible.
5. **Calm by default.** Animations are short (≤ 150 ms), optional, and off in e-ink mode and when
   the OS asks for reduced motion.

## Themes and tokens

All colours, radii, spacings and font sizes come from `theme::Tokens` (`ui-egui/src/theme.rs`),
never hard-coded. Themes:

| Theme | Use |
|---|---|
| **Stage** (dark) | Default for the app chrome; the page keeps its own filter |
| **Studio** (light) | Daylight practice |
| **E-ink** | Pure black and white, no gradients or shadows, no animation, large targets |
| **High contrast** | Follows the OS setting |

Page filters (READ-7) are separate from the app theme: Original, Dark (inverted luminance with
hue kept), Sepia (with intensity), Inverted.

## Reader layout

- **Turn zones.** By default the right 60% of the page turns forward and the left 25% turns back,
  the top 12% band shows the toolbar (exact proportions to be fixed in `docs/spec/page-turning.md`
  from observation, and configurable).
- **Half-page turns.** A thin divider marks the split; the incoming half fades in no faster than
  the turn animation.
- **Toolbar.** Library, setlist position, layout, filters, markup, bookmarks, shortcuts, metronome,
  audio, more. Each button dispatches a command; the command palette (⌘K / Ctrl+K) lists every one.
- **Markup bar.** Appears when the pen touches or the markup button is tapped: pen, highlighter,
  whiteout, erasers, text, stamps, lasso, colours, widths, layer picker, undo/redo, done.

## Library layout

Grid of first-page thumbnails or a dense list; a sidebar of folders, setlists, composers, tags,
keys and Recently Deleted; search field always visible; multi-select for bulk metadata and moves.

## Widgets

Shared widgets in `ui-egui/src/widgets/` (buttons, icon buttons, sliders with numeric entry,
segmented controls, colour swatches, list rows with swipe actions). Icons are original or from a
permissively licensed set (for example Lucide, ISC), with the licence next to them and rows in
`ATTRIBUTION.md`.

## Localisation

Every string goes through `tl!` and `ui-egui/src/i18n/<lang>.tsv`. Translations are clean-room:
written from the meaning of our English text, never copied from another app's translations. Layouts
must survive 40% longer strings and right-to-left scripts.

## Automation for visual checks

`cargo run -p scorecraft-ui-egui --example snapshot` renders any UI state offscreen, and
`ui.screenshot` over the control channel captures the live app. Every UI PR attaches before/after
PNGs of public-domain scores.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
