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
| Window: the floors are coherent with what the application can compose | **open** | the floor (1024 × 680) is a chosen number, not a derived one; the composition's own floors are not written down anywhere |
| Spine: a stable presence, it narrows but never vanishes | **held by the floor, not by a rule** | the spine is 190 px (56 collapsed) and is never squeezed — but nothing narrows it automatically, and the situation it would protect against cannot occur because the window's floor forbids it |
| Body: the catalogue keeps a usable width and stays dominant | **held horizontally, open vertically** | horizontally: 834 px at the floor, the catalogue dominates. Vertically: the readout falls to **6.5 px** when the drawer takes its share (§4.2) |
| Context surfaces: opening one never reduces the body below its floor | **open** | the nameplate keeps 260 px of a 372 px band while the catalogue falls to 6.5 px |
| Portrait and landscape: the composition adapts to the space, not to a declared orientation | **held, by not answering it** | no code reads the orientation; the composition is expressed in logical pixels and adapts by growing or shrinking the bands |
| Zoom: at every scale the content stays accessible | **held by construction, with one caveat** | the layout is in logical pixels (the viewport is divided by `px(1.0)`, `package_workstation.rs:627,629`) and the window's floor is a floor in logical pixels, so the physical window grows with the scale. The caveat is in §4.5 |
| Transitions: a change of width preserves the continuity | **partial** | the vertical budget has no owner, so a change of height is not a transition but a re-landing (§4.2) |
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

## 4. The findings

### 4.1 The window's floor is a number, not a derivation

The contract (§4.1) asks for
`min_window_width ≥ spine_floor + body_floor`. The repository has
`MIN_WINDOW_WIDTH = 1024` and a spine of 190, so the body gets 834 — and
nothing says that 834 is enough, because *the body's floor is not written
down*. The consequence is visible in the code: the toolbar's breakpoint rule
has three branches (`WorkbenchBreakpoint`, `state/query.rs:370-386`) and
**two of them cannot be reached**:

- the content is never below 834 px (the spine expanded) or 968 px (collapsed);
- `from_width` calls 834 "Wide" (≥ 640), and Medium (460–640) and Narrow
  (< 460) are out of reach.

So the `Narrow` branch of the sort label (`query_workbench.rs:332-345`) is
code that can never run, and the rule it belongs to is tested
(`test_query_workbench_breakpoints_wide_medium_narrow`) but never exercised.
**The gap is not a missing feature; it is a rule that is not connected to the
window's floor.** Two ways out, and the product decides: lower the floor and
author the narrow layout, or delete the two branches and say the toolbar is
always the wide one.

### 4.2 The vertical budget has no owner — *measured*

This is the finding that matters, and it is measured rather than argued.

At the window's floor (1024 × 680), the station composes like this:

| Band | Height |
|---|---|
| the bank (the seven sources) | 100.5 px |
| the readout (the catalogue) | 314.5 px |
| the splitter | 5 px |
| the nameplate (the plate) | 260 px |

Now the log drawer opens. It takes 220 px (its configured height) from the
bottom of the main column, and the header takes its band (≈ 88 px — measured
on the screen; the header's height is content-driven and is not a token). The
station's band is left with 372 px. The station does not know: **it measures
the window, not the band it occupies**
(`package_workstation.rs:627-634` — `window.viewport_size().height`). So its
nameplate still computes its share against 680 px, not 372 px.

Measured result, with the station in a 372 px band:

| Band | Height |
|---|---|
| the readout (the catalogue) | **6.5 px** |
| the nameplate (the plate) | 260 px — unchanged |

The catalogue — priority 2 — is reduced to a sliver, while the nameplate —
priority 3, the *caption* of that catalogue — keeps 70 % of the band. The
nameplate cannot shrink (`flex_shrink_0`), and the readout can shrink to
nothing (`flex_1`, `min_h(0)`): the ladder of the contract (§4.4) is
**inverted** — the secondary surface takes what the primary one needs.

The same arithmetic, taken further, shows the ceiling of the drawer is not
related to the floor of the window: the config allows a drawer of 600 px
(`config.rs:464-478`) and the window's floor is 680 px, so a drawer of 600
plus a 90 px header is already more than the window — the destination band
goes to zero and the header is clipped.

**The gap**: the vertical space is divided in three places, with three
notions of "the space" — the window, the band, and the config. The contract
(§4.3) asks for one owner. The work it would take:

1. the workspace computes the bands' heights once — the header's, the
   drawer's, and the destination's — and passes the destination's height
   down;
2. the header's height becomes a token (`UiMetrics`), because today it is
   content-driven and nobody can know it before the layout;
3. the station takes its height as an input, and the nameplate's share is
   computed against the band it occupies;
4. the drawer's height is bounded by a share of the budget, not by a constant
   range.

### 4.3 The spine is protected by the floor, not by a rule

The spine is 190 px (56 collapsed) and is never squeezed — good. But the
contract's ladder (§4.4) says the spine narrows *when the space runs short*,
and today the space can never run short because the window's floor forbids
it (§4.1). The user collapses the spine by hand, at any width; nothing
collapses it for them. The invariant is therefore true by accident, and it
would stop being true the moment the floor came down — which is exactly the
change §4.1 proposes.

### 4.4 The context surfaces are bands already

The contract's §4.4 asks for the context surfaces to give way first. They
are bands, so they *can* give way — but there is no rule that says they must:
the nameplate is `flex_shrink_0` (it never gives way) and the drawer is a
fixed configured height. The order the contract asks for is not implemented;
what exists is the opposite order.

### 4.5 The zoom holds by construction, with one caveat

The layout is in logical pixels: the viewport is divided by `px(1.0)`
(`package_workstation.rs:627,629`), and gpui's `viewport_size()` is already in
logical pixels on both backends (X11 divides by the scale factor; Wayland's
bounds are logical). The window's floor is a floor in logical pixels, so at
150 % the physical window grows and the logical composition keeps its room.
Nothing in the layout needs to change for the zoom.

The caveat: a floor is a *request*. A tiling compositor, or a screen smaller
than 1024 logical pixels, may grant a window that is narrower than the floor
— the window's floor and the composition's floor are two different things,
and only the first is enforced by the system. This is precisely why the
contract (§4.1) asks for the two levels to be tested separately, and why the
composition's own floors have to exist as numbers.

## 5. What the audit leaves to the product

These are decisions, not findings. The audit states the facts; the product
decides.

1. **The window's floor**: 1024 stays and the two unreachable branches are
   deleted (the toolbar is always the wide one), *or* the floor comes down and
   the narrow layout is authored. The first is a deletion; the second is a
   slice.
2. **The body's floor**: what is the narrowest width at which the catalogue
   is still a catalogue? The answer is the number the window's floor must be
   derived from.
3. **The vertical budget's owner**: the workspace, as the contract says, or
   somewhere else.
4. **The drawer's ceiling**: a share of the budget, or a constant.

## 6. The test plan this audit implies

| The rule | The test | State |
|---|---|---|
| the composition is measurable, and the audit's numbers are the numbers | `the_composition_is_measured_in_a_real_window` | **done** |
| the body's floor is written down, and the window's floor is derived from it | — | **open** (needs decision 2) |
| the vertical budget has one owner, and the context surfaces give way first | — | **open** (needs decision 3) |
| the two unreachable branches of the breakpoint rule are either reachable or gone | — | **open** (needs decision 1) |
| the composition at the floors, and at 150 %, keeps the body's floor | — | **open** (the harness can measure this once the floors exist) |

The harness makes the last one cheap: `VisualTestContext::simulate_resize`
resizes the window, and the bands are re-measured. What is missing is not the
instrument — it is the floors.

## 7. What was not done, and why

- **No new primitive was written.** The audit found the pieces: the window
  policy (`config.rs`), the geometry (`ui_metrics.rs`), the two divisions of
  the space (`content_width`, `nameplate_height`), the presentation policy
  (`state/motion.rs`). What is missing is an *owner* for the vertical budget,
  and that is a decision (5.3), not a component.
- **The narrow layout was not authored.** It cannot be: the window's floor
  makes it unreachable, and the floor is a decision.
- **The drawer was not bounded by the window.** The same: the ceiling is a
  decision (5.4), and bounding it without a budget would be a number in the
  wrong place.
