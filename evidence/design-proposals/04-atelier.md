# 04 — L'Atelier: a package manager as a workbench

*The packages are tools. The sources are trays. The operations are the things you do at
the bench. The user is at work.*

Image: `04-atelier.png` (1600×1000) · Source: `04-atelier.figcraft`

---

## 1. The navigation I reasoned out

**Two axes, two places, never mixed.** The seven sources are **trays**, and they sit in a
column on the left of the screen, permanently: ALPM, AUR, FLATPAK, APPIMAGE, CARGO, NPM,
TRACKED RELEASES. The operations are **tools**, and they are a rail along the top of the
work surface: FIND, INVENTORY, DELTA. A tray is chosen, a tool is chosen, and the surface
below shows what that tray looks like through that tool. The two axes never occupy the same
affordance, so "AUR + installed" and "Flatpak + updates" are two different gestures — one
along the column, one along the rail — and neither one is a mode you have to remember.

**The third operation is a verb, not a view.** *Remove* is not a browsing operation; you do
it to a thing you have selected. So REMOVE lives on the inspection card, next to INSTALL,
and not on the rail. That is the one place where the Atelier reading costs a little
explicitness, and I paid for it by writing the rail's three tools out loud (FIND, INVENTORY,
DELTA) with their tallies (1,284 / 612 / 18) so the user can see that a tool is a *lens*, not
a page.

**Where am I?** The header is the answer, and it never changes shape: `TRAY ■ ALPM ›
TOOL FIND › FILTER name ~ z`, with the tray carrying a signal-orange square, the tool
carrying the accent itself, and the filter written as the query the user typed. To the far
right of the same header, the **freshness of the answer**: `INDEX FRESHNESS · 4 MIN OLD ·
20:14:09`, with an orange dot that goes cold when the index goes stale. The state of the
*list* is a strip on the surface itself, above the column headings:
`6 SHOWN · 1,284 IN ALPM · FILTER name~z · SORT NAME ↑`, with a `DETAILED | COMPACT`
density switch at the right end — the compact list is not hidden, it is one key away
(`⌘⇧D`), and the switch shows it in the same place the user will look for it.

**What can I do here?** The actions are the ones that are true *for this source*. The card
for `zsh` says, in the card's own words: `extra — official repository`, `SIGNED BY THE
PACKAGERS. NO BUILD RECIPE — OFFICIAL PACKAGES ARE ALREADY BUILT.`, and at the foot of the
card: `THESE ACTIONS BELONG TO THE ALPM TRAY: NO BUILD, NO RECIPE.` The AUR tray says
`recipes, not binaries`; the FLATPAK tray says `sandboxed applications`. The rule is
engraved once on the bench itself, in the tray column, so it is a property of the space and
not of a help page: *A TRAY HOLDS ONLY ITS OWN KIND. A FLATPAK IS NEVER UPDATED FROM A CARGO
TRAY.*

**How do I get elsewhere?** The whole application is under the keyboard, and the keyboard is
printed on the bench, in the console's right-hand corner, as a legend a workshop would have
stencilled on the wall: `⌘1 FIND · ⌘2 INVENTORY · ⌘3 DELTA · ⌘4 CARD · ⌘5 CONSOLE ·
⌥←→ TRAY · ⌘F SEARCH · ⌘L LIST · ⌘⌥C COPY CMD · ESC CLEAR · ⌘⇧D DENSITY · ⏎ INSTALL`.
`⌥←`/`⌥→` walk the tray column; `⌘1…⌘3` pick the tool; `⌘4`/`⌘5` jump to the card and the
console. Escape clears the filter and puts the tray back in charge. The trays themselves are
the way back, and the selected one is unmistakable — orange edge, raised, count in orange.

**The empty bench.** The brief's detail to get right, and I gave it two homes. The seventh
tray, TRACKED RELEASES, is *empty*: a recessed, dashed slot with nothing in it and one
engraved invitation, `⌘F TO FOLLOW A RELEASE`. It says which kind of package would fit
there and it says what to do next. And on the lower half of the bench — the mat below the
list, past the fade — there is a second empty thing: a dashed **EMPTY SLOT** card reading
`AN OFFICIAL-REPOSITORY PACKAGE WOULD SIT HERE. ⌘F TO PLACE ONE`. The bench is not a
blank rectangle; a blank rectangle is a bug, and an empty slot is an invitation.

## 2. The material

**Everything is token-derived.** The palette is the brief's palette, used as given:
anthracite `#111418` is the ground; shell white `#F7F3ED` is the ink *and* the warmth; muted
`#A1A6AD` is the secondary; slate `#667085` is the hairline and the stem; signal orange
`#FF6A00` is the accent, with `#FFE5D2` for the rim light, `#FF8E40` for hover and
`#C95400` for the shadow side; royal blue `#4169E1` is held in reserve as the alternate
accent, the way the mark is held in reserve. Every intermediate tone in the image is
`mix(anthracite, shell-white, n)` or `mix(slate, …)` — warm neutrals, steel, cork — so the
material has warmth without a single invented hue. (The brief's *cutting-mat green* is the
one instruction I did not take literally: a green is not in the token set, so the cutting
mat is the **grid** and the **slate-tinted** surface instead of its colour.)

**Grain, bevel, shadow.** Every large surface carries a fine noise (the engine's `noise`
effect at 4–7%), so the panels read as pressed board rather than as `#111418` rectangles.
Bevels are drawn, not implied: a 1px light line on the top edge of a raised panel, an
inner shadow on the recessed ones (the search field, the command plate, the empty slot), and
a hairline under every divider. Shadows are shadows — soft, offset, coloured by their
distance from the ground — never black rectangles.

**Depth is the first thing the eye reads.** The stack, from the ground up: the tray column
is a cork-ish cheek; the tool rail is a steel channel with a lip; the work surface is a mat
laid on the bench, with a grid and a bench rule along its lower edge; the inspection card is
a card standing on the bench with a real shadow; the console is an apron across the front of
everything, separated from the mat by a shadow that falls *upward* onto it.

**Type is a workshop's.** IBM Plex Sans for the labels — plain, workhorse, the face of a
label on a box. IBM Plex Sans Condensed, bold, wide-tracked, uppercase for the tray names
and the tool names: stencilled capitals, the lettering a tray would be stamped with. IBM
Plex Mono for every figure that a machine owns — counts, versions, sizes, commands, log
lines, the freshness stamp, the keyboard legend. The name is `CALIX`, alone, with the shell
mark beside it at 70px, not a 16-pixel glyph in a corner.

**The mark.** A scallop shell as the cup of a chalice: nine ribs radiating from the hinge,
a gently scalloped margin with a rim light along it, a metal cup in front of the shell's
foot so the shell sits *in* it, and a brushed aluminium stem and foot below. The cup's
gradient runs light to dark across a cylinder; the shell's runs from `#FF8E40` on the lit
shoulder to `#C95400` in the shadow. It is the only saturated thing on the screen apart from
the accents, and it is big enough to be the product's voice.

## 3. The motion

Physical, and slightly heavy. Nothing in this design snaps.

- **Picking a tool up takes a moment and settles with a small rebound.** The chosen block
  in the rail lifts, tilts 2.2° off the rail, and its shadow separates from the steel
  beneath it. The trace is in the image: FIND is caught mid-lift, its orange edge raised
  above the channel it sits in.
- **The selected package is picked up off the bench.** The `zsh` row is tilted 0.55°, its
  orange edge bar is exposed, and its shadow has come away from the mat. The inspection
  card beside it is captioned `LIFTED FROM THE BENCH`, and the card is *for that package* —
  its version, its provenance, its dependencies, its files, the exact command.
- **A tray slides in with its contents already in place.** The ALPM tray is offset and
  raised, carrying its count and its three repositories (`core · extra · multilib`) with it.
- **Scrolling glides, and a hand stops.** The list is caught mid-settle: the next row
  (`zstd`) is dissolving into the mat through the fade, and the scroll thumb is still
  travelling. Nothing here jumped.
- **The console is never still.** The sync bar carries a bright head at 68% — a moving
  edge — and a second operation is queued behind it.

## 4. What I would change in a second pass

- **The empty bench deserves a whole screen, not a card.** The strongest thing in this
  proposal is the empty slot, and it is currently competing with six populated rows for
  attention. The second pass would put the empty state in its own composition — the tray
  open, the mat bare, one slot marked — and let the *populated* bench be the second image.
- **The count `6 SHOWN` is doing too much work in one line.** "6 shown of 1,284 in ALPM,
  filtered by name~z" is three facts in a row of monospace, and the eye has to parse it. The
  second pass would give the count a fixed position and a fixed typeface, the way an
  instrument would.
- **The two-axis model has a hole: a package that exists in more than one source.** With
  trays, `ripgrep` can be in the ALPM tray and in a Flatpak tray, and the bench shows them
  as two separate objects. The second pass would add a **tie-line** — a hairline and a
  count across the tray column, or a small "also in: 2 other trays" tag on the card —
  because separating the axes must not hide the overlap.
- **The card is too tall and too quiet.** Provenance, state, dependencies, files, the
  command: five plates, each with a stencilled heading, and the user has to read a lot
  before acting. The second pass would collapse dependencies and files into a scrollable
  region and keep the command plate permanently in view, because the command is the thing
  the user came for.
- **The tool rail's tallies are an assumption.** 1,284 / 612 / 18 are plausible, not
  known; if the real numbers are wildly different (1,284 in ALPM is the number the
  *header* claims, and the *rail* claims the same number for FIND, which are not the same
  thing), the rail's semantics need tightening. The rail should say how many packages the
  tool would *show*, not how many exist.
- **The mark wants a second colourway proof.** Royal Blue is in the tokens as the
  alternative accent, and I did not use it. The second pass would show the same screen
  with the shell in royal blue, to prove the identity survives the swap — and the accent
  would follow the mark, not the other way round.
