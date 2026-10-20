#!/usr/bin/env python3
"""Record the pre-Slice-08 UI state on a live Wayland session.

The Slice-07 contract asks Slice-09 to prove the navigation contract with runtime
captures.  A "before" has to exist before the change, or the comparison is
worthless, so this records the state of the application as it is at the
contract commit: one capture per place and per scope, each paired with the
control surface's own status document.

Usage:  capture.py <output-dir>

The output directory receives one PNG per state, one JSON per state (the
control surface's status), and a run.json describing the whole session.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

APP_CLASS = "live.gnu6.calix"
# The desktop dock floats over the bottom of the tiled window; the captures
# crop it away so the frames can be compared pixel for pixel later.
DOCK_OVERLAY_PX = 70

# name, control command, seconds to wait for the surface to settle
STATES = [
    ("browse-empty", ["calix-gpui", "navigate", "browse"], 3.0),
    ("browse-results", ["calix-gpui", "search", "ripgrep"], 4.0),
    ("installed", ["calix-gpui", "navigate", "installed"], 6.0),
    ("updates", ["calix-gpui", "navigate", "updates"], 5.0),
    ("news", ["calix-gpui", "navigate", "news"], 5.0),
    ("settings", ["calix-gpui", "navigate", "settings"], 3.0),
]


def run(cmd: list[str]) -> str:
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
    return (proc.stdout or "") + (proc.stderr or "")


def window_geometry() -> tuple[int, int, int, int]:
    """The global geometry of the application window, read from the compositor."""
    out = run(["hyprctl", "clients", "-j"])
    clients = json.loads(out)
    for client in clients:
        if (client.get("class") or "") == APP_CLASS:
            at = client["at"]
            size = client["size"]
            return int(at[0]), int(at[1]), int(size[0]), int(size[1])
    raise SystemExit("the application window is not mapped")


def capture(png: Path) -> dict:
    x, y, w, h = window_geometry()
    region = f"{x},{y} {w}x{h}"
    run(["grim", "-g", region, str(png)])
    if not png.exists():
        raise SystemExit(f"grim did not write {png}")
    # Crop the dock overlay off the bottom.
    cropped = png.with_name(png.stem + "-crop.png")
    run(["magick", str(png), "-crop", f"{w}x{h - DOCK_OVERLAY_PX}+0+0", "+repage", str(cropped)])
    cropped.replace(png)
    return {
        "file": png.name,
        "region": region,
        "cropped_px": {"bottom": DOCK_OVERLAY_PX},
        "bytes": png.stat().st_size,
        "sha256": hashlib.sha256(png.read_bytes()).hexdigest(),
    }


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: capture.py <output-dir>", file=sys.stderr)
        return 2
    out = Path(sys.argv[1]).resolve()
    out.mkdir(parents=True, exist_ok=True)

    started = datetime.now(timezone.utc).isoformat()
    captures: list[dict] = []
    statuses: list[dict] = []

    for name, command, settle in STATES:
        run(command)
        time.sleep(settle)
        status = run(["calix-gpui", "status", "--json"])
        (out / f"{name}.json").write_text(status)
        artifact = capture(out / f"{name}.png")
        artifact["state"] = name
        artifact["command"] = " ".join(command)
        artifact["settle_seconds"] = settle
        captures.append(artifact)
        print(f"captured {name}: {artifact['region']} -> {artifact['file']}", flush=True)

    run(["calix-gpui", "quit"])
    time.sleep(1.0)

    record = {
        "run_id": out.name,
        "kind": "evidence",
        "status": "ok",
        "started_at": started,
        "finished_at": datetime.now(timezone.utc).isoformat(),
        "objective": (
            "Record the state of the navigation before the Slice-08 implementation, "
            "so that the acceptance matrix of Slice-07 can be closed with a before/after "
            "comparison instead of a claim."
        ),
        "method": {
            "compositor": "Hyprland",
            "capture": "grim, on the window geometry read from `hyprctl clients -j`",
            "crop": f"the bottom {DOCK_OVERLAY_PX}px are cropped: the desktop dock floats over the window",
            "driver": "the control surface (`calix-gpui navigate`, `calix-gpui search`)",
            "pairing": "each capture is stored next to the control surface's status document for the same state",
        },
        "captures": captures,
        "status_documents": [f"{c['state']}.json" for c in captures],
    }
    (out / "run.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"run record: {out / 'run.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
