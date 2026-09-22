#!/usr/bin/env python3
"""RENDER-01 determinism harness: fiducial ROI location + pixel hashing.

The lab frames its confrontation box with a 1px magenta (#FF00FF) marker.
This tool locates that marker in a window capture, derives the fixed
preview ROI (marker bbox inset by 2px), and hashes the decoded normalized
RGBA buffer -- never the PNG container bytes.

Usage:
    roi_hash.py locate <capture.png>
    roi_hash.py hash <capture.png> <x,y,w,h>
    roi_hash.py compare <a.png> <b.png> <x,y,w,h>
"""
import hashlib
import json
import sys

from PIL import Image

MARKER = (255, 0, 255)
INSET = 2


def find_marker_bbox(im):
    px = im.load()
    w, h = im.size
    def is_marker(pixel):
        r, g, b = pixel[:3]
        return r > 200 and g < 80 and b > 200

    xs0, ys0 = w, h
    xs1, ys1 = -1, -1
    for y in range(h):
        for x in range(w):
            if is_marker(px[x, y]):
                if x < xs0:
                    xs0 = x
                if y < ys0:
                    ys0 = y
                if x > xs1:
                    xs1 = x
                if y > ys1:
                    ys1 = y
    if xs1 < 0:
        raise SystemExit("ERROR: fiducial marker #FF00FF not found in capture")
    top = [(x, ys0) for x in range(xs0, xs1 + 1) if is_marker(px[x, ys0])]
    bottom = [(x, ys1) for x in range(xs0, xs1 + 1) if is_marker(px[x, ys1])]
    left = [(xs0, y) for y in range(ys0, ys1 + 1) if is_marker(px[xs0, y])]
    right = [(xs1, y) for y in range(ys0, ys1 + 1) if is_marker(px[xs1, y])]
    expected_width = xs1 - xs0 + 1
    expected_height = ys1 - ys0 + 1
    if not all(
        len(edge) == expected and len(edge) > 0
        for edge, expected in (
            (top, expected_width),
            (bottom, expected_width),
            (left, expected_height),
            (right, expected_height),
        )
    ):
        raise SystemExit(
            f"ERROR: fiducial marker #FF00FF truncated bbox={xs0},{ys0},{expected_width},{expected_height}"
        )
    return (xs0 + INSET, ys0 + INSET, xs1 - xs0 + 1 - 2 * INSET, ys1 - ys0 + 1 - 2 * INSET)


def hash_roi(im, roi):
    x, y, w, h = roi
    crop = im.crop((x, y, x + w, y + h)).convert("RGBA")
    raw = crop.tobytes()
    digest = hashlib.sha256(raw).hexdigest()
    colors = crop.getcolors(maxcolors=1 << 24)
    return {
        "roi": {"x": x, "y": y, "w": w, "h": h},
        "sha256": digest,
        "bytes": len(raw),
        "unique_colors": len(colors) if colors is not None else -1,
    }


def parse_roi(s):
    parts = [int(v) for v in s.split(",")]
    if len(parts) != 4:
        raise SystemExit("ERROR: ROI must be x,y,w,h")
    return tuple(parts)


def main(argv):
    if len(argv) < 3:
        raise SystemExit(__doc__)
    cmd = argv[1]
    if cmd == "locate":
        im = Image.open(argv[2]).convert("RGBA")
        x, y, w, h = find_marker_bbox(im)
        print(json.dumps({"x": x, "y": y, "w": w, "h": h}))
    elif cmd == "hash":
        im = Image.open(argv[2]).convert("RGBA")
        print(json.dumps(hash_roi(im, parse_roi(argv[3])), indent=2))
    elif cmd == "compare":
        a = Image.open(argv[2]).convert("RGBA")
        b = Image.open(argv[3]).convert("RGBA")
        roi = parse_roi(argv[4])
        ha = hash_roi(a, roi)
        hb = hash_roi(b, roi)
        print(json.dumps({
            "roi": ha["roi"],
            "a": ha["sha256"],
            "b": hb["sha256"],
            "identical": ha["sha256"] == hb["sha256"],
        }, indent=2))
        if ha["sha256"] != hb["sha256"]:
            raise SystemExit(1)
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main(sys.argv)
