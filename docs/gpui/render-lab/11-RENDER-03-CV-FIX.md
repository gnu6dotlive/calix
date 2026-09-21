# RENDER-03-CV-FIX — Reference Semantics and Comparison Contract

Status: authorized and locally audited.

## Reference authority

The G atlas is row-major (`1,2 / 3,4 / 5,6 / 7,8`) and has been regenerated.
QC sheets are now distinct: `qc-G.png`, `qc-H.png`, `qc-I.png`, `qc-J.png` and
`qc-K.png`. K remains ordinal only:

```text
mapping_basis = ordinal
semantic_validation = UNVALIDATED
```

## Comparison modes

`references/atlas/comparison-modes.json` assigns all 46 fixtures before any
rerun:

```text
DIRECT_SPATIAL       8
SEMANTIC_FEATURE    20
TEXTURE_STATISTICAL  4
REFERENCE_AMBIGUOUS 14
```

The first visual-gap ledger is retained as diagnostic-only. Its aspect-ratio
normalization must not be interpreted as a fidelity verdict.

## Controlled comparison path audit

The existing path is:

```text
FixtureDef
  -> RecipeCatalog / plan_for_recipe
  -> compile_stock_gpui(RecipePlan) -> AnyElement
  -> fixed 340px Render Lab confrontation box
  -> fiducial ROI (currently 481x336)
```

There is no existing CLI command, IPC field, test invocation, or Render Lab
route that supplies an explicit per-fixture canvas width, height, background,
and bounds to a framebuffer renderer. The current result is:

```text
MISSING_COMPARISON_RENDER_PATH
IPC IMPACT: none, because no IPC change was made
```

No new canvas or protocol field is implemented in CV-FIX. A future bounded
test-only or local Render Lab canvas must be specified separately before
R03-C2. Until then, R03-H, R03-I and R03-J remain paused/blocked.
