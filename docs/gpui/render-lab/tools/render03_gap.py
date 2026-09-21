#!/usr/bin/env python3
"""Calibrate and measure multi-axis Render Lab image gaps.

This tool intentionally does not assign a universal quality score. Inputs must
have equal dimensions and explicit provenance is carried in the output.
"""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter, ImageOps


def load_rgba(path: Path) -> list[tuple[int, int, int, int]]:
    image = Image.open(path).convert("RGBA")
    return list(image.get_flattened_data())


def image_size(path: Path) -> tuple[int, int]:
    with Image.open(path) as image:
        return image.size


def metrics(reference: Path, output: Path) -> dict[str, object]:
    reference_size = image_size(reference)
    output_size = image_size(output)
    if reference_size != output_size:
        raise ValueError(f"dimension mismatch: reference={reference_size} output={output_size}")

    ref = load_rgba(reference)
    got = load_rgba(output)
    changed = [a != b for a, b in zip(ref, got)]
    deltas = [sum(abs(a[channel] - b[channel]) for channel in range(4)) / 4 for a, b in zip(ref, got)]
    rgb_deltas = [
        sum(abs(a[channel] - b[channel]) for channel in range(3)) / 3
        for a, b in zip(ref, got)
    ]
    return {
        "dimensions": list(reference_size),
        "changed_pixel_count": sum(changed),
        "changed_region_ratio": sum(changed) / len(changed),
        "mean_absolute_rgba_delta": sum(deltas) / len(deltas),
        "mean_absolute_rgb_delta": sum(rgb_deltas) / len(rgb_deltas),
        "max_absolute_rgb_delta": max(rgb_deltas),
        "alpha_only_change": all(
            not different or a[:3] == b[:3]
            for different, a, b in zip(changed, ref, got)
        ),
    }


def difference_image(reference: Path, output: Path, destination: Path) -> None:
    with Image.open(reference).convert("RGBA") as ref, Image.open(output).convert("RGBA") as got:
        if ref.size != got.size:
            raise ValueError(f"dimension mismatch: reference={ref.size} output={got.size}")
        ImageOps.autocontrast(ImageChops.difference(ref, got)).save(destination)


def calibration_cases(destination: Path) -> None:
    size = (64, 64)
    base = Image.new("RGBA", size, (40, 60, 80, 255))
    cases = {
        "near-identical": base.copy(),
        "uniform-color-shift": Image.new("RGBA", size, (50, 70, 90, 255)),
        "localized-color-shift": base.copy(),
        "geometry-displacement": Image.new("RGBA", size, (40, 60, 80, 255)),
        "blur-mismatch": base.filter(ImageFilter.GaussianBlur(3)),
        "banding": Image.new("RGBA", size, (40, 60, 80, 255)),
        "over-bloom": base.copy(),
        "texture-frequency-mismatch": base.copy(),
        "missing-feature": base.copy(),
    }
    cases["near-identical"].putpixel((32, 32), (41, 61, 81, 255))
    for y in range(16, 48):
        for x in range(16, 48):
            cases["localized-color-shift"].putpixel((x, y), (100, 110, 120, 255))
    cases["geometry-displacement"].paste((220, 180, 40, 255), (12, 12, 36, 36))
    for y in range(size[1]):
        cases["banding"].paste((40 + (y // 8) * 5, 60, 80, 255), (0, y, size[0], y + 1))
    cases["over-bloom"].paste((100, 120, 140, 255), (12, 12, 52, 52))
    for y in range(0, size[1], 2):
        cases["texture-frequency-mismatch"].paste((90, 100, 110, 255), (0, y, size[0], y + 1))
    cases["missing-feature"].paste((220, 180, 40, 255), (24, 24, 40, 40))

    destination.mkdir(parents=True, exist_ok=True)
    base.save(destination / "reference.png")
    rows = []
    for name, image in cases.items():
        path = destination / f"{name}.png"
        image.save(path)
        rows.append({"case": name, "metrics": metrics(destination / "reference.png", path)})
    (destination / "calibration.json").write_text(
        json.dumps({"schema_version": 1, "reference": "reference.png", "cases": rows}, indent=2) + "\n"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    calibrate = subparsers.add_parser("calibrate")
    calibrate.add_argument("destination", type=Path)
    measure_parser = subparsers.add_parser("measure")
    measure_parser.add_argument("reference", type=Path)
    measure_parser.add_argument("output", type=Path)
    measure_parser.add_argument("--provenance", type=Path, required=True)
    measure_parser.add_argument("--json", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "calibrate":
        calibration_cases(args.destination)
        return
    provenance = json.loads(args.provenance.read_text())
    result = {
        "schema_version": 1,
        "reference": str(args.reference),
        "output": str(args.output),
        "provenance": provenance,
        "metrics": metrics(args.reference, args.output),
    }
    args.json.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
