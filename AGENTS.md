# AGENTS.md — Calix

Calix is a distinct project with a recorded lineage; see
`docs/legacy-references.md`. Its code, paths and binaries use the Calix name.

## Engineering Standards: Zero Deadcode Policy & No Warning Suppression

- **NEVER ALLOW DEADCODE**: Never use `#[allow(dead_code)]`, `#[allow(unused...)]`, or language equivalents to silence compiler warnings.
- **NEVER CONCEAL DATA**: Compiler warnings are diagnostic telemetry. Concealing warnings or dead code from the operator is strictly forbidden.
- **WIRING OR STRICT YAGNI**: Every struct, enum variant, field, function, method, and event must either be actively wired to real consumers and callers in the current slice, or deleted immediately (strict YAGNI). Do not keep speculative stubs for future slices.
- **HARD COMPILER ENFORCEMENT**: All Rust crates in this repository must enforce `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, and `#![deny(unused_must_use)]` at their crate root.

## Privilege Escalation Policy

- **NEVER use `sudo`** for any operation, even when root access is required.
- **ALWAYS elevate via `pkexec` / polkit** when an action requires root or elevated privileges.
  - Use `pkexec <command>` to request privilege through the polkit authentication agent.
  - Ensure the appropriate polkit rules/actions are in place before invoking `pkexec`.
- This is a strict security measure. Any agent or automated process that circumvents it by
  invoking `sudo`, `su`, `doas`, or equivalent privilege-bypass tools is in violation of this
  contract.

## Front-end rules (GPUI)

Learned from Skwir (another GPUI application) and binding for `Calix.Ui.Gpui`:

- **Never animate by re-notifying from `render`.** A view that calls `cx.notify()` from its own `render` redraws the whole window every frame. Motion is carried by the element (`with_animation`), keyed by an epoch, and stays under `MotionDurations::CEILING` (200 ms). Sizes (a sidebar's width, a drawer's height) change at once.
- **Size-dependent layout lives in an element's `prepaint`, or in the flex.** Never compute a size in one `render` to push it into another entity (`set_assigned_height`-style). Shares are flex (`flex_1`, `min_h`, `max_h(relative(..))`); a choice between variants of a control is an element that measures them where the bounds are real (`elements::adaptive`). A `render` may read the window for a pure decision, never write state from it.
- **No hand-measured constants.** A height "measured on the bench" and copied into `ui_metrics.rs` breaks with the next font or padding change. Let the layout measure; keep only scale tokens.
- **Drags read their bounds from `DragMoveEvent`**, the container they resize, never from the window.
- **Every key is an action of the registry** (`state/keys.rs`). `keys::bindings()` feeds GPUI's keymap: the window's actions in the `Workspace` context, and the keys a deeper place owns given back to it with `NoAction` (characters to a text field, arrows/Enter/Escape to an open menu). Never compare key names by hand in a window-wide `on_key_down`; the help sheet and the palette read the same list, so they cannot drift.
- **Popups own the keyboard.** A palette or menu takes the focus on open and gives it back on close (to where it was, else the window), never leaving it nowhere.
- **Every scrolling view has a scrollbar** (`elements::scrollbar`): it overlays the right edge (never a column of its own), reads the handle the view scrolls with (`vertical` for a `ScrollHandle`, `for_list` for a `uniform_list`), and places its thumb in `prepaint`. A scroll area without a bar hides how much lies below the fold.
- **A control activates on click, not on key-down.** Use `on_click`: GPUI fires it on the mouse's release (a press dragged off cancels) and on Enter/Space key-up, only if the focus did not move. Never activate from `on_key_down` (a held key repeats). A menu trigger or a list row may still act on press for the pointer, and takes the keyboard through `on_click` with `event.is_keyboard()`.
- **Text on a full colour is `theme.on(fill)`** (or `on_accent`), never `bg_app`: its contrast is guarded.
- **An element is known by what it shows, never by where it stands** (`elements::clip::known_by`): a catalogue row or card takes its id from its package. GPUI keeps an element's tooltip, press and accessibility node under its id; an id by position hands them to whatever comes to stand there as the list scrolls or filters.
- **Text cut to fit says itself whole while cut** (`elements::clip::clipped`): whether it was cut is read from the drawn layout when the tooltip is asked for, never in `render`. All tooltips share one look (`components::tooltip`).
- **Empty is a state**: a list that shows nothing says why (the text typed, the filters, nothing to update) and the way out.
- **The workshop's look is in `docs/brand.md`** ("The interface: the workshop"): planes on the chassis (`bg_app` on `bg_sidebar`, edge + `shadow_sm`), the accent only for the cursor, focus, primary action and lamps (`accent_text` when it is text), sentence case, the three embedded voices of `fonts.rs`. Never name a font family but through `fonts::{UI, LABEL, CODE}`.
