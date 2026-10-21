# 05 — A reading of the four proposals

Four authors, four briefs, one product. This is what the proposals say, what
they cost, and what they agree on.

## The four, in one line each

| | The metaphor | The navigation it produced |
|---|---|---|
| **01 Le Comptoir** | a package manager as a shop counter at night | the *wall*: the sources are drawers in a cabinet, the three operations are shelves of the main surface, and the operation is a physical till-switch that moves between three positions |
| **02 L'Instrument** | a package manager as a precision instrument | the *panel*: the sources are channels of a selector with a needle, the operations are a three-position function rocker, and the update set is a gauge attached to the selected channel rather than a list |
| **03 La Carte** | a package manager as a chart | the *chart*: the sources are territories in a legend, the operation is a bearing written in the header on a compass rose, and the keyboard is the helm |
| **04 L'Atelier** | a package manager as a workbench | the *bench*: the sources are trays, the operations are a rail of tools above the surface, and a selected tool is picked up, its inspection card laid beside it |

## What they agree on, without having spoken

Four blind authors, one product, and they converged on five things. That
convergence is worth more than any one of the images.

1. **The two axes are separated.** Every proposal puts the seven sources in the
   left column and the three operations in a rail at the top of the surface. Not
   one of them let a source and an operation share a list. This is the failure
   the brief named, and four independent readings found it independently.
2. **The mark is present.** Not a glyph in a corner: a chalice whose cup is a
   scallop shell, at the top left, with `CALIX` alone beside it.
3. **The non-interchangeability is made explicit.** The Instrument tags each
   channel `BIN` / `SRC` / `SBX`; the Atelier states the rule of the bench — *a
   tray holds only its own kind*; the Carte's legend says what each territory
   holds and how much of it is out of date; the Comptoir writes the kind under
   each counter. A recipe can be built, a prebuilt binary cannot — and the
   design says so before the user asks.
4. **The freshness of the answer is a first-class fact.** All four put the age
   of the reading somewhere permanent. A package manager that cannot say when
   it last looked is decoration.
5. **Two densities, both lists.** Every proposal offers a detailed row and a
   compact row, and every one of them is careful to say the compact one is *not*
   a grid of cards.

## What each one costs

**Le Comptoir** is the warmest and the most human, and it is the one that risks
drifting into skeuomorphism: the shelves, the wood, the brass. Its content area
draws empty shelves *above* the list, which read as broken rows rather than as a
state — the same mistake the Atelier makes with its empty slot, and both are
fixable by moving the empty state out of the flow.

**L'Instrument** is the most disciplined, and the most severe. Monospaced
everywhere makes the dense list heavy, and the instrument's own doubt is the
interesting one: is the function axis global, or does each channel carry its own
set? A Flatpak keeps an inventory, ALPM keeps another, Cargo keeps none — and
the Instrument's own answer (draw the absence: `NO BUILD RECIPE — REPO BINARY`)
is a mitigation, not a resolution.

**La Carte** is the most evocative and by far the most textured — the contour
lines, the fog, the halo on a selected landmark. It is also the most
overstuffed: the left column carries the legend, the symbol key, the two inks
and the keyboard at once, and the legend's isotype strip is mostly empty cells
for the small territories, which the author names as the first thing to change.

**L'Atelier** is the most productive metaphor, and it produced the sharpest
sentence of the four: *a tray holds only its own kind*. The rule, the picked-up
selection, the empty slot that says which kind of package would fit there, the
inspection card lifted from the bench, the log with its timestamps — it is a
workspace, not a museum. Its grain is the risk: at 12px a noise that is right on
a 4K monitor is mud on a 1080p one.

## What the brief could not tell them

The brief described the product, not the project's rules. Two proposals print
`sudo pacman -Syu …` as the command the application would run — and the
application does not do that: it elevates through polkit, and the command it
shows is `calix install standard <name>`. That is a detail of the proposals, not
a defect in the product, but it is where the implementation will differ.

## The three questions the authors left each other

They were not allowed to discuss the design, so they asked the product instead.
These three are real, and they change the implementation.

1. **How far should a package manager show the command it will run?** The
   Instrument asks it, and notes that a command is only honest for one of the
   seven sources.
2. **Should the frequency of use decide the weight in the navigation?** The
   Carte asks it: a daily Arch user touches ALPM every day and the tracked
   releases almost never — yet all seven are named, and a legend that gives a
   rarely-used shore the same weight as the harbour town may be the wrong map.
3. **Should the navigation let a user discover that one thing lives in more
   than one place?** The Atelier asks it: `ripgrep` exists as an ALPM package,
   as a Flatpak, and as a Cargo crate, and AUR is not a binary at all. Does
   each source answer on its own, or does the product show that one package
   lives in several places?

### The answers

1. **The card shows the product's own command** — `calix install standard
   ripgrep` — and the log shows what the backend actually ran, underneath. The
   product does not dress as a terminal, and the command stays honest per
   source.
2. **A principled order, the rare sources grouped.** The seven keep a fixed,
   semantic rank — the binaries, then the recipes, then the sandbox, then the
   followed releases — and the rarely used ones are grouped rather than given
   the same weight as ALPM. The counts already say what the frequency is.
3. **Yes: the card says where else the package lives.** "Also in: AUR · Flatpak
   · AppImage", with each version. The inventory carries it, per source.

### A fourth question, from the Comptoir

The seven sources are not interchangeable: an ALPM package has no build recipe,
a Cargo crate has no repository to be removed from, an AppImage has no
dependencies to show. **When a user lands on a source whose operations differ —
or on an operation a source does not support at all — where does the interface
tell them what they cannot do here, without turning every screen into a list of
disclaimers?**

The four proposals answer it four times, in four registers: the Instrument tags
each channel `BIN` / `SRC` / `SBX` and draws the absence; the Atelier states the
rule of the bench — *a tray holds only its own kind* — and marks the tray that
cannot be followed; the Carte's legend says what each territory does not hold;
the Comptoir writes the kind under each counter.

The answer, then, is that **the absence is expressed by the absence of the
action**, and the reason lives in the source's own entry. A source that cannot
be removed from simply has no *Remove*; the reason is one line under the
counter. The interface states a limit by *not offering the thing*, and the
source's entry says why — which is the opposite of a disclaimer, and the reason
it survives the four proposals.


## The reading, from here

The four are not four skins. They are four answers to *what kind of thing a
package manager is*, and each answer produces a different navigation, a
different depth, a different motion. The choice is not cosmetic, and it is not
theirs to make: it is the product's.

What the four prove, and what the next slice inherits, is the **separation of
the two axes** — the sources on the left, the operations on the top, never
merged — and the **explicit non-interchangeability** of the sources. Those two
are in every proposal, and they are what the requalification's contract
already asks for.
