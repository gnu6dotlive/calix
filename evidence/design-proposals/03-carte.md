# 03 — La Carte

*A package manager as a chart.*

**The screen is a chart.** The sources are territories, the packages are landmarks,
the installed ones are ports, and the update is what the chart says is out of
date. The user is a navigator reading a coastline.

---

## The navigation I reasoned out

The brief's hard problem is that **sources and operations are two different
axes** and the failure mode is mixing them. So the design refuses to put them
in the same control. They live at opposite ends of the screen and in opposite
grammars:

| axis | where it lives | what it is |
|---|---|---|
| **source** (where a package comes from) | the **left rail** — the chart's legend | a *territory*, chosen by a chart symbol, a name, a count, a coverage strip |
| **operation** (what you do with it) | the **header** — a heading | a *bearing*, chosen on a compass rose, written as `000° ALL LANDMARKS` |

The operation is never a tab and the source is never a tab. They are a
compass and a coastline. That is the whole separation: **a bearing is a
direction you turn to, a territory is a shore you sail to**, and the screen
never shows them as siblings in a list of eight things.

**Where am I.** The header always says all three things at once, in one band:
the bearing (`000° · ALL LANDMARKS`, on a compass rose whose needle is at
000°), the territory (`ALPM · OFFICIAL REPOSITORIES`, with `core · extra ·
multilib` beneath it as the chart's annotation), and the state of the answer
(`9,812 LANDMARKS ON CHART`, `CHARTED 11:42 · ALPM SYNCED 4 MIN AGO`).

**What can I do here.** The chart card for the selected landmark carries only
the actions that are true for *this source at this bearing*: `UPDATE TO 141.0`,
`REFRESH`, `REMOVE`. And it states, in the same card, what this territory
cannot do — *BINARY FROM THE OFFICIAL REPOSITORY — NO RECIPE TO BUILD* — and
then names the place where the missing capability lives: *the AUR is where
recipes live — steer there with ⌥← ⌥→*. A Flatpak is never updatable from a
Cargo source, and this design never offers the button.

**How do I get elsewhere.** The keyboard is the helm, and it is drawn in the
rail as a helm: `⌘1…⌘5` turn the bearing, `⌥← ⌥→` steer the territory, `⌘F`
find, `⌘L` back to the landmark list, `⌘⌥C` copy the install command, `ESC`
bearing return. The five bearings are also the five marks on the compass rose
in the header, numbered, so the pointer route and the keyboard route are the
same route drawn twice.

**What is the state of the list.** Three places, never mixed: the count and
the freshness in the header, the filters in force in the sheet's own strip
(`ALL REPOS / CORE / EXTRA / MULTILIB`, with the active one lit), and the
scale (`DETAILED / COMPACT`).

### The legend, which is the part I care about most

A chart's legend is not a menu, so this one states facts instead of offering
buttons. Each territory carries a chart symbol (square = ALPM, diamond = AUR,
ring = Flatpak, rounded square = AppImage, hexagon = Cargo, triangle = NPM,
pennant = tracked releases), the size of the territory (`92,431`), and an
**isotype coverage strip** — eighteen cells, filled in proportion to how much
of that territory is on this machine — with the exact figures under it
(`1,204 PORTED · 37 OUTDATED`). Above the territories, one line states the
reach of the current bearing across the whole chart: *BEARING 000° REACHES
EVERY TERRITORY — NOTHING HELD BACK*. Under `⌘3` that same line would read
*37 OF 9,812 ON THIS BEARING*, and the key's fifth row (`HELD BACK — OFF THIS
BEARING`, with a struck-through ring) is the symbol for whatever the bearing
cannot reach. Coverage is a fact, and the legend is where the fact is.

### The two densities

The sheet is showing the **detailed** list: landmark, provenance, size, state,
and a line of description. The **compact** list is not described, it is
*shown*: the chart card's last section, `ALSO IN ALPM`, is the same list at
compact scale — the same rows, the same columns, less of them — and the scale
control in the sheet's strip says so in as many words
(`COMPACT SCALE — THE SAME ROWS, THE SAME COLUMNS, LESS OF THEM`).

---

## The material

**Translucent and deep.** The ground is anthracite `#111418` with a 16 px
ruled weave over it — a texture you find by looking for it and lose
immediately. The five plates (header, rail, sheet, card, console) are glass:
they are 6–12 % white over the ground, so the light behind them shows
through, and each one is raised by a soft shadow and caught by a one-pixel
lip along its top edge. The chart card is the highest plate: it is lighter
than the sheet, it throws the deepest shadow, and it has a Signal-Orange 100
crest on its left edge where the light would find it.

**A chart's weather.** The sheet carries a horizon: a soft band of fog across
its top, a graticule (vertical every 147 px, horizontal every 44 px), two
depth contours, and soundings — small mono numerals scattered on the water
(`12.4`, `18`, `7.5`) that sit behind the data and belong to nothing. The
navigation layers are all *panelled off* the data: they live in their own
frame, offset seven pixels from the sheet, so the chart's ground is never
quite aligned with the landmarks on it.

**A chart's type.** IBM Plex Sans, wide and calm, letterspaced wide for
territories and headings and set small and subordinate for annotations;
IBM Plex Mono for every figure — versions, sizes, counts, coverage captions,
commands. Numbers line up because they are all the same face.

**The mark.** A scallop shell as the cup of a chalice: a fan of ribs with a
scalloped rim, sitting in a metal cup, on a brushed aluminium stem and foot.
It is 72 px in the header, not a glyph in a corner; it is a watermark seal on
the console's margin; and it appears twice more, small, in the legend's
**THE INKS** row — Signal Orange and Royal Blue — because a legend states its
inks. The shell is orange by default and royal blue as the alternative, and
the metal is a real gradient with a lit lip, not a grey fill.

---

## The motion

**Travel.** Turning to a bearing pans the chart. The three layers move at
different rates — the ground (graticule, contours, soundings) moves least,
the landmarks move most, the labels ride between them — so the movement has
depth rather than being a repaint. In the still, the ground layer is caught
seven pixels ahead of the landmarks, which is what a pan in progress looks
like.

**A lens settling.** The selected landmark is ringed in Signal Orange, and the
ring carries a second, larger and fainter ring offset from it: the overshoot
of a lens coming to rest.

**A compass turning.** The needle in the header points at 000°, and behind it
a ghost needle and the arc it swept are still visible at 090° — the turn from
the previous bearing, caught in the act.

**A scroll with weight.** The sheet's scroll thumb sits at the top of its
track with a faint trail below it: the list has just been flung and is still
settling. The eleventh row is clipped by the plate's edge, mid-row.

**A glide.** Scrolling the landmarks is a glide with weight, not a jump: the
rows carry an ease-out, the scroll thumb decelerates into place, and the
sheet's shadow on the console moves with it.

---

## What I would change in a second pass

1. **The coverage strip is honest but thin.** Eighteen cells at 9 px cannot
   say "1,204 of 9,812" with much precision, and for Cargo and NPM the true
   share is a sliver. I would give the legend a **logarithmic** scale, or
   state the ratio as figures only and drop the strip, rather than keep a bar
   that is 99 % empty cells. A chart that lies by rounding is not a chart.
2. **The bearing is a heading but the search is not a bearing.** `⌘4 FIND ON
   THE CHART` is a bearing on the rose, but the find field in the sheet's
   strip is a field — two routes to the same operation. I would either make
   find a true bearing (the field becomes the heading) or take find off the
   rose and leave it as the field it is. Right now it is both.
3. **The map layers risk decoration.** The soundings and the contours are
   beautiful and they are not data. I would test the sheet with them removed
   and keep them only if the chart still reads as a chart. If they go, the
   graticule stays: it is the one thing that says "coordinates" and it is
   also the one thing that can later carry the *real* coordinates of a
   package's provenance.
4. **The rail is doing two jobs.** The legend and the helm are both in it,
   and the helm is the part that grows when the application grows. I would
   move the helm to a footer strip along the bottom of the screen, above the
   console, and let the legend own the full height of the margin.
5. **Density is a control that changes the data, not the type.** `DETAILED` and
   `COMPACT` currently change how much of each row is drawn. I would make the
   compact scale also drop the description *and* the note column, so the two
   scales are two different tables, not one table with two amounts of text —
   that is what "less of them" should mean.

---

## The artefact

`03-carte.figcraft` is the source: one frame, `Calix — La Carte`, 1600 × 1000.
`03-carte.png` is its export, and because the brief's command uses `--scale 2`
the PNG is 3200 × 2000 px — a 1600 × 1000 screen at 2× density, so the 8 px
map annotations stay legible.

The generator that produced the FigCraft file is kept beside it at
`build/03-carte.build.py` (with the command script it emits at
`build/03-carte.build.jsonl`), so the drawing is reproducible rather than
hand-edited.

*Colours: the design-system tokens only — anthracite, shell white, muted,
slate, signal orange 100/300/700, signal orange, royal blue 300 and royal
blue. Nothing was invented; the depth comes from alpha, gradients and
shadows over the ground, not from new colours.*
