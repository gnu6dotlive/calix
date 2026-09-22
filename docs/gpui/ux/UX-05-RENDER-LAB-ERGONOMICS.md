# UX-05 - Render Lab Ergonomics & Settings Entry

Status: `IMPLEMENTED` + `RUNTIME_PARTIAL`

## Scope

UX-05 is isolated from `RENDER-04-P2-FLICKER-01`. The worktree starts at
`ab3f581d63d83874f5ca0a4e925739b484b2b059`; no Blade renderer, scene ordering,
shader, pipeline, or P2 callback code was changed.

## Implemented

- Settings has a UI-only, default-collapsed `ADVANCED SETTINGS` section.
- The section exposes `settings_render_lab_btn` and navigates through the
  existing `NavDestination::RenderLab` authority.
- Render Lab remains absent from the global sidebar.
- Render Lab has `render_lab_back_settings` and returns through
  `NavDestination::Settings`.
- The ledger has UI-only, default-expanded state in `WorkspaceView`, with
  stable toggle ID `render_lab_ledger_toggle` and a 280px expanded / narrow
  collapsed width.
- The catalog retains `overflow_scroll()` and now uses a dedicated
  `ScrollHandle`; no virtualization or renderer rewrite was introduced.
- Keyboard Enter and Space handlers are wired for both collapse controls and
  the Render Lab launch/back actions.
- No persistence, manifest, CLI, protocol, or semantic `RenderLabState`
  fields were added.

## Validation

```text
cargo fmt --check       PASS
cargo check --locked    PASS
cargo test --locked render_lab  PASS: 37
cargo test --locked settings    PASS: 11
cargo test --locked     PASS: 178
git diff --check        PASS
```

Runtime binary:

```text
/mnt/workbench/pkgbuild/ux-05-render-lab/target/debug/shelly-gpui
sha256: 121175a06319a4e96de933c1d0f40ebd32fa3c90ce117a9b7a4d6bb4ffabab4a
WAYLAND_DISPLAY=wayland-1
xwayland=false
```

The Settings collapsed state and Render Lab expanded state were captured
successfully. The native multi-window Hyprland session prevented a reliable
automated collapsed-ledger interaction: the pointer reached the control and
focused it, but the subsequent key event was delivered to another active
window. The state transition itself is covered by the UI-local implementation
and unit tests; the collapsed runtime screenshot is therefore not claimed as
valid evidence.

## Gate

```text
Render Lab discoverable from Settings: YES
Render Lab in global sidebar: NO
Advanced Settings collapsible: YES
Advanced Settings persistence: NO
Render Lab back to Settings: YES
Ledger collapsible: YES
Ledger semantic state changed: NO
Catalog independently scrollable: YES, dedicated ScrollHandle
Scroll position stable during selection: PRESERVED; human smoothness remains runtime-observed
Keyboard controls: YES
Reduce-motion animation added: NO; controls snap immediately
Renderer architecture changed: NO
P2 flicker mixed into UX change: NO
Push: NO
```
