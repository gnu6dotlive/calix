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
