# Le Comptoir — a package manager as a shop counter at night

**Proposal 01 of 4.** One screen, 1600×1000, dark. The whole application.

The source is `evidence/design-proposals/01-comptoir.figcraft`; it is built by
`evidence/runs/design-01-comptoir-20261020/build.py`, which emits the FigCraft
command log `build.jsonl`.

---

## 1. The navigation I reasoned out

The product has two axes and the failure the brief warns about is mixing them.
So the screen has exactly two navigations, and they are different *kinds* of
thing — one is a wall you pull from, the other is a mechanism you move.

**The wall (left column) is WHERE.** Seven drawers, one per source, always
present, in a fixed vertical run: ALPM, AUR, FLATPAK, APPIMAGE, CARGO, NPM,
TRACKED RELEASES. Each drawer carries its name, a one-word descriptor of what
that source actually *is* (official binaries / build recipes / sandboxes /
single binaries / crates / node packages / git releases), a count, and its key.
A user's hand learns the order of that column in an hour, which is the whole
point of making it a wall: it never moves, so it never has to be explained
twice.

**The till (right column) is WHAT.** The three operations — search, installed,
updates — are three positions of one physical object. Not a row of buttons: a
milled aluminium channel with three detents and a brass lever that *travels*
along it. The lever is in the third position in this frame (UPDATES), and the
channel carries a polished groove recording where it has been. This is the
detail I was asked to get right, so I made it the only object in the interface
that is allowed to be bright brass.

**The shelf (centre) is the WORK.** The three operations are literally the
three shelves of the case. Two of them — SEARCH and INSTALLED — sit in the
case behind, pushed back, in shadow, with their goods barely visible. The
third, UPDATES, has been pulled out of the case and forward onto the counter;
the empty slot it came from is still open, labelled `SHELF OUT`. The wall is
always there; the shelf changes. That is the thesis of the brief, drawn.

**Where does `remove` go?** Nowhere as a shelf. Removal is a verb you do to a
thing, not a list you look at; there is no "list of things to remove" that is
interesting on its own. So REMOVE is a button on the detail board, next to
PIN and CHANGELOG, and the detail board also carries the exact command line.
That is why the till has three positions and not four.

**The two densities.** The DETAILED list is the one shown: package, version
delta, provenance, size delta, state, and a line of description. The COMPACT
list is not drawn as a second screen; it is made legible as a possibility in
three places — the `ROWS` control in the header, which shows the two row
shapes literally (a two-line row and a one-line row) with `⌘L` beside it; the
compartment legend inside the open ALPM drawer, which is a genuine compact
list of the same typographic system; and the keymap. The control makes the
alternative concrete without splitting the screen.

**The answer to the four questions, and where each lives:**

| | where it is answered |
|---|---|
| Where am I? | header: `SOURCE / ALPM` + the repository chips `core` `extra` `multilib`, `OPERATION / UPDATES`, `FILTER / outdated`, `8 shown · 812 in ALPM`, `SYNCED 4 MIN AGO`; the till names the operation; the wall's pulled drawer names the source |
| What can I do here? | the counter rail, explicitly scoped `FOR ALPM · UPDATES` (`UPDATE ALL · 8`, `PIN`, `CHANGELOG`); the detail board's `BUILD — official binary — no recipe`; per-row state chips (`OUTDATED`, and `HELD` on a pinned row, which is why its action is not "update") |
| How do I get elsewhere? | the seven drawers with their keys; `⌥↑ ⌥↓` to walk the wall; the till with `⌥← ⌥→`; `⌘F` in the filter; `⌘L` by ROWS; `⌘⌥C` on the command; `esc` in the console; and the whole map engraved on a brass plate at the foot of the wall |
| What is the state of the list? | count and filter in the header, filter chip again on the rail, the shelf's brass plate reading `UPDATES 8 OF 812`, and the age of the answer at the top right, always in the same place |

**The keyboard.** `⌘1`–`⌘5` jump to the five sources a user visits most
(ALPM, AUR, FLATPAK, APPIMAGE, CARGO); `⌥↑`/`⌥↓` walk the entire wall,
including NPM and Tracked releases, which have no number of their own;
`⌥←`/`⌥→` move the till; `⌘F` filters the shelf; `⌘L` switches density;
`⌘⌥C` copies the command; `esc` goes back or cancels. Every one of these is
printed in the interface, near the thing it controls.

---

## 2. The material

Warm dark wood and blackened brass, and nothing outside the token set.

**The wood** is the anthracite ground with the signal-orange token composited
over it at low alpha — literally `#111418` plus `#FF6A00` at 4–14%. That is
why the surfaces are warm browns rather than grey panels, without a single
invented hue. The grain is a noise effect in **slate**, blended `overlay`, at
50% density and one document unit — you do not see it, you notice it.

**The brass** is signal-orange 700 (`#C95400`), darkened toward the ground:
the drawer pulls, the shelf's plate, the rail, the engraved keymap. The rim
light is signal-orange 100. Only the till's lever is polished brass
(`#FF8E40` → `#FF6A00` → `#C95400`); it is the one object allowed to be
bright, because it is the thing you move.

**The counter** is wood. **The till's plate** is brushed aluminium — slate
with a muted highlight and a fine grain — because a cold metal mechanism
mounted on a warm wooden counter is the clearest possible statement that the
operation is a *control* and the shelf is a *place*. The mark's stem is the
same aluminium, so the identity's material reappears as the mechanism's.

**The mark.** A scallop shell as the cup of a chalice: seven ribs fanning from
the hinge, a scalloped margin, the fan seated in a metal cup with a lit rim,
and a brushed aluminium stem with a knop and a foot. It is drawn as vectors,
not a glyph: 84 px tall in the header, where it is the first thing the eye
lands on after the wordmark, and again — engraved as a line drawing, the shop's
crest — on the back panel of the case. `CALIX` stands alone: no framework
name, no version, no subtitle. The shell's fill is a single variable, so the
Royal Blue alternative is one substitution away; I did not put blue on this
screen, because the brief for this direction is warm wood and blackened brass,
and one accent that means one thing beats two that mean two.

**The lips.** Every shelf, drawer, rail and board has a lit lower edge. The
shelf's front rail catches a warm light along its bottom; the drawer fronts
have a brass pull and a chamfered top edge; the case's shelves have a lip.
This is the cheapest way to make a flat rectangle read as a solid object.

**The depth, which is the point.** The screen is a stack of planes: the case
behind, the pulled shelf forward of it, the front rail in front of the goods,
the selected row lifted off the shelf, the detail board raised off the ground
with a shadow under it, the till above the detail board, the header fascia
above everything, the console recessed below everything. A selected object
lifts and throws a soft shadow on what is behind it — the `ffmpeg` row sits
2 px above its neighbours, with a lit top edge and a real shadow on the row
below. The drawer pulls out of the wall and casts its shadow on the drawer
below it. That is the layering the brief asks for, and it is what the eye
reads first.

---

## 3. The motion

The shelves slide.

**Pulling a drawer.** The drawer front comes out of the wall and down, its
box interior visible from above, and its shadow falls on the drawer beneath
it. The goods inside arrive at different depths — the front row first, the
back row last — so the drawer reads as a volume, not a rectangle. The open
ALPM drawer in this frame is caught mid-pull: the interior is exposed, the
front has moved eight pixels right, and the drawer below is already in its
shadow.

**Moving the till.** The lever travels along the channel from one detent to
the next, decelerating into it. It does not cross-fade; it *slides*, and the
groove it polishes into the channel is the memory of where it has been. In
this frame the groove runs from SEARCH to UPDATES and the lever is at the
end of it.

**A selection lifts.** The chosen object comes up off the shelf about two
pixels, settles with a small bounce — a thing picked up and put back down —
and its shadow separates from the shelf beneath it.

**The list scrolls.** The rows glide with a little weight; they do not jump.
The frame catches a scroll still settling: the top row is fading into the
shadow under the shelf above it, and the last row is passing behind the
shelf's front rail, cut in half by it.

**Where the motion would be visible, the image carries its trace:** the
polished groove in the till, the lever parked at the end of it; the empty slot
marked `SHELF OUT`; the drawer pulled out of the wall; the list clipped top
and bottom; the progress bar at 62% in the console; the running job in flight.

---

## 4. What I would change in a second pass

1. **The empty slot is seventy pixels of nothing.** It says `SHELF OUT`, which
   is a caption, not a use. It should hold the shelf's ghost — a dim outline
   of what is on it — or it should be shorter and the space given to the list.
   Right now the case earns its area but the hole does not.
2. **The compact list should be drawn, not implied.** The `ROWS` control
   shows the shape of the alternative; a real second mode, with the same rows
   and fewer columns, would prove the column model survives compression.
3. **The till needs an off position and a real detent.** It has three stops
   and no rest. A spring return, or a fourth detent for "all sources", would
   make it a mechanism rather than a slider.
4. **The crest is decoration.** It should be a live mark — a lamp that burns
   when a source has work waiting, or the count of the source you last
   touched. A shop's crest that means nothing is a sticker.
5. **The wall only holds sources.** There is no place for "everything", or for
   a pinned package, or for the four packages a user keeps pinned across
   sources. One more drawer would have to appear at the top of the wall, and
   that changes the muscle memory the whole idea depends on.
6. **AUR's drawer should show recipes.** `61,204` is a number; the drawer
   should say what a recipe is and how many are broken, because "build
   recipes, not binaries" is the fact that separates it from ALPM and it is
   currently only a descriptor word.
7. **The accent should be per-source.** Royal Blue exists in the system and
   nothing on this screen shows it. A Flatpak that is an update, an AUR that
   is a build, a Cargo crate that is a pin — one accent per source would
   answer "where did this come from" from the row itself, and would let the
   seven sources stop being seven identical drawers.
8. **The state chip is too far from the name.** `OUTDATED` lives four
   columns to the right of the package name. In the compact list that column
   has to go somewhere, and the answer is probably next to the name.

---

## 5. How it was made

`evidence/runs/design-01-comptoir-20261020/build.py` emits the command log;
`figcraft-cli run build.jsonl --out 01-comptoir.figcraft` builds the document;
`figcraft-cli export` renders it. 422 commands, one process per file.

The PNG was rendered and read back nine times. The corrections that mattered:
the noise effect paints grains at full strength, so the grain colour has to
sit close to the surface it lies on — slate over wood, muted over aluminium
(`overlay`) — the first attempt covered the whole screen in orange static; the
shell's flanks were drawn on the wrong side of the hinge, which made the mark
read as an umbrella; the fan's ribs overshot the scalloped margin and had to
be pulled back inside it; the cup was narrower than the fan's flanks and left
a sliver of shell escaping below its rim; the till's lever sat on top of the
channel instead of in it; and the wood's first pass was opaque, which let the
shelf's front rail show the row behind it.

Fonts are system fonts: IBM Plex Sans for the goods, IBM Plex Sans Condensed
for the labels, IBM Plex Mono for the figures. On a machine without them the
render falls back.
