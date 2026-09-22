# UX-05R - Render Lab Interaction Recovery

Status: `CODE_ESTABLISHED` + `RUNTIME_PARTIAL` + `HUMAN_VERIFIED_PENDING`

## Baseline

Baseline candidate: `fd882bac` (`feat(ux): improve Render Lab access and panel ergonomics`).
This worktree contains only the UX-05R repair; the canonical checkout and P2
work remain untouched.

## Ledger Root Cause

The baseline `render_lab_ledger_toggle` had focus, keyboard, hover, and
`cursor_pointer` behavior, but no `on_mouse_down` handler. Therefore a mouse
click could focus the control without invoking the state mutation. UX-05R adds
the same `on_toggle` action used by Enter and Space to the left mouse handler.

## Catalog Root Cause

Temporary instrumentation measured the same scroll handle after layout in both
the baseline and repaired worktrees:

```text
fd882bac baseline:
  bounds=260x2693
  max_offset.y=0

UX-05R:
  bounds=260x551
  max_offset.y=2142
```

The baseline catalog expanded to its content height, so `overflow_scroll()`
had no viewport range. UX-05R clips the flex body with `overflow_hidden()` and
uses `overflow_y_scroll()` on the catalog viewport. No virtualization,
renderer, or navigation change was needed.

## Runtime Boundary

The repaired binary starts on native Wayland with `xwayland=false`, one known
experimental Shelly process, and the exact isolated executable recorded in the
JSON evidence. Existing CLI commands opened Render Lab and Settings; these are
legitimate control-path checks, not proof of mouse/wheel interaction.

No synthetic input was used for UX closure. Consequently the following remain
open until an operator validates them directly:

```text
Ledger mouse collapse/expand: NOT VERIFIED
Ledger Enter/Space: NOT VERIFIED
Catalog wheel/deep scrolling: NOT VERIFIED
Selection position preservation: NOT VERIFIED
Continuous scroll smoothness: HUMAN_VERIFIED_PENDING
```

## Quality Gates

```text
cargo fmt --check: PASS
cargo check --locked: PASS
cargo test --locked: PASS, 178
cargo test --locked render_lab: PASS, 37
cargo test --locked settings: PASS, 11
git diff --check: PASS
jq empty evidence/runs/ux05r-render-lab-interactions.json: PASS
```

## Scope Gate

```text
renderer modified: NO
P2 mixed in: NO
persistence added: NO
new navigation authority: NO
synthetic input evidence: NO
main modified: NO
push: NO
```
