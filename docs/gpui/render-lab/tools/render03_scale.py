#!/usr/bin/env python3
"""Run a bounded offline scale sweep over already captured fixture buffers."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

from render03_measure import measure


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    ledger = json.loads(args.ledger.read_text())
    rows = []
    for row in ledger["rows"]:
        artifact = Path(row["artifacts"][0]).parent
        reference = Image.open(artifact / "reference-normalized.png").convert("RGBA")
        output = Image.open(artifact / "output-normalized.png").convert("RGBA")
        scales = []
        for name, factor in (("small", 0.5), ("canonical", 1.0), ("large", 2.0)):
            size = (max(1, round(reference.width * factor)), max(1, round(reference.height * factor)))
            ref_scaled = reference.resize(size, Image.Resampling.BILINEAR)
            output_scaled = output.resize(size, Image.Resampling.BILINEAR)
            scales.append({"scale": name, "dimensions": list(size), "resample_policy": "ResizeLinear", "metrics": measure(ref_scaled, output_scaled)})
        rows.append({"fixture_id": row["fixture_id"], "scales": scales, "texture_regeneration": "not measured; output is a captured RGBA buffer"})
    args.output.write_text(json.dumps({"schema_version": 1, "rows": rows}, indent=2) + "\n")


if __name__ == "__main__":
    main()
