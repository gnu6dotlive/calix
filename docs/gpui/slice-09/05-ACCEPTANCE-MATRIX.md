# 05 — The acceptance matrix

Every rule of the slice, and how it is proven. A rule with no proof is a
wish; a rule proven by a test is a fact; a rule proven by the eye is a fact
only until the next change.

| # | The rule | The proof | State |
|---|---|---|---|
| A1 | The bank carries the sources of the scope, each with the kind of thing it holds and the count of the scope's rows in it | `state::sources::tests::the_axis_offers_what_the_scope_can_hold_and_counts_it`; `every_source_states_what_it_holds`; `the_counts_follow_the_rows` | proved |
| A2 | The two families are separate: the stores first, the installation methods last | `the_stores_come_before_the_installation_methods`; the order in `PackageSourceKind::ALL` and `NavScope::sources()` | proved |
| A3 | A source's count follows the scope and the filter, never the last search | `the_axis_follows_the_filter` | proved |
| A4 | The reason of an absent action lives in the source's own entry | `the_limits_of_a_source_are_the_reasons_why_an_action_is_absent`; the inspector's action row writes the reason where the button would have been | proved by the test; the rendering is seen by the eye |
| A5 | The card says where else the name is found, with the versions | `the_card_says_where_else_the_name_is_found`; the overview's *ALSO IN* band | proved by the test; the rendering is seen by the eye |
| A6 | The bank is always present, and it is reachable by the keyboard | the apertures are focusable, tab stops, and answer Enter and Space; the bank is in the plate at every width | proved by the eye |
| A7 | The keyboard leaves the surface | the key map is a translucent overlay, **derived from the registry**; the surface carries no key glyph; the overlay is opened by the `?` disc in the header, or by holding the modifier alone for three seconds | **done**: `state/keys.rs` is the single source of truth ✗ — the handler asks it what a key means ✗, and the sheet is derived from it ✗; `the_registry_is_the_source_of_the_bindings` ✗, `the_sheet_draws_without_panicking` ✗. The gesture is not verified by hand ✗ (the synthetic keys of this machine never reach the client ✗) |
| A8 | The sources leave the filters menu | the menu still offers them, bound to the same selection | **not done**: the menu keeps the sources as the narrow fallback |
| A9 | The detail is a nameplate, a full-width band under the readout | the plate: bank → readout → edge → nameplate; the vertical splitter and the right-hand column are gone | **done**; the nameplate's bounds are proved by `the_nameplate_stays_inside_its_bounds_at_the_heights_a_tiling_compositor_uses` and `the_nameplate_never_swallows_the_readout` |
| A10 | The log is a tape: the product's own command, then what the backend actually ran | the operation log is a band at the bottom of the frame; it now carries the product's command (`$ calix install standard ripgrep`) above the backend's output | **done**; the product's command is the console model's `product_command`, set when the operation starts |
| A11 | The scope is a dial, one object with three positions, and it does not look like the bank | `components/scope_dial.rs`: a machined disc, three detents on the arc, a needle thrown at the detent; the header's segmented switcher is gone | **done**; the needle's throw is `ease_out_back` (`the_overshoot_passes_the_target_and_comes_back`), the positions are `the_three_positions_are_on_the_upper_arc` |
| A12 | The readout is a recess cut into the plate, and the values are measured against a shared scale | the measurement column: a bar with an end-marker, drawn against the scope's shared scale, with the exact figure in its own column; the scale is logarithmic and the header says so | **done** |
| A15 | The recess keeps its frame when the table is empty | the readout is a recess — its own ground, a hairline, and the rows engraved into it rather than laid on it. The ruler — the column names and the scale — stays when the table is empty and a query matched nothing: a calibrated glass that waits reads as glass, not as a panel that failed to draw | **done**, by the eye |
| A16 | The state where there is no table yet is a screen, not an empty table | the discovery screen (the scope, the four searchable sources, how to use it) replaces the readout entirely, and the ruler goes with it — a header over columns that do not exist would be a lie | **done**, by the eye |
| A13 | The value that changes re-reads and flashes once | the count in the bank flashes once when it changes | **done**, by the eye |
| A14 | The curves are tokens, not formulas | `state/motion.rs`: three named Bézier curves ✗ — out ✗, in-out ✗, spring ✗ — evaluated by Newton-Raphson ✗, and the application invents no others ✗ | **done**: `the_curves_are_tokens_and_they_behave` ✗ |

## The reference

The motion, the keyboard and the performance discipline were studied on a
reference application — the Artcraft launcher ✗ — and what was taken from it
is written down in `evidence/references/artcraft-launcher-20261020.md` ✗. The
study names what was *not* taken ✗: the physical key codes ✗ (so the registry
is not yet layout-independent ✗), the binding presets ✗, the availability
gates ✗, and the remapping ✗.

## The motion

The movement is part of the arrangement, not a decoration on it. What is in
the application now, and what it does:

| Where | What | The law | The proof |
|---|---|---|---|
| The bank's arrival | the seven apertures come in as a wave: they overlap, the gaps shrink, and the anchors travel further and longer | the waterfall entry | `the_cascade_overlaps_and_its_gaps_shrink` |
| The needle's throw | the needle passes its detent and comes back | the overshoot | `the_overshoot_passes_the_target_and_comes_back` |
| The panel's switch | the outgoing panel accelerates, the cut lands mid-motion, the incoming panel continues in the same direction and decelerates, and it ignites at 35 % instead of fading in | the cut on the curve | `the_cut_entry_ignites_mid_path_and_keeps_the_direction` |
| A value that changes | the count re-reads and flashes once; it does not glide into place | the instrument's readout | the eye |
| The list's scroll | a wheel notch is a movement, not a jump | the eased hand | `the_wheel_moves_the_list_by_steps` |

The user can ask for less movement: `reduce_motion` snaps the needle, skips
the bank's arrival, and leaves the scroll as it is.

## The gaps of this run

- The bank's apertures are seven equal cells. On a narrow window the note
  under the name is cut rather than wrapped: the band stays a band, and the
  note is the first thing to lose its room.
- The nameplate's empty state is centred in a wide band. The Instrument's
  empty nameplate names the channel instead of a package; this one says
  *No Package Selected* and centres it.
- The tape is the drawer as it was. It shows the backend's output; it does
  not yet put the product's own command on the line above the backend's
  actual run.
- The readout is the list, measured but not yet recessed: the Instrument's
  readout is a recess cut into the plate, ruled, with the grid visible. The
  measurement is there, the recess is not.
- The measurement's scale is logarithmic, and the header says so. A linear
  scale would bury every small package under a sliver, and the sizes span
  orders of magnitude.
- The recess's ground is darker than the plate's by five points out of 255 —
  the same subtlety as the Instrument's mock, where the inset is a hairline
  and a shade, not a hole.
- The nameplate's empty state is centred in a wide band, where the
  Instrument's names the channel. The application's says *No Package Selected*,
  which is honest about the selection but says nothing about the scope.
