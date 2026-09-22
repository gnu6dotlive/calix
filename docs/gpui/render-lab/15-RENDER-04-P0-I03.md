# RENDER-04-P0-I03 - Blade -> GPUI Presentation Interop

Status: `GPU_NATIVE_INTEROP_BLOCKED` + `CPU_READBACK_PRESENTATION_ESTABLISHED`

## Starting Evidence

I02 closed at commit `9b12b4e280df667a20d4d450985ce3bf623351fc` with a public
`blade-graphics 0.7.1` context, validated WGSL, and a created render pipeline.
GPUI presentation was not tested there.

## Interop Seam Audit

| Seam | Result | Evidence |
| --- | --- | --- |
| GPU texture -> GPUI | `NOT_FOUND` / `PRIVATE_ONLY` | GPUI exposes `RenderImage::new` from CPU `image::Frame`; no public constructor accepts a `blade_graphics::Texture` or external texture handle. GPUI's Blade renderer/context are private `pub(crate)` internals. |
| Blade -> CPU | `AVAILABLE` | Public `TextureUsage::COPY`, `BufferDesc { memory: Memory::Shared }`, `copy_texture_to_buffer`, `Context::submit`, `Context::wait_for`, and mapped `BufferPiece::data()`. |
| CPU pixels -> GPUI | `AVAILABLE` | Public `RenderImage::new(vec![Frame::new(RgbaImage)])`, then `img(Arc<RenderImage>)`. |

No GPU-native path was attempted after the audit because the required public
GPUI import seam is absent. I03-A is therefore classified
`GPU_NATIVE_INTEROP_BLOCKED`, not `ZERO_COPY_SHADER_IN_GPUI`.

## Context / Device Relationship

```text
CUSTOM BLADE CONTEXT: YES
SEPARATE FROM GPUI RENDERER CONTEXT: YES / independently created
SHARED DEVICE OR TEXTURE IDENTITY: NOT OBSERVABLE / NOT ASSUMED
```

The probe creates its own `blade_graphics::Context`. GPUI's Blade context is
owned by private GPUI platform code. I03 does not extract native handles or
assume that the two contexts share a `VkDevice`, queue, allocator, or texture
identity.

## I03-A GPU-Native Attempt

```text
RESULT: GPU_NATIVE_INTEROP_BLOCKED
```

The public GPUI image path accepts CPU image frames, not public Blade texture
resources. The required bridge would require private GPUI renderer types or
native handle/device interop, both outside this probe. No unsafe handle
extraction, GPUI patch, renderer fork, or external-memory machinery was added.

## I03-B CPU Readback Attempt

```text
RESULT: CPU_READBACK_PRESENTATION_ESTABLISHED
```

Source:

```text
Shelly.Ui.Gpui/src/render_lab/experimental.rs
Shelly.Ui.Gpui/examples/render04_p0_i03_blade_readback.rs
```

The one-shot path is:

```text
WGSL UV fragment shader
  -> Blade Rgba8Unorm target texture, 8x8
  -> Shared CPU buffer via copy_texture_to_buffer
  -> wait_for completion
  -> RgbaImage
  -> gpui::RenderImage
  -> img(RenderImage)
  -> Render Lab experimental stage
```

The target is deliberately `8x8`, not the `427x87` canonical metrology ROI;
I03 tests interop and provenance rather than visual fidelity. The copied size
is `8 * 8 * 4 = 256` bytes.

The Render Lab view displays the output under the non-canonical identity:

```text
experiment.render04-i03-shader-present
```

It is labeled `EXPERIMENTAL` and is not present in `FixtureCatalog`, the
canonical fixture matrix, or the capability ledger.

## Compile Evidence

```text
cargo fmt --check
cargo check --locked
cargo check --example render04_p0_i03_blade_readback --locked
```

All passed. The known dependency warning remains visible:

```text
proc-macro-error2 v2.0.1
future-incompat warning
non-blocking for I03
```

The nested local `[workspace]` boundary in `Shelly.Ui.Gpui/Cargo.toml` was
verified with `cargo metadata --locked`; the package resolves as its own
workspace without changing the parent `/home/tension_atoi/Cargo.toml`.

## Runtime Evidence

Command:

```text
cargo run --example render04_p0_i03_blade_readback --locked
```

Result:

```text
CPU_READBACK_PRESENTATION_READY
bytes_copied=256
elapsed_to_submit_micros=244461
elapsed_to_readback_micros=244596
elapsed_to_image_micros=245247
timing_semantics=cumulative from one-shot start; not independent phase durations
image_bytes=256
exit code: 0
```

The timing includes one-shot context, shader, pipeline, target, and buffer
initialization. It is not a steady-state frame benchmark. No animation or
per-frame recreation loop was introduced.

Runtime binary:

```text
/mnt/workbench/target/debug/examples/render04_p0_i03_blade_readback
```

The normalized Shelly presentation run used clean commit
`c8c6c577fb1eb71464c8db5732ff258a6fe3aa43`, with native Wayland confirmed by
the mapped `shelly-gpui` client (`xwayland: false`).

## Pixel Provenance Evidence

The fragment shader writes:

```text
R = position.x / 8.0
G = position.y / 8.0
B = 0.25
A = 1.0
```

Readback samples:

| Point | RGBA |
| --- | --- |
| `(0, 0)` | `[16, 16, 64, 255]` |
| `(4, 4)` | `[143, 143, 64, 255]` |
| `(7, 7)` | `[239, 239, 64, 255]` |

These values match the UV shader within the probe tolerance and are not a
hard-coded image or external PNG.

## Presentation Evidence

Capture from the exact commit:

```text
/mnt/workbench/pkgbuild/render-04-p0-i03-exact/render-lab.png
```

The native Wayland Shelly window visibly contains the UV gradient under the
label `EXPERIMENTAL · experiment.render04-i03-shader-present`. The same window
also shows the canonical field confrontation separately; the probe was not
substituted for a canonical fixture.

## Cost Observations

```text
render target: 8x8 Rgba8Unorm
CPU bytes copied: 256
elapsed from start through encode + submit: 244461 us
elapsed from start through completion + readback: 244596 us
elapsed from start through image construction: 245247 us
```

These are cumulative timestamps from the start of the one-shot, not independent
phase durations. They include context, shader, pipeline, target, and buffer
initialization and are coarse infrastructure data, not a performance claim.
I04 must use phase deltas for cost comparisons. Optimization is out of scope.

## Observed Constraints

- GPU-native sharing was not established because GPUI has no public external Blade texture import seam.
- The custom Blade context and GPUI's private renderer context remain separate.
- CPU readback and CPU `RenderImage` ingestion are sufficient to cross the presentation boundary.
- The Render Lab can host a one-shot experiment without adding catalog, persistence, or product authority.

## Architectural Implications

The statement now established is narrowly:

```text
CUSTOM WGSL -> BLADE OFFSCREEN DRAW -> CPU READBACK -> GPUI IMAGE -> VISIBLE RENDER LAB
```

This does not establish zero-copy interop, a shader ABI, production renderer
authority, performance suitability, animation architecture, or a material
system.

## Next Experiment

`RENDER-04-P0-I04` may investigate GPU-native host integration and the real
cost/constraint of sharing resources with GPUI. Keep the successful CPU bridge
active only as this experimental Render Lab proof; do not promote it to product
or canonical fixture authority.
