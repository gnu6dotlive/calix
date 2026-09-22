# RENDER-04-P0-I04 - GPU-Native Host Integration

Status: `PUBLIC_HOST_INTEROP_BLOCKED` + `MINIMAL_GPUI_HOOK_IDENTIFIED` + `MEMORY_SHARING_ESTABLISHED_SYNC_BLOCKED`

## Starting Authority

```text
base commit: 2bd60d852637698fcf5321804190c695de0fa6d7
worktree: clean at probe start
I03 control: CPU readback presentation established
I03 native sharing: not established
```

I04 did not modify GPUI, vendor dependencies, add a patch section, or retain
compile-probe source. All temporary probes were deleted after their diagnostics
or runtime observations were recorded.

## GPUI Renderer Ownership Map

The locked `gpui 0.2.2` Wayland path has this ownership shape:

```text
WaylandClientState (pub(crate))
  gpu_context: BladeContext (private GPUI module/type)
      gpu: Arc<blade_graphics::Context> (pub(super) field)

WaylandWindowState (public type, private fields)
  renderer: BladeRenderer (private GPUI module/type)
      gpu: Arc<blade_graphics::Context> (private field)
```

Evidence in the resolved source:

- `WaylandClientState::gpu_context` is a private field initialized once by `BladeContext::new()`.
- `WaylandWindowState::new` receives `&BladeContext` and constructs `BladeRenderer::new(...)` from it.
- `WaylandWindowState::draw` calls the private `state.renderer.draw(scene)`.
- `gpui::platform::blade` is a private module and re-exports Blade types only with `pub(crate)`.
- `Element::paint` receives only `&mut Window` and `&mut App`, not a renderer, context, command encoder, texture, or frame host.

The public `gpui::Window::gpu_specs()` exposes GPU information, not resource or
renderer ownership.

## Public Integration Seam Audit

| Candidate | Visibility | Result |
| --- | --- | --- |
| `Window` renderer/context accessor | Public `Window`, no matching methods | `NOT_FOUND` |
| `Element::paint` renderer access | Public trait, only `Window`/`App` arguments | `NOT_APPLICABLE` for raw GPU access |
| GPUI `Scene` custom GPU primitive | Scene is renderer-internal; no public custom primitive insertion API | `PRIVATE_ONLY` |
| GPUI `PlatformWindow`/renderer injection | Platform modules and renderer implementation are private | `PRIVATE_ONLY` |
| `RenderImage` external GPU backing | `RenderImage::new` accepts decoded CPU `image::Frame`; no Blade texture constructor | `NOT_FOUND` |
| Blade external texture export | Public `ExternalMemorySource`, `get_external_texture_source` on public Blade context | `AVAILABLE` |
| Blade external texture import | Public `TextureDesc.external` on another public Blade context | `AVAILABLE` |
| Blade external synchronization | Public context submit/wait is context-local; no public external semaphore handoff found | `NOT_FOUND` |
| GPUI consumption of imported Blade texture | No public GPUI import or texture wrapper | `PRIVATE_ONLY` |

## Compile Probes

### I04-A01 - Window Renderer Access

Attempted:

```rust
window.renderer()
```

Result: `COMPILE_BLOCKED`

```text
error[E0599]: no method named `renderer` found for mutable reference
`&mut gpui::Window` in the current scope
```

### I04-A02 - Element GPU Context Access

Attempted from a real `Element::paint` implementation:

```rust
window.gpu_context()
```

Result: `COMPILE_BLOCKED`

```text
error[E0599]: no method named `gpu_context` found for mutable reference
`&mut gpui::Window` in the current scope
```

These are not renamed-API guesses remaining unresolved: the source ownership
audit shows the required types and fields are private to GPUI's platform layer.

## Same-Context Probe

```text
Can Shelly access GPUI's renderer context publicly? NO
Can Shelly reuse GPUI's Blade context? NOT EXPRESSIBLE
Same Blade Context? NOT OBSERVABLE FROM PUBLIC API
```

The custom I03 context cannot be treated as GPUI's context. GPUI creates its
own `BladeContext` and owns the `BladeRenderer` objects. The public API does not
expose the `Arc<blade_graphics::Context>` stored inside either object, nor the
renderer command encoder used for the GPUI frame.

No same-context compile or runtime claim is made.

## External Resource Probe

The public Blade API does expose a limited external-memory path. A temporary
probe created an `8x8 Rgba8Unorm` texture with:

```text
ExternalMemorySource::Fd(None)
```

The exporter returned:

```text
Fd(Some(31))
```

A second independently initialized public `blade_graphics::Context` imported
that source through `TextureDesc.external` and returned the same external source
metadata. Both contexts reported:

```text
NVIDIA GeForce RTX 3070
driver: NVIDIA 615.71.09
```

This establishes only:

```text
EXPORT_FROM_CUSTOM_BLADE: AVAILABLE
IMPORT_INTO_OTHER_BLADE_CONTEXT: AVAILABLE
```

It does not establish:

```text
IMPORT_INTO_GPUI_RENDERER: PRIVATE_ONLY
same logical device: NOT OBSERVABLE
same allocator: NOT OBSERVABLE
GPUI sampling: NOT ATTEMPTED
```

The two contexts sharing the same reported physical device is evidence of
device selection, not proof of shared `VkDevice`, queue, allocator, or Blade
context identity.

## Synchronization Findings

```text
MEMORY_SHARING: ESTABLISHED BETWEEN TWO PUBLIC BLADE CONTEXTS
SYNCHRONIZATION: NOT ESTABLISHED
```

`ExternalMemorySource` transfers a memory handle only. The audited public API
does not expose an external semaphore/fence export/import path that connects
the custom context's completion to GPUI's renderer queue. The public
`Context::submit` and `Context::wait_for` operations are local to a context and
cannot establish GPUI consumption ordering.

No sleep, device-idle claim, CPU polling workaround, or unsafe native handle
extraction was used.

## Runtime Evidence

Command:

```text
cargo run --example render04_p0_i04_c01_external_blade_probe --locked
```

Output:

```text
EXPORTED_SOURCE=Fd(Some(31))
EXPORT_DEVICE=DeviceInformation { is_software_emulated: false, device_name: "NVIDIA GeForce RTX 3070", driver_name: "NVIDIA", driver_info: "615.71.09" }
IMPORT_DEVICE=DeviceInformation { is_software_emulated: false, device_name: "NVIDIA GeForce RTX 3070", driver_name: "NVIDIA", driver_info: "615.71.09" }
IMPORTED_SOURCE=Some(Fd(Some(31)))
```

The temporary source was deleted after this runtime evidence was captured.
No GPUI surface, scene, or presentation was attempted by this probe.

## Integration Burden

```text
new direct dependencies: 0
new retained unsafe blocks: 0
temporary probe unsafe blocks: Context::init only, existing Blade API requirement
new native handles: 1 exported POSIX FD in temporary probe
new synchronization primitives: 0
new GPUI internal touchpoints required: 2 private ownership layers
new retained lifetime-bearing state: 0
renderer fork/vendor/patch: 0
```

The external-memory route is therefore technically reachable beside GPUI but
does not reduce the host integration boundary: GPUI still cannot import or
consume the resource through its public API, and synchronization remains open.

## Minimal Missing Hook

```text
Need:
one frame-scoped, renderer-owned custom draw seam in GPUI that runs against
the existing GPUI Blade context/command encoding stage and can submit a
custom primitive or texture operation before the GPUI frame is finalized.

Lifetime:
borrowed for one render frame; no global GPU service or persistent texture
authority required.

Frame stage:
inside the private BladeRenderer draw path, with access to the same context
and the active render command encoder/target.

Because:
public Element::paint receives Window/App only; RenderImage owns CPU frames;
and private BladeRenderer owns the context, command encoder, and surface.

Does not require:
renderer replacement, a new application lifecycle, a global GPU service,
a public shader ABI, a material system, or a product-level texture registry.
```

Exposing only a read-only `Arc<blade_graphics::Context>` would not be enough:
it would still not provide the active GPUI frame target, command encoder,
resource lifetime, or synchronization point. The smallest useful hook is a
safe frame-scoped host callback/custom primitive interface, not a standalone
context getter.

## Architectural Implication

I04 establishes:

```text
PUBLIC_HOST_INTEROP_BLOCKED
MINIMAL_GPUI_HOOK_IDENTIFIED
EXTERNAL_RESOURCE_MEMORY_ACCESSIBLE
SYNCHRONIZATION_NOT_ESTABLISHED
```

The result is not `ZERO_COPY_ESTABLISHED`. It also does not prove that
zero-copy is impossible. It identifies the missing public capability at the
actual GPUI renderer boundary.

## Next Experiment

`RENDER-04-P0-I05` may authorize a narrowly bounded GPUI hook experiment based
on this finding: a frame-scoped custom Blade draw callback or equivalent
renderer-owned primitive seam. The hook should be evaluated before any
external-memory or zero-copy architecture is generalized.
