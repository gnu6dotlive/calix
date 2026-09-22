# RENDER-04-P0-I02 - Renderer Boundary / Blade Probe

Status: `PIPELINE_CREATED`

## Hypothesis

The Shelly crate can reach the renderer stack used by locked `gpui 0.2.2`,
compile one minimal WGSL shader, and create one GPU render pipeline outside the
public `gpui::Window` abstraction.

## Locked Renderer Dependency Authority

| Question | Result |
| --- | --- |
| GPUI renderer backend crate | `blade-graphics` |
| Resolved version/source | `0.7.1`, crates.io registry, lock checksum `e71cfb73b98eb9f58ee84048aa1bdf4e7497fd20c141b57523499fa066b48fed` |
| GPUI version | `0.2.2`, crates.io registry |
| Direct dependency before I02 | No; Blade was transitively resolved by GPUI |
| Direct dependency after I02 | Yes, exact `blade-graphics = "=0.7.1"` for this probe |
| Public crate usable externally | Established; registry crate exports `Context`, `ContextDesc`, `ShaderDesc`, `RenderPipelineDesc`, `TextureFormat`, and public shader/pipeline methods/traits |
| Shader representation | WGSL parsed and validated through Naga 25.0.1 |
| Device/context ownership | Public `blade_graphics::Context`, created by `unsafe Context::init(ContextDesc)` |
| Shader creation API | `Context::try_create_shader(ShaderDesc)` |
| Pipeline creation API | `Context::create_render_pipeline(RenderPipelineDesc)` |

The GPUI-side Blade module remains private: `gpui::platform::blade` is a
private module and re-exports its `BladeContext`/`BladeRenderer` with
`pub(crate)`. I02 does not use those private GPUI types. It uses the exact
public `blade-graphics 0.7.1` crate already present in the locked dependency
graph.

## Public API Seam

```text
blade_graphics::Context::init(ContextDesc)
blade_graphics::Context::try_create_shader(ShaderDesc)
blade_graphics::Context::create_render_pipeline(RenderPipelineDesc)
```

This seam is public because the types and methods are exported by the
`blade-graphics` library crate, not reached through `gpui::platform`.

## Implementation Attempt

Source:

```text
Shelly.Ui.Gpui/examples/render04_p0_i02_blade_probe.rs
```

The probe:

1. Initializes a non-presenting Blade context with validation enabled.
2. Loads a constant-color WGSL vertex/fragment pair.
3. Validates the shader with `try_create_shader`.
4. Creates a triangle-list render pipeline targeting `Rgba8Unorm`.
5. Drops the pipeline and exits without creating a surface or presenting.

No GPUI window, Render Lab state, product view, texture presentation, or
public Shelly ABI is involved.

## Compile Result

```text
COMPILE: PASS
```

Commands:

```text
cargo check --example render04_p0_i02_blade_probe --locked
cargo check --example render04_p0_i02_blade_probe --offline
```

The initial `--locked` attempt correctly rejected the new direct dependency
until the existing lockfile was updated. The resulting lockfile retains the
already resolved `blade-graphics 0.7.1`; no GPUI or Blade version was changed.

## Runtime Result

```text
PIPELINE_CREATED
exit code: 0
```

Command:

```text
cargo run --example render04_p0_i02_blade_probe --offline
```

Runtime provenance:

```text
git HEAD before I02 commit: d7934dc60522263de22ce0daa55517caf8cd1128
working tree: dirty during probe, only Cargo/example changes
target binary: /mnt/workbench/target/debug/examples/render04_p0_i02_blade_probe
presentation: not attempted
surface: not created
```

The runtime reached context initialization, shader validation, and pipeline
creation. No adapter telemetry was added because the public probe did not need
to invent or expose it.

## Observed Constraint

The direct Blade route is accessible from Shelly, but it is independent of
GPUI's private `BladeContext` and `BladeRenderer`. I02 proves a pipeline can
exist beside GPUI in the same Rust application stack; it does not prove that
the pipeline can render into a GPUI surface.

The parent `/home/tension_atoi/Cargo.toml` is an unrelated workspace that does
not include Shelly. A local empty `[workspace]` table was added to Shelly's
manifest so the existing standalone crate and lockfile could be checked
without modifying that external workspace.

## Architectural Implication

The first barrier is not Blade dependency access, WGSL compilation, GPU context
initialization, or pipeline creation. A later probe may investigate GPUI
presentation interop, but no shader manager, renderer fork, resource registry,
or product integration is justified by I02.

## Next Iteration

`RENDER-04-P0-I03` may test whether one isolated Blade result can be presented
inside one Render Lab surface. Keep that as a separate probe. Do not claim
custom shader support in GPUI until I03 succeeds.
