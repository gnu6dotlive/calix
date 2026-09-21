# RENDER-03 — Visual Gap and Composition Ceiling

Status: authorized measurement slice. No shader host or product-surface change
is in scope.

## Measurement rules

- Render Lab fixtures are the quantitative harness; live product screenshots are not an oracle.
- Reference provenance, dimensions, alpha handling, color format and transforms are explicit.
- Metrics are multi-axis observations, not a universal quality score.
- Structural metrics come from the canonical RENDER-02 `RecipePlan`.
- Runtime measurements use only quantities directly observed by the harness.
- `SHADER_REQUIRED` requires a documented visual, performance or scale burden of proof.

## Calibration

`tools/render03_gap.py calibrate <directory>` produces deliberate cases for
near-identity, color drift, localized change, displacement, blur, banding,
bloom and missing features. The resulting JSON is a calibration artifact, not
a production threshold.

## Output vocabulary

Each fixture receives a separate disposition:

```text
SUFFICIENT
SUFFICIENT_WITH_COST
FIDELITY_GAP
PERFORMANCE_GAP
REFERENCE_AMBIGUOUS
NEEDS_HUMAN_REVIEW
```

Candidate burden is recorded separately and never changes historical RENDER-01
capability classes automatically.
