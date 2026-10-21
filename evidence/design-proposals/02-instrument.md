# Calix — L'Instrument

*A package manager as a precision instrument. Everything is a readout; nothing is decorative.*

The image: one desktop screen, 1600 × 1000, dark. The header is the instrument's face.
The left column is the channel bank and the delta gauge. The centre is the readout.
The right is the detail panel. The bottom band is the console.

---

## 1. The navigation I reasoned out

**Two selectors, orthogonal, never merged.** The **channel** is *where a package comes
from* — a vertical bank of the seven sources on the left, with a needle on the selected
one. The **function** is *what you are doing to it* — a three-position rocker above the
readout: `SEARCH`, `INVENTORY`, `DELTA`. The panel is the same object in every state;
only the reading changes. The two axes are never allowed to share a control: no list
ever contains a source and an operation in the same menu, because the moment they mix,
"update" and "from the AUR" become indistinguishable.

**The channel is the primary axis, and it is the only thing that moves.** A function is
applied to the *selected channel*, so the readout always answers "what does ALPM see
through SEARCH?". The delta is therefore not a global list — it is a **gauge attached to
the channel**: a chart-recorder trace of ALPM's update set over the last 24 hours, with
the count as a figure beside it. The update set is a *measurement of a channel*, not a
list that has forgotten where it came from.

**The class tag carries the non-interchangeability.** Each channel is tagged `BIN`
(ALPM, AppImage, Tracked), `SRC` (AUR, Cargo, NPM — a recipe, not a binary) or `SBX`
(Flatpak). That single tag is the difference that matters: a recipe can be *built*,
a prebuilt binary cannot. The tag is what lets the instrument refuse the wrong action
before the user asks.

### Where am I?

Four places, always in the same place:

| | where |
|---|---|
| **the source** | the needle on `ALPM` in the channel bank; the readout window's `CHANNEL` cell |
| **the operation** | the lit segment of the function rocker; the readout window's `FUNCTION` cell |
| **the filter** | the readout window's `FILTER` cell and the filter field itself |
| **the state of the list** | `14 MATCHES · SHOWING 1–8 OF 14`, the scrollbar's phosphor edge, and the freshness block |

The header's readout window is a recessed, milled LCD: three cells, `CHANNEL`,
`FUNCTION`, `FILTER`. The live value is the only phosphor in it. The header also carries
the two facts an instrument must never hide — **when the answer was read**
(`READ 14:32:07`, `AGE 2m 41s`, always in the same place, always in the same typeface)
and the size of the corpus (`173,306 PKG · 7 SOURCES`, the exact sum of the seven
channel counts).

### What can I do here?

Three mechanisms, all keyed to the *current* channel:

- **`CAPABILITIES · ALPM`** in the left column: the channel's actions as physical keys —
  `INSTALL`, `UPDATE`, `REMOVE`, `FILES` — and, stated in words, the one that is *not*
  available: `NO BUILD RECIPE — REPO BINARY`. Absence is drawn, not greyed out.
- **The class tag on every channel**, with the legend `BIN PREBUILT · SRC RECIPE ·
  SBX SANDBOX`, so the user can see which channels are buildable before switching to one.
- **The `APPLY` row in the detail panel**, carrying only what can be done to the
  selected package, plus the exact command that would run
  (`sudo pacman -Syu --needed ripgrep`) so the operation is legible before it is armed.

### How do I get elsewhere?

- **A keyboard that covers the whole application, always visible.** Twelve keys, drawn
  as keycaps on a plate in the console: `⌘1`–`⌘3` the three functions, `⌘4`/`⌘5` the
  detail panel and the console, `⌥←`/`⌥→` the channels, `⌘F` the filter, `⌘L` the list,
  `⌘⌥C` copy the command, `⌘D` the density, `⌘R` re-read, `ESC` back. Nothing in the
  application is reachable only by mouse.
- **`ALSO IN` — AUR · FLATPAK · APPIMAGE.** The other channels where the selected
  package also exists, in the one colour reserved for *elsewhere*. They are the way out
  of the current channel without hunting.
- **The header readout** is the way back: it always names the state you are in, and the
  channel bank is always on the left, so the route back to a known point is visible.

### What is the state of the list?

Count (`14 MATCHES`), position (`SHOWING 1–8 OF 14`), the filters in force (the filter
field carries the query; the filter is a *reading*, not a hidden mode), the density in
force (`DETAILED` lit, `COMPACT` dim but present, with a three-line glyph of the
compact form and the note `COMPACT = 1 LINE / PKG`), and the freshness of the whole
answer (`AGE 2m 41s`).

---

## 2. The material

**Machined, anodised, engraved — and every surface is a plate, not a rectangle.**

- The chassis is *darker* than the ground; the panels are plates raised out of it. A
  plate carries a 1px light along its top edge, a dark line along its bottom edge, a
  soft light falling from the top, and a drop shadow. Value hierarchy follows
  attention: channel bank (darkest) → readout → detail panel (brightest).
- **Recesses are cut, not laid on.** The readout window, the filter field, the trace
  plate and the command strip are darker with an inner shadow at the top edge. The
  alternation raised / cut / raised is what makes the screen read as a machined
  instrument rather than a stack of cards.
- **The grid is visible, faintly, behind the rows.** The list's column rules sit behind
  the type; the figures are aligned on them. The trace has its own grid. It is the
  texture you notice only when you look for it.
- **The figures are monospaced.** IBM Plex Mono for every identifier, version, size,
  count, time and command; IBM Plex Sans for prose and for the engraved section labels
  (`CHANNEL`, `DELTA · LIVE`, `DEPENDENCIES`, `FILES`, `ALSO IN`, `COMMAND`, `APPLY`).
  No figure is proportional, so columns line up on the decimal.
- **The accent is a phosphor, used the way a phosphor is used: only where the reading
  is live.** The selected channel, the selected function, the packages with an update
  available, the delta, the live head of the trace, the operation in flight, the two
  cursors, the `RE-READ` pip. Everything else is shell white, muted, or slate.
- **Royal blue is not a second accent — it is the accent for *elsewhere*.** It is used
  exactly once in the design: the `ALSO IN` chips, the other channels where the
  selected package exists. One meaning, one colour, and a direct answer to "how do I
  get elsewhere".

### The mark

A scallop shell as the cup of a chalice: nine ribs fanned from the hinge, the fan in
signal orange with the shadow side in signal orange 700 and a rim light in signal
orange 100 along the top-left edge; the hinge sits in a brushed-aluminium nacelle; below
it the knop, the stem and the foot, all aluminium with a cylindrical highlight and fine
brushing. It is **76 × 89** — present, not a glyph in a corner — and it is a vector
(imported from SVG), so the ribs are editable paths in the source, not a raster. The
name is `CALIX`, alone: no framework, no version, no subtitle.

---

## 3. The motion

**Nothing moves except the needle.** The panel is a machine; the machine's moving part
is the selector.

- **Channel change — a damped sweep.** The needle leaves the old channel, accelerates,
  and settles with a small overshoot, like a galvanometer: ~160 ms, a decaying spring,
  no bounce. Nothing else on the panel changes position. In the still, the needle is
  caught *mid-sweep* — tilted a few degrees off its rest with a ghost of its previous
  position trailing behind it, which is the trace of the motion.
- **A value that changes does not animate into place: it re-reads.** The digits swap in
  one frame. Only the part that changed flashes once in phosphor and decays. No
  sliding numbers, no count-up, no tween.
- **The delta is a measurement, not a list.** The trace is a chart recorder: when new
  updates arrive it extends one step to the right, and the head dot pulses once. The
  count changes in the same frame; the trace *is* the animation, and the growing trace
  is the thing the user reads.
- **The list scroll settles.** The scrollbar carries a phosphor edge at the reading
  position, and the last row is cut by the fold — the list is caught mid-scroll, and
  the row that is half off the edge says so.
- **The function switch is a switch.** The selected segment lifts and the others
  settle; the readout re-reads. No slide, no wipe, no page transition — a different
  reading, not a different place.
- **The console is where the time goes.** The progress bar is the only continuously
  moving thing in the application, and it moves because a transaction is moving.

---

## 4. What I would change in a second pass

1. **Freshness should be per channel, not just for the reading in front of you.** Right
   now only ALPM's age is visible. The instrument should show that AUR was read six
   minutes ago and AppImage an hour ago — a staleness pip on each channel row would
   do it, and it would change the character of the channel bank completely.
2. **The two action mechanisms should be one.** Today `CAPABILITIES · ALPM` is
   channel-level and `APPLY` is package-level; they should be the same keys, lighting
   up for the selected package, so the instrument never says the same thing twice.
3. **The compact list deserves to be drawn, not captioned.** I show a glyph and a note
   (`COMPACT = 1 LINE / PKG`); the real thing — the same rows and the same columns with
   the description dropped — should be a first-class second state with its own column
   header, and the specimen of it should be visible in the density control.
4. **The trace needs units and a second mode.** Axis labels in updates-per-hour, and
   the ability to plot "updates by source" when the channel is *all* of them, which is
   where the trace earns its keep across seven sources.
5. **The console's middle is empty and I left it empty on purpose.** A real console
   would carry a strip chart of the transaction rate; I would rather leave the space
   than invent a metric I cannot stand behind.
6. **The mark loses its growth lines at 76px.** At 96px they survive; the header would
   have to grow to 116 and the list would lose a row, which is a trade I made the
   other way.
7. **The `ALL` channel.** Seven sources is one too many to scan at a glance. A
   synthetic `ALL` position above ALPM — with the class tag replaced by a count of
   mixed provenance — would let the instrument answer "what can I install" without
   making the user walk the bank. I left it out because a synthetic channel is a
   compromise, and this proposal is about not compromising the two axes.

---

## 5. The source

`02-instrument.figcraft` — one page, 402 nodes, a single 1600 × 1000 frame named
`CALIX`. Built headless with `figcraft-cli run` from a 343-step JSONL script; the
shell is an SVG import, so it remains editable vector geometry. Rendered at 2× and
reduced to 1600 × 1000.

Colours are the tokens as given: anthracite `#111418` ground, shell white `#F7F3ED`
ink, muted `#A1A6AD`, slate `#667085`, signal orange `#FF6A00` with 100/300/700 for
rim light, hover and shadow, royal blue `#4169E1` for *elsewhere*. Every other surface
is one of those values at low opacity over the ground, or a black at low opacity cut
into it — the plates are lit anthracite, not new colours.
