# 03 — L'Atelier

*A package manager as a workbench.*

**The packages are tools. The sources are trays. The operations are the things
you do at the bench. The user is at work.**

Two screens: the bench with work on it, and the bench the moment it is
opened, empty.

---

## The arrangement, and why

The screen is the bench **seen from above**, in plan. There is no sidebar and
no panel column: there is a surface, and things lie on it, and the screen's
edges are the bench's edges. Three edges carry the application.

**The far edge — the rack of trays.** Above the bench, a row of seven
pigeonholes, one per source, each a shallow box seen from above with its
label **stencilled on the front lip** and its contents counted on the inside
face. This is the **source axis**. It is not a list and it is not a set of
tabs: it is a rack of physical boxes, and the one you are working from is
*pulled* — its box is open, its lip is lit, its name is stencilled in Signal
Orange, a tick is chalked beside it, and you can see two or three tools lying
inside it. The other six are shut. The rack is also where the
non-interchangeability is stated before the user asks: the AUR's lip reads
`RECIPES — NO BINARIES`, the AppImage's reads `NO DEPENDENCIES`, the NPM's
reads `GLOBAL INSTALLS ONLY`, the tracked releases' reads `FOLLOWED FROM GIT
HOSTS`.

**The middle — the mat.** The work surface is a dark cutting mat, ruled at
40 mm, with knife marks and a worn patch. The tools lie on it in rows, and
the **inspection card is a loose object on the mat**, laid down *beside* the
tool you picked up — not a panel in a column, but a card with a steel clip
on its top edge and a shadow that falls on the mat. When you pick a
different tool, the card is laid down somewhere else.

**The near edge — the cork rail.** The bench's front rail, carrying a cork
strip with the steel edge along it. On it: the **command the product shows**
(`$ calix install standard ripgrep`, in Signal Orange) and, directly
underneath, **what the backend actually ran** (`sudo pacman -S --needed
ripgrep`, then the backend's own output, then the tick). To the right of it
the operation currently in flight, with its bar mid-way.

**The operation axis is the job, chalked on the mat.** `FIND · STOCK · DELTA`
is written on the surface you are working on, in chalk, with the job in hand
ringed in chalk. It is the one thing on the bench that the user has *set up*
rather than picked up. It cannot be confused with the trays because it is
not a control among controls — it is handwriting on the bench, in a different
material, at a different edge.

So the two axes are separated by **material and by edge**, not by
convention: one is a rack of boxes above the work, the other is a mark on the
work itself. The first round's failure — eight things in one list, so that
"source" and "operation" become siblings — cannot happen here, because
nothing on this screen is a list of eight things.

**The question this arrangement answers: what is within reach, and what is
out of it.**

- **Within reach** is the mat: the list, the slot you write in, the place
  where the card will be laid. Everything the user does happens here.
- **Above and out of reach** is the rack: seven kinds of thing, only one of
  them open. To change kind, you reach up and pull a different box down.
- **Below and out of reach** is the log: what has already happened. You read
  it; you do not touch it.
- **Under everything** is the bench itself, the only continuous surface, and
  the only thing that is always there.

**The backboard** at the very top carries the identity and the age of the
answer: `CALIX` and the shell-chalice mark on the left, and on the right, in
one fixed place and one fixed typeface, `CHARTED 11:42 · STANDARD SYNCED
4 MIN AGO`. The freshness never moves and never changes face, on either
screen.

**The keyboard leaves the surface.** There is not a single key hint anywhere
on either screen — no ⌘ glyph, no shortcut chip. The map lives in a help
overlay, one keystroke away, and the surface belongs to the work. The
operations are discoverable by their names, which are on the bench in
English: `FIND`, `STOCK`, `DELTA`.

---

## The information budget — eight kinds, and what waits

What is on the surface, all at once:

1. **Identity** — the mark and `CALIX`.
2. **Freshness** — when the answer was read, in the backboard's right.
3. **The trays** — the source axis: symbol, name, size, on hand, due.
4. **The job** — the operation axis, and the slot that belongs to it.
5. **The list's state** — the count, the filters in force (`REPOS ALL`), the
   scale switch.
6. **The list** — the tools on the mat.
7. **The card** — the picked-up tool: version, provenance, dependencies,
   files, the command, the two actions that apply to it.
8. **The log** — the backend's record, and what is in flight.

That is eight. What waits, and where it waits:

| waits | behind |
|---|---|
| the whole key map | a help overlay, one keystroke away |
| the other 11 files of the package | a gesture on the card's `14 FILES` line |
| the diff between the installed build and the new one | a gesture on the version line |
| the dependency tree | a gesture on `DEPENDENCIES` |
| the per-repository filter chips | a click on the pulled tray's lip |
| an AUR recipe's build log | a separate view, when a build is running |
| the full contents of every tray | the tray itself, when it is pulled |
| settings, keys, account | a separate window |

The **compact density** is kept visible without being on the surface as a
list: the scale switch on the mat carries a **scale model of the other
layout** — three tall rows for `DETAILED` (lit, because that is the scale in
use) and seven dense rows for `COMPACT` — the way a drawing carries a small
elevation of the elevation it is not showing.

---

## The empty region, and what it is for

On the quiet screen the mat's lower two-thirds are empty, and they are empty
**on purpose**.

A workbench is a workspace, and an empty workspace is not an absence of
information — it is the invitation. The emptiness here is *composed*:

- the mat's **engraved grid** is still there, so the emptiness has a scale
  and a place;
- **one dashed slot** sits where the first row will be, and it says what
  would fit in it: *A PACKAGE FROM THE STANDARD TRAY FITS HERE · WRITE A NAME
  IN THE SLOT TO FILL THE BENCH*;
- the **place where the card will be laid** is outlined, so the reader can
  see that the mat has a second room and that the card is what goes in it;
- in **chalk**, low on the mat, the state of the bench and the rule that
  governs it: *THE BENCH IS CLEAR* and *THE JOB AND THE TRAY DECIDE WHAT YOU
  CAN PICK UP*;
- the rail below says `NOTHING RUN YET`, and the tray above still says what
  is waiting — `1,204 ON HAND · 37 DUE` in the standard tray.

So the empty bench answers "what can I do here" before the user types
anything: the slot says which kind of package fits, the job says what the
bench is set up for, and the rack says what is waiting. The empty region is
also the reason the arrangement has to hold: a panel layout would have had
to fill that rectangle with something, and this one does not have to.

---

## The material

**Warm neutral greys, steel and cork, and no hue that is not a token.** The
rule is *invent nothing outside these*, so the brief's *cutting-mat green*
is not a colour here: the mat is a darker plate of anthracite ruled in
slate, and it is a mat because of its grid, its knife marks and its worn
patch, not because of its colour. The cork is a warm speckle — muted and
Signal Orange 700 at low alpha, tiled — and the steel is a gradient with a
lit edge. Everything warm is shell white at low alpha over the ground, which
is what makes the greys warm without inventing a beige.

- **The bench** is a large radial wash of shell white over anthracite, warm
  at the centre and falling away at the edges.
- **The mat** is a plate set *down* on the bench: a real drop shadow, a
  hairline edge, a 40 mm grid, knife marks from other work, and a cup ring.
- **The trays** are boxes: a dark well with an inner shadow, a steel lip
  that catches the light, and the label stencilled on the lip.
- **The card** is the lightest object on the bench, a plate of paper with a
  steel clip, a bevel, and a shadow that falls on the mat and not on the
  rows.
- **The rail** is cork with a steel edge along the top, and it is the only
  warm-textured surface, which is why the log reads as the softest thing
  here.
- **Type**: IBM Plex Sans, a plain workhorse, for every label; IBM Plex Mono
  for every figure and for the **stencilled capitals on the tray lips** —
  the stencils are mono, uppercase, tracked wide, and they read as paint
  sprayed through a template.

---

## The motion

Physical, and slightly heavy. Nothing snaps.

- **Picking a tool up** takes a moment: the name tilts a degree and a half
  off the mat, its shadow separates (it grows, softens and offsets), and it
  settles with a small rebound — the tilt overshoots by a fraction and comes
  back. The tool is the only thing that moves; the mat's annotations stay
  flat, because they are engraved on the mat.
- **A tray slides** down out of the rack with its contents already in place:
  the box travels, the tools inside it arrive with it, and the name is
  stencilled on the lip the whole time.
- **Scrolling glides** with a little inertia at the end, the way a hand
  stops — the list does not jump, and the last row is caught mid-way by the
  mat's edge.
- **The card is laid down** with a weight to it: it lands, the shadow
  tightens under it a beat later.

The traces in the stills: the picked-up tool is still tilted and its shadow
has not re-joined the mat; the in-flight operation's bar is mid-way with a
bright head on it; the log's last line is written and the tick is already
there; the last row of the list is cut by the mat's edge in the middle of
its description; the cup ring and the knife marks are the bench's history.

---

## What I would change in a second pass

1. **The card has nowhere to go.** It is 460 px wide and it moves to sit
   beside whatever you picked up; pick something in the last three rows and
   the card runs off the bottom of the mat. Either the mat scrolls under a
   fixed card, or the card is clamped to the mat and stops following the
   selection — and the second is a different idea of the metaphor.
2. **The rack is loud.** The counts (9,812 / 92,431 / 3,290,441) are the
   biggest type on the screen after the package names, and the first
   question at the bench is *which kind of tool do I need today*, not *how
   many of them exist*. I would test the rack at a glance and probably
   demote the counts to the same weight as the on-hand figures.
3. **Chalk is a lovely control for a job you set up once.** `FIND · STOCK ·
   DELTA` is handwriting, and handwriting is not what you want from the thing
   you use daily. It may deserve to be a job card clipped to the rack — an
   object with a place, like the trays.
4. **The compact density is a promise the surface does not keep.** It exists
   as a scale model. One keystroke should swap the mat to the real thing,
   and I have not shown that the mat can hold 1,204 rows at that density
   without the descriptions surviving as ghosts.
5. **The rail is a queue of one.** When two operations run, the cork has to
   hold two histories and a progress bar, and I have not designed that. It
   is the part of this layout I trust least.
6. **The bench is very empty in the middle of the quiet screen.** It is
   composed, but I would test whether the chalk line earns its place or
   whether the slot alone is enough invitation.

---

## The artefact

`03-atelier.figcraft` holds both screens as two top-level frames,
`03-atelier-loaded` and `03-atelier-quiet`; the two PNGs are its export at
2× (3200 × 2000 px for a 1600 × 1000 screen), named by their frames. The
generator that produced the file is at `r2/build/03-atelier.build.py`, with
the command script it emits beside it, so the drawing is reproducible rather
than hand-edited.
