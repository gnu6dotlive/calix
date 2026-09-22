# RENDER-04-P0-I05 - Minimal Same-Context Frame Hook

Status: `SAME_CONTEXT_CUSTOM_DRAW_ESTABLISHED` + `DIRECT_FRAME_TARGET_DRAW_ESTABLISHED` + `BOUNDED_REGION_ESTABLISHED`

## Starting Authority

```text
base commit: 42ab6c11
GPUI source: exact gpui 0.2.2 crate copy
crate digest: 68dca63d7c278b338df11b5e607ab07ca0eb07f476b288f592ba8aa0d332267d
workbench: /mnt/workbench/pkgbuild/render-04-p0-i05
```

The canonical GPUI registry source was not modified. Shelly was compiled with a
temporary Cargo CLI patch pointing at an isolated and hashed workbench copy only;
no patch section was added to the repository manifest or lockfile.

## Hook Shape

The workbench GPUI copy adds a frame-scoped callback queue to `Scene`:

```text
Window::enqueue_gpu_frame_callback(callback)
  -> next_frame.scene callback queue
  -> BladeRenderer::draw(scene)
  -> after normal scene batches, before submit/present
  -> borrowed GpuFrame { existing Context, active RenderCommandEncoder }
```

The callback has no renderer/context getter and no global GPU authority. It can
only encode commands during the active frame. The callback queue is cleared by
the normal scene lifecycle.

## Temporary Shelly Consumer

The temporary `render_lab::experimental::gpui_frame_probe` used a real GPUI canvas. Its paint
callback queues a WGSL fullscreen triangle, applies a `128x128` scissor derived
from the canvas bounds, and draws directly into the live GPUI swapchain pass.

The probe does not create a Blade context, allocate an offscreen target, read
back pixels, construct a `RenderImage`, or composite on the CPU. The temporary
pipeline is destroyed after the draw; persistent renderer-owned resource
lifetime remains a later hook-design question.

The consumer was removed after the runtime capture so the canonical Shelly
checkout remains buildable against stock crates.io `gpui 0.2.2`.

The reproducible temporary changes are retained as:

- `evidence/patches/render04-p0-i05-gpui-0.2.2.patch` - hook only
- `evidence/patches/render04-p0-i05-gpui-resolution-adjustment.patch` - standalone `blade-graphics = "=0.7.1"` resolution adjustment
- `evidence/patches/render04-p0-i05-shelly.patch` - temporary Render Lab consumer

The first runtime shader attempt was `SHADER_COMPILE_BLOCKED`: Blade/Naga
rejected the direct `vec4` vertex output shape. An explicit `VertexOutput`
structure fixed the diagnostic shader. The final provenance shader uses
`fract(position.xy / 32.0)` plus a checker response, so the bounded region is
visibly non-uniform without saturating at the observed framebuffer coordinates.

## Verification

```text
cargo check --features wayland                         PASS (GPUI workbench)
cargo check --config patch.crates-io.gpui.path=...      PASS (Shelly temporary consumer)
Wayland runtime                                         PASS
same-context frame callback                             OBSERVED
direct current-frame target draw                        OBSERVED
bounded 128x128 shader output                           OBSERVED
distinct pixel provenance                               OBSERVED
```

Runtime evidence recorded in:

- `evidence/runs/render04-p0-i05.json`
- `evidence/logs/render04-p0-i05.log`
- `/mnt/workbench/pkgbuild/render-04-p0-i05/shelly-i05-provenance.png`

The runtime screenshot shows the non-uniform checker/coordinate shader square
below the existing I03 CPU-readback image in the Render Lab window.

## Limits

This is a workbench-only GPUI hook proof, not a product-level GPUI fork. It
establishes same-context command encoding, direct current-frame target draw,
bounded visible output, and post-scene/pre-submit-present ordering. It does not
establish arbitrary GPUI scene ordering, a stable upstream API, persistent
pipeline ownership, shader registry semantics, or a production custom-renderer
architecture.

## Pipeline Lifetime Audit

In `blade-graphics 0.7.1` Vulkan, `destroy_render_pipeline` calls
`vkDestroyPipeline` immediately after destroying the pipeline layout. The
audited source provides no deferred-destruction or completion guarantee for a
pipeline whose commands have been recorded but not submitted. Therefore:

```text
PIPELINE_LIFETIME_AFTER_ENCODING: NOT ESTABLISHED
PERSISTENT_PIPELINE_LIFETIME: NOT ESTABLISHED
```

The temporary consumer destroys its diagnostic pipeline to avoid retaining an
unbounded resource, but this is not a claim that the ordering is safe for a
production implementation. A pipeline cache belongs to a later applied
experiment.

## Final Contract

```text
SAME_CONTEXT_CUSTOM_DRAW_ESTABLISHED: YES
DIRECT_FRAME_TARGET_DRAW_ESTABLISHED: YES
BOUNDED_REGION_ESTABLISHED: YES
SECOND_BLADE_CONTEXT: NO
CPU_READBACK: NO
CPU_COMPOSITING: NO
COMPOSITION_ORDER: POST-SCENE / PRE-SUBMIT-PRESENT
STABLE_GPUI_API: NO
CANONICAL_PATCH_ADOPTED: NO
STOCK_SHELLY_CHECKOUT_RESTORED: YES
PUSH: NO
```
