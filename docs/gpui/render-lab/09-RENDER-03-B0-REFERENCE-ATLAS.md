# RENDER-03-B0 — Canonical Reference Atlas

Status: verified locally. The machine-readable authority is
`references/atlas/reference-atlas.json`.

The atlas maps all 46 fixture IDs to raw RGBA crops from the two immutable G–K
board assets. It records source SHA-256, board/cell identity, crop rectangle,
crop SHA-256, label where the board provides one, and the explicit selection
method. Crops are not resized or color-converted.

```text
G  8/8
H  8/8
I  8/8
J  8/8
K 14/14
total 46/46
```

The G/H/I/J board uses the visible board-cell boundaries. Board K uses the
declared `3 + 3 + 4 + 4` cell convention. This convention is explicit in the
atlas and is not treated as an authority inherited from earlier code.

QC sheets with numbered rectangles are stored in `references/atlas/qc/`.
Primary crops are stored in `references/atlas/crops/`. The QC images are review
artifacts; the JSON and raw crops are the comparison authority.

No Render Lab output ROI is used as a reference source. Existing RENDER-01
ROIs remain current GPUI outputs only.
