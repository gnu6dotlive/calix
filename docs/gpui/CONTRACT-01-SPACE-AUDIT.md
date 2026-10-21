# CONTRACT 01 — The audit of the spatial contract

**Audit 1.0 — 2026-10-21 — against `main` at `d2d25f5b`**

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
| Window: the floors are coherent with what the application can compose | **held** | the floor (720 × 480) is derived: the catalogue's minimum (560) plus the folded spine (56). Tested: `the_window_floor_is_derived_from_the_catalogue_and_the_spine` |
| Spine: a stable presence, it narrows but never vanishes | **held by a rule** | the spine folds itself when the catalogue would lose its room, and takes back the user's shape when the window gives it room. Tested: `the_spine_folds_when_the_catalogue_needs_the_room` |
| Body: the catalogue keeps a usable width and stays dominant | **held horizontally, qualified against the bands** | horizontally: 664 px at the floor (the spine folded), and the three toolbar forms are measured. **The rows themselves are not yet measured** (§4.7) |
| Context surfaces: opening one never reduces the body below its floor | **held** | the drawer cannot take the workspace's 320 px whatever the request, and below 120 px it overlays instead of squeezing. The plate yields first: the catalogue goes from 6.5 px to 170.5 px in a 372 px band. Tested: `the_drawer_never_takes_the_workspace_floor`, `the_composition_is_measured_in_a_real_window` |
| Portrait and landscape: the composition adapts to the space, not to a declared orientation | **held, by not answering it** | no code reads the orientation; the composition is expressed in logical pixels and adapts by growing or shrinking the bands |
| Zoom: at every scale the content stays accessible | **held by construction, with one caveat** | the layout is in logical pixels (the viewport is divided by `px(1.0)`) and the window's floor is a floor in logical pixels, so the physical window grows with the scale. The caveat is in §4.5 |
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
min_window_width  = catalogue_min_width + sidebar_collapsed   = 560 + 56 = 616
min_window_height = the height at which the band still holds the bands   = 480
```

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

### 4.5 The zoom holds by construction, with one caveat

The layout is in logical pixels: the viewport is divided by `px(1.0)`
(`package_workstation.rs:627,629`), and gpui's `viewport_size()` is already in
logical pixels on both backends (X11 divides by the scale factor; Wayland's
bounds are logical). The window's floor is a floor in logical pixels, so at
150 % the physical window grows and the logical composition keeps its room.
Nothing in the layout needs to change for the zoom.

The caveat: a floor is a *request*. A tiling compositor, or a screen smaller
than the floor, may grant a window that is narrower than the floor — the
window's floor and the composition's floor are two different things, and only
the first is enforced by the system. This is precisely why the contract
(§4.1) asks for the two levels to be tested separately, and why the
composition's own floors have to exist as numbers.

### 4.6 The case that fails, kept

**Decision 1 — the failing case is retained.**

A compositor that grants 600 × 480 — less than the requested floor of
720 × 480 — produces a content width of 544 px (the spine folded): **16 px
below the catalogue's qualified minimum of 560**. The composition cannot
honour its promise there. The bench keeps the case
(`the_spine_folds_when_the_catalogue_needs_the_room`) rather than hiding it:
the catalogue's *rows* are not measured in the bench (§4.7), so what 544 px
costs the rows is not yet known. That is the open qualification.

### 4.7 What the bench still cannot see

The harness draws the application's own views and reads the bounds the window
actually drew. It reads the bands, the buttons, the search field, the mark
that travels, the toast, the menu — everything it can.

It does **not** read the catalogue's rows: the `uniform_list` that holds them
resolves its *viewport* in the bench (600 × 400 in a probe) but never paints
its items, and the probe found no configuration of the harness in which they
appear — the item measurement inside the list's measured layout is what
fails. The limitation is therefore explicit and unchanged:

- the catalogue's **viewport** is measured ✓;
- the catalogue's **rows** are not ✗;
- the rows' *content* is therefore not measured at 560 px either.

The qualification of the catalogue's minimum against its own contents is
**open**. The number 560 is the operator's target, and it is *consistent*
with the measured geometry (the spine folds, the toolbar's three forms are
measured, the catalogue's viewport is measured) — but it is not yet qualified
against the rows themselves.

## 5. What the audit leaves to the product

These are decisions, not findings. The audit states the facts; the product
decides.

1. ~~The window's floor~~ — **decided** (decision 1): derived, 720 × 480 as
   the qualification target, the narrow layout authored (the spine folds,
   the three toolbar forms exist and are measured).
2. ~~The body's floor~~ — **decided** (decision 2): 560 px as the
   qualification target, *not yet qualified against the rows* (§4.7).
3. ~~The vertical budget's owner~~ — **decided** (decision 3): the workspace.
4. ~~The drawer's ceiling~~ — **decided** (decision 4): a share of the
   budget, and the overlay below 120 px.

What is left to decide, after the measurements:

5. **The catalogue's minimum against its rows.** The bench cannot read them
   (§4.7). Either the rows are measured another way — the row component
   drawn directly, outside the list — or 560 stays a *target* and the
   qualification waits for an instrument that can see the list.
6. **The plate at a small band.** The plate yields first (§4.2), which means
   that at the workspace's minimum it shows its chrome and a thin body. The
   alternative — the plate as an overlay, like the drawer — is not decided.

## 6. The test plan this audit implies

| The rule | The test | State |
|---|---|---|
| the composition is measurable, and the audit's numbers are the numbers | `the_composition_is_measured_in_a_real_window` | **done** |
| the body's floor is written down, and the window's floor is derived from it | `the_window_floor_is_derived_from_the_catalogue_and_the_spine` | **done** |
| the vertical budget has one owner, and the context surfaces give way first | `the_drawer_never_takes_the_workspace_floor`, `the_budget_is_a_function_and_it_does_not_oscillate`, `the_composition_is_measured_in_a_real_window` | **done** |
| the drawer's ceiling is a share and a floor, and a shallow drawer overlays | `the_drawer_ceiling_is_a_share_and_a_floor`, `a_shallow_drawer_overlays_instead_of_squeezing` | **done** |
| the spine folds itself, and the failing case is kept | `the_spine_folds_when_the_catalogue_needs_the_room` | **done** |
| the three forms of the toolbar are reachable and distinct | `the_three_forms_of_the_toolbar_are_measured` | **done** |
| the catalogue's *rows* are measured at the catalogue's minimum | — | **open**: the `uniform_list` does not paint its items in the bench (§4.7) |
| the composition at 150 % keeps the catalogue's floor | — | **open**: the same instrument, at another scale |

The harness makes the last two cheap once the list can be seen:
`VisualTestContext::simulate_resize` resizes the window, and the bands are
re-measured. What is missing is not the instrument — it is the list.

## 7. What was not done, and why

- **No new primitive was written.** The pieces were found and re-used: the
  window policy (`config.rs`), the geometry (`ui_metrics.rs`), the two
  divisions of the space (`content_width`, `nameplate_height`), the
  presentation policy (`state/motion.rs`). What was missing was an *owner*
  for the vertical budget — a responsibility, not a component.
- **The catalogue's rows are not measured.** The `uniform_list` resolves its
  viewport in the bench but never paints its items (§4.7). The rows' own
  measurement is the open qualification of the catalogue's minimum.
- **The zoom was not measured at 150 %.** The layout is in logical pixels
  and the floor is too, so the reasoning holds; but the *measurement* of a
  150 % window is not in the bench yet, for the same reason as the rows.
- **The plate's overlay alternative was not built.** The plate yields first
  instead (§4.2), which is the contract's ladder; the alternative — the plate
  as an overlay — is a decision left open (§5.6).
- **No screenshot was taken.** The desktop was not touched during this
  work: the evidence is the bench's measurements, which are reproducible and
  do not need a pointer.
