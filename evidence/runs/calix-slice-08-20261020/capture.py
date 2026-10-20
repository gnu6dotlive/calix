#!/usr/bin/env python3
"""Record the Slice-08 surfaces on a live Wayland session.

One capture per scope and per place, each paired with the control surface's own
answer, so the picture and the machine-readable state can be read together.

Usage: capture.py <output-dir> [binary]
"""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

APP_CLASS = "live.gnu6.calix"


def hypr(*args: str) -> str:
    return subprocess.run(["hyprctl", *args], capture_output=True, text=True).stdout


def window() -> tuple[int, int, int, int]:
    for client in json.loads(hypr("clients", "-j")):
        if (client.get("class") or "") == APP_CLASS:
            at, size = client["at"], client["size"]
            return int(at[0]), int(at[1]), int(size[0]), int(size[1])
    raise SystemExit("the application window is not mapped")


def shot(path: Path) -> dict:
    x, y, w, h = window()
    subprocess.run(["grim", "-g", f"{x},{y} {w}x{h}", str(path)], capture_output=True)
    if not path.exists():
        raise SystemExit(f"grim did not write {path}")
    return {"file": path.name, "region": f"{x},{y} {w}x{h}", "bytes": path.stat().st_size}


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: capture.py <output-dir> [binary]", file=sys.stderr)
        return 2
    out = Path(sys.argv[1]).resolve()
    out.mkdir(parents=True, exist_ok=True)
    binary = sys.argv[2] if len(sys.argv) > 2 else "calix-gpui"

    captures = []
    for name, target in [
        ("scope-results", "results"),
        ("scope-installed", "installed"),
        ("scope-updates", "updates"),
        ("place-news", "news"),
        ("place-settings", "settings"),
    ]:
        subprocess.run([binary, "navigate", target], capture_output=True)
        time.sleep(3.5)
        status = subprocess.run([binary, "status", "--json"], capture_output=True, text=True)
        (out / f"{name}.json").write_text(status.stdout)
        captures.append({"state": name, "target": target, **shot(out / f"{name}.png")})
        print(f"captured {name}", flush=True)

    record = {
        "run_id": out.name,
        "kind": "evidence",
        "objective": "Record the Slice-08 surfaces: the context header, the query surface in every scope, and the places.",
        "method": {
            "compositor": "Hyprland",
            "capture": "grim, on the window geometry read from `hyprctl clients -j`",
            "driver": "the control surface (`calix-gpui navigate`, `calix-gpui status --json`)",
            "pairing": "each capture is stored next to the control surface's status document",
        },
        "captures": captures,
    }
    (out / "run.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"run record: {out / 'run.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
