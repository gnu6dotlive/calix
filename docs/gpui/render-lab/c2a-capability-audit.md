# RENDER-03 C2A Capability Authority Audit

Status: analysis was read-only against the runtime and dependency sources. This document is the resulting local, uncommitted documentation mutation until the dedicated closure commit.

## Authority Layers

| Capability | Recipe IR | Shelly compiler | Locked GPUI 0.2.2 | Authority conclusion |
|---|---|---|---|---|
| Solid fill | `FillSpec::Solid` | `div().bg(...)` | `Background` solid and `Window::paint_quad` | Used and established |
| Two-stop linear gradient | `FillSpec::LinearGradient2` | `linear_gradient(angle, from, to)` | `BackgroundTag::LinearGradient`; constructor accepts two stops | Used and established |
| Arbitrary multi-stop linear gradient | Not represented | Not compiled | `LinearColorStop` is public, but `Background`'s tag, color array, and color-space fields are crate-private; its public storage is exactly `[LinearColorStop; 2]` | User code cannot construct a `Background::LinearGradient` with more than two stops on locked `gpui 0.2.2`; no shader conclusion |
| Radial gradient | Not represented | Not compiled | No radial background tag/constructor found in the audited stock renderer path | No constructor/tag found in audited stock path |
| Rounded rectangle | `BorderRadius` | GPUI rounded style methods | `PaintQuad.corner_radii` | Used and established |
| Borders | `BordersSpec` | Stock border APIs | `PaintQuad.border_widths`, `border_color`, `border_style` | Used and established |
| Shadows | `ShadowSpec` | Stock shadow APIs | `Window::paint_shadows`, `BoxShadow` | Used and established |
| Opacity/layering | RGBA/HSLA plus child order | Style opacity and overlapping children | Element opacity multiplies painted primitives | Used and established |
| Paths/strokes | `Streamlines` only | `PathBuilder::stroke` + `paint_path` | `Window::paint_path`, `PathBuilder` | Available and used, but current representation is narrow |
| Repeated primitives | No repeat node | Explicit path loop/expansion | Primitive calls are available | Composition cost/capability question, not shader evidence |
| Rectangular clipping | Not represented | Not exposed by recipe compiler | `ContentMask` and `with_content_mask` | Backend available, current recipe/compiler not wired |
| Non-rectangular masking | Not represented | Not compiled | Source states only rectangular content masks are supported in the audited stock path | Not found in audited stock path |
| Transforms | Not represented in current recipe nodes | Not compiled for recipe graph | SVG transformations exist; no general recipe transform path found | Not available in current slice |

## Attribution Consequences

- G01, G02, G06 and G08 remain recipe-level candidates for palette, stop, alpha, or layering correction.
- G05 is recipe/composition-remediable only within the current two-stop linear-gradient IR; a true arbitrary multi-stop ramp would require an IR/compiler extension, not a shader by itself.
- G03 is not evidence of a GPUI primitive limit. Its current four-band appearance is selected by the recipe; a smoother result needs either better composition or a deliberately extended gradient representation.
- G04 has stock path support, but the current `Streamlines` node does not encode the reference's full visual semantics. This is a representation/composition gap, not an established shader gap.
- G07 has rounded quads, layering, gradients, and shadows available in stock GPUI. Its current nested solid rectangles are therefore not evidence of a backend limit.
- No fixture establishes shader burden. The audit does not authorize shader work.

## Source Anchors

- `src/render_lab/graph.rs`: `FillSpec`, `RecipeNode`, and `compile_stock_gpui()`.
- Locked source `gpui-0.2.2/src/color.rs:758-797`: two-stop `linear_gradient` and `LinearColorStop`.
- Locked source `gpui-0.2.2/src/color.rs:687-698`: `Background` has crate-private fields and a fixed `[LinearColorStop; 2]` carrier; this answers the public/private multi-stop check as `NO`.
- Locked source `gpui-0.2.2/src/window.rs:2803-2871`: shadows, quads, paths, and rectangular content-mask propagation.
- Locked source `gpui-0.2.2/src/style.rs:306-317`: `BoxShadow`.
- Locked source `gpui-0.2.2/src/window.rs:1278-1300`: rectangular-only `ContentMask`.
