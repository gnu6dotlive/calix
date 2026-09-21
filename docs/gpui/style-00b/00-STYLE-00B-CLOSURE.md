# STYLE-00B — Surface Projection Wiring: Closure

**Status**: CLOSED & RATIFIED  
**Baseline**: `801674a2d9fed03c25f213b55d59d26cb7999057`

---

## 1. Contract Summary

STYLE-00B was authorized to wire `apply_surface_projection` into every primary
surface container in the Shelly GPUI UI, exercising the visual style system built
in STYLE-00A against the proven paint magnitudes established by RENDER-01/02.

The ratified boundary was:

- **In scope**: NavigationRail, QueryChrome, ResultSurface, Menu surfaces — all four primary
  surface roles — wired to `apply_surface_projection` with runtime `VisualStyleId` propagation.
  Confrontation matrix (16-cell STYLE-00B grid in Render Lab).
- **Out of scope**: Inspector panel, console, toast overlays, news view. No new surface roles.
  No STYLE/GPUI shader work. No product-adoption surface integration beyond the four primary roles.

---

## 2. Implementation Commits

| Commit | Description |
|---|---|
| `feat(style-00b): wire apply_surface_projection to all primary surfaces` | Implementation |
| `docs(style-00b): closure, confrontation matrix, effect resolution upgrade` | This document |

> Baseline: `801674a2d9fed03c25f213b55d59d26cb7999057`  
> Implementation: `8e7d6d74e75bec51517eee432005453b8bad376b`  
> Closure: `b0fab6deab9884cd90f8543034c29287941d0018`
> Final evidence / published HEAD: `94d7186f65c3e254d98e0d442ab78f5037e0c2a8`
>
> Historical evidence note: the R2 Standard screenshot A/B gate was left
> incomplete. R3/R3B then demonstrated that live product-window `grim` hashes
> were not deterministic, so that gate was superseded by the R4 structural
> paint-contract audit. STYLE-00B remains CLOSED & RATIFIED.

---

## 3. Surface Wiring Map

| Surface Role | Component / View | Before STYLE-00B | After STYLE-00B |
|---|---|---|---|
| `NavigationRail` | `SidebarProps` / `sidebar.rs` | Hard-coded `.bg(theme.bg_sidebar)` | `apply_surface_projection(…, NavigationRail, …)` |
| `QueryChrome` | `QueryWorkbenchProps` / `query_workbench.rs` | Hard-coded `.bg(theme.bg_sidebar)` | `apply_surface_projection(…, QueryChrome, …)` |
| `ResultSurface` | `PackageCardProps` / `package_card.rs` | Hard-coded `.bg(bg_color)` | `apply_surface_projection(…, ResultSurface, …)` |
| `Menu` | `MenuSurfaceProps` / `menu.rs` | `.bg(theme.bg_surface).shadow_lg()` | `apply_surface_projection(…, Menu, …)` |

`apply_surface_projection` is generic over `E: Styled` to support both `Div` and
`Stateful<Div>` (required by `MenuSurface` which uses `.id(…)`).

---

## 4. VisualStyleId Propagation Chain

```
GpuiConfig.visual_style
  └── WorkspaceView
        ├── PackageWorkstationConfig.visual_style
        │     └── PackageWorkstationView.visual_style
        │           ├── QueryWorkbenchProps.visual_style   → QueryChrome
        │           └── PackageCardProps.visual_style      → ResultSurface
        ├── SidebarView::new(visual_style)                 → NavigationRail
        │     └── SidebarProps.visual_style
        └── RenderLabViewProps (matrix only, no surface role)
              └── MenuSurfaceProps.visual_style            → Menu (settings dropdown)
```

Runtime update paths (both update all wired surfaces):
1. `apply_committed_settings_to_runtime()` — live toggle while app is running
2. Save-settings path — persists and re-propagates on next run

---

## 5. Effect Resolution Upgrade (RENDER-01/02 Evidence Integration)

`visual_style/resolver.rs` updated from STYLE-00A's "all Unknown" baseline to reflect
empirical confrontation results:

| Effect Kind | STYLE-00A | STYLE-00B |
|---|---|---|
| `ContactDepth` | `Unknown` (hypothesis) | `Native` — proven via stock BoxShadow (h01, contact-shadow) |
| `RimResponse` | `Unknown` (unevaluated) | `Native` — proven via stock perimeter borders (rim-highlight-outline) |
| `Microstructure` | `Unknown` (deferred) | `TextureProof` — proven via deterministic immutable textures (Board K/J, 16 fixtures) |
| `BackdropBlur` | `Unknown` | `Unknown` — window-level appearance exists; per-surface path unproven in GPUI 0.2.2 |

New `EffectResolution::is_active()` predicate: `requested && resolved != Unknown`.

`StyleProjection` gains a `paint: SurfacePaintProjection` field, computed by
`SurfacePaintProjection::compute(style, role, scheme)` at resolution time.
`StyleProjection` derives `PartialEq` only (not `Eq`) since `SurfacePaintProjection`
contains `f32` fields.

---

## 6. New Modules

| Module | Purpose |
|---|---|
| `src/visual_style/paint.rs` | `apply_surface_projection<E: Styled>` — the central paint seam. Takes `(div, role, style, base_bg, theme)`. Generic over `Styled` trait. |
| `src/visual_style/projection.rs` | `SurfacePaintProjection` — computed paint magnitudes: `surface_opacity`, `content_scrim`, `rim_active`, `shadow_active`, `rim_color`, `shadow_alpha`. |

Both modules re-exported from `visual_style/mod.rs`:
```rust
pub use paint::apply_surface_projection;
pub use projection::SurfacePaintProjection;
```

---

## 7. Render Lab: STYLE-00B Confrontation Matrix

A 16-cell confrontation matrix added to `views/render_lab.rs`:

- **Dimensions**: 4 surface roles × 4 (style × scheme) combos
  - Roles: NavigationRail, QueryChrome, ResultSurface, Menu
  - Combos: Standard/Dark, Standard/Light, Transparency/Dark, Transparency/Light
- **Backdrop**: High-contrast split (blue / rose) to expose surface opacity
- **Telemetry footer**: Per-cell `opacity%`, `scrim%`, `rim`, `shadow` pills
- **Access**: `⊞ Style Matrix (STYLE-00B)` button in catalog column; `⊞ Show Style Matrix` in fixture header bar; `shelly render-lab fixture clear` via CLI

New types added to `render_lab.rs` at module scope (not inside `impl`):
- `ClearFixtureHandler` type alias
- `MatrixRoleRow` struct (module-scope to satisfy Rust struct-in-impl restriction)

`render_lab/state.rs`: `set_fixture_by_id_str()` now accepts `"none"`, `"clear"`, or
empty string to clear the active fixture (enabling `shelly render-lab fixture clear`).

---

## 8. Ancillary Changes

- **`theme.rs`**: Added `Theme::is_dark() -> bool` predicate (`bg_app.r < 0.5`),
  consumed by `apply_surface_projection` to select appropriate surface blend.
- **`docs/gpui/render-lab/07-RENDER-02-CLOSURE.md`**: Minor doc polish (no semantic change).
- **`docs/gpui/render-lab/05-ACCEPTANCE.md`**: Minor doc polish (no semantic change).

---

## 9. Quality Gates

| Gate | Requirement | Outcome |
|---|---|---|
| `cargo fmt --check` | 0 formatting diffs | **PASS** |
| `cargo clippy -- -D warnings` | 0 warnings (zero-deadcode policy) | **PASS** |
| `cargo test` | ≥ 177 tests passing | **PASS** — 177 passed, 0 failed |
| Neutrality test renamed | `test_resolution_neutrality_all_unknown_in_style_00a` → `test_resolution_evidence_resolution_in_style_00b` | **PASS** — asserts per-effect RENDER evidence |
| Paint projection test | `test_resolve_deterministic_across_all_roles_styles_schemes` extended | **PASS** — asserts `paint` field correctness |

---

## 10. What This Does Not Change

- No new surface roles beyond the four primary ones.
- No `PackageInspectorView`, `OperationConsoleView`, `NewsView`, or toast overlays wired.
  These remain with their existing `theme.*` backgrounds; future chantier.
- No product-adoption integration (STYLE/ASTRA surface registry).
- No custom shaders. No RENDER-03 work.
- `shelly render-lab` visual regression: 46/46 fixtures still bit-identical to RENDER-01 baseline.
