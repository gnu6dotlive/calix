# RENDER-03 — Measurement Run Status

## First visual-gap run disposition

The first C–F visual-gap corpus is **DIAGNOSTIC_ONLY** and is not valid for
`FIDELITY_GAP`, `PERFORMANCE_GAP`, `SHADER_REQUIRED`, or human quality
verdicts. It exposed an atlas G mapping defect, incomplete QC output,
preview-canvas versus intrinsic-primitive geometry, aspect-ratio-changing
`ResizeLinear`, semantic carrier differences, and unvalidated ordinal K
mapping.

The previous offline resize experiment is named **R03-E1 — OFFLINE
RESAMPLING SENSITIVITY**. It is not a GPUI scale sweep and does not measure
texture regeneration.

The first controlled C–F run is stored outside the repository under
`/mnt/workbench/evidence/render-03/visual-gap/`.

```text
fixtures measured                 46/46
reference atlas entries           46/46
structural RENDER-02 joins        46/46
difference products               3 per fixture
scale classes                     small / canonical / large
runtime measurement               CLI recipe acquisition wall time
GPU metrics claimed               none
```

The runtime ledger calls its measurement
`render_lab_metrics_cli_wall_ms`. It is not GPU frame time, GPU execution time,
draw calls, VRAM, or application render time.

The current visual ledger leaves every row at `NEEDS_HUMAN_REVIEW`. The raw
reference/output comparison is valid and reproducible, but the observed gaps
are broad enough that an automated threshold would conflate crop semantics,
recipe approximation and visual fidelity. No `FIDELITY_GAP`,
`PERFORMANCE_GAP`, or `SHADER_REQUIRED` conclusion is emitted from this run.

SSIM and CIEDE2000 are intentionally `NOT_IMPLEMENTED`; no local dependency
or reference implementation was available without adding an unapproved
dependency. RGB/RGBA, edge and alpha observations remain available.

The next required step is bounded human review of outliers and metric
disagreements using the generated per-board review sheets. Until that review is
recorded, R03-J is not closed and no burden candidate is supported.
