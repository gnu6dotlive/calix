#!/usr/bin/env python3
"""Measure the observable Render Lab CLI recipe-query wall time.

This is a CLI acquisition benchmark, not a GPU frame or draw-call benchmark.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import time
from pathlib import Path
from statistics import mean, median


def samples(binary: str, recipe: str, warmup: int, count: int) -> list[float]:
    command = [binary, "--json", "render-lab", "metrics", recipe]
    for _ in range(warmup):
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    values = []
    for _ in range(count):
        start = time.perf_counter_ns()
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        values.append((time.perf_counter_ns() - start) / 1_000_000)
    return values


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    index = min(len(ordered) - 1, round((len(ordered) - 1) * fraction))
    return ordered[index]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--recipes", type=Path, required=True)
    parser.add_argument("--binary", default="calix-gpui")
    parser.add_argument("--warmup", type=int, default=2)
    parser.add_argument("--samples", type=int, default=10)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    recipes = json.loads(args.recipes.read_text())["data"]
    rows = []
    for item in recipes:
        recipe = item["recipe_id"]
        values = samples(args.binary, recipe, args.warmup, args.samples)
        rows.append({
            "recipe_id": recipe,
            "measurement": "render_lab_metrics_cli_wall_ms",
            "samples": len(values),
            "warmup": args.warmup,
            "p50_ms": median(values),
            "p95_ms": percentile(values, 0.95),
            "max_ms": max(values),
            "spread_ms": max(values) - min(values),
            "mean_ms": mean(values),
        })
    result = {
        "schema_version": 1,
        "measurement_scope": "CLI recipe structural-metrics acquisition wall time",
        "not_claimed": ["GPU frame time", "GPU execution time", "draw calls", "VRAM"],
        "environment": {"platform": platform.platform(), "python": platform.python_version(), "cpu": platform.processor(), "pid": os.getpid()},
        "rows": rows,
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
