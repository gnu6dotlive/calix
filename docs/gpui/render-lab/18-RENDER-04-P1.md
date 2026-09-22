# RENDER-04-P1 - Applied Optical Rendering: local-focus

Status: `APPLIED_EXPERIMENT_ESTABLISHED` + `PRODUCTION_ARCHITECTURE_DEFERRED`

## Reference And Baseline

```text
fixture: optical.local-focus
recipe: i06-central-luminance-boost/r1
reference: docs/gpui/render-lab/references/atlas/crops/optical.local-focus.png
reference dimensions: 200x170, 8-bit sRGB
historical stock ROI: docs/gpui/render-lab/evidence/render01/i/optical.local-focus.roi.png
historical ROI dimensions: 481x336
capability: Composable
```

The atlas crop contains the original fixture card and a white header band. The
historical ROI is the normalized dark composition produced by the existing stock
recipe. The P1 surface displays the atlas reference explicitly as `REFERENCE ·
200x170`; A and B use equal `220x170` confrontation panels. This mapping is
intentional and not a silent resize claim.

The current stock plan is:

```text
nodes=2
expanded_ops=3
depth=2
fills=2
gradients=0
borders=0
shadows=1
textures=0
```

A is the existing `FixtureDef -> RecipePlan -> compile_stock_gpui` path. It
renders a centered `140px` white rounded disc over `rgb(0e141c)` with one
`60px` blur / `30px` spread shadow.

## Fresh Hook Reproduction

A fresh stock `gpui 0.2.2` copy was created under:

```text
/mnt/workbench/pkgbuild/render-04-p1/upstream/gpui-0.2.2
```

The I05 patch and the standalone `blade-graphics = "=0.7.1"` resolution
adjustment applied cleanly and compiled before P1 changes were added.

## P1-A0 Pipeline Residency

The smallest lifetime-correct location is the existing renderer-owned
`BladePipelines` struct. P1 adds one `local_focus` pipeline there:

```text
BladePipelines::new
  create shader + pipeline once per renderer
BladeRenderer::draw
  borrow pipeline through frame-scoped GpuFrame
  bind three vec4 parameters and draw
BladeRenderer::destroy
  existing wait_for_gpu()
  destroy BladePipelines, including local_focus
```

Runtime logs show initialization once per renderer and no per-frame pipeline
creation. Destruction occurs with the existing renderer teardown after the
renderer waits for its last sync point. No generic cache or ShaderManager was
introduced.

## Parameter Path And Compiler Discoveries

The shader receives real frame-local data through three explicit uniform
`vec4`s:

```text
bounds = origin_x, origin_y, width, height
focus  = center_x, center_y, inner_radius, outer_radius
color  = intensity, base_r, base_g, base_b
```

The first P1 uniform layout was blocked at runtime (`Plain size 52`, expected
`64`). An explicit padding attempt was blocked again (`Plain size 64`, expected
`80`). The final three-`vec4` layout compiled and ran. This is the minimal
parameter path for this specimen, not a generic material ABI.

The local UV is derived from the same bounds used for the scissor:

```text
uv = (framebuffer_position - bounds.origin) / bounds.size
aspect-corrected distance = (uv - center) * (bounds.width / bounds.height, 1)
```

No magic framebuffer size or time uniform is used.

## Visual Iterations

```text
B0  center=(0.5,0.5), inner=0.18, outer=0.80, intensity=0.90
    Observation: functioning radial field, but halo too broad and edge too bright.

B1  center=(0.5,0.5), inner=0.12, outer=0.62, intensity=0.90
    Observation: localized the focus and reduced the broad halo.

B2  center=(0.5,0.5), inner=0.08, outer=0.55, intensity=0.72
    Observation: better peak/falloff balance; visibly reads as a local focus
    rather than a white disc, while remaining softer than the stock disc.
```

The same B2 shader was rendered at both `220x170` and `256x128`. The resize
probe retained its center and radial character without a magic-size artifact.

Evidence captures are retained in the P1 workbench:

- `p1-local-focus-b0.png`
- `p1-local-focus-b1.png`
- `p1-local-focus-b2.png`
- `p1-final-comparison.png`
- `stock-a1.png` / `shader-b2.png`

## Comparison Metrics

The following samples come from the final `220x170` crops. They are 8-bit
numeric sRGB-derived values and are therefore approximate; the target surface
was `Bgra8Unorm`, not an independently calibrated sRGB comparison target.

```text
                         Stock A       Shader B2
center RGB               255,255,255   198,204,212
top-edge RGB             43,65,89      20,26,34
top-left RGB             27,40,55      14,20,28
centerline radial luma   1.00 -> .19   .797 -> .076
radial symmetry           disc/blur     centered, aspect-corrected
```

The centerline profiles show the essential difference: A has a hard bright
plateau followed by a single shadow falloff, while B has a continuous radial
field. No universal quality score or winner is declared.

## Composition Order And Limits

The B draw is still the I05 seam:

```text
POST-SCENE / PRE-SUBMIT-PRESENT
```

It is therefore an overlay after all GPUI scene batches. P1 does not establish
that local-focus can be inserted behind text, inside one surface, below borders,
or between arbitrary GPUI layers. That remains an `ORDERING_FIDELITY_LIMIT`.

Color comparison is `COLOR_SPACE_COMPARISON: APPROXIMATE` because the reference
is sRGB PNG data while the runtime target is `Bgra8Unorm` and no calibration
conversion was introduced.

## Final P1 Gate

```text
REFERENCE UNDERSTOOD: YES, with explicit atlas/ROI mapping
STOCK GPUI A RUNNING: YES
SAME-CONTEXT SHADER B RUNNING: YES
SAME VISUAL BOUNDS: YES (220x170), plus 256x128 resize probe
SHADER PARAMETER PATH: YES (three vec4 uniforms)
SAFE EXPERIMENTAL PIPELINE LIFETIME: YES within renderer teardown
CPU READBACK: NO
SECOND GPU CONTEXT: NO
SHADER VISUAL ITERATIONS: B0, B1, B2
REMAINING MISMATCH: understood as stock hard disc vs shader radial field and color-space approximation
STRUCTURAL BURDEN: A = 2 nodes / 3 expanded ops / 1 shadow; B = 1 resident pipeline + 1 draw + 3 vec4 uniform block
COMPOSITION ORDER LIMITATION: documented
PRODUCTION ARCHITECTURE: NOT RATIFIED
```
