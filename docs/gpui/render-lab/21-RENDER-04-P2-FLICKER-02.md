# RENDER-04-P2-FLICKER-02 - Frame-State Containment & Callback Residency

Status: `SCISSOR_LEAK_ESTABLISHED` + `CALLBACK_FRAME_RESIDENCY_MISMATCH_ESTABLISHED`

## Scope

This is the causal follow-up to FLICKER-01. The canonical checkout was not
modified and started at `6fce3747fa0e1254efd4a37d32ba1d29b6a708dd`. The
experiment used the fixed fixture `optical.energized-wire-glow` and the
existing experimental GPUI Blade seam in an isolated worktree.

FLICKER-01 evidence was copied byte-for-byte before the experiment. Its
historical measurements remain unchanged.

## Phase A: Scissor Containment

F2-A and F2-B used the same binary/source, fixture, window geometry, workspace,
and native Wayland path. The only runtime difference was the existing
`SHELLY_P2_SCISSOR_RESET` hook.

| Run | Reset | Client geometry | Frames | Paint/enqueue/callback | Reset count |
| --- | --- | --- | ---: | --- | ---: |
| F2-A | absent | `[1939,399]`, `1062x1902`, workspace 11 | 80 | `76 / 76 / 76` | 0 |
| F2-B | present | `[1939,399]`, `1062x1902`, workspace 11 | 281 | `76 / 76 / 76` | 76 |

Both runs had one Shelly client, `xwayland=false`, and identical normalized
Hyprland geometry. F2-A reproduced the clipped native state after the ordered
callback left the probe scissor active. F2-B restored the full target scissor
after each callback and the clipped native state was absent in the matched
capture. The custom probe still executed in F2-B.

Classification:

```text
SCISSOR_LEAK_ESTABLISHED: YES
GPU_DYNAMIC_STATE_LEAK_ESTABLISHED: YES
GPU_DYNAMIC_STATE_LEAK: SCISSOR
SCISSOR_ONLY_CAUSALITY_REJECTED: NO
```

This establishes scissor causality only. No generic GPU state restore was
tested or introduced.

## Phase B: Scene Residency

The temporary renderer instrumentation logged the `Scene` address, callback
counts consumed by each renderer draw, frame sequence, and callback dispatch.
In the frozen B post-scene run:

```text
frame 24: scene_ptr=0x56359d7fae20 post_callbacks=1, B callback seq=25
frame 25: scene_ptr=0x56359d7fae20 post_callbacks=0, no B callback
frame 26: scene_ptr=0x56359d7fae20 post_callbacks=1, B callback seq=26
```

The same `Scene` was therefore rendered after callback consumption without a
post-scene callback on frame 25. The run recorded 79 scene draws, 79 B paints,
and 78 B callbacks. This establishes a callback residency/cadence mismatch,
but not a shader-visible flicker: the eight same-geometry captures were
byte-identical and no visual absent/present transition was correlated to the
single no-callback draw.

Classification:

```text
CALLBACK_FRAME_RESIDENCY_MISMATCH_ESTABLISHED: YES
SHADER_CALLBACK_CADENCE_FLICKER_ESTABLISHED: NO
```

The evidence supports the narrower statement that the current one-shot
callback is not guaranteed to accompany every render of a resident `Scene`.
It does not prove that this gap caused the operator's shader flicker.

## State Containment Audit

| State | Modified by callback | Restored by experiment | Evidence |
| --- | --- | --- | --- |
| pipeline | YES | no explicit restore | `GpuFrame::draw_energized_wire` binds the experimental pipeline; native batches bind their own pipelines |
| scissor | YES | F2-B only | `GpuFrame::set_scissor_rect`; full target reset logged after callback |
| viewport | NO | n/a | no viewport call in callback seam |
| blend constants | NO | n/a | no blend-constant call in callback seam |
| stencil/reference | NO | n/a | no stencil call in callback seam |
| bindings | YES | no generic restore | `draw_energized_wire` binds its parameter buffer; subsequent native encoders bind their own resources |

Only scissor produced the reproduced native corruption under this protocol.
The pipeline and binding observations are recorded as seam requirements, not
as separately established corruption causes.

## Operator Mapping

```text
OPERATOR_STATE_A -> PARTIAL
  matched to the native-complete / custom-callback-present state in F2-B

OPERATOR_STATE_B -> PARTIAL
  matched to the clipped native state in F2-A; shader-present/absent cadence
  was not independently established
```

The captures include unrelated desktop windows, so they are evidence of the
controlled renderer delta rather than clean product screenshots.

## Architecture Conclusion

1. A custom draw can execute in the existing GPUI Blade context: `YES`.
2. The current callback prototype composes safely: `NO` for canonical adoption;
   scissor containment is missing and Scene-resident callback semantics are
   not guaranteed.
3. A future seam minimally requires GPU-state containment, Scene/frame
   residency semantics, deterministic insertion ordering beyond `DrawOrder`,
   resource lifetime guarantees, and explicit bounds/clipping authority.

The ordered callback remains research-valid but architecturally insufficient:
`DrawOrder=4` is not a unique insertion key. No `PrimitiveId`, `BatchId`,
global ordering token, scene ABI, renderer framework, or production patch was
created.
