# Annotation: the markup toolbar

- **Feature-map ids:** ANNO-1, ANNO-3, ANNO-4, ANNO-6, ANNO-7, ANNO-8, ANNO-12, ANNO-14, PAGE-7, NAV-1
- **Source:** observed in enScore on Windows by the project owner, 2026-10-31 (screenshots 24 and
  25, [index](screenshots/README.md#markup)), plus the owner's description of what ScoreCraft's
  markup should do
- **Observer:** Brinsleym

## What happens

1. Entering markup on a score shows one toolbar strip across the top of the reader. From left to
   right it holds:
   1. A **Draw with Touch** toggle (a hand icon with its label).
   2. A padlock button (the **touch lock**, below).
   3. The pen, shown selected (highlighted in an accent colour) by default.
   4. A highlighter.
   5. An eraser.
   6. A text tool (two letter "A"s of different sizes).
   7. A rectangle-selection tool (a dashed square).
   8. A ruler.
   9. Eight round colour swatches: black (selected, ringed in the accent colour), red, blue,
      green, amber, purple, brown and grey.
   10. A palette button that opens a custom colour picker.
   11. A slider that sets the width.
2. The owner's account of how it behaves and what ScoreCraft should do:
   - In markup, **only a pen draws** by default. A mouse (and a finger) doesn't mark up.
     **Draw with Touch** lets the mouse and touch draw ink too.
   - The rectangle tool draws a rectangle on the page. The rectangle is then used to **snip and
     paste** bars (Score Snips), to **create a shortcut** (Score Shortcuts), or to **move, copy or
     delete** the ink inside it.
   - The **padlock is a touch lock**: when it is on, every touch input is ignored. It can only be
     toggled while markup (ink) mode is on.
   - The **eraser** has *stroke* and *pixel* options, which appear in the bar when the eraser is
     selected.
   - The **width slider is per tool**: each tool keeps its own width.
   - The **eight swatches can be changed**. The palette button opens a fixed grid of extra colours
     (screenshot 25): six columns by five rows, roughly black, white and four greys; crimson,
     red, orange-red, orange, amber and yellow; lime, green, sea green, cyan, blue and indigo;
     violet, deep purple, peach, tan, brown and dark brown; then pastel pink, orange, yellow,
     green, blue and lavender. It offers only these colours, not a free colour picker. Clicking
     one replaces the **currently selected swatch** with that colour.

## Decisions taken from this note

- The **ruler** keeps its function, but its button is **hidden by default**; a setting shows it
  (owner, 2026-10-31, correcting an earlier "leave it out").
- The rectangle tool replaces the freehand lasso as the way to select ink.

## Open questions

- How enScore's ruler behaves: how it is placed, moved and turned, and how strokes follow it.
- Whether the eight swatches are shared by pen and highlighter or each tool has its own row.
