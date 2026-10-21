# The four briefs

A design exploration for Calix, the package manager for Arch Linux, run as four
independent blind proposals.

**The rules of the game.** Each author works alone, from their brief alone. They
must not look at the running application, at its screenshots, or at the Calix
repository: the point is to design from a description of the product and its
principles, not to re-skin something that exists. They may leave each other a
note — a question, a doubt, an observation about the *medium* — but they may not
discuss the design, and they may not look at each other's work.

**The output.** One image per author, a desktop screen of the application
(1600×1000, dark), the FigCraft source that produced it, and a short rationale.
The four images must be visibly different: not four palettes of one idea.

**The shared spine.** Every proposal designs the same product, so the four are
comparable.

---

## The product

Calix installs, updates and removes software on Arch Linux. A user comes to it
to answer one of three questions: *what can I install*, *what do I have*, and
*what is out of date*. The software comes from seven sources, which are not
interchangeable and are usually managed by different people:

| Source | What it is |
|---|---|
| **ALPM** | the official repositories — `core`, `extra`, `multilib` |
| **AUR** | the user-contributed repository: build recipes, not binaries |
| **Flatpak** | sandboxed applications, distributed as Flatpak |
| **AppImage** | single self-contained binaries |
| **Cargo** | Rust crates, installed from `crates.io` |
| **NPM** | Node packages, installed globally |
| **Tracked releases** | binaries followed from GitHub, GitLab or Gitea |

Because they are not interchangeable, the navigation must **separate the
sources from the operations**. A user filters by *where a package comes from*;
a user also acts on *what they want to do with it* — search, see what is
installed, see what can be updated, remove. Those are two different axes, and a
navigation that mixes them is the failure this exercise exists to avoid.

Every proposal must therefore answer, visibly and in the design itself:

1. **Where am I?** — the source, the operation, and the state of the list.
2. **What can I do here?** — the actions that make sense for this source and this
   operation. A Flatpak package cannot be updated from a Cargo source; a package
   in the official repositories has no build recipe.
3. **How do I get elsewhere?** — a way back, and a keyboard that covers the
   whole application.
4. **What is the state of the list?** — the count, the filters in force, and
   how fresh the answer is.

**The work surface.** A list of packages, with a detail panel for the selected
one: its version, its provenance, its dependencies, its files, and the exact
command that would install it. Below the surface, a console that shows the
operations in flight. Above it, a header that always says where the user is.

**Two densities, both lists.** The application offers a *detailed* list — the
package, its provenance, its size, its state, a line of description — and a
*compact* list — one dense line per package, for when the user knows what they
are looking for. The compact list is not a grid of cards; it is a list with
less in each row. A proposal may show one of them, but it must show that the
other is possible: the same rows, the same columns, less of them.

**The identity.** A scallop shell as the cup of a chalice: a fan of ribs
sitting in a metal cup on a brushed aluminium stem. The mark is the product's
voice and it must be *present* — not a 16-pixel glyph in a corner. The name is
`CALIX`, alone; no framework name, no version, no subtitle. The shell is
Signal Orange by default and Royal Blue as an alternative; the stem is brushed
aluminium.

**The depth.** The current surfaces read as flat: one plane, no texture, no
light, no material. A proposal must show *layers* — a surface that sits on
another, an edge that catches the light, a panel that is raised, a texture that
you notice only when you look for it. Depth is the point of the exercise, and
it is what the eye reads first.

**The motion.** The list must scroll smoothly. A selection must arrive from
where it was. A change of source or operation must read as a change of place,
not as a repaint. A proposal states its motion in the rationale even though a
still image cannot show it — and where the motion would be visible, the image
should carry its *trace*: a transition half-finished, a surface mid-flight, a
scroll that is still settling.

**The tokens.** The colours are design system tokens, taken as they are:

| Token | Hex | Role |
|---|---|---|
| anthracite | `#111418` | the ground |
| shell white | `#F7F3ED` | the ink, and the surface's warmth |
| muted | `#A1A6AD` | secondary text |
| slate | `#667085` | hairlines, the stem |
| signal orange 100 | `#FFE5D2` | the rim light |
| signal orange 300 | `#FF8E40` | hover |
| signal orange | `#FF6A00` | the accent |
| signal orange 700 | `#C95400` | the shadow side |
| royal blue 300 | `#87A0F4` | the alternative accent, hover |
| royal blue | `#4169E1` | the alternative accent |

White on signal orange is an approved pairing. Nothing outside these tokens is
invented.

---

## Brief 1 — Le Comptoir

*A package manager as a shop counter at night.*

The sources are counters along a wall, each with its own till, its own wood,
its own lamp. The packages are objects on the shelves. The user is a regular
who knows the shop.

**The thesis.** The navigation is the *wall*, and the operations are the
*counter*: the sources are the vertical run of drawers in the left column, and
the three operations — search, installed, updates — are the three shelves of the
main surface, reached by pulling a drawer rather than by clicking a page. The
wall is always there; the shelf changes. A user's hand knows the wall after an
hour.

**The material.** Warm dark wood and blackened brass. The surfaces are not
grey panels: they are shelves, with a lip at the bottom that catches a warm
light. A selected object *lifts* off its shelf, throwing a soft shadow on what
is behind it. The type is a shop's: a plain grotesk for the goods, a condensed
one for the labels, and prices and sizes in figures that line up.

**The motion.** The shelves slide. Pulling a drawer slides the whole shelf in
from the side, with the objects moving at different depths — the front row
fastest, the back row slowest, so the shelf reads as a volume. A selection
lifts and settles, with a faint bounce, like a thing picked up and put back.

**The detail to get right.** The store's *till* — the operation control — is
not a row of buttons. It is a physical switch: the three operations are three
positions of one object, and the object moves between them.

---

## Brief 2 — L'Instrument

*A package manager as a precision instrument.*

Everything is a readout. Nothing is decorative. The user is reading an
instrument that happens to install software.

**The thesis.** The navigation is an *instrument panel*: the sources are
channels of a selector, and the operations are functions of the instrument
(search, inventory, delta). A channel is chosen, a function is chosen, and the
readout below shows what that channel sees through that function. The panel
is the same object in every state; only the readings change. Nothing moves
except the needle.

**The material.** Machined, anodised, engraved. The surfaces are flat but not
*plain*: each is a milled plate with a bevelled edge — a light above, a shadow
below — so that the stack of plates has real thickness. The accent is a
phosphor, and it is used the way a phosphor is used: sparingly, exactly where
the reading is live. The figures are monospaced and aligned on a grid, and the
grid is visible, faintly, behind the rows.

**The motion.** Instantaneous where a machine would be, and nowhere else. The
selector's needle travels to a channel with a short, damped sweep. A value
that changes does not animate into place; it re-reads, and the parts that
changed flash once. The delta — what can be updated — is drawn as a live trace
beside the numbers, so the update set is a measurement, not a list.

**The detail to get right.** The *freshness* of the reading. An instrument that
does not say when it was read is decoration. The age of the answer is part of
the panel, always, in the same place, in the same typeface.

---

## Brief 3 — La Carte

*A package manager as a chart.*

The screen is a chart: the sources are territories, the packages are
landmarks, the installed ones are ports, and the updates are what the chart
says is out of date. The user is a navigator reading a coastline.

**The thesis.** The navigation is *spatial*: the left column is the chart's
legend and territory rail, and the operation is a *heading* — a bearing —
written in the header. The user does not "switch to the updates view"; the user
*turns* to it, and the chart turns with them. The back path is a bearing
return, and the keyboard is the helm: it steers between territories and
bearings without the pointer.

**The material.** Translucent and deep. The chart is a stack of translucent
plates over a dark ground, with a slow fog at the horizon, and the landmarks
sit on it with a soft halo. The type is a chart's: wide, calm, spaced, with
labels that behave like map annotations — smaller, offset, subordinate to the
territory names.

**The motion.** Travel. Changing the territory pans the chart, with the layers
moving at different rates — the ground, then the near landmarks, then the
labels — so the movement has depth. A selected landmark is ringed, and the
ring settles with a slight overshoot, the way a lens does. Scrolling the
landmarks is a glide with weight, not a jump.

**The detail to get right.** The *legend*. A chart's legend is not a menu; it
says what the territories hold, how much of each is on the chart, and which
parts of it are unreachable by the current bearing. The coverage of a source
is a fact, and the legend is where it is stated.

---

## Brief 4 — L'Atelier

*A package manager as a workbench.*

The packages are tools. The sources are trays. The operations are the things
you do at the bench. The user is at work.

**The thesis.** The navigation is the *bench itself*: the trays of the sources
are on the left, each labelled and each holding only what it can hold, and the
operations are a *rail of tools* along the top of the surface — find, inventory,
delta — rather than a mode. The tool is chosen, the tray is chosen, and the
surface below is the work laid out on the bench.

**The material.** Warm neutral greys, a cutting-mat green, steel, cork. The
surfaces have grain: a fine noise, a soft bevel, a shadow that is a shadow and
not a black rectangle. A selected tool is *picked up*: it tilts a degree, its
shadow separates from the bench, and the detail panel is the tool's own
inspection card laid beside it. The typography is a workshop's: a plain
workhorse face for the labels, and stencilled capitals for the tray names.

**The motion.** Physical, and slightly heavy. Picking a tool up takes a
moment and settles with a small rebound. A tray slides in with its contents
already in place. Scrolling the bench's tools glides with a little inertia at
the end, the way a hand stops.

**The detail to get right.** The *empty bench*. What a bench looks like when
there is nothing on it, and what it invites you to do next — the tray
suggesting a search, the empty slot saying which kind of package would fit
there.

---

## The output, for each author

| Artefact | Path |
|---|---|
| The image | `evidence/design-proposals/<n>-<name>.png` — 1600×1000, dark, the whole screen |
| The source | `evidence/design-proposals/<n>-<name>.figcraft` |
| The rationale | `evidence/design-proposals/<n>-<name>.md` — the navigation you reasoned out, the material, the motion, and what you would change in a second pass |
| The note | `evidence/design-proposals/notes/<n>-<name>.md` — a question to the other three. Not a design discussion. |

**How to build.** `figcraft-cli`, one process per file, so nothing is shared:

```sh
figcraft-cli run build.jsonl --out design.figcraft
figcraft-cli export design.figcraft -o design.png --scale 2
```

`build.jsonl` is a list of `{"command": id, "params": {...}}`; a step can refer to an
earlier one with `{"$step": n}`. The catalogue is 561 commands — look them up with
`figcraft-cli commands --filter <text> --json` rather than guessing. A frame
created without `sizingH`/`sizingV` is fixed at 100×100 and clips its children;
a frame that must size to its content needs `"sizingH": "hug"`.

**The loop that matters.** Build, export, *look at the PNG*, fix, export again.
The engine does not lie about the structure, but only the image tells you
whether it looks right.

**Do not** open the Calix repository, the running application, or any screenshot
of it. Design from the brief.
