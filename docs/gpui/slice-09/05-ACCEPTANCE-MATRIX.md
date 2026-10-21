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
| A8 | The sources leave the filters menu | the menu is gone. The sources are the bank's, and the menu's only other content was the state — which has its own button at every reachable width. `menu_item_count` and `activate_menu_item` count the states the scope offers, the same list the menu draws | **done**: the menu, its items (`MenuCheckItem`, `MenuDivider`) and the three settings it existed to read are deleted; the wide toolbar carries Search, State, Sort, View — seen at 1080 × 935 |
| A9 | The detail is a nameplate, a full-width band under the readout | the plate: bank → readout → edge → nameplate; the vertical splitter and the right-hand column are gone | **done**; the nameplate's bounds are proved by `the_nameplate_stays_inside_its_bounds_at_the_heights_a_tiling_compositor_uses` and `the_nameplate_never_swallows_the_readout` |
| A10 | The log is a tape: the product's own command, then what the backend actually ran | the operation log is a band at the bottom of the frame; it now carries the product's command (`$ calix install standard ripgrep`) above the backend's output | **done**; the product's command is the console model's `product_command`, set when the operation starts |
| A11 | The scope is a dial, one object with three positions, and it does not look like the bank | `components/scope_dial.rs`: a machined disc, three detents on the arc, a needle thrown at the detent; the header's segmented switcher is gone | **done**; the needle's throw is `ease_out_back` (`the_overshoot_passes_the_target_and_comes_back`), the positions are `the_three_positions_are_on_the_upper_arc` |
| A12 | The readout is a recess cut into the plate, and the values are measured against a shared scale | the measurement column: a bar with an end-marker, drawn against the scope's shared scale, with the exact figure in its own column; the scale is logarithmic and the header says so | **done** |
| A15 | The recess keeps its frame when the table is empty | the readout is a recess — its own ground, a hairline, and the rows engraved into it rather than laid on it. The ruler — the column names and the scale — stays when the table is empty and a query matched nothing: a calibrated glass that waits reads as glass, not as a panel that failed to draw | **done**, by the eye |
| A16 | The state where there is no table yet is a screen, not an empty table | the discovery screen (the scope, the four searchable sources, how to use it) replaces the readout entirely, and the ruler goes with it — a header over columns that do not exist would be a lie | **done**, by the eye |
| A13 | The value that changes re-reads and flashes once | the count in the bank flashes once when it changes | **done**, by the eye |
| A14 | The curves are tokens, not formulas | `state/motion.rs`: three named Bézier curves ✗ — out ✗, in-out ✗, spring ✗ — evaluated by Newton-Raphson ✗, and the application invents no others ✗ | **done**: `the_curves_are_tokens_and_they_behave` ✗ |
| A17 | The selection is a carrier: a bar that travels from the row it marked to the row chosen, and keeps its departure through the repaints the voyage crosses | `the_carrier_leaves_from_the_line_it_occupied`, `a_repaint_does_not_throw_the_carrier_at_its_destination`, `the_carrier_sits_on_the_line_it_marks`; seen at 1080 × 935: the bar crossing two rows between the frames taken at 80 ms and 107 ms | **done** |
| A18 | The scope enters from the side the dial turned | `the_scope_dirties_the_side_of_the_cut`, `the_cut_lands_where_it_was_aimed`; seen: the readout 35 px off its seat on the first frame after the change, at rest by the fourth | **done** |
| A19 | The nameplate enters with the selection | `PLATE_ENTRY`, `PLATE_ENTRY_TRAVEL`; seen: the name's top at 673, then 652, then 651, over 80 ms | **done** |
| A20 | An entry is a displacement, not a fade — and it moves | `left`/`top` on a flow child is ignored by this version of gpui: a probe of 240 px over two seconds moved nothing. Every animated surface is therefore laid out absolutely inside its band, and the offset is applied there; a layer of opacity around the virtualised readout is refused, because it dims the whole window | **done**: the probe, then the frames after it |
| A21 | The bank has a front: the anchor carries its full name and what it holds, the narrow apertures carry their short name, and the seven counts share a line | `anchor_index` (ties go to the first), `the_stores_come_before_the_installation_methods`; the short names are `PackageSourceKind::short_label` | **done**, by the eye |

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
| The scope's change | the readout is a cut: the new scope's content lands displaced from the side the dial turned, and closes the distance | the cut, positional | `the_cut_lands_where_it_was_aimed`, `the_scope_dirties_the_side_of_the_cut`; the frames at 1080 × 935 |
| The selection's move | the carrier leaves the row it marked and travels to the row chosen | the travelling marker | `the_carrier_leaves_from_the_line_it_occupied`, `a_repaint_does_not_throw_the_carrier_at_its_destination`, `the_carrier_sits_on_the_line_it_marks`; the frames |
| The rail's mark | the accent bar leaves the entry it marked and travels to the entry chosen, while the fill lights the destination at once | the travelling marker — one law, two surfaces, one porteur | `the_carrier_leaves_from_the_line_it_occupied` (the same marker, moved into `state::motion`), `the_rail_mark_lands_on_the_row_it_marks`; `evidence/runs/slice-09-rail-mark-20261021/rail-travel.png` |
| The plate's arrival | the nameplate's content lands 22 px low and rises into its band | the cut, positional | the frames |
| The panel's switch | the outgoing panel accelerates, the cut lands mid-motion, the incoming panel continues in the same direction and decelerates, and it ignites at 35 % instead of fading in | the cut on the curve | `the_cut_entry_ignites_mid_path_and_keeps_the_direction` |
| A value that changes | the count re-reads and flashes once; it does not glide into place | the instrument's readout | the eye |
| The list's scroll | a wheel notch is a movement, not a jump | the eased hand | `the_wheel_moves_the_list_by_steps` |

A travel takes `Curve::IN_OUT`; an entry takes `Curve::OUT`. The out curve is
front-loaded — at a tenth of the time it has covered about half the distance —
so a mark sent along it arrives before the eye has left the row it departed:
it reads as a jump with a tail, not as a voyage. The carrier and the rail's
mark both travel, and both take the aller-retour.

The user can ask for less movement: `reduce_motion` snaps the needle, skips
the bank's arrival, snaps the rail's mark onto its entry, and leaves the
scroll as it is.

## The gaps of this run

- The two narrow breakpoints cannot be reached. The window's minimum width is
  1024 px (`config::MIN_WINDOW_WIDTH`) and the sidebar is 190 px expanded, so
  the content is never under 834 px: the workbench is always Wide. That is why
  the filters menu could never open: it existed only to hold the state where
  the state button was hidden, and that place does not exist. The menu is
  deleted rather than left as a fallback that cannot fall. If the application
  is ever to run under 1024 px, the narrow layout has to be authored — and the
  minimum lowered with it — not resurrected from a branch that never ran.
  `Narrow` still shortens the sort label; that branch is unreachable too, and
  it is left for the pass that decides the window's floor.
- The rail row's focus ring is the accent, as the mark is: a clicked entry
  wears both, and its left edge is the two marks merged. During the 220 ms of a
  travel the destination already wears the ring while the bar is still in
  flight — two accent marks, briefly, on two rows. The ring leaves as soon as
  focus moves to the list, and the bar is drawn over it: recorded rather than
  differentiated, because gpui 0.2.2 has no `focus_visible`, and a ring that
  only the keyboard raises would need its own interaction state.
- The keyboard's state menu had never matched the drawn one: it believed four
  states while the menu drew `PackageStateFilter::options_for(scope)` — two in
  Installed, one in Updates. It now counts and activates the same list. The
  synthetic keyboard of this machine never reaches the client, so it is proved
  by the rule, not by a keypress.
- The inspector's tab body still animates its opacity only. Its `left` offset
  falls on a flow child, where this version of gpui ignores it, so the fade
  carries the change alone.
- The carrier's geometry inside the virtualised list is proved by the rule
  (`carrier_top`) and by the frames, not by a test that lays the list out: a
  `uniform_list` does not resolve its layout in the test harness.
- The synthetic keyboard of this machine never reaches the client, so A7's
  three-second gesture is still unproven by hand. The keys that were checked
  were checked with the pointer.
- The measurement's scale is logarithmic, and the header says so. A linear
  scale would bury every small package under a sliver, and the sizes span
  orders of magnitude.
- The recess's ground is darker than the plate's by five points out of 255 —
  the same subtlety as the Instrument's mock, where the inset is a hairline
  and a shade, not a hole.
- Below about a thousand pixels the seven apertures crowd. The anchor keeps
  its extra basis, the short names hold, and the note under the anchor is the
  first thing to lose its room.
