# 04 — Le Comptoir: a package manager as a shop counter at night

*Round two. Two states of one arrangement.*

| | |
|---|---|
| Loaded | `04-comptoir-loaded.png` — the counter after a search, `ri` on the slate |
| Quiet | `04-comptoir-quiet.png` — the counter the moment the shop opens |
| Source | `04-comptoir.figcraft` (two frames: *Le Comptoir — loaded*, *Le Comptoir — quiet*) |

---

## 1. The arrangement, and why

The first round put the sources in a column on the left. This round removes the column
entirely and replaces the whole application with **one counter, seen from the customer
side, with the shop's board on the wall behind it**. There is no rail of sources, no
sidebar, no tab bar: the seven sources are **seven counters on a board**, a horizontal band
across the top of the screen, and the operations are **one physical switch** — a till —
standing on the counter in the foreground. The two axes cannot touch, because they are not
the same kind of object: one is a wall of counters, the other is a thing you move your hand
across.

The arrangement is built in **four depth planes**, and the reading order is front to back:

1. **The near edge of the counter (foreground, y 872–1000).** A till receipt, still coming
   out of the printer, running off the bottom of the frame. It is the only element that
   breaks the frame's edge, and it carries the backend's actual commands —
   `pacman -Sy --needed ripgrep`, `verifying signature ... ok` — in ink on pale paper, at
   a smaller size than anything else, because a log is a record and not a headline.
2. **The counter's surface (y 606–870).** Three props stand on the wood, and they are
   objects, not panels: the **till** at the left (the operation switch), the **ticket** in
   the middle (the selected package's description, propped up where you would prop a
   shop's descriptive card), and the **slate** at the right (your request, written in
   chalk, in a wooden frame).
3. **The shelf (y 282–556).** The packages stand on a plank as objects: a small pale label
   pinned to each, the version and provenance printed on the card, a line of description,
   and a price tag hooked on the shelf's lip with the size and the stock state. The lip
   itself is engraved with the state of the list. A brass pendant lamp hangs over the
   middle of the shelf, and the pool it makes is what the eye lands on.
4. **The wall (y 0–282).** Panelled, with a chair rail, and carrying the **board**: seven
   cells, each with a lamp, a name, a word for what that counter holds, and a count. The
   hanging sign is in front of the board's left end, in the foreground plane, with the
   scallop shell lit above the wordmark.

**Why this and not a column.** The product's premise is that a user comes for one thing,
takes it and goes. A column of seven sources says "here is a catalogue"; a counter says
"here is what is on the counter, here is what is behind it, and here is what is out of
stock". The board says *behind* — you can see that there are seven counters and that one of
them is empty, without being able to see into any of them. The counter says *on the
counter* — five objects, one of them in your hand. The receipt says *what just happened*.
Nothing lists the whole shop.

**The two states are the same arrangement.** Nothing moves, nothing collapses, no panel
appears or disappears. In the loaded state the counter's middle holds the ticket, the shelf
holds five packages, and the receipt is printing. In the quiet state the same three props
are in the same places — the till at rest, the slate asking — and the only things missing
are the things that have not happened yet: no package has been asked for, so the shelf is
bare, and nothing has been selected, so the counter's middle is bare. **The arrangement is
the arrangement in both; only the shop's contents change.**

## 2. The information budget — eight things, and what waits

On the surface, at once, in the loaded state:

| # | kind of information | where |
|---|---|---|
| 1 | **the seven counters** — name, what it holds, how much, which one is lit | the board |
| 2 | **the freshness of the answer** — `ALPM INDEX · 4 MIN OLD` | the board's rail, right end, always |
| 3 | **the operation** — FIND / INVENTORY / DELTA, and the handle's position | the till |
| 4 | **the request** — the query being asked for | the slate |
| 5 | **the list** — name, version, provenance, description; size and stock state | the packages and their tags |
| 6 | **the state of the list** — how many on the shelf, how many in the shop, the sort; and the density switch | engraved on the shelf's lip |
| 7 | **the selected package** — version, provenance, size, dependencies, files, state, and the command Calix would run | the ticket on the counter |
| 8 | **the log** — what the backend actually ran | the receipt |

That is the whole surface. **What waits, and where it went:**

- **The key map.** Off the surface entirely — a help overlay. There is not one key glyph
  anywhere on either screen. The slate says *type a name*, which is a gesture, not a key.
- **The other six counters' contents.** Their names, their kinds and their counts are on
  the board; their goods are not on the counter, because you are not at that counter.
- **The dependency tree and the full file list.** The ticket states the first line of each
  (`libpcre2-8 · gcc-libs · zlib`, `/usr/bin/rg · man1/rg.1.gz`); the rest waits behind a
  gesture on the ticket.
- **Remove.** It is an action on a thing you hold, so it is not on the board or the till;
  it belongs on the ticket, and in the quiet state there is nothing to remove yet.
- **The history.** Only the current operation is printed. The receipt is the last thing
  that happened, not a log you can scroll.

**The two densities are two lists.** `DETAILED` is what the loaded screen shows: a package
with its label, its version, its provenance, a line of description, and a tag with its
size. `COMPACT` is the same packages, the same columns, less in each: the tag and the
name only. The switch is at the end of the shelf's lip, where a shop's own fixture would
be, and the state line next to it says which one is in force.

## 3. The empty region, and what it is for

**The counter's middle is empty on purpose in the quiet state**, and the empty region is
the same region the ticket will occupy when something is selected. A shop counter before
an order is a bare counter, and the space is not a hole to be filled: it is the place the
thing you ask for will be put down. It is composed rather than accidental because

- the pendant lamp is on, and the pool it makes falls exactly on that bare wood, so the
  emptiness is **lit**, and shaped, with a soft edge rather than a flat fill;
- the bare shelf above it keeps its furniture — the plank's edge, the chair rail on the
  wall, the lip, and the empty brass eyelets where the price tags will hang, so the shelf
  reads as a fitted shop fitting waiting for stock, not as a rendering that failed;
- the two props that survive (the till at rest, the slate with the question) hold the
  left and the right of the frame, so the empty middle is framed, not abandoned.

In the loaded state the same space holds the ticket, and the deliberate emptiness moves
to **the wall's left corner, under the hanging sign** — the sign needs air around it, and
a lit bare wall is what the end of a shop looks like at night. That corner is the region I
would defend as the proposal's empty space, and it exists in both states.

## 4. The material

**The wood is warm without inventing a colour.** The token set has no brown, so the wood
is `anthracite` mixed with the accent's shadow side — `mix(#111418, #C95400, 0.05…0.20)`
— which is the only warmth the palette admits, and it is a *token* warmth, not a new hue.
The result reads as dark oiled wood rather than brown plastic because the lamp pools
(`#FFE5D2` at 6–20% over it) do the colouring, not the base tone.

**The metal is the slate family.** Blackened brass is `mix(slate, anthracite, 0.3…0.7)`,
the lamps' shades and the till are lit on one side and dark on the other, and the board's
rail is the darkest of them. The stem of the mark is the same family, so the shell in its
cup belongs to the same object world as the till.

**Paper is the inversion.** The shop's own labels are pale (`mix(shell-white, anthracite,
0.52)`) with the package name in **dark ink**, and the receipt is paler still with the
backend's commands in dark ink. These are the only places in the interface where the ink
flips, and that is exactly how a shop works: the objects are lit and dark, the paper is
bright and printed on.

**Surfaces have grain and a bevel.** Every wood plane carries the engine's fine noise; the
plank, the lip and the counter each have a lit top edge and a dark underside; the
shelves' front edge catches a 2px rim of `#FFE5D2`. Shadows are soft, offset, and
coloured by their distance from the ground — the board drops one, the ticket drops one,
the sign drops one, the packages drop small ones onto the plank.

**The type is a shop's.** A plain grotesk (IBM Plex Sans) for the goods, because that is
what the goods say; condensed capitals (IBM Plex Sans Condensed) for the labels — the
counter names, the till's positions, the stock states; and IBM Plex Mono for every figure a
machine owns: counts, versions, sizes, the command, the log, the freshness stamp, the
shelf's engraved state. The mark is on the hanging sign at 72px, and `CALIX` is alone
under it.

## 5. The motion

- **The shelf slides.** Changing counter or changing tool slides the shelf in from the
  side, and the objects move at different depths: the front row fastest, the back row
  slowest, so the shelf reads as a volume rather than a repaint.
- **A selection lifts and settles with a small bounce.** The chosen package is *taken off
  the shelf*: it rises 28px, tilts 2.6°, and throws a soft shadow onto the shelf and on
  the package behind it. The place it came from is left visible — a clean rectangle in the
  dust on the plank. The trace of the motion is in the image: the object is still in the
  air, and its shadow has already arrived.
- **The till's handle moves between its positions.** The loaded screen catches it in
  travel, tilted 5°, with two fading ghosts of itself behind it and the FIND detent lit
  ahead of it. In the quiet state it is at rest, in the FIND notch, with no trail.
- **The receipt is still printing.** It runs out of the frame; there is more of it below
  the screen than above.

## 6. What I would change in a second pass

- **The board is too even.** Seven cells of identical weight read as a menu, not as a
  shop. A real counter board would carry live facts: the AUR cell showing how many recipes
  are currently failing to build, the Flatpak cell showing a pending update, one cell
  closed for the night. The board should be alive, and I only used it to enumerate.
- **The compact list is a switch, not a drawing.** I show `COMPACT` as a position, but the
  brief wants two lists and only one of them exists in the image. The second pass would
  draw the compact shelf as a second arrangement of the same plank — flatter packages, the
  tag carrying the state alone, the description gone — so the two densities can be
  compared rather than asserted.
- **The receipt cannot grow.** A log on a paper strip is charming until there are twenty
  lines of it. The second pass would make it a roll that unrolls upward, and would decide
  what the surface shows when an operation fails.
- **Removing has no home.** "Take it back" is a real action of a real shop, and I left it
  off the surface entirely, which is a dodge. The second pass would put a small brass
  `RETURN` beside the ticket, and work out what it means to return something that came
  from AUR.
- **The quiet state might need a third state.** "Open, but nothing asked for" is one
  moment; the shop *closed* — lamps off, board dark, the till shuttered — is another, and
  it may be the more honest empty. The arrangement should survive that too.
- **The wall's left corner is doing too much.** It is empty in both states, and it is
  holding the sign's shadow and the light falloff. If the second pass gives the corner a
  job, the emptiness has to move somewhere else, and I would rather find that place by
  drawing than by arguing.
