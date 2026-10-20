#!/usr/bin/env python3
"""Measure the workstation's geometry on a live Wayland session.

The contract says the inspector stays inside [320, min(520, 3/7·w_c)] and the
list pane at or above 340px. This measures what is actually drawn: the rail's
border and the splitter are read from the pixels of a grim capture of the
mapped window.

Usage: measure.py [tag]
"""

from __future__ import annotations

import collections
import json
import subprocess
import sys
from pathlib import Path

from PIL import Image

APP_CLASS = "live.gnu6.calix"
RAIL = 190.0
SPLITTER = 5.0
LIST_FLOOR = 340.0
INSPECTOR_FLOOR = 320.0
INSPECTOR_CAP = 520.0
INSPECTOR_RATIO = 3.0 / 7.0


def hypr(*args: str) -> str:
    return subprocess.run(["hyprctl", *args], capture_output=True, text=True).stdout


def window() -> tuple[int, int, int, int]:
    for client in json.loads(hypr("clients", "-j")):
        if (client.get("class") or "") == APP_CLASS:
            at, size = client["at"], client["size"]
            return int(at[0]), int(at[1]), int(size[0]), int(size[1])
    raise SystemExit("the application window is not mapped")


def edges(png: Path) -> list[tuple[int, int]]:
    """The vertical edges of the capture, with how many sampled rows carry them.

    The splitter is a full-height border: it is the edge with the strongest
    count. The table's own column separators only exist in the header row, so
    they must not be mistaken for it.
    """
    image = Image.open(png).convert("RGB")
    width, height = image.size
    pixels = image.load()
    counter: collections.Counter = collections.Counter()
    for y in range(60, height, 40):
        for x in range(1, width):
            here, before = pixels[x, y], pixels[x - 1, y]
            if sum(abs(a - b) for a, b in zip(here, before)) > 40:
                counter[x] += 1
    return list(counter.most_common(8))


def measure(tag: str) -> dict:
    x, y, w, h = window()
    png = Path(f"/var/tmp/calix-shots/measure-{tag}.png")
    png.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["grim", "-g", f"{x},{y} {w}x{h}", str(png)], capture_output=True)
    found = edges(png)
    # Le bord du rail est la première arête de pleine hauteur ; le séparateur
    # est la suivante. Les deux sont comptées sur les mêmes lignes.
    rail_edge = max(
        (x for x, _ in found if x < RAIL + 40),
        key=lambda x: next(c for e, c in found if e == x),
        default=None,
    )
    if rail_edge is None:
        raise SystemExit(f"le bord du rail est introuvable dans la capture : {found}")
    after_rail = [(x, c) for x, c in found if x > rail_edge + 200]
    if not after_rail:
        raise SystemExit(f"le séparateur est introuvable dans la capture : {found}")
    splitter = max(after_rail, key=lambda pair: pair[1])[0]
    list_width = splitter - RAIL
    content = w - RAIL
    inspector = content - SPLITTER - list_width
    cap = min(INSPECTOR_CAP, content * INSPECTOR_RATIO)
    ok = (
        list_width >= LIST_FLOOR - 1
        and inspector >= INSPECTOR_FLOOR - 1
        and inspector <= cap + 1
    )
    return {
        "tag": tag,
        "window_px": w,
        "rail_px": RAIL,
        "splitter_px": splitter,
        "list_px": list_width,
        "inspector_px": inspector,
        "inspector_cap_px": round(cap, 1),
        "inspector_floor_px": INSPECTOR_FLOOR,
        "list_floor_px": LIST_FLOOR,
        "within_bounds": ok,
    }


if __name__ == "__main__":
    tag = sys.argv[1] if len(sys.argv) > 1 else "probe"
    result = measure(tag)
    print(json.dumps(result, indent=2))
