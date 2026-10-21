# L'Instrument — a Calix package manager as a precision instrument

**Proposal 01 of 4, second round.** Two screens: `01-instrument-loaded.png` and
`01-instrument-quiet.png`, both 1600×1000, dark. The source is
`evidence/design-proposals/r2/01-instrument.figcraft` — one document, two
frames — built by `evidence/design-proposals/r2/build/01-instrument.py`.

---

## 1. The arrangement, and why

**The instrument is a single milled faceplate read from the top down. There is
no left column, no right card and no bottom bar. Everything is a full-width
band, and the bands are stacked in the order the eye should travel: what is
being read, what is being read *from*, what it means, and what the machine
actually did.**

```
┌──────────────────────────────────────────────────────┬──────────────┐
│  identity                          READ · AGE   (?)  │  the header  │  0 – 84
├──────────────────────────────────────────────────────┼──────────────┤
│  SOURCE ▸ ALPM  AUR  FLATPAK  APPIMAGE  CARGO  NPM …  │  the bank    │  84 – 262
│  seven apertures, each with its kind and its count    │              │
├──────────────────────────────────────────────────────┤  the bay     │
│  SCOPE · PERMITS · FORBIDS   actions   SHOWN/FILTER  │  a totaliser │
│                                                      │  a knob      │  262 – 384
│                                                      │  a legend    │
├──────────────────────────────────────────────────────┴──────────────┤
│  PACKAGE   INSTALLED  AVAILABLE  PROVENANCE   Δ MiB ▏ SIZE  STATE      │
│  ▏ the readout: a window cut into the plate, ruled, with the delta    │  384 – 760
│    measured against a scale                                            │
├──────────────────────────────────────────────────────────────────────┤
│  the specification: a nameplate of engraved cells                     │  768 – 878
├──────────────────────────────────────────────────────────────────────┤
│  ○ ○ ○  the record: a paper tape of what the backend actually ran     │  884 – 1000
└──────────────────────────────────────────────────────────────────────┘
```

**Why the sources are a horizontal bank and not a column.** The first round's
four proposals all put the seven sources in a left column, which made the
source axis look like a menu. On an instrument, a set of *ranges* is a row of
switch positions — a signal generator's band selector, a receiver's range
bank. Seven equal apertures across the top of the faceplate read as one
control with seven positions, and they carry the thing the first round kept
burying: what each source *is*. Every aperture states its kind in three
letters — `BIN`, `SRC`, `SBX`, `ONE`, `CRATE`, `NODE`, `GIT` — and its count
beneath. A user learns that AUR holds recipes and ALPM holds binaries by
looking at the bank, before any package is selected.

**Why the function is a rotary and not a switch bank.** The two axes must not
share a control, and they should not even look alike. The sources are a
*bank*: many positions, equal, browsable, each showing a fact about itself.
The function is a *single object with three positions*, and it lives in its
own bay on the right of the plate, above the readout it governs. The knob is a
machined disc standing proud of the plate, knurled, with a phosphor pointer
thrown at one of three engraved detents on a scale arc. Turning it changes
what the window below is reading — it never changes the source. The bay also
holds the totaliser (the size of what is pending) and the density legend, so
every *control* on this instrument is in one place and every *reading* is
elsewhere.

**Why the readout is a window and not a list on the plate.** The list is the
only thing on the instrument that changes constantly, so it is the only thing
that gets its own material: a recess cut into the faceplate, with a shadow
at the top lip, a bright edge at the bottom, and a fine engraved grid ruled
through the glass. The figures sit on that grid. Everything else on the
plate is solid.

**Why the detail is a nameplate, not a card.** The specification of the
selected package is a single row of engraved cells running the full width
under the window — version, provenance, depends-on, files, and the command —
divided by engraved rules, like the legend plate under a measuring
instrument's dial. It is not a separate object floating to the right of the
list; it is the list's caption, and it reads across.

**Why the log is a tape.** The backend's own output is the machine's record,
and a machine's record is a strip of paper with perforations, coming out of
a slot. The tape carries the product's command (`calix install standard
ripgrep`) on one line and what the backend actually ran (`pacman -Syu
ripgrep`) on the next. The distinction between what you asked for and what
was executed is the most useful thing a package manager can show and it
deserves to be the last thing on the screen, in the one band that is not a
reading.

**The keyboard is not here.** There is no key map on the surface, because a
key map is a document, not an instrument. The surface is the work: seven
apertures, a knob, a legend, a window, a nameplate, a tape. The way into the
help overlay is one engraved `?` disc in the corner of the header — the only
affordance on the plate that is not part of the reading.

---

## 2. The information budget — eight kinds, and what waits

**On the surface, at once:**

| | kind | where |
|---|---|---|
| 1 | **The freshness of the reading** | header, top right: `READ 20:14:31Z` · `AGE 4 MIN`, in one typeface, in one place, always |
| 2 | **The source bank** — which channel, what it is, how much it holds | the seven apertures |
| 3 | **The function** | the rotary and its engraved scale |
| 4 | **The list, and the state of the list** — the rows, the count, the filter | the window, and `SHOWN 8 OF 812` · `FILTER OUTDATED` |
| 5 | **The scope of action** — what this source and this function permit, and what they forbid | `PERMITS UPDATE · PIN · IGNORE · HOLD`, `FORBIDS BUILD — NOTHING TO BUILD`, and the action buttons |
| 6 | **The delta, as a measurement** | the bars against a calibrated scale, and the totaliser |
| 7 | **The specification of the selection** | the nameplate band |
| 8 | **The record** — what the product was asked to do, what the backend ran, what is in flight | the tape |

**What waits, and where it went:**

- the key map — a help overlay behind the `?`;
- the full file list — the nameplate gives the count and one path;
- the full dependency list — three names and `+2 MORE`;
- the other six sources' contents — the bank gives their kind and their count;
- AUR's build recipes and logs — not shown at all, because they only exist for `SRC`;
- previous runs — the tape holds four lines, not a history;
- the compact list — the legend beside the knob shows its *shape* (one line per row, the same columns, less of them), and the switch is a click away;
- the changelog — nowhere; it is a web page, not a reading.

---

## 3. The empty region, and what it is for

**The readout window, in the quiet screen, is empty on purpose. It is the
largest region of the proposal and it holds nothing.**

This is not a gap. It is the instrument's dial. When Calix opens with a
source engaged and `FIND` selected but no query entered, there is no reading
to display, and the honest thing is to display nothing rather than to fill
1520 × 376 pixels with suggestions, placeholders or an onboarding panel that
would have to be dismissed before the instrument could be used. The window is
kept *composed* so that the emptiness reads as deliberate:

- the recess is physical — a shadow under the top lip, a bright edge along
  the bottom, a milled bevel — so it reads as glass set into a plate, not as
  an undrawn rectangle;
- the engraved grid and the row rules stay visible through it, because a dial
  keeps its calibration when the needle reads nothing;
- the rest of the plate does not go quiet with it. The source bank is full,
  the bay reads its total, the nameplate names the *channel* rather than a
  package (`ALPM · BINARY REPOSITORY · core · extra · multilib · 2,481
  PACKAGES`), the scope line says `AWAITING — NOTHING TO READ`, the action
  buttons are present but dim, and the tape says `NO OPERATIONS RECORDED`.

So the empty region is the one place on the instrument that is *about* the
instrument's readiness, and everything around it is about the work. The
arrangement holds: the quiet screen is the same plate with one window blank,
and the band order, the bay and the nameplate are all still in place.

---

## 4. The material

Machined, anodised, engraved. Three kinds of lettering, and the rule is
absolute:

- **engraved** — slate `#667085`: every label, heading, scale and tick. Cut
  into the plate, so it is darker than the surface and never competes.
- **inked** — shell white `#F7F3ED`: every value. The package name is the
  only grotesk on the plate; everything with a number in it is IBM Plex
  Mono, on the grid.
- **phosphor** — signal orange `#FF6A00`, and nowhere else. It marks what
  is *live*: the engaged channel's rule, the knob's pointer and its bright
  tip, the selected row's caret, the delta bars, the command's edge, and the
  print head on the tape. Four or five places on the whole face.

The plates are anthracite with the slate token laid over them as a vertical
sheen, a light hairline along the top edge, a shadow along the bottom edge
and a real drop shadow beneath, so the stack has thickness. Their grain is a
slate noise effect blended `overlay` at one document unit — you do not see
it; you notice it. The whole faceplate carries an engraved 40-unit grid,
strengthened every 200 units, and the window's glass is ruled on the same
grid as the figures that sit on it.

The mark is the identity, present and not a corner glyph: a scallop shell of
seven ribs in a metal cup on a brushed aluminium stem, 74 px tall in the
header, its shell in signal orange and its metal in slate and muted. The
name is `CALIX`, alone.

---

## 5. The motion

Instantaneous where a machine would be, and nowhere else.

- **The knob snaps.** It does not sweep; it is thrown to its detent and
  stops. The pointer is the only thing that moves.
- **A value re-reads and flashes once.** It does not animate into place. In
  the loaded frame the newly-read `AVAILABLE` figures sit one step brighter
  than the installed ones, and the selected row's available version carries
  the phosphor — the trace of a value that has just been re-read.
- **The delta is a measurement, not a list.** Each row's pending size is a
  bar on a shared scale, ticked at 0, 32, 64, 96 and 128 MiB, with a bright
  end-marker at the reading and the exact figure in the column to the right.
  A package of 1.2 MiB is a sliver; `linux` at 124 MiB nearly fills the
  scale. The eye gets the order of magnitude before it reads a digit, which
  is what a bar chart on a scope is for.
- **Nothing scrolls into place.** The rows are on the dial.
- **The tape prints.** The record is written line by line; the print head
  sits at the end of the live line, and the progress bar is read against the
  tape's own scale.

The stills carry the traces: the knob parked at `DELTA`, the pointer lit; the
newly-read available version lit in the selected row; the print head at the
end of a running line; the delta bars with their end markers.

---

## 6. What I would change in a second pass

1. **The bank is a row of equal apertures, and equal is a lie.** AUR holds
   sixty-one thousand recipes and AppImage holds forty-two single binaries;
   they get the same plate, the same weight and the same place. A second
   pass would let the aperture's height or its rule vary with what the
   source holds — the wall would then be readable at a glance.
2. **The delta scale is linear, and that hides the small ones.** `ripgrep`'s
   1.2 MiB is a 4-pixel sliver. A log scale, or a second row of the scale
   for the sub-megabyte range, would let a 84 KiB update be seen. As it
   stands the bar is honest and nearly invisible, and I am not sure that is
   the right trade.
3. **The function rotary has three positions and no home.** There is no rest
   position, no "all sources", no off. The bank needs an eighth aperture
   for "everything", and the knob needs a fourth detent, and both would
   change the muscle memory the arrangement is trying to build.
4. **The nameplate is a row of cells and cells are a table.** It reads
   correctly as a legend plate, but a card is what a card is good at, and I
   have traded the ability to grow the detail for the ability to read the
   detail across. If the detail grew a provenance graph, this band would
   not hold it.
5. **The compact list is only a legend.** The two glyphs beside the knob say
   what the compact row looks like, which satisfies legibility but not
   demonstration. The second density deserves to be drawn once, even at the
   cost of the loaded screen's calm.
6. **The tape's perforations are decoration.** They say "paper", but the
   record is text, not paper; a reader may find them twee. If the tape were
   given a real edge — a torn edge, a timestamp gutter, a column ruler — the
   band would earn the perforation.
7. **The empty window could be wrong.** I have argued that it is the dial,
   and the calibration stays visible, and everything around it is full. But
   a user arriving at a large blank rectangle may read it as an error rather
   than as readiness. One engraved word in the middle would fix that at the
   cost of the emptiness; I have left it out, and I am not certain that is
   the right call.

---

## 7. How it was made

`evidence/design-proposals/r2/build/01-instrument.py` emits the command log;
`figcraft-cli run` builds the document; `figcraft-cli export --node` renders
each frame. 702 commands, one process per file.

Both PNGs were rendered and read back seven times. The corrections that
mattered: the knob's radial gradient had its axis on the diagonal and painted
a band across the disc, so the knob is built from two linear gradients and a
crescent instead; twenty-eight separate knurl vectors did not render, so the
knurl is one vector with twenty-eight sub-paths; the action buttons were
positioned by multiplying each button's own width by its index, so they
overlapped; the delta scale's figure column drew the number and its unit at
the same right edge; the density legend was drawn inside the window's
heading row and collided with the `SIZE` heading; the source bank's seven
apertures needed a fifth kind of label that did not exist; and the second
screen's nameplate overflowed its cells by a dozen pixels in three places.
