# RENDER-04-P2-FLICKER-01 - Flicker Isolation

Status: `SCISSOR_LEAK_STRONGLY_INDICATED` + `DRAW_ORDER_INSUFFICIENT_AS_INSERTION_KEY` + `CALLBACK_CADENCE_MISMATCH_OBSERVED`

## Scope

The experiment used the fixed fixture `optical.energized-wire-glow` in an
isolated workbench. The canonical checkout remained unchanged at
`ab3f581d63d83874f5ca0a4e925739b484b2b059`.

Topology checks found one experimental Shelly process and one matching native
Wayland client. Duplicate-window topology was not causal evidence.

## Matrix

```text
F0 stock       528 frame draws; no experimental callbacks
F1 post-scene  276 frame draws; B paint=4, enqueue=4, callback=3
F2 ordered     264 frame draws; C paint=1, enqueue=1, callback=1
F2 reset       164 frame draws; C paint=2, enqueue=2, callback=2; scissor reset=2
F3 combined    1227 frame draws; B/C paint=13/13, enqueue=13/13, callback=11/11
```

The F2 capture shows native UI clipped to the ordered probe rectangle after the
probe calls `set_scissor_rect`. The minimal corrective experiment restores the
full framebuffer scissor after every callback. Its execution is logged, but
the reset and non-reset captures were not geometry-identical enough to claim a
clean visual disappearance test.

## Ordered Audit

The ordered callback is dispatched as `order=4` before a native batch with
`order=4`. The same frame contains multiple native batches at order 4, so the
draw order is not a unique insertion key. This is an ordering limitation, not
proof of temporal instability.

## Cadence Audit

F3 records 13 paints and enqueues but only 11 callbacks for both probes while
1227 frame draws occur. This establishes callback coalescing or consumption
between enqueue and dispatch; it does not alone establish flicker causality.

## Classification

```text
DUPLICATE_WINDOW_TOPOLOGY_CAUSAL: NO
SCISSOR_LEAK_ESTABLISHED: STRONGLY_INDICATED
DRAW_ORDER_INSERTION_UNSTABLE: NO
DRAW_ORDER_INSUFFICIENT_AS_INSERTION_KEY: YES
CALLBACK_CADENCE_FLICKER: NOT ESTABLISHED
```

No renderer fix was applied to the canonical checkout. The workbench deltas
are retained as evidence patches and the raw logs are retained under
`evidence/logs/`.
