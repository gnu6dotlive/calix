#!/usr/bin/env python3
"""Run the bounded RENDER-03 visual gap measurement over the 46 atlas entries."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from statistics import mean

from PIL import Image, ImageChops, ImageFilter, ImageOps


def rgba_sha(image: Image.Image) -> str:
    return hashlib.sha256(image.convert("RGBA").get_flattened_data()).hexdigest()


def normalize(reference: Image.Image, output: Image.Image) -> tuple[Image.Image, Image.Image, str, str]:
    reference = reference.convert("RGBA")
    output = output.convert("RGBA")
    if reference.size == output.size:
        return reference, output, "Identity", "Identity"
    return reference, output.resize(reference.size, Image.Resampling.BILINEAR), "Identity", "ResizeLinear"


def measure(reference: Image.Image, output: Image.Image) -> dict[str, object]:
    ref = list(reference.get_flattened_data())
    got = list(output.get_flattened_data())
    changed = [a != b for a, b in zip(ref, got)]
    rgb_delta = [sum(abs(a[i] - b[i]) for i in range(3)) / 3 for a, b in zip(ref, got)]
    rgba_delta = [sum(abs(a[i] - b[i]) for i in range(4)) / 4 for a, b in zip(ref, got)]
    alpha_delta = [abs(a[3] - b[3]) for a, b in zip(ref, got)]
    ref_edges = reference.convert("L").filter(ImageFilter.FIND_EDGES)
    got_edges = output.convert("L").filter(ImageFilter.FIND_EDGES)
    edge_changed = sum(a != b for a, b in zip(ref_edges.get_flattened_data(), got_edges.get_flattened_data()))
    return {
        "dimensions": list(reference.size),
        "changed_pixel_count": sum(changed),
        "changed_region_ratio": sum(changed) / len(changed),
        "mean_absolute_rgb_delta": mean(rgb_delta),
        "mean_absolute_rgba_delta": mean(rgba_delta),
        "max_absolute_rgb_delta": max(rgb_delta),
        "mean_absolute_alpha_delta": mean(alpha_delta),
        "edge_changed_pixel_count": edge_changed,
        "edge_changed_region_ratio": edge_changed / len(changed),
        "alpha_only_change": all(not different or a[:3] == b[:3] for different, a, b in zip(changed, ref, got)),
    }


def save_artifacts(reference: Image.Image, output: Image.Image, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    reference.save(destination / "reference-normalized.png")
    output.save(destination / "output-normalized.png")
    absolute = ImageChops.difference(reference, output)
    ImageOps.autocontrast(absolute.convert("RGB")).save(destination / "absolute-diff.png")
    ref_edges = reference.convert("L").filter(ImageFilter.FIND_EDGES)
    got_edges = output.convert("L").filter(ImageFilter.FIND_EDGES)
    ImageOps.autocontrast(ImageChops.difference(ref_edges, got_edges)).save(destination / "edge-diff.png")
    color = Image.new("RGB", reference.size)
    color.putdata([(min(255, abs(a[0] - b[0]) * 3), min(255, abs(a[1] - b[1]) * 3), min(255, abs(a[2] - b[2]) * 3)) for a, b in zip(reference.get_flattened_data(), output.get_flattened_data())])
    color.save(destination / "color-diff.png")


def load_structural(path: Path) -> dict[str, dict[str, int]]:
    rows = json.loads(path.read_text())["fixtures"]
    return {row["data"]["recipe"]: row["data"]["metrics"] for row in rows}


def run(atlas_path: Path, output_root: Path, structural_path: Path, destination: Path) -> None:
    atlas = json.loads(atlas_path.read_text())
    structural = load_structural(structural_path)
    rows = []
    for fixture_id, entry in sorted(atlas["fixtures"].items()):
        reference_path = atlas_path.parent / "crops" / f"{fixture_id}.png"
        manifests = list(output_root.rglob(f"{fixture_id}.manifest.json"))
        if len(manifests) != 1:
            raise FileNotFoundError(f"expected one controlled output manifest for {fixture_id}, got {len(manifests)}")
        output_manifest = manifests[0]
        output_path = output_manifest.with_name(output_manifest.name.removesuffix(".manifest.json") + ".roi.png")
        if not output_manifest.exists() or not output_path.exists():
            raise FileNotFoundError(f"missing controlled output for {fixture_id}")
        output_manifest_data = json.loads(output_manifest.read_text())
        reference_raw = Image.open(reference_path).convert("RGBA")
        output_raw = Image.open(output_path).convert("RGBA")
        reference, output, reference_transform, output_transform = normalize(reference_raw, output_raw)
        artifact_dir = destination / "artifacts" / fixture_id
        save_artifacts(reference, output, artifact_dir)
        metrics = measure(reference, output)
        recipe = output_manifest_data["recipe"]
        rows.append({
            "fixture_id": fixture_id,
            "board": entry["board"],
            "recipe_id": recipe,
            "capability_class_r01": output_manifest_data.get("verdict", "UNKNOWN"),
            "reference": {
                "source_asset": entry["source_asset"],
                "source_sha256": entry["source_sha256"],
                "atlas_crop_sha256_rgba": entry["crop_sha256_rgba"],
                "raw_dimensions": list(reference_raw.size),
            },
            "output": {
                "manifest": str(output_manifest),
                "backend": output_manifest_data.get("backend"),
                "seed": entry.get("seed"),
                "frozen_time": output_manifest_data.get("clock_time"),
                "raw_dimensions": list(output_raw.size),
            },
            "comparison": {
                "target_dimensions": list(reference.size),
                "transform_reference": reference_transform,
                "transform_output": output_transform,
                "alpha_policy": "RGBA; alpha included in RGBA delta",
                "color_space_policy": "raw source RGBA; no color conversion",
            },
            "metrics": metrics,
            "structural_metrics_r02": structural.get(recipe),
            "artifacts": [str(artifact_dir / name) for name in ("absolute-diff.png", "edge-diff.png", "color-diff.png")],
            "preliminary_disposition": "NEEDS_HUMAN_REVIEW",
            "review_status": "PENDING",
        })
    destination.mkdir(parents=True, exist_ok=True)
    (destination / "render03-ledger.json").write_text(json.dumps({"schema_version": 1, "fixture_count": len(rows), "rows": rows}, indent=2) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--atlas", type=Path, required=True)
    parser.add_argument("--outputs", type=Path, required=True)
    parser.add_argument("--structural", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    run(args.atlas, args.outputs, args.structural, args.destination)


if __name__ == "__main__":
    main()
