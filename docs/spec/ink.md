# Ink and markup

> **Last reviewed:** 2026-10-31 · **Last updated:** 2026-10-31 · **Change:** minor (owner's direction on the toolbar; earlier: first version) · **Target:** enScore 3.7 (2026)

How the player marks up a score: pens, highlighters, whiteout, erasers, text notes, stamps,
layers, selection, shapes, and exporting the result. Written for milestone M4.

## Sources

- Public listings S1, S2, S4 and S8: [../observations/public-listings-2026-10-28.md](../observations/public-listings-2026-10-28.md)
  ("Markup", "Export", the S8 notes on highlighters and tiers). They name the capabilities: pen,
  finger ("draw with touch") and mouse input; Apple Pencil and Windows Ink with pressure; pencil
  shortcuts; highlighters that blend with the page like real ones; custom pen colours; whiteout
  that covers a mark so a new one can be written over it; fingerings; typed text notes; stroke
  and pixel erasers; annotation layers that can be hidden for a performance; PDFs exported with
  ink and layers.
- Observation: [../observations/anno-markup-toolbar.md](../observations/anno-markup-toolbar.md),
  the markup toolbar on Windows and the owner's direction for ScoreCraft's markup (2026-10-31):
  pen-only drawing by default with a *Draw with Touch* switch, a touch lock, the tool set, eight
  changeable swatches with a fixed palette of extra colours (screenshot 25), a width slider per
  tool, eraser options, a rectangle tool for snips, shortcuts and moving ink, and a ruler whose
  button is hidden unless a setting shows it.
- Platform documentation on palm rejection: Windows' *Ignore touch input when I'm using my pen*
  setting ([guide](https://geekrewind.com/how-to-enable-or-disable-ignore-touch-input-when-using-pen-in-windows-11/),
  [Microsoft hardware docs](https://learn.microsoft.com/vi-vn/windows-hardware/customize/desktop/pen-and-ink));
  Android's palm flags ([Stylus palm rejection](https://developer.android.com/develop/adaptive-apps/cookbook/stylus-palm-rejection)).
- Standard: ISO 32000-1 §12.5 (annotations: `/Ink`, `/FreeText`, `/Square`, `/Line`, appearance
  streams) and §11.3.5 (blend modes: `Multiply`) for export (EXP-2).
- Standard: SMuFL 1.4 (Standard Music Font Layout), for the code points stamps use (ANNO-9).
- Architecture rules: annotations are stored in the unrotated source page's PDF user space
  ([architecture §1.1](../architecture.md#11-rules-we-adopt-from-day-one), §5), so they survive
  page edits, and the imported file is never rewritten (AGENTS.md rule 3).
- The listings don't say how these tools behave in detail. Where they are silent, this spec picks
  behaviour that suits a player marking a part on a stand, and lists the choice under
  [Open questions](#open-questions) for an observation pass ([G17](../gaps.md#g17-unknowns-behaviour-not-yet-observed)).

## Terms

- **Markup mode**: the reader with the markup toolbar shown. Outside it, pointers turn and pan
  pages as in [page-turning](page-turning.md) and nothing draws.
- **Tool**: what a pen does on the page in markup mode. Exactly one tool is active: pen,
  highlighter, eraser, text or rectangle select. The ruler is a guide shown alongside them.
- **Stroke**: the samples one pen-down to pen-up gesture produced, with a tool, a colour and a
  nominal width.
- **Page space**: points on the source page, y up, before the score's own rotation or crop.
  Everything placed on a page is kept in page space.
- **Active layer**: the layer new markings go into.

## Requirements

### The markup toolbar

- Entering markup mode shows one toolbar, in this order: *Draw with Touch*, touch lock, pen,
  highlighter, eraser, text, rectangle select, ruler (only when the *Show ruler button* setting is
  on; it is off by default); then eight colour swatches, a palette button and a width slider. The
  active tool and the selected swatch are marked.
- With the eraser selected, its *stroke* and *pixel* options appear in the bar next to it.
- The pen is active the first time; afterwards markup opens with the last tool, colours, widths
  and swatches.
- Leaving markup mode (or turning the page with a pedal) keeps every marking and the tool
  settings. Page turns by pedal and keys keep working in markup mode.

### ANNO-1 Pen

- With the pen active, a pen drawn across a page leaves ink along its path (a mouse or finger
  too when *Draw with Touch* is on, ANNO-14). Ink is drawn while the pointer moves; it doesn't wait for the pointer to lift.
- One pen-down to pen-up is one stroke and one undo step.
- A stroke that starts on a page belongs to that page. Samples that leave the page are kept (ink
  may run into the margin) but a stroke never crosses onto another page.
- A tap without movement leaves a dot as wide as the pen.
- Ink stays sharp at every zoom: it is kept as geometry, not as pixels.
- Smoothing removes the jitter of a hand-held stylus but keeps the corners of handwritten
  fingerings and accents: a direction change sharper than 60° is not rounded off.
- Samples closer than 0.1 pt to the previous one are dropped. A stroke holds at most 10,000
  samples; a longer gesture ends the stroke and continues as a new one.

### ANNO-2 Pressure

- Where the device reports pressure, the stroke is wider where the pen pressed harder. Width at a
  sample is `width × (floor + (1 − floor) × pressure^γ)` with floor 0.25 and γ 0.6 by default, so a
  light touch still leaves a visible line. Devices without pressure draw at full width.
- The width changes smoothly along the stroke; the first sample of a stroke doesn't show a blob or
  a spike when the device's first pressure reading is low.
- Highlighter and whiteout ignore pressure: their width is constant.

### ANNO-3 Highlighter

- Draws a translucent, wide, constant-width band with flat ends.
- Blends like a real highlighter: it darkens the page by its colour (multiply), so black notation
  under it stays black and the paper takes the colour. It never covers notation.
- No build-up within a stroke: where one stroke crosses itself, the colour is the same as where it
  doesn't. Two separate strokes over the same place do darken it again, as two passes would.
- Default: yellow at 40% (`#ffe60066`), 12 pt wide. A *stronger* highlighter is the same tool at a
  higher alpha.

### ANNO-4 Colours, widths and presets

- Eight swatches, black, red, blue, green, amber, purple, brown and grey to start with. Tapping a
  swatch selects it and gives the active tool its colour.
- The swatches can be changed. The palette button opens a fixed grid of 30 colours (six columns,
  five rows: black, white and greys; reds, oranges and yellows; greens, cyan, blue and indigo;
  purples and browns; pastels). It offers only these colours, not a free colour picker. Picking
  one replaces the **selected swatch** with it and gives the active tool that colour. The swatches
  are kept between launches. ScoreCraft chooses its own values for every colour.
- The width slider is per tool: it shows and sets the active tool's own width: pen 0.25 to 12 pt,
  highlighter 4 to 40 pt, eraser 2 to 60 pt.
- Pen, highlighter and whiteout each keep their own colour and width (picking red for the pen
  doesn't change the highlighter); changing them affects new strokes only. The highlighter takes a
  swatch's hue at its own translucency.
- A player can save the current tool, colour and width as a named preset, recall it in one action,
  and delete it. Presets are kept between launches.

### ANNO-5 Whiteout

- Whiteout is the pen in white: picking the palette's white (or a swatch set to it) for the pen
  makes it paint whiteout. The toolbar has no separate whiteout button. Choosing another colour
  returns the pen to ink.
- Paints an opaque band in the page's paper colour (white on a normal page) that covers printed
  marks under it. Pen strokes drawn after it, on the same or a higher layer, show on top of it.
- Under a display filter (dark, sepia) whiteout shows in that filter's paper colour, so it still
  reads as blank paper.
- Default width 10 pt.

### ANNO-6 Stroke eraser

- The toolbar has one eraser with two modes, *stroke* (the default) and *pixel* (ANNO-7), chosen
  from the options that appear when the eraser is selected. The eraser keeps the mode it was last
  used in.
- With the eraser in stroke mode, every stroke the eraser touches is removed whole. The eraser is a
  disc 8 pt across in page space; a stroke is touched when any part of its drawn shape is within
  the disc.
- One eraser gesture is one undo step, however many strokes it removed. A gesture that touched
  nothing records nothing.
- Only strokes on visible, unlocked layers are erased. Text notes and stamps are not erased by
  either eraser (they are deleted by selecting them, ANNO-12).

### ANNO-7 Pixel eraser

- With the eraser in pixel mode, erases only the part of a stroke under the eraser disc (default 8
  pt across). What remains of the
  stroke becomes one or more new strokes, each keeping the original's tool, colour, width and
  pressure along its remaining samples. A stroke erased entirely is removed.
- One gesture is one undo step; undo restores the original stroke.

### ANNO-8 Text notes

- A text note is a box on a page holding typed text, with a font size (6–144 pt), a colour, bold
  and italic, and an optional filled background.
- Placing a note puts its top-left corner where the player tapped; the box grows to fit the text.
- Notes can be edited (text and style), moved and deleted. Each is one undo step.
- Empty text is not a note: finishing an edit with no text deletes it.

### ANNO-9 Stamps

- A stamp is a music symbol placed on a page: fingerings 0–5, dynamics (pp, p, mp, mf, f, ff, sfz,
  fp), bowings (up bow, down bow), breath mark, caesura, fermata, accent, staccato, tenuto,
  sharp, flat, natural, and eyeglasses ("watch").
- Symbols come from a SMuFL font under the OFL ([fonts](../standards/fonts.md)); without that font a
  built-in fallback drawing is used, so stamps always show.
- Placing a stamp centres it where the player tapped, at a size the player picks (8–96 pt); it can
  be moved, resized, recoloured and deleted like any selection (ANNO-12).

### ANNO-10 Layers

- Every score has at least one layer. A new score has one, "Layer 1", which is active.
- Layers can be added, renamed, reordered, shown or hidden, locked or unlocked, and deleted
  (deleting the last layer is refused; deleting a layer deletes its markings, as one undo step).
- Layers draw bottom first, in their list order; within a layer, markings draw in the order they
  were made.
- Hidden layers are not drawn in the reader and can be left out of an export. A locked layer is
  drawn but can't be drawn on, erased or edited.
- The active layer can't be hidden or locked while the player draws; drawing on a hidden or locked
  active layer is refused with an error the player sees, never silently dropped.

### ANNO-11 Pen shortcuts

- A stylus barrel button or double-tap (where the platform reports one) switches to a tool the
  player chooses (default: the stroke eraser) and switching again returns to the previous tool. The
  eraser end of a stylus always erases with the stroke eraser.

### ANNO-12 Rectangle select: move, copy, delete; snips and shortcuts

- The rectangle tool drags out a rectangle on one page. When the drag ends, the player chooses
  what to do with it:
  - **Move**, **copy** or **delete** the ink inside it: every marking whose shape lies mostly
    inside the rectangle (more than half its samples, or its box's centre for notes and stamps).
    Moving drags the markings; copying leaves the originals and places copies (new ids) where the
    player drops them, on the same page or another. A selection can also be scaled about its
    centre (0.1× to 10×) and recoloured. Each action is one undo step.
  - **Snip** (PAGE-7): the rectangle's bars are copied as a Score Snip and pasted where the player
    taps, typically at the bottom of the page before a turn.
  - **Shortcut** (NAV-1): the rectangle becomes a Score Shortcut; the player then picks the page
    (and optionally the place) it jumps to.
- An empty selection offers only snip and shortcut. Snips and shortcuts are specified with
  setlists (M6); the rectangle tool is their way in.

### ANNO-13 Undo and redo

- Every markup step above is undoable and redoable, in the score's own history (shared with its
  page edits). Undo during a stroke cancels the stroke in progress.

### ANNO-14 Pen only and Draw with Touch (palm rejection)

- By default only a pen draws in markup mode. A mouse, and a finger or palm on a touch screen,
  never mark up: they turn, pan and zoom as outside markup mode, so a resting hand leaves no ink.
- The *Draw with Touch* switch lets the mouse and touch draw (and erase, write text and select)
  like a pen, at full width since they have no pressure. It is off by default and kept between
  launches.
- **Palm rejection while the pen draws.** While a pen is touching the screen, every touch is
  ignored: a hand leaning on the screen never turns, pans, zooms or (with *Draw with Touch*)
  draws. In detail:
  - a touch that starts while the pen is down is ignored until that finger lifts, even if the pen
    lifts first;
  - a touch gesture already in progress when the pen goes down is cancelled: a pending tap never
    turns the page, and a pan or pinch stops where it is;
  - when no pen is touching, touch works as normal: tapping and swiping turn pages, two fingers
    pinch to zoom and pan, so using the pen and then zooming with two fingers needs no switch;
  - the mouse is never rejected;
  - a touch the platform marks as a palm or cancels (Android 13 and later flag palm touches as
    cancelled) is dropped as if it never happened, and any ink or gesture it started is undone.
- **Touch lock** (the padlock): while it is on, every touch is ignored on the page: no drawing,
  erasing, turning, panning or zooming by touch. The pen, the mouse, keys and pedals keep working.
  It can be switched only in markup mode and applies only there; leaving markup mode lets touch
  turn pages again, and the lock is as the player left it next time. It is off by default.
- The platforms help but can't be relied on alone: Windows has an *Ignore touch input when I'm
  using my pen* setting (Pen & Windows Ink) that suppresses touch while the pen is near the screen,
  but the user can turn it off and some devices hide it; Android only marks palm touches, so the
  app has to drop them. ScoreCraft's own rule above works either way and doesn't fight the
  platform's.

### ANNO-17 Ruler

- The ruler is a straight edge laid over the page to rule lines against (bar lines, slurs, a
  cut). Its toolbar button is hidden by default and appears when the *Show ruler button* setting
  is on; the ruler works the same either way (an agent can show it by command).
- Showing it places it across the middle of the page on screen, horizontal. Dragging it (one
  finger, the mouse, or the pen on the ruler body) moves it; two fingers turn it. Its angle is
  shown while it turns and snaps to 0°, 45° and 90° within 2°.
- While it is shown, a pen or highlighter stroke that starts within 12 pt of the ruler's edge is
  drawn straight along that edge, from where it started to where the pen's movement along the
  edge ended. Strokes that start further away are drawn freehand.
- The ruler is a guide, not a marking: it is never stored in the score or exported. Hiding it, or
  leaving markup mode, removes it from the screen.

### ANNO-15 The file is never changed

- No markup action writes to the imported PDF. Markings live in the library, keyed by source page,
  in page space.
- Markings stay on their place in the music when the page is rotated, cropped, moved, hidden and
  shown again, or when the page range changes. Deleting a page from the score keeps its markings
  in the library until the score itself is deleted.

### ANNO-16 Shapes

- With the shape tool, a stroke that is nearly straight becomes a straight line, and a closed
  stroke that is nearly round becomes an ellipse; anything else stays freehand. The result is a
  stroke like any other.

### EXP-2 Export with ink

- Exporting writes a new PDF; the imported file is untouched. The player chooses which layers go
  in (by default, the visible ones) and how:
  - **flattened**: markings drawn into the page content, so every reader shows them the same;
  - **annotations**: pen and highlighter strokes as `/Ink` annotations (highlighter with a
    `Multiply` blend in its appearance), text notes as `/FreeText`, stamps as `/Stamp`, each with an
    appearance stream so readers that ignore annotation dictionaries still draw them.
- Page order, rotation, crop, hidden pages and the page range of the score are applied to the
  exported pages.

### Drawing and colours under filters

- Ink keeps its own colour under sepia. Under dark, ink darker than mid grey is drawn light (its
  lightness inverted, hue kept) so a black pen stays legible on a dark page; highlighter colours
  keep their hue.

### Speed (P5)

- Pen-down to first visible ink within one display frame where the platform allows it, with
  predicted samples drawn ahead and replaced by real ones (perf scenario P5).
- When a stroke is committed, the page shows the committed ink in place of the live stroke with no
  frame in between showing neither.
- Committing, erasing and undoing redraw only the page they changed; turning pages never waits on
  ink.

## Acceptance tests

| Id | Given | When | Then |
|---|---|---|---|
| INK-1 | An open library score, pen active | a stroke is drawn across page 1 | page 1 has one stroke on the active layer, in page space, and one undo step |
| INK-2 | INK-1 | the player undoes | the stroke is gone; redo brings it back unchanged |
| INK-3 | Page 1 rotated 90° | a stroke is drawn from the screen's top left to bottom right | the stroke's samples, mapped through the page's rotation, land where they were drawn; rotating back to 0° keeps them on the same notes |
| INK-4 | A stroke with pressures 0.1 then 1.0 | its outline is built | it is narrower at the start than the end, and never narrower than 25% of the width |
| INK-5 | A highlighter stroke that crosses itself over white paper | the page is composed | the crossing pixel has the same colour as a pixel the stroke covers once |
| INK-6 | A highlighter over black notation | the page is composed | the notation pixel stays black |
| INK-7 | Whiteout over notation, then a pen stroke over it | the page is composed | the notation is covered by paper colour and the pen shows on top |
| INK-8 | Three strokes, the eraser crosses two | one stroke-eraser gesture | two strokes are removed in one undo step |
| INK-9 | A straight stroke | the pixel eraser crosses its middle | two strokes remain, one on each side, with the original's style; undo restores the one |
| INK-10 | The active layer is locked | a stroke is drawn | it is refused with an error, and nothing changes |
| INK-11 | A hidden layer with a stroke | the page is composed | the stroke is not drawn |
| INK-12 | A stroke on page 2 | page 2 is moved to the end of the score | the stroke is still on that page and on the same notes |
| INK-13 | A stroke of 12,000 samples | it is submitted | it is refused (UI splits long strokes before 10,000) |
| INK-14 | A dark filter and a black pen stroke | the page is composed | the stroke is drawn light |
| INK-15 | A sharp 90° corner drawn with jitter | it is smoothed | the corner survives within 1 pt |
| INK-16 | Any markup command | it gets adversarial params | it returns an error, never panics (`panic_hunt`) |
| INK-17 | Markup mode, *Draw with Touch* off | a stroke arrives from a mouse or touch | nothing is drawn; the same stroke from a pen draws |
| INK-18 | *Draw with Touch* on | a stroke arrives from a mouse | it draws at full width |
| INK-20 | The pen is drawing | a touch starts, moves and lifts | nothing turns, pans or zooms; the stroke is unaffected |
| INK-21 | A one-finger touch is down (no pen) | the pen touches the screen | the touch is cancelled: lifting it doesn't turn the page |
| INK-22 | No pen is touching | two fingers pinch | the page zooms, as outside markup mode |
| INK-23 | The platform cancels a touch as a palm | it had started a stroke (*Draw with Touch* on) | the stroke is removed |
| INK-24 | Markup mode, touch lock on | a touch taps the right third, pinches, or (with *Draw with Touch*) draws | nothing happens; a pen still draws |
| INK-25 | The second swatch selected | the player picks the palette's lavender | the second swatch becomes lavender and the active tool draws in it |
| INK-26 | The palette | a colour that isn't one of its 30 is asked for | it is refused |
| INK-27 | The ruler shown horizontally across the page | a pen stroke starts 5 pt from its edge and wobbles along it | the stored stroke is a straight line along the edge |
| INK-28 | The pen with white picked | a stroke is drawn | it is a whiteout stroke |
| INK-19 | Pen red, highlighter yellow | the player picks blue with the highlighter active | new highlighter strokes are blue at the highlighter's translucency; the pen is still red |

INK-27 comes with the ruler. INK-20 to INK-23 are `scorecraft-input` tests of the palm filter the shell routes pointers through. INK-1 to INK-3, INK-8 to INK-13, INK-16 to INK-19, INK-24 to INK-26 and INK-28 are engine tests
(`crates/engine/src/annotate_cmds.rs`, `crates/engine/tests/panic_hunt.rs`); INK-4 and INK-15 are
`scorecraft-ink` tests; INK-5 to INK-7, INK-11 and INK-14 are `scorecraft-render` tests. Text,
stamps, layers, selection, shapes and export get their acceptance tests when their part of M4
lands, and are added to this table.

## Not in scope

- Score Snips (PAGE-7) and Score Shortcuts (NAV-1): they are annotations in the model but belong to
  M6 with setlists.
- Importing ink from `.4sc` (IMP-5) and from PDF annotations (IMP-7): M9.
- Handwriting recognition and tilt-shaded pencil strokes: not in the public listings.
- Syncing markings between devices: M8 (the model already stamps every marking with a hybrid
  logical clock).

## Open questions

- Does enScore's highlighter build up where two strokes overlap, and does it have flat or round
  ends? (S8 says only that it blends "like real ones".)
- Eraser size and whether it is adjustable; whether the pixel eraser leaves the stroke's pressure
  profile intact.
- Which symbols the stamp palette offers by default, and whether custom stamps can be made from a
  selection (the feature map's ANNO-9 wording).
- Whether rejecting touches for a moment after the pen lifts (a hand landing as the pen comes
  off) is worth the delay it adds to a deliberate tap; ScoreCraft starts without such a delay.
- How enScore's ruler is placed, moved and turned, and how strokes follow it; ANNO-17 is
  ScoreCraft's own design until observed.
- Whether pen and highlighter share the eight swatches or each has its own row; ScoreCraft shares
  one row, each tool remembering its own selected swatch.
- What the stylus shortcut switches to by default.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-31 | minor | Owner's corrections: the padlock is a touch lock; the ruler stays (ANNO-17) with its button hidden by default; eraser options in the bar; width per tool; changeable swatches with a fixed 30-colour palette that replaces the selected swatch; whiteout is the pen in white |
| 2026-10-31 | minor | Owner's direction: palm rejection while the pen draws, without disturbing taps and two-finger zoom; platform behaviour noted |
| 2026-10-31 | minor | Owner's direction: pen-only by default with *Draw with Touch*, the toolbar, eight swatches, custom colour and width slider, one eraser with two modes, rectangle select (move, copy, delete, snip, shortcut) replaces the lasso, no ruler |
| 2026-10-31 | major | First version for M4: pen, pressure, highlighter, whiteout, erasers, text notes, stamps, layers, presets, selection, shapes, pen shortcuts, pen only, export with ink |
