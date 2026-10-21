# The four briefs — second round

The first round answered the *navigation* question and missed the *spatial* one.
Four authors produced four skins of the same three-column application, every one
of them maximalised, none of them leaving a single region empty. The second
round changes the terms.

**The rules of the game.** Each author works alone, from their brief alone. They
must not look at the running application, at its screenshots, at the Calix
repository, **nor at the first round's four images** — they are in the repository,
and they are the thing this round exists to get past. They may leave each other a
note — a question, a doubt — but not a design discussion.

**The new terms, and why.**

1. **The arrangement must be argued, not inherited.** The first round's four
   proposals all had the same skeleton: a column at the left, a list in the
   middle, a card at the right, a bar at the bottom. That skeleton is the
   application's existing topology, and treating it as a given is how four
   metaphors became four coats of paint.

   **The shape "a left column, a list, a right card, a bottom bar" is
   forbidden.** So is any arrangement that is the first round's with different
   content in it. Each proposal must find an *arrangement* and argue it: what
   deserves the eye, in what order, and what can wait. Two proposals that
   share an arrangement have failed, however different they look.

2. **The empty space is a resource.** A screen with half its area empty can be
   *rich* — the eye reads the emptiness as a field, and the few things on it as
   marks. The first round filled every region because the brief asked for four
   questions to be answered at once, on the surface, always.

   **A budget: at most eight distinct kinds of information on the surface at
   once.** Everything else waits — behind a gesture, a key, a smaller plane, or
   a moment. And one region of each proposal is *allowed* to be empty, on
   purpose, and the rationale must say what that emptiness is for.

3. **Two screens per proposal.** The *loaded* one, and the *quiet* one — the
   state right after opening, before any search. The second is the proof: it
   shows whether the arrangement holds when there is little to show, and whether
   the emptiness is composed or accidental.

4. **The keyboard leaves the surface.** Three of the first round's four put the
   key map on screen, because the brief had put it there. A key map belongs in
   a help overlay, and the surface belongs to the work.

5. **Everything the first round agreed on stays.** The two axes never share a
   control. The non-interchangeability of the sources is stated before the user
   asks. The freshness of the answer is a first-class fact. Two densities, both
   lists, and the compact one is not a grid of cards.

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

The user filters by *where a package comes from*; the user also acts on *what
they want to do with it* — find, inventory, delta. Two axes, never merged. The
answer to "what can I do here" depends on both: a Flatpak package cannot be
updated from a Cargo source, an official-repository package has no build recipe,
an AppImage has no dependencies to show.

**The work surface.** A list of packages, with a detail for the selected one:
its version, its provenance, its dependencies, its files, and the command that
would install it. The command the product shows is its own — `calix install
standard ripgrep` — and the log underneath shows what the backend actually ran.

**The two densities.** A *detailed* list — the package, its provenance, its
size, its state, a line of description — and a *compact* list, one dense line per
package. The compact list is not a grid of cards.

**The identity.** A scallop shell as the cup of a chalice: a fan of ribs sitting
in a metal cup on a brushed aluminium stem. The mark is the product's voice and
it must be present, not a glyph in a corner. The name is `CALIX`, alone.

**The tokens.** Design system tokens, taken as they are: anthracite `#111418`
(the ground) · shell white `#F7F3ED` (ink) · muted `#A1A6AD` (secondary text) ·
slate `#667085` (hairlines, the stem) · signal orange 100 `#FFE5D2` (rim light) ·
signal orange 300 `#FF8E40` (hover) · signal orange `#FF6A00` (the accent) ·
signal orange 700 `#C95400` (shadow side) · royal blue 300 `#87A0F4` · royal blue
`#4169E1` (the alternative accent). White on signal orange is an approved
pairing. Nothing outside these tokens is invented.

---

## Brief A — Le Comptoir, by the author of L'Instrument

*A package manager as a shop counter at night.*

The sources are counters along a wall, each with its own till, its own wood,
its own lamp. The packages are objects on the shelves. The user is a regular
who knows the shop.

**The arrangement is yours to find, and it may not be a column of drawers.**
A counter can be faced, walked along, reached across. What the shop knows is
that the customer comes for one thing, takes it, and goes — so the surface
should say what is on the counter, what is behind it, and what is *out of
stock*, without listing the whole shop.

The material: warm dark wood and blackened brass. Shelves with a lip that
catches a light. A selected object lifts off its shelf and throws a soft shadow
on what is behind it. Prices and sizes in figures that line up.

The motion: shelves slide; the objects move at different depths, so the shelf
reads as a volume. A selection lifts and settles with a faint bounce.

The detail to get right: the till — the operation control — is not a row of
buttons. It is a physical switch, and the object moves between its positions.

## Brief B — L'Instrument, by the author of La Carte

*A package manager as a precision instrument.*

Everything is a readout. Nothing is decorative. The user is reading an instrument
that happens to install software.

**The arrangement is yours to find, and it may not be a channel bank beside a
readout.** An instrument is not necessarily a rectangle with a list in it: it is
a set of parts, each with a function, arranged so that the reading is
unmistakable and the rest is quiet.

The material: machined, anodised, engraved. Each surface a milled plate with a
bevelled edge — a light above, a shadow below — so the stack has real
thickness. The accent is a phosphor, used sparingly, exactly where the reading
is live. Monospaced figures on a visible grid.

The motion: instantaneous where a machine would be, and nowhere else. A value
that changes re-reads and flashes once rather than animating into place.

The detail to get right: the freshness of the reading. An instrument that does
not say when it was read is decoration.

## Brief C — La Carte, by the author of L'Atelier

*A package manager as a chart.*

The sources are territories, the packages are landmarks, the installed ones are
ports, and the updates are what the chart says is out of date. The user is a
navigator reading a coastline.

**The arrangement is yours to find, and it may not be a legend beside a
landscape.** A chart can be a strip, a spread, a fold; a map can be read from
anywhere. The question the arrangement must answer: where is the user looking,
and what is the first thing they see there?

The material: translucent and deep, plates over a dark ground, a slow fog at
the horizon, landmarks with a soft halo. The type is a chart's: wide, calm,
spaced, with labels that behave like map annotations.

The motion: travel. Changing the territory pans the layers at different rates,
so the movement has depth. A selected landmark is ringed, and the ring settles
with a slight overshoot.

The detail to get right: the legend. It says what the territories hold, how
much of each is on the chart, and which parts are unreachable by the current
bearing. The coverage of a source is a fact, and the legend is where it is
stated.

## Brief D — L'Atelier, by the author of Le Comptoir

*A package manager as a workbench.*

The packages are tools. The sources are trays. The operations are the things
you do at the bench. The user is at work.

**The arrangement is yours to find, and it may not be a column of trays beside
a mat.** A bench can be approached from one side or from two; a drawer can be
above, below, or out. The question the arrangement must answer: what is within
reach, and what is out of it?

The material: warm neutral greys, a cutting-mat green, steel, cork. The surfaces
have grain, a soft bevel, a shadow that is a shadow and not a black rectangle.
A selected tool is picked up: it tilts a degree, its shadow separates from the
bench, and the inspection card is laid beside it.

The motion: physical, slightly heavy. Picking a tool up takes a moment and
settles with a small rebound. A tray slides in with its contents already in
place.

The detail to get right: the empty bench. What a bench looks like when there is
nothing on it, and what it invites you to do next — the empty slot saying
which kind of package would fit there.

---

## The output, for each author

| Artefact | Path |
|---|---|
| The loaded screen | `evidence/design-proposals/r2/<n>-<name>-loaded.png` — 1600×1000, dark |
| The quiet screen | `evidence/design-proposals/r2/<n>-<name>-quiet.png` — 1600×1000, dark, the state after opening |
| The source | `evidence/design-proposals/r2/<n>-<name>.figcraft` |
| The rationale | `evidence/design-proposals/r2/<n>-<name>.md` — the arrangement and why, the information budget (what is on the surface, what waits), the empty region and what it is for, the material, the motion, the second-pass changes |
| The note | `evidence/design-proposals/r2/notes/<n>-<name>.md` — a question to the other three. Not a design discussion. |

**How to build.** `figcraft-cli`, one process per file, so nothing is shared:

```sh
figcraft-cli run build.jsonl --out design.figcraft
figcraft-cli export design.figcraft -o design.png --scale 2
```

`build.jsonl` is a list of `{"command": id, "params": {...}}`; a step can refer to
an earlier one with `{"$step": n}`. The catalogue is 561 commands — look them up
with `figcraft-cli commands --filter <text> --json` rather than guessing. A frame
created without `sizingH`/`sizingV` is fixed at 100×100 and clips its children; a
frame that must size to its content needs `"sizingH": "hug"`. Auto layout owns
child positions: set `layout`, `gap`, `padding`, `mainAlign`, `crossAlign`,
`sizingH`/`sizingV` — never hand-place children of an auto-layout frame. Fills
and strokes are lists painted bottom-first. `effects` carries drop shadows. A
gradient is `{"type":"linearGradient","stops":["#7048E8","#1098AD"]}`.

**The loop that matters.** Build, export, *look at the PNG*, fix, export again.
The engine does not lie about the structure, but only the image tells you
whether it looks right — and in this round, only the image can tell you whether
the space breathes.

**Do not** open the Calix repository's application code, do not run or screenshot
the running Calix application, and **do not open the first round's images** in
`evidence/design-proposals/*.png` — they are the thing this round exists to get
past.
