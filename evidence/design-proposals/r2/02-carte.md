# Calix — La Carte (round 2)

*A package manager as a chart. The sources are territories, the packages are
points on a coast, the installed ones are ports, and the updates are what the
chart says is out of date.*

Two screens: the **loaded** chart, with an answer plotted on it, and the
**quiet** chart, the state right after opening, before any search.

---

## 1. The arrangement, and why

**One sheet, not four widgets.** The application is a single continuous sheet
of chart, read top to bottom in four bands, each of which is a part of a chart
rather than a part of an application:

| band | y | what it is |
|---|---|---|
| the title margin | 0–112 | the mark, `CALIX`, the bearing, the query, the freshness |
| **the chart** | 112–596 | the land, the coast, the points, the soundings, the scales, the inset, the track |
| the key | 596–660 | the seven territories: what they hold, how much is charted, what this bearing cannot reach |
| the table of points | 660–1000 | the list, in two densities |

Round one put a navigation column on the left, a list in the middle, a detail
card on the right and a log bar along the bottom — and every proposal in the
round was that shape with different furniture. This arrangement is argued
against that:

- **There is no detail card.** The detail is an **inset drawn on the chart** —
  a plate in the sheet's corner, attached by a leader line to the point it
  describes, with a hairline rule and a `DETAIL` label, the way a chart carries
  an inset. The reader's eye travels from the point, along the leader, to the
  data. The list and the detail are not neighbours, because they are not the
  same kind of thing: one is an index, the other is an enlargement.
- **There is no left column.** The two axes are deliberately *not adjacent* and
  not the same shape: the **bearing** is a compass rose in the title margin,
  three rays and one needle; the **territory** is a column in the key band
  under the chart. You read the sheet from the top down and the pair frames
  everything between them. They never share a control, and neither of them is
  a list of tabs.
- **There is no bottom bar.** The log is the **ship's track**, drawn on the
  water in the chart's lower left: a dotted line with waypoints, the last one
  lit. The operations in flight are where a navigator would read them, and
  the backend's actual command (`pacman -S --needed ripgrep`) is annotated at
  the waypoint, not in a panel that pretends to be a terminal.
- **The list is the chart's own index.** A chart of lights has a table of
  lights; this chart has a **table of points** — the same points, as rows,
  with the chart's own symbol in the first column so the two are visibly the
  same thing. The list is full width because it is the sheet's bottom margin,
  not a column between two panels.

**Where the user looks.** The first thing is the **coast**: a profile drawn
through the answer's points, with the territory's name written across the land
above it in wide, calm, letterspaced type. The second thing is the selected
point's ring. The detail is not next to the list; it is next to the chart,
attached to the point.

**The chart's grammar** (stated on the sheet, in *How to read this chart*):
the coast is the catalogue; every point on it is a package; its depth is its
size in MiB on a logarithmic scale; a filled square is a port (installed); a
ringed point is out of date; a glowing point is being installed now. The
horizontal axis is the catalogue's order, with a scale bar under the plot.

---

## 2. The information budget

**Eight kinds of information are on the surface at once, and no more:**

1. **The place** — the mark, `CALIX`, the territory's name and class on the
   land, the bearing (rose + word).
2. **The query** — the one field on the sheet.
3. **The freshness** — `SOUNDED 14:32:07 · 2m 41s AGO`, always in the same
   place, always in the same typeface, always the right end of the margin.
4. **The chart** — coast, points, soundings, graticule, the two scales, the
   notes.
5. **The key** — the seven territories: what each holds (`BIN · REPOSITORIES`),
   how much of it is held, how much is charted, and which two hold no index
   and so cannot be reached by this bearing at all.
6. **The table of points** — the list, the count (`POINTS 1–8 OF 14`), the
   position, the density in force.
7. **The detail inset** — version, provenance, dependencies, files, the
   product's own command, and the command the backend actually ran.
8. **The track** — the operations in flight and what has already run.

**What waits, and where:**

- **The key map.** No key glyph appears anywhere on the surface. The keyboard
  is a help overlay, one keystroke away, and it is the only place the map
  lives.
- **The other two bearings' readings.** `INVENTORY` and `DELTA` are rays on
  the rose; their tables and their charts are waiting behind them.
- **The compact table's rows.** The density switch shows the two densities as
  two glyphs; only one table is drawn, and the other waits.
- **The whole file list.** The inset shows two files of 214.
- **The AUR's build recipes**, the per-territory inventories, and any
  aggregate of all seven territories.

---

## 3. The empty region, and what it is for

**The water below the coast is empty on purpose.**

In the quiet screen it is the largest region on the sheet, and nothing is
plotted in it: no coast, no points, no track, no inset. That emptiness is
composed rather than accidental, and three things compose it:

- The chart is **charted before it is plotted**. The graticule is ruled, the
  depth scale is cut and labelled `0.1 / 1 / 10 MiB`, the scale bar is drawn
  and numbered `1 … 14`, the horizon is hazed with fog. The instrument is
  ready; the reading has not been taken.
- The **key is complete** — the reader can see all seven territories, their
  holdings, and the two that this bearing cannot reach, before a single point
  exists.
- The **notes are present**, so the chart can be read the moment it has
  something on it.

The water is the space the answer will occupy. The quiet screen and the loaded
screen are the same sheet with and without a reading on it, and the difference
between them is entirely the answer: a coast, its points, its soundings, a
track, an inset, and seven rows of a table. Nothing else moves.

(The other half of the composition is the **land** above the coast: it holds
only the territory's name and its annotation, and it is empty because land is
not where packages are.)

---

## 4. The material

**Translucent and deep.** The sheet is one dark ground. The land is a
translucent plate laid over it, catching a slow fog that fades upward into
the horizon; the water below the coast is the ground itself. The coast is a
single hairline in pale grey. The inset and the key are translucent plates
with hairline borders, not filled cards. Points are symbols with a soft halo
rather than dots with a border.

**The type is a chart's.** The territory's name is wide, calm and letterspaced
in slate — a map annotation, subordinate to the ink, placed across the land
where a chart writes a region's name. Every other label is smaller, offset and
letterspaced: `CHANNEL`-scale headings are 8–9px; the figures are monospaced
and aligned on the graticule; the descriptions are the only proportional
type on the sheet, and they are prose.

**The accent is a phosphor, used only where the reading is live**: the point
being installed, the waypoint in flight, the selected point's ring, the
leader that connects the inset to its point, the caret, the freshness pip.
The land and the coast are grey; only what is *now* is orange.

---

## 5. The motion

**Travel, and nothing else.**

- **Changing the territory pans the sheet**, and the layers move at different
  rates: the ground and the graticule at the base rate, the points re-plotted
  a half-beat later (they are not carried — they are re-sounded), the
  annotations trailing by a frame. The sheet reads as a move across a
  coastline, not as a repaint.
- **Turning the bearing turns the rose** with a short damped sweep, and the
  chart turns with it. The rose is the only control that rotates; nothing
  else on the sheet changes position.
- **The selected point's ring is drawn and settles** with a slight overshoot,
  the way a lens does.
- **Scrolling the table is a glide with weight** — the rows carry, they do
  not jump. The last row is cut by the sheet's edge in the loaded screen, so
  the table is caught mid-glide.
- **A live operation pulses once** on the track and the waypoint's halo
  breathes; it does not spin.

---

## 6. What I would change in a second pass

1. **The horizontal axis is the point order, not the catalogue position.**
   That makes the coast a profile of the *answer* rather than of the source —
   the honest version places each point at its real position in the source's
   index, so the coast is the source's own shape and the answer is scattered
   along it. The price is that a query like `rg` clusters in the source's
   `r` section, and the sheet becomes mostly empty water. That is the right
   emptiness for a chart, and I did not earn it in this pass.
2. **The depth scale is logarithmic, because the sizes are skewed.** A linear
   scale would be more honest and would make 90% of the points a single line
   on the datum. The second pass should try a square-root scale, or keep the
   size in the point's symbol area and let the coast be an honest shape.
3. **The inset is a panel in all but name.** It should be an actual
   enlargement: a ×4 patch of the coast around the selected point, with the
   point in place and the data beside it — then it is a chart of the chart.
4. **The key is a band, not a key box.** Seven territories in a row is
   legible, but a chart's key belongs in a corner, and the corner is taken by
   the inset. A second pass would trade the two.
5. **The quiet screen needs one more line** in the empty water saying what
   will appear there. The composed emptiness is legible to a cartographer and
   to nobody else.
6. **The track needs a direction** — a ship's track has an arrowhead, and
   without one the log reads as three unrelated commands.
7. **The compact density is a glyph, not a table.** It should be drawn, once,
   somewhere quiet.

---

## 7. The source

`02-carte.figcraft` — one page, two top-level frames: `CALIX loaded` and
`CALIX quiet`, each 1600×1000. Built headless from a JSONL script; the shell
mark and the compass rose are SVG imports, so they stay editable vector
geometry. The coast is a Catmull-Rom spline sampled and written out as a
dense polyline, because the engine's path parser approximates cubics.

Colours are the tokens as given — anthracite ground, shell white ink, muted
and slate for everything secondary, signal orange only for what is live. Every
other surface is one of those values at low opacity over the ground.
