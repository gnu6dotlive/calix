# CONTRACT 01 — The audit of the spatial contract

**Audit 1.3 — 2026-10-21 — against `feat/contract-01-row-qualification`
(merged into `main` as `c2b8d6b7`)**

*Version 1.0 was the audit of the state before the decisions. Version 1.1
records the implementation of the four decisions. Version 1.2 adds the
qualification of the catalogue's row: the measurement that corrected the
minimum from 560 to 660, and the reconciliation of the window's floor that
the operator asked for. Version 1.3 adds the qualification of the **list**
that holds the rows — and the bench's blind spot, named: a list whose height
is not given is laid out at zero, and draws nothing.*

This audit confronts [`CONTRACT-01-SPACE.md`](CONTRACT-01-SPACE.md) with the
repository as it stands. It does not propose new primitives: it inventories
what already exists, measures what the contract asks for, and names the gaps
that need an architectural change.

Everything numeric below was **measured on the real views**, in the test
harness, on the machine this audit was written on. Where a number comes from
the screen rather than from the harness, it says so.

---

## 1. The method

The harness draws the application's own views and reads the bounds the window
actually drew. That makes the composition *measurable*, which is what the
contract's §6.5 demands. The audit's instrument is the test
`the_composition_is_measured_in_a_real_window`
(`views/package_workstation.rs`): it draws the real station — the bank, the
readout, the nameplate — in a real window, at the window's floor, and then
draws it again in a band shorter than the window, which is what the log
drawer does when it opens.

The one thing the harness cannot lay out is the `uniform_list` of the
catalogue itself (its measured layout never resolves there). Every band
*around* it resolves, which is what the audit needs.

## 2. The verdict

| Invariant | Verdict | The evidence |
|---|---|---|
| Window: the floors are coherent with what the application can compose | **held** | the floor (720 × 480) is derived and the budget adds up: the row's floor (495) + the name's share (165) + the folded spine (56) = 716 → 720. Tested: `the_window_floor_is_derived_from_the_catalogue_and_the_spine`, `the_window_floor_budget_adds_up` |
| Spine: a stable presence, it narrows but never vanishes | **held by a rule** | the spine folds itself when the catalogue would lose its room, and takes back the user's shape when the window gives it room. Tested: `the_spine_folds_when_the_catalogue_needs_the_room` |
| Body: the catalogue keeps a usable width and stays dominant | **held, and qualified against the row and the list** | the catalogue's minimum is 660 px, measured on the real row and the real names (§4.7), and the list that holds the rows draws them at every width (§4.8); at the window's floor the catalogue has 664 px; the three toolbar forms are measured |
| Context surfaces: opening one never reduces the body below its floor | **held** | the drawer cannot take the workspace's 320 px whatever the request, and below 120 px it overlays instead of squeezing. The plate yields first — and when the band cannot hold the catalogue's six rows *and* the plate's chrome, it leaves the flow entirely and comes over the catalogue, summoned by the same edge (§4.10). Tested: `the_drawer_never_takes_the_workspace_floor`, `the_plate_leaves_the_flow_before_the_catalogue_is_reduced`, `the_composition_is_measured_in_a_real_window` |
| Portrait and landscape: the composition adapts to the space, not to a declared orientation | **held, by not answering it** | no code reads the orientation; the composition is expressed in logical pixels and adapts by growing or shrinking the bands |
| Zoom: at every scale the content stays accessible | **held, and qualified as far as the harness allows** | the layout is in logical pixels and the scale factor does not leak into the geometry — measured at the bench's fixed 2.0 factor (§4.5). A real 150 % window is not measured: the harness cannot be set to it |
| Transitions: a change of width preserves the continuity | **held** | the vertical budget has one owner, so a change of height is a re-budget, not a re-landing; the spine's fold is animated like a travel, not a jump |
| Extensibility: a new surface uses the same primitives | **held** | the plate, the drawer and the sidebar are all bands; the render lab is a surface, not a place (`the_render_lab_is_not_a_place`) |

## 3. What already exists — the inventory

The brief asks not to invent what is already there. This is what there is.

### 3.1 The window's policy (exists)

- `config::MIN_WINDOW_WIDTH = 1024.0`, `MIN_WINDOW_HEIGHT = 680.0`
  (`config.rs:89-90`), enforced twice: by the config's own sanitiser
  (`sanitize_window_size`, `config.rs:159-168`) and by the compositor
  (`WindowOptions.window_min_size`, `main.rs:128-131`).
- The requested size comes from the user's config (1440 × 900 by default,
  `config.rs:64-66`), and the *mapped* size is what counts — the code says
  so explicitly, with the numbers that motivated it
  (`content_width`'s doc, `package_workstation.rs:69-80`).

### 3.2 The geometry tokens (exist)

`ui_metrics.rs` holds the plate's geometry: the spine (190 / 56), the rows
(36 / 30 / 32), the nameplate's bounds (floor 168, share 0.55), the splitter
(5). They are constants, and they are tested
(`ui_metrics::tests::test_splitter_and_sidebar_geometry_invariants`).

### 3.3 The division of the space (exists — twice, and that is the problem)

- **Horizontally**: `content_width(viewport, sidebar) = viewport - spine`
  (`package_workstation.rs:78-80`), one place, tested
  (`the_content_width_follows_the_mapped_window`).
- **Vertically**: `nameplate_bounds` / `nameplate_height`
  (`package_workstation.rs:46-56`), one place, tested
  (`the_nameplate_stays_inside_its_bounds_at_the_heights_a_tiling_compositor_uses`).
- The **drawer's** height is a third place: the user's config, bounded by
  [120, 600] (`config.rs:466-478`), and not bounded by anything else.

### 3.4 The presentation policy (exists)

`state/motion.rs` holds the curves as tokens, the durations, the reduced-motion
policy and the *porteur* — the mark that travels, shared by the list, the rail
and the view commutateur. Every entry is now measured on the image
(`slice-09/05-ACCEPTANCE-MATRIX.md`, A20–A24).

### 3.5 The keyboard (exists)

`state/keys.rs` is the single registry of the bindings, and the surface
carries no key glyph. The invariant "the keyboard leaves the surface" is
implemented, not wished for.

### 3.6 The surfaces (exist)

| Surface | Where it lives | How it shares the space |
|---|---|---|
| the spine | `views/sidebar.rs` | a fixed width, collapsed by the user |
| the body | the station's readout | `flex_1`, `min_h(0)`, `overflow_hidden` |
| the plate (the nameplate) | `views/inspector/` | a bottom band with a fixed height and a drag |
| the log (the drawer) | `views/operation_console.rs` | a bottom band with an animated height |
| the render lab | `views/render_lab.rs` | a surface that opens over the place, not a place |

The composition is already "the detail is a band, not a card in a column" —
the conclusion of the second design round (§ `evidence/design-proposals/06-ROUND-2.md`).

## 4. The findings — after the operator's decisions

**Decisions 1 to 4 are implemented.** What follows is the measured state, and
what is still open.

### 4.1 The window's floor is derived, and the catalogue's minimum with it

**Decision 1 and 2 — implemented.**

The window's floor is no longer a chosen number. It is derived from the
composition's own floors:

```
min_window_width  = the row's floor (495) + the name's share (165) + the folded spine (56)
                  = 660 + 56 = 716 → 720
min_window_height = the height at which the band still holds the bands   = 480
```

The name's share is 165 px: the identity glyph (14), its gap (8), and the
p90 name of the real packages (143). **This is the correction of the
reconciliation the operator asked for**: the earlier derivation
(`720 = 560 + 56`) was wrong on both terms — it did not add up (616 ≠ 720),
and the 104 px it left over had no name. The corrected budget has no
leftover: the 104 px were the name — the space the row does not give to
its text, and that the text takes. The test
`the_window_floor_budget_adds_up` checks the sum.

The contract's §4.1 asks for `min_window_width ≥ spine_floor + body_floor`;
the repository now has exactly that relation, and it is tested
(`the_window_floor_is_derived_from_the_catalogue_and_the_spine`).

The spine is not a victim of the floor: it **folds itself** when the
catalogue would lose its room (`spine_is_forced_collapsed`,
`effective_sidebar_width`, tested). It is a rule of the ladder, not a state:
when the window takes its room back, the spine takes back the shape the user
gave it. The spine never disappears — it keeps its five destinations, in
their compact form.

**The unreachable branches are now reachable and tested.** The toolbar's
breakpoint rule has three forms, and each of them is a *different* layout,
measured in the bench (`the_three_forms_of_the_toolbar_are_measured`):

| Form | The content width | What the commands say |
|---|---|---|
| Wide | ≥ 640 px | what they do: "Sort: Relevance ▾" |
| Medium | 460–640 px | what they *are*: "Sort: Rel ▾" |
| Narrow | < 460 px | their word alone: "Rel ▾", "State ▾" |

The Medium form had no distinct behaviour before — it was the Wide form with
a different threshold. It has one now, and the Narrow form shortens the state
button too.

The Narrow form is reachable in the app even though the catalogue's
qualified minimum forbids it in the *requested* size: a compositor that
grants less than the floor produces exactly that content width. The bench
keeps the failing case (§4.6).

**The maximum**: there is none. The compositor and the display's work area
decide; the application governs its internal composition and its components'
constraints.

### 4.2 The vertical budget has one owner — *measured, fixed*

**Decision 3 — implemented.**

The workspace is the owner. It computes the budget **once** per frame, from
the window's real allocation and the heights of the chrome, and it attributes
an explicit height to the station and to the drawer
(`vertical_budget`, `VerticalBudget`, in `views/workspace.rs`).

The rule the operator stated is now the invariant of the code:

> No child surface may calculate its usable height from the full window when
> it occupies only a portion of that window.

- The station no longer measures the window: it receives its band
  (`set_assigned_height`) and composes in it.
- The header's geometry comes from the token system
  (`UiMetrics::CONTEXT_HEADER_HEIGHT`), and the header *is* that height —
  the token is not an estimate, it is what the component lays out.
- The drawer participates in the same allocation (§4.3).
- The horizontal split keeps its existing shared mechanism
  (`content_width`), now with the catalogue's minimum in it.

**Measured, before and after.** The composition at the window's floor:

| | Before | After |
|---|---|---|
| the bank | 100.5 px | 100.5 px |
| the readout (the catalogue) | 314.5 px | 314.5 px |
| the nameplate (the plate) | 260 px | 96 px — its floor |
| in a 372 px band: the readout | **6.5 px** | **170.5 px** |
| in a 372 px band: the nameplate | 260 px | 96 px |

The plate's floor was 168, which was its *comfortable* height, and a
comfortable height is a luxury: at the workspace's minimum it left the
catalogue with less than a row and a half. The plate's floor is now its own
chrome — the name, the actions, the tabs (96 px) — and the plate grows only
when the user pulls it, up to a ceiling computed *after* the catalogue's own
floor (`nameplate_bounds`).

### 4.3 The drawer's ceiling comes from the budget — *implemented*

**Decision 4 — implemented.**

```
A = the height available after the permanent bands (the header)
P = max(0, min(0.40 × A, A − W_min))        W_min = 320
```

`P` is the drawer's *total* band, its header included. The drawer's body is
capped by `P` and the station keeps the rest — which is at least `W_min`,
always, whatever the request (`the_drawer_never_takes_the_workspace_floor`).

The 600 px ceiling of the settings is gone as a *rule*: it is now the user's
preference, bounded by the budget. A drawer of 600 px in a 900 px window
takes 236.8 px — the share — and the catalogue keeps 320.

Below 120 px of usable body, the drawer stops being a split: it **overlays**
the catalogue (`drawer_overlays`), keeping its full height and displacing
nothing (`a_shallow_drawer_overlays_instead_of_squeezing`). The context is
preserved, the catalogue keeps its place, and the drawer is reversible.

### 4.4 The spine folds itself

**Decision 2 — implemented.** See §4.1. The rule is one function, tested
(`the_spine_folds_when_the_catalogue_needs_the_room`), and the failing case
is kept in the same test.

### 4.5 The zoom — qualified as far as the harness allows, and no further

**What the harness can do, and cannot.** The test platform's window has a
**fixed scale factor of 2.0** — it is hard-coded in gpui's test platform
(`platform/test/window.rs:136`). The bench therefore cannot be set to 1.5,
and no amount of cleverness in the bench will change that.

**What that does *not* change: the composition.** The layout is computed in
logical pixels — the viewport is divided by `px(1.0)`, gpui's `viewport_size()`
is logical on both backends (X11 divides by the scale factor; Wayland's bounds
are logical), and the text metrics are logical too. The scale factor only
multiplies the *physical* size. So the composition at 150 % is the composition
at 100 %, in a physically larger window.

**And that is now measured** (`the_composition_is_in_logical_pixels`): at the
bench's 2.0 factor, the row is 600 × 36 — the logical numbers, exactly. The
scale factor does not leak into the geometry, and the contract's floors are
logical (660, 720, 480) in the same way.

So the zoom is qualified *as far as the instrument allows*: the layout is
scale-independent, and the floors are logical. What is **not** measured is a
window at 1.5 on a real screen — the harness cannot be set to it, and the
machine's compositor is not the bench.

**The caveat that remains**: a floor is a *request*. A tiling compositor, or
a screen smaller than the floor, may grant a window that is narrower or
shorter than the floor — the window's floor and the composition's floor are
two different things, and only the first is enforced by the system. This is
why the contract (§4.1) asks for the two levels to be tested separately.

### 4.6 The case that fails, kept

**Decision 1 — the failing case is retained.**

A compositor that grants 600 × 480 — less than the requested floor of
720 × 480 — produces a content width of 544 px (the spine folded): **116 px
below the catalogue's qualified minimum of 660**. The composition cannot
honour its promise there: the row's floor is 495 px, and the name would
have 27 px. The bench keeps the case
(`the_spine_folds_when_the_catalogue_needs_the_room`) rather than hiding it.

### 4.7 The row, qualified — the catalogue's minimum, measured

**The row is now measurable.** The catalogue's row was extracted into a
production component (`PackageTable::render_selectable_row` — the same
markup, the same styles, the same gesture, the same hover, the same
selection) and the bench draws *that*. The cells carry handles
(`package_row_name`, `package_row_version`, `package_row_source`,
`package_row_measurement`, `package_row_size`, `package_row_status`), and
the bench reads the bounds the window actually drew.

**What the row costs before the name** — measured, not estimated:

| The row's parts | Width |
|---|---|
| the version | 105 px |
| the source | 105 px |
| the measurement (the bar against the scale) | 96 px |
| the size | 80 px |
| the status | 85 px |
| the row's padding | 24 px |
| **the row's floor** | **495 px** |

Below 495 px the name is gone *and* the row's right end is cut: the fixed
columns do not shrink, so the row overflows and the status is the first
thing to disappear.

**The name's share** — the cell is what the fixed columns leave, and the
name's text is 22 px less (the identity glyph and its gap):

| The content | the name's cell | the name's text |
|---|---|---|
| 495 px | 0 px | 0 px — the row is at its floor |
| 520 px | 25 px | 3 px |
| **560 px** | **65 px** | **43 px — six characters** |
| 600 px | 105 px | 83 px |
| 640 px | 145 px | 123 px |
| 720 px | 225 px | 203 px |

**The names are the real ones.** The sample is drawn from the inventory of
the machine the measurement was taken on — 1884 installed packages — sampled
across its distribution (median 10 characters, p80 15, p90 18, max 44) and
its extreme, and measured in pixels:

| the name | its width |
|---|---|
| `bc` | 17 px |
| `7zip` | 34 px |
| `zotero` | 51 px |
| `alsa-utils` (the median) | 85 px |
| `intel-ucode` | 93 px |
| `libxkbcommon` | 101 px |
| `aws-sdk-cpp-s3` (the p80) | 118 px |
| `vlc-plugin-alsa` | 127 px |
| `ruby-power_assert` (the p90) | 143 px |
| `python-jaraco.classes` | 177 px |
| `networkmanager-vpn-plugin-openvpn` | 278 px |
| `intel-oneapi-compiler-dpcpp-cpp-runtime-libs` (the longest) | 370 px |

**The qualification.** The name is whole when the content is
`495 + 22 + the name's width`:

- for the p80 (127 px): **644 px**;
- for the p90 (143 px): **660 px**;
- for the longest (370 px): 887 px.

**The catalogue's qualified minimum is therefore 660 px**, and the 560 px
that was tried first **does not hold**: at 560 the name shows 43 px of text
— six characters — and the row becomes a list of truncated names. The
failing case is kept in the test
(`the_row_is_measured_at_and_around_the_catalogue_minimum`), not hidden.

### 4.8 The catalogue's list, qualified — and the bench's blind spot, named

**The list is qualified now.** The `uniform_list` — the real one, with the
real row — draws its rows in the bench, and the bench reads their bounds
(`the_catalogue_list_draws_its_rows_at_every_width`):

| the content | the row the list drew | the name's cell |
|---|---|---|
| 400 px | 400 × 36 | 0 px |
| 480 px | 480 × 36 | 0 px |
| 495 px | 495 × 36 | 0 px |
| 520 px | 520 × 36 | 25 px |
| 560 px | 560 × 36 | 65 px |
| **660 px** | **660 × 36** | **165 px** |
| 720 px | 720 × 36 | 225 px |
| 800 px | 800 × 36 | 305 px |

The row is the catalogue's real row, at the catalogue's real height (36 px),
taking the list's width, and the name's cell is exactly what the fixed
columns leave — the same rule as the row alone, now proved *in the list*.

**What the earlier probe missed, and why.** The first probe found a list
whose viewport resolved (600 × 400) but whose items never appeared. The
cause was found by reading the list's *own* measurement of its items
(`UniformListScrollState::last_item_size`):

```
item:     600 × 0     ← the viewport's height: zero
contents: 600 × 180   ← 36 px × 5 items: the list measured the row
```

**A `uniform_list` whose height is not given is laid out at zero by the
measured-layout path, and then draws no row at all.** The bench's earlier
fixture gave the list its height through `flex_1`, which the measured path
does not resolve; the bench now gives it a definite height, which is the
condition of reading it — not a change to the product. The application's own
list gets its height from the flex layout of the readout, and the row it
measures is the same row (36 px).

The lesson is kept in the test's own comment, because it is the kind of thing
that would be rediscovered as a mystery: *a list that is not measured is a
list that is not drawn.*

**What is proved, and what is not:**

| What is proved | By |
|---|---|
| the row's geometry, and what the name keeps at each width | the extracted row, drawn directly, measured |
| **the list drawing the real row, at the real height, at every width** | the real `uniform_list`, measured in the bench |
| the catalogue's viewport, and the three toolbar forms | the bench, on the real views |
| the list's own measurement of its rows (36 px, the content size) | `UniformListScrollState::last_item_size` |
| the list's *virtualisation* — that it recycles rows when scrolled | **proved**: `the_list_draws_only_the_rows_it_shows` — a hundred rows, a window that shows a dozen: the hundredth is not drawn, and after a scroll of fifty rows the list draws the fiftieth at the top and has *recycled* the first |
| the list's own scrolling, and the application handing the wheel back in steps | **proved by the application**: `the_wheel_moves_the_list_by_steps` (the bench has no wheel over a list; the application's own scrolling is proved on the structure the list uses) |

So the qualification of the catalogue is now real: the row, and the list that
holds it. What remains unproved is the list's behaviour *under* a scroll —
which is the application code, and is proved by the application's own
scrolling test (`the_wheel_moves_the_list_by_steps`, on the same structure
the list uses).

### 4.9 The row's ladder — the concession, done better than a concession

The concession that would have made 560 hold was to *drop* the measurement
bar (96 px): the size has its own column, so the bar is the row's most
droppable part. Dropping it is a loss, and a loss is a poor answer.

The row now has a **ladder**, and the ladder is expressed in the flex
factors — the flexbox distributes a deficit in proportion to them, so the
numbers *are* the order:

| the cell | gives way | why |
|---|---|---|
| the source | 4 | the most droppable: the name almost always says it, and the inspector gives it in full |
| the measurement | 2 | the instrument's reading — the size's exact figure has its own column |
| the version | 1 | the last of the secondary columns |
| the size, the state | 0 | the figure, and the state of the package: they never shrink |
| **the name** | — | the key: it never shrinks, and it has a floor of 165 px of cell (the p90 of the real names, 143 px of text, plus the identity glyph and its gap) |

Measured, in the list, at every width:

| the content | the name | the version | the source | the bar | the size | the state |
|---|---|---|---|---|---|---|
| 400 px | **165** | 60 | 0 | 0 | 80 | 85 |
| 480 px | **165** | 77 | 0 | 49 | 80 | 85 |
| 520 px | **165** | 84 | 21 | 61 | 80 | 85 |
| 560 px | **165** | 90 | 45 | 71 | 80 | 85 |
| **660 px** | 165 | 105 | 105 | 96 | 80 | 85 |
| 720 px | 225 | 105 | 105 | 96 | 80 | 85 |

At and above the catalogue's qualified minimum, the row is **exactly the row
it was**: nothing changes at 660 px and above. Below it, the row degrades
instead of breaking: the name keeps its floor, the state and the size keep
their room, and the source — the column that says *where* — is the one that
shortens. The row's own floor falls from 495 to **383 px**, so a row that
used to be cut on the right now fits.

Nothing is dropped: at 400 px every cell is still there, and the source and
the bar are the ones that shorten.

### 4.10 The plate, the catalogue, and the band that is too short

**The catalogue's vertical floor was missing.** The audit noted it as an open
item: the body had a *width* floor (660) but no *height* floor, and the
height it had (72 px) was two rows — a strip, not a catalogue. Six rows
(216 px) is a catalogue you can scan.

With that floor, the arithmetic changes shape: the band that holds the bank
(100.5), the edge (5), the catalogue (216) and the plate's chrome (96) is
**417.5 px** — and the workspace's minimum band is 320. The plate and the
catalogue cannot both stay in the flow.

**So the plate leaves the flow.** It comes *over* the catalogue, summoned by
the same edge that resized it — the same gesture, but it lifts the sheet
instead of resizing it, and pulls it back down to dismiss it. The catalogue
takes the band whole.

| the window | the band | the plate | the catalogue |
|---|---|---|---|
| 480 (the minimum) | 356 | **over the catalogue**, summoned | 250.5 px — 6 rows |
| 560 | 436 | in the flow, at its chrome | 234.5 px — 6 rows |
| 680 | 556 | in the flow | 359.5 px — 9 rows |

This is the contract's ladder, applied to the band: the context surface
yields *entirely* before the body is reduced — and the body's floor is now
written down.

**The risk, stated.** The gesture is the existing one (the edge drag), and
the layout is measured; but the *feel* of a sheet that comes from the bottom
cannot be verified by the bench, which has no pointer. The measurement proves
the composition, not the motion's quality.


## 5. What the audit leaves to the product

These are decisions, not findings. The audit states the facts; the product
decides.

1. ~~The window's floor~~ — **decided** (decision 1): derived, 720 × 480 as
   the qualification target, the narrow layout authored (the spine folds,
   the three toolbar forms exist and are measured).
2. **The body's floor** — **decided and qualified** (decision 2): the target
   of 560 px **did not hold** when measured against the real row (§4.7); the
   qualified minimum is **660 px**, measured from the row's fixed columns
   and the real names.
3. ~~The vertical budget's owner~~ — **decided** (decision 3): the workspace.
4. ~~The drawer's ceiling~~ — **decided** (decision 4): a share of the
   budget, and the overlay below 120 px.

What is left to decide, after the measurements:

5. **~~The catalogue's minimum against its rows.~~** **Done** (§4.7): the row
   was extracted, measured, and the minimum corrected from 560 to **660** —
   the value the measurement supports. What remains open is the *list*
   (§4.8), not the row.
6. **The plate at a small band.** The plate yields first (§4.2), which means
   that at the workspace's minimum it shows its chrome and a thin body. The
   alternative — the plate as an overlay, like the drawer — is not decided.
7. **The row at a narrow width.** The measurement names the concession
   exactly: without the measurement bar (96 px, measured), 560 px would hold
   the p90 name whole. Whether the instrument's reading is worth 96 px of
   the name is a product decision, and it is not taken here.

## 6. The test plan this audit implies

| The rule | The test | State |
|---|---|---|
| the composition is measurable, and the audit's numbers are the numbers | `the_composition_is_measured_in_a_real_window` | **done** |
| the body's floor is written down, and the window's floor is derived from it | `the_window_floor_is_derived_from_the_catalogue_and_the_spine` | **done** |
| the vertical budget has one owner, and the context surfaces give way first | `the_drawer_never_takes_the_workspace_floor`, `the_budget_is_a_function_and_it_does_not_oscillate`, `the_composition_is_measured_in_a_real_window` | **done** |
| the drawer's ceiling is a share and a floor, and a shallow drawer overlays | `the_drawer_ceiling_is_a_share_and_a_floor`, `a_shallow_drawer_overlays_instead_of_squeezing` | **done** |
| the spine folds itself, and the failing case is kept | `the_spine_folds_when_the_catalogue_needs_the_room` | **done** |
| the three forms of the toolbar are reachable and distinct | `the_three_forms_of_the_toolbar_are_measured` | **done** |
| the row's geometry, and the name's share at every width | `the_row_is_measured_at_and_around_the_catalogue_minimum` | **done** |
| the row's ladder: the name is never cut, the state and the size never shrink | `the_catalogue_list_draws_its_rows_at_every_width` | **done** |
| the list draws only the rows it shows, and recycles them | `the_list_draws_only_the_rows_it_shows` | **done** |
| the composition is in logical pixels, whatever the scale | `the_composition_is_in_logical_pixels` | **done** (the harness's scale factor is fixed at 2.0) |
| the plate leaves the flow before the catalogue is reduced | `the_plate_leaves_the_flow_before_the_catalogue_is_reduced`, `the_composition_is_measured_in_a_real_window` | **done** |
| the catalogue's minimum is the measured one, and the budget adds up | `the_window_floor_budget_adds_up` | **done** |
| the catalogue's rows are drawn by the list, at every width | `the_catalogue_list_draws_its_rows_at_every_width` | **done** |
| a click on the view commutateur changes the view | `a_click_on_the_switcher_changes_the_view` | **done** |
| a click on a rail entry changes the place or the scope | `a_click_on_a_rail_entry_changes_the_place_or_the_scope` | **done** |
| a click on a bank aperture toggles that source | `a_click_on_an_aperture_toggles_that_source` | **done** |
| a click on a catalogue row selects that package | `a_click_on_a_row_selects_that_package` | **done** |
| the composition at 150 % keeps the catalogue's floor | — | **open**: the same instrument, at another scale |

The harness makes the last one cheap: `VisualTestContext::simulate_resize`
resizes the window, and the bands are re-measured.

### The interaction layer, and the protocol of a click

Since v0.2.5 the release notes said the *clicks* were not proved — the
synthetic pointer never reached the Wayland client on the machine where the
evidence was taken. It stole the focus, and it landed nowhere.

The gap was never in the capability. The **harness** dispatches the event
straight into the window: a real hit test, a real propagation, a real
handler — with no compositor, no pointer, and no stolen focus. Four gates
stand, on the real components: the view commutateur, the five entries of
the rail, the seven openings of the bank, and a row of the catalogue. The
rows are addressable *by name* (`package_row-<name>`), so a click lands on
the package the test means and not on the first row of the list.

**The protocol**, which belongs to the bench and not to the application: *a
click is a sequence of events separated by frames*, exactly as it is for a
user. The first click moves the focus, the focus changes the style, the
window repaints, and the next click is dispatched against the hitboxes of
that repaint. Without the frame between them, the second click lands
nowhere.

Two false diagnoses came out of that discovery, and the data corrected
both: the travelling mark does *not* steal the click, and a percentage
height did *not* fail to resolve. The list was simply not in the tree,
because the session's default scope is the *results* — which were empty.

## 7. What was not done, and why

- **No new primitive was written.** The pieces were found and re-used: the
  window policy (`config.rs`), the geometry (`ui_metrics.rs`), the two
  divisions of the space (`content_width`, `nameplate_height`), the
  presentation policy (`state/motion.rs`). What was missing was an *owner*
  for the vertical budget — a responsibility, not a component.
- **The list's virtualisation is now qualified** (§4.8): it draws only the
  rows it shows, and it recycles them on a scroll.
- **The 150 % scale is qualified as far as the harness allows** (§4.5): the
  layout is scale-independent and the floors are logical, but the harness's
  scale factor is fixed at 2.0 and a real 150 % window is not measured.
- **The plate's sheet cannot be felt by the bench.** The composition when it
  overlays is measured (§4.10); the *gesture* — the edge lifting it, and
  pulling it back down — is the existing drag, but the bench has no pointer
  and the motion's quality is not verified.
- **The zoom was not measured at 150 %.** The layout is in logical pixels
  and the floor is too, so the reasoning holds; but the *measurement* of a
  150 % window is not in the bench yet, for the same reason as the rows.
- **The plate's overlay alternative was not built.** The plate yields first
  instead (§4.2), which is the contract's ladder; the alternative — the plate
  as an overlay — is a decision left open (§5.6).
- **No screenshot was taken.** The desktop was not touched during this
  work: the evidence is the bench's measurements, which are reproducible and
  do not need a pointer.
