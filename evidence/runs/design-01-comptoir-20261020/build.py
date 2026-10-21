#!/usr/bin/env python3
"""Le Comptoir — a Calix package manager proposed as a shop counter at night.

Builds the FigCraft document `01-comptoir.figcraft` from a command list.
Run:  python3 01-comptoir.py && figcraft-cli run 01-comptoir.jsonl --out ../01-comptoir.figcraft

Nothing here invents a colour: every fill is a design-system token, and the
warmth of the wood and the blackened brass is produced by compositing the
signal-orange token over the anthracite ground with alpha.
"""

import json
import math
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
# the command log is written beside this file, under this file's own name
OUT_JSONL = os.path.join(HERE, os.path.splitext(os.path.basename(__file__))[0] + ".jsonl")

# ---------------------------------------------------------------- tokens ----
ANTH = "#111418"   # anthracite, the ground
SHELL = "#F7F3ED"  # shell white, the ink
MUTED = "#A1A6AD"  # secondary text
SLATE = "#667085"  # hairlines, the stem
O100 = "#FFE5D2"   # signal orange 100, rim light
O300 = "#FF8E40"   # signal orange 300, hover
ORANGE = "#FF6A00"  # signal orange, the accent
O700 = "#C95400"   # signal orange 700, the shadow side
B300 = "#87A0F4"
BLUE = "#4169E1"

# ----------------------------------------------------------------- fonts ----
GROTESK = "IBM Plex Sans"
GROTESK_SB = "IBM Plex Sans SemiBold"
COND = "IBM Plex Sans Condensed"
COND_SB = "IBM Plex Sans Condensed SemiBold"
MONO = "IBM Plex Mono"

# ----------------------------------------------------------------- layout ----
W, H = 1600, 1000
HDR = 96
MAIN_T, MAIN_B = 110, 872
CON_T = 886

L_X, L_W = 24, 300     # the wall
C_X, C_W = 342, 848    # the counter
R_X, R_W = 1208, 368   # the till and the detail board

steps = []


def emit(cmd, params):
    steps.append({"command": cmd, "params": params})
    return len(steps) - 1


def R(i):
    return {"$step": i}


def alpha(hexcol, a):
    """#RRGGBB + an alpha byte -> #RRGGBBAA (token colours only)."""
    return "%s%02X" % (hexcol, a)


def wood(a=0x18, b=0x05, angle=None):
    """Warm dark wood: the anthracite ground with the signal-orange token
    laid over it.  The first paint is opaque, so a shelf is a solid board."""
    if angle is None:
        frm, to = [0, 0], [0, 1]
    else:
        frm, to = angle
    return [ANTH,
            {"type": "linearGradient", "stops": [alpha(ORANGE, a), alpha(ORANGE, b)],
             "from": frm, "to": to}]


def grad(stops, frm, to):
    return [{"type": "linearGradient", "stops": stops, "from": frm, "to": to}]


def rect(x, y, w, h, fills=None, parent=None, **props):
    p = {}
    if fills is not None:
        p["fills"] = fills
    p.update(props)
    params = {"x": x, "y": y, "width": w, "height": h, "props": p}
    if parent is not None:
        params["parent"] = parent
    return emit("shape.rectangle", params)


def ellipse(x, y, w, h, fills=None, parent=None, **props):
    p = {}
    if fills is not None:
        p["fills"] = fills
    p.update(props)
    params = {"x": x, "y": y, "width": w, "height": h, "props": p}
    if parent is not None:
        params["parent"] = parent
    return emit("shape.ellipse", params)


def poly(x, y, w, h, parent=None, **props):
    p = {}
    p.update(props)
    params = {"x": x, "y": y, "width": w, "height": h, "props": p}
    if parent is not None:
        params["parent"] = parent
    return emit("shape.polygon", params)


def vec(path, parent=None, **props):
    p = {}
    p.update(props)
    params = {"path": path, "x": 0, "y": 0, "props": p}
    if parent is not None:
        params["parent"] = parent
    return emit("vector.create", params)


def txt(s, x, y, size=13, family=GROTESK, fill=SHELL, parent=None, width=None,
        align=None, ls=None, style=None, opacity=None, lh=None, caps=None, **extra):
    p = {"fontSize": size, "fontFamily": family}
    p["fills"] = [fill] if isinstance(fill, str) else fill
    if align is not None:
        p["textAlign"] = align
    if ls is not None:
        p["letterSpacing"] = ls
    if style is not None:
        p["fontStyle"] = style
    if opacity is not None:
        p["opacity"] = opacity
    if lh is not None:
        p["lineHeight"] = lh
    if caps is not None:
        p["textCase"] = caps
    p.update(extra)
    params = {"text": s, "x": x, "y": y, "props": p}
    if width is not None:
        params["width"] = width
    if parent is not None:
        params["parent"] = parent
    return emit("text.create", params)


def rtxt(s, right_x, y, size=13, family=MONO, fill=MUTED, parent=None, **extra):
    """Right-aligned text: a box whose right edge is right_x."""
    w = 460
    return txt(s, right_x - w, y, size, family, fill, parent, width=w,
               align="right", **extra)


def ctxt(s, centre_x, y, size=13, family=MONO, fill=MUTED, parent=None, **extra):
    w = 460
    return txt(s, centre_x - w / 2, y, size, family, fill, parent, width=w,
               align="center", **extra)


def shadow(color="#000000A6", off=(0, 8), radius=16, spread=0):
    return [{"type": "dropShadow", "color": color, "offset": list(off),
             "radius": radius, "spread": spread}]


def inner(color="#000000A6", off=(0, 4), radius=10, spread=0):
    return [{"type": "innerShadow", "color": color, "offset": list(off),
             "radius": radius, "spread": spread}]


def noise(size=1.0, density=0.5, color=SLATE, secondary=None, opacity=None,
          blend="overlay"):
    """Fine grain.  The engine paints grains at full strength, so the grain
    colour has to sit close to the surface it lies on: slate over the warm
    wood, muted over the aluminium."""
    e = {"type": "noise", "size": size, "density": density, "color": color,
         "blend": blend}
    if opacity is not None:
        e["opacity"] = opacity
    if secondary:
        e["mode"] = "duo"
        e["secondary"] = secondary
    return [e]


# =========================================================== the identity ====
# A scallop shell as the cup of a chalice: a fan of ribs sitting in a metal
# cup on a brushed aluminium stem.  Drawn in a 120 x 140 local box.

def mark_geometry():
    cx, cy, rx, ry = 60.0, 52.0, 48.0, 38.0
    n = 7
    depth = 9.0

    def pt(t):
        return (cx + rx * math.cos(t), cy - ry * math.sin(t))

    pts = [pt(math.pi * i / n) for i in range(n + 1)]
    crests = []
    for i in range(n):
        tm = math.pi * (i + 0.5) / n
        p = pt(tm)
        crests.append((p[0] + depth * math.cos(tm), p[1] - depth * math.sin(tm)))

    # the fan: a scalloped margin over the top, flanks down to the hinge
    d = ["M %.2f %.2f" % pts[0]]
    for i in range(n):
        c = crests[i]
        e = pts[i + 1]
        d.append("Q %.2f %.2f %.2f %.2f" % (c[0], c[1], e[0], e[1]))
    d.append("C 24.00 74.00 42.00 88.00 60.00 92.00")
    d.append("C 78.00 88.00 96.00 74.00 %.2f %.2f" % pts[0])
    d.append("Z")
    shell = " ".join(d)

    # the ribs, from the hinge to just inside the margin
    ribs = []
    for i in range(n):
        tm = math.pi * (i + 0.5) / n
        p = pt(tm)
        ribs.append("M 59.00 89.00 L %.2f %.2f" % (
            p[0] + 0.9 * math.cos(tm), p[1] - 0.9 * math.sin(tm)))
    rib = " ".join(ribs)

    # a rim light along the upper-left of the margin
    hl = []
    for i in range(n // 2, n):
        tm = math.pi * (i + 0.5) / n
        p = pt(tm)
        hl.append("M %.2f %.2f L %.2f %.2f" % (
            p[0] + 4.3 * math.cos(tm), p[1] - 4.3 * math.sin(tm),
            p[0] + 1.2 * math.cos(tm), p[1] - 1.2 * math.sin(tm)))

    cup = ("M 11.00 64.00 "
           "C 11.00 71.50 34.00 75.50 60.00 75.50 "
           "C 86.00 75.50 109.00 71.50 109.00 64.00 "
           "C 109.00 81.00 87.00 94.00 60.00 96.00 "
           "C 33.00 94.00 11.00 81.00 11.00 64.00 Z")
    lip = ("M 11.00 64.00 C 11.00 71.50 34.00 75.50 60.00 75.50 "
           "C 86.00 75.50 109.00 71.50 109.00 64.00")
    cupshade = ("M 13.00 60.00 C 13.00 67.50 35.00 71.50 60.00 71.50 "
                "C 85.00 71.50 107.00 67.50 107.00 60.00")
    knop = ("M 46.00 92.00 C 46.00 99.00 51.00 102.00 60.00 102.00 "
            "C 69.00 102.00 74.00 99.00 74.00 92.00 Z")
    shaft = "M 53.00 99.00 L 67.00 99.00 L 68.00 127.00 L 52.00 127.00 Z"
    foot = ("M 44.00 125.00 L 76.00 125.00 "
            "C 78.50 125.00 78.50 137.00 76.00 137.00 L 44.00 137.00 "
            "C 41.50 137.00 41.50 125.00 44.00 125.00 Z")
    return {"shell": shell, "ribs": rib, "hl": " ".join(hl), "cup": cup,
            "lip": lip, "cupshade": cupshade, "knop": knop, "shaft": shaft,
            "foot": foot}


MARK = mark_geometry()
_NUM = re.compile(r"-?\d+(?:\.\d+)?")


def mark(parent, x, y, height, style="lit", dx=0.0, dy=0.0):
    """Draw the mark with its top-left at (x, y), `height` tall.

    style 'lit'    — the product mark: orange shell, aluminium stem.
    style 'engrave'— the same silhouette cut into a wooden back panel.
    """
    s = height / 140.0
    ox, oy = x + dx, y + dy

    def path_of(key):
        nums = _NUM.sub(lambda m: "%.2f" % (float(m.group(0)) * s), MARK[key])
        out = []
        for tok in nums.split():
            if tok in ("M", "C", "Q", "L", "Z"):
                out.append(tok)
            else:
                out.append(float(tok))
        res = []
        k = 0
        while k < len(out):
            t = out[k]
            if isinstance(t, str):
                res.append(t)
                k += 1
                continue
            res.append("%.2f %.2f" % (t + ox, out[k + 1] + oy))
            k += 2
        return " ".join(res)

    if style == "lit":
        vec(path_of("shell"), parent,
            fills=grad([O300, ORANGE, O700], [0.12, 0.0], [0.68, 1.0]),
            name="calix-shell")
        vec(path_of("ribs"), parent, fills=[],
            strokes=[alpha(O700, 0xE6)], strokeWeight=4.2 * s,
            strokeCap="round", name="calix-ribs")
        vec(path_of("hl"), parent, fills=[],
            strokes=[alpha(O100, 0xB0)], strokeWeight=2.4 * s,
            strokeCap="round", name="calix-rimlight")
        # the cup's shadow falling on the shell
        vec(path_of("cupshade"), parent, fills=[],
            strokes=[alpha(ANTH, 0x55)], strokeWeight=3.0 * s,
            strokeCap="round", name="calix-cup-shadow")
        vec(path_of("cup"), parent,
            fills=grad([MUTED, SLATE, ANTH], [0.0, 0.0], [1.0, 0.35]),
            name="calix-cup")
        vec(path_of("lip"), parent, fills=[],
            strokes=[alpha(O100, 0x96)], strokeWeight=2.2 * s,
            strokeCap="round", name="calix-lip")
        for key in ("knop", "shaft", "foot"):
            vec(path_of(key), parent,
                fills=grad([SLATE, MUTED, SLATE, ANTH], [0.0, 0.0], [1.0, 0.0]),
                name="calix-stem")
    else:
        vec(path_of("shell"), parent, fills=[],
            strokes=[alpha(O300, 0x30)], strokeWeight=2.0 * s,
            strokeJoin="round", name="calix-shell-engraved")
        vec(path_of("ribs"), parent, fills=[],
            strokes=[alpha(O300, 0x18)], strokeWeight=1.6 * s,
            strokeCap="round", name="calix-ribs-engraved")
        vec(path_of("cup"), parent, fills=[],
            strokes=[alpha(O300, 0x30)], strokeWeight=2.0 * s,
            strokeJoin="round", name="calix-cup-engraved")
        for key in ("knop", "shaft", "foot"):
            vec(path_of(key), parent, fills=[],
                strokes=[alpha(O300, 0x30)], strokeWeight=2.0 * s,
                strokeJoin="round", name="calix-stem-engraved")


# ============================================================== the data =====
SOURCES = [
    ("ALPM", "official binaries", "2,481", "⌘1"),
    ("AUR", "build recipes", "61,204", "⌘2"),
    ("FLATPAK", "sandboxes", "187", "⌘3"),
    ("APPIMAGE", "single binaries", "42", "⌘4"),
    ("CARGO", "crates", "9,318", "⌘5"),
    ("NPM", "node packages", "3,706", "⌥↓"),
    ("TRACKED RELEASES", "git releases", "27", "⌥↓"),
]

PACKAGES = [
    ("ripgrep", "14.1.0", "14.1.1", "core · x86_64", "+1.2 MiB", "OUTDATED",
     "Fast recursive grep that respects .gitignore."),
    ("linux", "6.11.5", "6.12.1", "core · x86_64", "+124 MiB", "OUTDATED",
     "The Linux kernel and modules."),
    ("ffmpeg", "7.1", "8.0", "extra · x86_64", "+37.8 MiB", "OUTDATED",
     "Tools for transcoding, streaming and playback of multimedia."),
    ("python-requests", "2.32.3", "2.33.0", "extra · any", "+84 KiB", "OUTDATED",
     "Python HTTP for Humans."),
    ("postgresql", "17.4", "17.5", "extra · x86_64", "+4.1 MiB", "OUTDATED",
     "Object-relational database system."),
    ("nvidia-dkms", "550.144", "555.58", "multilib · x86_64", "+92 MiB", "HELD",
     "NVIDIA driver module built for the running kernel."),
    ("foot", "0.16.2", "0.17.0", "extra · x86_64", "+612 KiB", "OUTDATED",
     "A fast GPU-driven terminal emulator."),
    ("alacritty", "0.13.2", "0.14.0", "extra · x86_64", "+1.8 MiB", "OUTDATED",
     "GPU-accelerated terminal emulator."),
]

SELECTED = 2  # ffmpeg

DEPS = ["gmp", "libvpx", "x264", "dav1d", "rubberband", "sdl2", "speex", "zimg"]

KEYS = [
    ("⌘1 – ⌘5", "jump to a source"),
    ("⌥↑  ⌥↓", "walk the wall"),
    ("⌥←  ⌥→", "move the till"),
    ("⌘F", "filter the shelf"),
    ("⌘L", "list density"),
    ("⌘⌥C", "copy the command"),
    ("esc", "back / cancel"),
]


# ================================================================ the build ==
root = R(emit("frame.create", {
    "name": "Calix — Le Comptoir",
    "x": 0, "y": 0, "width": W, "height": H,
    "props": {"fills": [ANTH], "clip": True},
}))
emit("page.setBackground", {"color": ANTH})

# the ground: a warm pool of lamp light over the whole shop
rect(0, 0, W, H, [{"type": "radialGradient",
                   "stops": [alpha(ORANGE, 0x0C), alpha(ANTH, 0x00)],
                   "from": [0.34, 0.02], "to": [0.42, 0.95],
                   "width": [1.9, 1.5]}], root)

# ================================================================ the fascia =
rect(0, 0, W, HDR, wood(0x1A, 0x04), root, effects=noise(1.0, 0.50, SLATE))
rect(0, 0, W, HDR, grad([alpha(O100, 0x0C), alpha(ANTH, 0x00)], [0, 0], [0, 1]), root)
rect(0, HDR - 3, W, 3, [alpha(O700, 0x60)], root)
rect(0, HDR - 4, W, 1, [alpha(O300, 0x38)], root)

mark(root, 30, 6, 84, "lit")

txt("CALIX", 116, 24, 34, GROTESK_SB, SHELL, root, ls=5)
rect(116, 64, 118, 1, [alpha(O300, 0x40)], root)


def header_plate(x, w, label, value, value_fill=SHELL):
    rect(x, 24, w, 48, [alpha(ANTH, 0x66)], root, cornerRadius=4,
         strokes=[alpha(SLATE, 0x55)], strokeWeight=1)
    txt(label, x + 12, 30, 8.5, COND, alpha(MUTED, 0xAA), root, ls=2.0)
    txt(value, x + 12, 44, 17, COND_SB, value_fill, root, ls=2.0)


header_plate(288, 104, "SOURCE", "ALPM", ORANGE)
for i, (tag, tw) in enumerate([("core", 50), ("extra", 58), ("multilib", 68)]):
    tx = 404 + sum([50, 58][:i]) + i * 8
    rect(tx, 34, tw, 24, [alpha(ORANGE, 0x14)], root, cornerRadius=12,
         strokes=[alpha(ORANGE, 0x4A)], strokeWeight=1)
    ctxt(tag, tx + tw / 2, 40, 10, COND, O300, root, ls=1.2)
header_plate(600, 128, "OPERATION", "UPDATES")
header_plate(740, 118, "FILTER", "outdated")
txt("8 shown · 812 in ALPM", 876, 44, 12, MONO, alpha(SHELL, 0xC0), root)

# the other list, shown as a possibility
rect(1128, 24, 1, 48, [alpha(SLATE, 0x66)], root)
txt("ROWS", 1150, 27, 9, COND, MUTED, root, ls=1.8)
rect(1150, 41, 28, 5, [ORANGE], root, cornerRadius=1)
rect(1150, 49, 17, 4, [alpha(ORANGE, 0x77)], root, cornerRadius=1)
txt("detailed", 1186, 38, 10.5, COND, SHELL, root, ls=1.2)
rect(1248, 41, 28, 5, [alpha(SLATE, 0x99)], root, cornerRadius=1)
txt("compact", 1284, 38, 10.5, COND, MUTED, root, ls=1.2)
txt("⌘L", 1338, 38, 10.5, MONO, alpha(MUTED, 0xC0), root)

# how fresh the answer is
rect(1384, 24, 1, 48, [alpha(SLATE, 0x66)], root)
rtxt("SYNCED", 1576, 27, 9, COND, MUTED, root, ls=1.8)
rtxt("4 MIN AGO", 1576, 41, 12, MONO, SHELL, root)
ellipse(1494, 45, 7, 7, [ORANGE], root, effects=shadow(alpha(ORANGE, 0x99), (0, 0), 6))

# ========================================================== the wall of drawers
rect(L_X, MAIN_T, L_W, MAIN_B - MAIN_T, wood(0x12, 0x04), root,
     effects=noise(1.0, 0.50, SLATE))
rect(L_X, MAIN_T, L_W, 2, [alpha(O300, 0x28)], root)
rect(L_X, MAIN_B - 2, L_W, 2, [alpha(ANTH, 0x99)], root)

txt("SOURCES", L_X + 12, MAIN_T + 12, 10, COND, MUTED, root, ls=2.2)
txt("⌥↑ ⌥↓", L_X + 210, MAIN_T + 12, 11, GROTESK, alpha(MUTED, 0xB0), root)

FRONT_H = 62
PITCH = 70


def drawer(y, name, desc, count, key, pulled=False, dx=0):
    x = L_X + 6 + dx
    w = 270
    if not pulled:
        rect(L_X + 6, y - 3, w, FRONT_H + 6, [alpha(ANTH, 0x88)], root,
             cornerRadius=3)
    rect(x, y, w, FRONT_H, wood(0x24, 0x08), root, cornerRadius=3,
         effects=shadow("#000000C0" if pulled else "#000000A0",
                        (3 if pulled else 0, 11 if pulled else 4),
                        20 if pulled else 8))
    rect(x, y, w, 1, [alpha(O300, 0x30)], root)
    rect(x, y + FRONT_H - 1, w, 1, [alpha(ANTH, 0xB0)], root)
    rect(x + 6, y + 6, w - 12, FRONT_H - 12, [alpha(ANTH, 0x38)], root,
         cornerRadius=2, strokes=[alpha(ORANGE, 0x1E)], strokeWeight=1)
    txt(name, x + 18, y + 11, 13, COND_SB, SHELL, root, ls=1.8,
        opacity=1.0 if pulled else 0.78)
    txt(desc, x + 18, y + 34, 10, COND, MUTED, root, ls=0.6,
        opacity=1.0 if pulled else 0.75)
    rtxt(count, x + w - 24, y + 12, 12.5, MONO, SHELL if pulled else MUTED, root)
    rtxt(key, x + w - 24, y + 34, 9.5, MONO,
         ORANGE if pulled else alpha(MUTED, 0x99), root)
    # the blackened-brass pull
    rect(x + w - 16, y + 19, 8, 24, grad([alpha(O700, 0xD0), alpha(ANTH, 0xFF)],
                                         [0, 0], [1, 0]), root,
         cornerRadius=4, effects=shadow("#000000A0", (1, 3), 5))
    rect(x + w - 16, y + 19, 1, 24, [alpha(O300, 0x40)], root, cornerRadius=1)


# The pulled drawer shows its inside: the source's own compartments, one dense
# line each — the compact list, glimpsed in the drawer.  The drawer is drawn
# last, so it sits in front of the drawer below it.
for i, (name, desc, count, key) in enumerate(SOURCES[1:]):
    drawer(282 + i * PITCH, name, desc, count, key)

# the open box of the pulled drawer, seen from above
vec("M 38.00 212.00 L 50.00 166.00 L 296.00 166.00 L 308.00 212.00 Z", root,
    fills=grad([alpha(O700, 0x16), alpha(ANTH, 0xFF)], [0, 0], [0, 1]),
    name="drawer-interior")
rect(50, 165, 246, 2, [alpha(O700, 0xAA)], root)
rect(50, 167, 246, 1, [alpha(O100, 0x44)], root)
txt("core 2,103   extra 361   multilib 17", 64, 176, 9.5, MONO, MUTED, root)
# the inner side walls
vec("M 38.00 212.00 L 50.00 166.00 L 50.00 184.00 L 44.00 212.00 Z", root,
    fills=[alpha(ANTH, 0xDD)], name="drawer-wall-l")
vec("M 308.00 212.00 L 296.00 166.00 L 296.00 184.00 L 302.00 212.00 Z", root,
    fills=[alpha(ANTH, 0xDD)], name="drawer-wall-r")

drawer(212, "ALPM", "official binaries", "2,481", "⌘1", pulled=True, dx=8)

# the keymap, engraved on a brass plate
KEY_Y = 712
rect(L_X + 6, KEY_Y, 270, MAIN_B - KEY_Y, grad([alpha(O700, 0x2E), alpha(O700, 0x10)],
                                               [0, 0], [0, 1]), root, cornerRadius=4,
     strokes=[alpha(O700, 0x70)], strokeWeight=1, effects=shadow("#000000A0", (0, 6), 12))
rect(L_X + 8, KEY_Y + 2, 266, 1, [alpha(O300, 0x44)], root)
txt("THE KEYS", L_X + 20, KEY_Y + 10, 9, COND, alpha(O300, 0xCC), root, ls=2.0)
for i, (k, label) in enumerate(KEYS):
    yy = KEY_Y + 28 + i * 15
    txt(k, L_X + 20, yy, 10.5, GROTESK, O300, root)
    txt(label, L_X + 116, yy, 10, COND, MUTED, root)

# ============================================================ the counter ====
rect(C_X, MAIN_T, C_W, 50, wood(0x1E, 0x06), root, cornerRadius=4,
     effects=noise(1.0, 0.50, SLATE))
rect(C_X, MAIN_T, C_W, 1, [alpha(O300, 0x46)], root)
rect(C_X, MAIN_T + 49, C_W, 1, [alpha(ANTH, 0xAA)], root)

rect(C_X + 16, MAIN_T + 12, 270, 26, [alpha(ANTH, 0xDD)], root, cornerRadius=4,
     effects=inner("#000000B0", (0, 3), 8))
txt("⌘F", C_X + 26, MAIN_T + 19, 11, MONO, ORANGE, root)
txt("outdated", C_X + 64, MAIN_T + 17, 12, MONO, SHELL, root)
txt("×", C_X + 254, MAIN_T + 18, 13, GROTESK, MUTED, root)
txt("FOR  ALPM · UPDATES", C_X + 312, MAIN_T + 20, 9.5, COND, MUTED, root, ls=1.8)


def brass_button(x, y, w, label, primary=False, key=None):
    h = 26
    if primary:
        rect(x, y, w, h, grad([O300, ORANGE], [0, 0], [0, 1]), root, cornerRadius=4,
             effects=shadow(alpha(O700, 0xA0), (0, 4), 8))
        txt(label, x + 12, y + 8, 10.5, COND_SB, SHELL, root, ls=1.4)
    else:
        rect(x, y, w, h, [alpha(O700, 0x1C)], root, cornerRadius=4,
             strokes=[alpha(O700, 0x88)], strokeWeight=1)
        txt(label, x + 12, y + 8, 10.5, COND, O300, root, ls=1.4)
    if key:
        txt(key, x + w - 26, y + 8, 9.5, MONO, alpha(O300, 0xA0), root)


brass_button(C_X + 512, MAIN_T + 12, 118, "UPDATE ALL · 8", primary=True)
brass_button(C_X + 640, MAIN_T + 12, 58, "PIN")
brass_button(C_X + 708, MAIN_T + 12, 96, "CHANGELOG")
brass_button(C_X + 814, MAIN_T + 12, 34, "…")

# ============================================================== the case =====
CASE_T, CASE_B = 174, 440
rect(C_X, CASE_T, C_W, CASE_B - CASE_T, wood(0x16, 0x05), root,
     effects=noise(1.0, 0.50, SLATE))
rect(C_X, CASE_T, C_W, CASE_B - CASE_T, [
    {"type": "radialGradient", "stops": [alpha(O300, 0x16), alpha(ORANGE, 0x00)],
     "from": [0.26, 0.0], "to": [0.32, 1.05], "width": [1.5, 1.2]}], root)
rect(C_X, CASE_T, C_W, 3, [alpha(O700, 0x60)], root)
rect(C_X, CASE_T + 3, C_W, 1, [alpha(O100, 0x30)], root)

# the shop's crest, cut into the back panel
mark(root, C_X + 612, 182, 206, "engrave")

# --- the two shelves still in the case --------------------------------------
def dim_shelf(y, label):
    for dx, dw, dh in [(0, 128, 15), (140, 88, 11), (240, 104, 14), (356, 72, 9),
                       (440, 92, 12)]:
        rect(C_X + 22 + dx, y - 4 - dh, dw, dh, [alpha(ORANGE, 0x16)], root,
             cornerRadius=2, effects=shadow("#00000088", (0, 5), 8))
        rect(C_X + 22 + dx, y - 4 - dh, dw, 1, [alpha(O300, 0x28)], root)
    rect(C_X, y, 560, 18, wood(0x28, 0x0A), root,
         effects=shadow("#000000A0", (0, 7), 14))
    rect(C_X, y, 560, 1, [alpha(O300, 0x38)], root)
    rect(C_X, y + 16, 560, 2, [alpha(O300, 0x40)], root)
    txt(label, C_X + 14, y + 5, 9, COND, alpha(MUTED, 0xAA), root, ls=2.0)


dim_shelf(214, "SEARCH")
dim_shelf(292, "INSTALLED")

# --- the empty slot the updates shelf was pulled from ------------------------
rect(C_X, 316, 560, 70, [alpha(ANTH, 0xC0)], root,
     effects=inner("#000000C0", (0, 9), 20))
rect(C_X + 10, 322, 540, 1, [alpha(O700, 0x55)], root)
rect(C_X + 10, 380, 540, 1, [alpha(O700, 0x55)], root)
txt("SHELF OUT", C_X + 560 - 92, 348, 9, COND, alpha(MUTED, 0x99), root, ls=2.0)

# ====================================================== the pulled-out shelf =
SHELF_T, SHELF_B = 386, MAIN_B
# the shelf's own shadow, thrown up onto the back panel
rect(C_X - 8, SHELF_T - 18, C_W + 16, 280, [alpha(ANTH, 0xAA)], root,
     effects=shadow("#000000C8", (0, -14), 28))
# the shelf's back board
rect(C_X, SHELF_T, C_W, SHELF_B - SHELF_T, wood(0x14, 0x04), root)
rect(C_X, SHELF_T, C_W, 130, grad([alpha(O300, 0x10), alpha(ORANGE, 0x00)],
                                   [0, 0], [0, 1]), root)
rect(C_X, SHELF_T, C_W, 2, [alpha(O300, 0x44)], root)
# the shelf's own thickness, its two ends
rect(C_X, SHELF_T, 3, SHELF_B - SHELF_T, [alpha(O700, 0x44)], root)
rect(C_X + C_W - 3, SHELF_T, 3, SHELF_B - SHELF_T, [alpha(ANTH, 0xCC)], root)

ROW_H = 56
ROW_TOP = SHELF_T


def package_row(y, p, selected):
    name, old, new, prov, size, state, desc = p
    name_col = SHELL if selected else alpha(SHELL, 0xC4)
    if selected:
        rect(C_X + 8, y - 2, C_W - 16, ROW_H - 2, [alpha(ORANGE, 0x16)], root,
             cornerRadius=4, effects=shadow("#000000C8", (0, 10), 22))
        rect(C_X + 8, y - 2, C_W - 16, 1, [alpha(O300, 0x36)], root, cornerRadius=2)
        rect(C_X + 8, y - 2, 3, ROW_H - 2, [ORANGE], root, cornerRadius=2,
             effects=shadow(alpha(ORANGE, 0x99), (0, 0), 8))
    else:
        rect(C_X + 16, y + ROW_H - 1, C_W - 32, 1, [alpha(SLATE, 0x30)], root)
    lift = 2 if selected else 0
    txt(name, C_X + 24, y + 11 - lift, 16, GROTESK_SB, name_col, root)
    txt(old, C_X + 258, y + 13 - lift, 12.5, MONO, alpha(MUTED, 0xAA), root)
    txt("→", C_X + 316, y + 13 - lift, 12.5, MONO, alpha(O300, 0xAA), root)
    txt(new, C_X + 336, y + 13 - lift, 12.5, MONO,
        O300 if selected else alpha(O300, 0xC0), root)
    txt(prov.upper(), C_X + 434, y + 14 - lift, 10.5, COND, alpha(MUTED, 0xC0),
        root, ls=1.4)
    rtxt(size, C_X + 728, y + 13 - lift, 12, MONO, alpha(MUTED, 0xC0), root)
    held = state == "HELD"
    cw = 78 if not held else 60
    chipx = C_X + C_W - 24 - cw
    rect(chipx, y + 9 - lift, cw, 20, [alpha(SLATE if held else ORANGE, 0x1E)],
         root, cornerRadius=10,
         strokes=[alpha(SLATE if held else ORANGE, 0x80)], strokeWeight=1)
    ctxt(state, chipx + cw / 2, y + 14 - lift, 9, COND_SB,
         alpha(MUTED, 0xC0) if held else O300, root, ls=1.4)
    txt(desc, C_X + 24, y + 33 - lift, 12, GROTESK, alpha(MUTED, 0xC0), root)


for i, p in enumerate(PACKAGES):
    package_row(ROW_TOP + i * ROW_H, p, i == SELECTED)

# the shadow the shelf above throws on the top of the list — a scroll still settling
rect(C_X, SHELF_T - 6, C_W, 52, grad([alpha(ANTH, 0xFF), alpha(ANTH, 0x00)],
                                     [0, 0], [0, 1]), root)

# the front rail of the shelf, in front of the goods
rect(C_X, 806, C_W, SHELF_B - 806, wood(0x26, 0x07), root,
     effects=shadow("#000000C8", (0, -12), 20))
rect(C_X, 806, C_W, 1, [alpha(O300, 0x38)], root)
rect(C_X, 806, C_W, 28, grad([alpha(O300, 0x14), alpha(ORANGE, 0x00)],
                              [0, 0], [0, 1]), root)
rect(C_X, SHELF_B - 6, C_W, 3, [alpha(O300, 0x62)], root)
rect(C_X, SHELF_B - 3, C_W, 3, [alpha(O700, 0x99)], root)
# the shelf's brass plate
rect(C_X + C_W - 16 - 214, 824, 214, 26, [alpha(O700, 0x33)], root, cornerRadius=3,
     strokes=[alpha(O700, 0x88)], strokeWeight=1)
txt("UPDATES", C_X + C_W - 202, 830, 11, COND_SB, O300, root, ls=1.8)
txt("8 OF 812", C_X + C_W - 112, 831, 10.5, MONO, alpha(MUTED, 0xC0), root)
txt("copy command  ⌘⌥C", C_X + 16, 831, 10, COND, alpha(MUTED, 0xAA), root)
rect(C_X + C_W - 16 - 230, 828, 9, 22, grad([alpha(O700, 0xD0), alpha(ANTH, 0xFF)],
                                             [0, 0], [1, 0]), root,
     cornerRadius=4, effects=shadow("#000000A0", (1, 3), 5))
rect(C_X + C_W - 16 - 230, 828, 1, 22, [alpha(O300, 0x40)], root, cornerRadius=1)

# ================================================================= the till ==
T_T, T_B = MAIN_T, 230
rect(R_X, T_T, R_W, T_B - T_T, grad([alpha(SLATE, 0xC4), alpha(SLATE, 0x88),
                                      alpha(ANTH, 0xF2)], [0, 0], [0, 1]),
     root, cornerRadius=5, effects=shadow("#000000C8", (0, 10), 22))
rect(R_X, T_T, R_W, 1, [alpha(O100, 0x66)], root)
rect(R_X, T_B - 2, R_W, 2, [alpha(O700, 0x55)], root)
rect(R_X, T_T, R_W, T_B - T_T, [alpha(MUTED, 0x08)], root, cornerRadius=5,
     effects=noise(0.8, 0.60, MUTED))
# the brush marks of a milled plate
for i in range(4):
    rect(R_X + 6, T_T + 10 + i * 26, R_W - 12, 1, [alpha(O100, 0x12)], root)

txt("OPERATION", R_X + 16, T_T + 9, 9, COND, alpha(O100, 0x99), root, ls=2.0)
rtxt("⌥←  ⌥→", R_X + R_W - 16, T_T + 9, 11, GROTESK, alpha(O100, 0x99), root)

CH_X, CH_Y, CH_W, CH_H = R_X + 16, T_T + 36, R_W - 32, 32
rect(CH_X, CH_Y, CH_W, CH_H, [alpha(ANTH, 0xEE)], root, cornerRadius=6,
     effects=inner("#000000C0", (0, 6), 14))
# the polished groove the switch has worn travelling from the first position
rect(CH_X + 52, CH_Y + 14, 234, 4, [alpha(O300, 0x1E)], root, cornerRadius=2)
for cx_ in (CH_X + 56, CH_X + 168, CH_X + 280):
    rect(cx_ - 5, CH_Y + CH_H - 7, 10, 3, [alpha(O700, 0x88)], root, cornerRadius=1)
# the lever, in the third position
LV_X, LV_Y, LV_W, LV_H = CH_X + 258, CH_Y + 3, 44, 26
rect(LV_X, LV_Y, LV_W, LV_H, grad([O300, ORANGE, O700], [0, 0], [0.2, 1]), root,
     cornerRadius=8, effects=shadow("#000000D0", (0, 7), 14))
rect(LV_X + 4, LV_Y + 3, LV_W - 8, 3, [alpha(O100, 0xAA)], root, cornerRadius=2)
for i in range(2):
    rect(LV_X + 13, LV_Y + 12 + i * 6, LV_W - 26, 2, [alpha(O700, 0x99)], root,
         cornerRadius=1)

for label, cxx in (("SEARCH", CH_X + 56), ("INSTALLED", CH_X + 168),
                   ("UPDATES", CH_X + 280)):
    active = label == "UPDATES"
    ctxt(label, cxx, CH_Y + CH_H + 14, 10, COND_SB if active else COND,
         SHELL if active else alpha(MUTED, 0xA0), root, ls=1.5)
    if active:
        rect(cxx - 24, CH_Y + CH_H + 29, 48, 2, [ORANGE], root)

# ========================================================= the detail board ===
D_T, D_B = 246, MAIN_B
rect(R_X, D_T, R_W, D_B - D_T, wood(0x14, 0x04), root,
     effects=noise(1.0, 0.50, SLATE))
rect(R_X, D_T, R_W, 1, [alpha(O300, 0x40)], root)
rect(R_X, D_T + 1, R_W, 1, [alpha(O100, 0x12)], root)
rect(R_X, D_B - 2, R_W, 2, [alpha(ANTH, 0xCC)], root)
rect(R_X, D_T, R_W, D_B - D_T, grad([alpha(O300, 0x0A), alpha(ORANGE, 0x00)],
                                    [0, 0], [0, 1]), root)

PX = R_X + 22
PWD = R_W - 44

name, old, new, prov, size, state, desc = PACKAGES[SELECTED]
txt(name, PX, D_T + 20, 26, GROTESK_SB, SHELL, root)
chipw = 82
rect(R_X + R_W - 22 - chipw, D_T + 20, chipw, 22, [alpha(ORANGE, 0x1E)], root,
     cornerRadius=11, strokes=[alpha(ORANGE, 0x80)], strokeWeight=1)
ctxt("OUTDATED", R_X + R_W - 22 - chipw / 2, D_T + 26, 9, COND_SB, O300, root, ls=1.4)
txt("extra · x86_64 · official binary", PX, D_T + 54, 11.5, MONO, MUTED, root)

rect(PX, D_T + 76, PWD, 1, [alpha(SLATE, 0x4A)], root)

txt("VERSION", PX, D_T + 90, 9, COND, alpha(MUTED, 0xAA), root, ls=2.0)
txt(old, PX, D_T + 106, 21, MONO, alpha(MUTED, 0xC0), root)
txt("→", PX + 62, D_T + 108, 20, MONO, alpha(O300, 0xC0), root)
txt(new, PX + 88, D_T + 106, 21, MONO, O300, root)
rtxt("RELEASED  30 SEP 2026", R_X + R_W - 22, D_T + 100, 9.5, MONO,
     alpha(MUTED, 0xAA), root)
rtxt("INSTALLED  11 APR 2026", R_X + R_W - 22, D_T + 116, 9.5, MONO,
     alpha(MUTED, 0xAA), root)

rect(PX, D_T + 140, PWD, 1, [alpha(SLATE, 0x4A)], root)

rows = [
    ("SOURCE", "alpm · extra · x86_64"),
    ("BUILD", "official binary — no recipe"),
    ("SIGNED", "alectra (PGP, valid)"),
    ("SIZE", "94.6 → 132.4 MiB (+37.8)"),
]
for i, (k, v) in enumerate(rows):
    yy = D_T + 152 + i * 30
    txt(k, PX, yy + 4, 9, COND, alpha(MUTED, 0xAA), root, ls=1.8)
    txt(v, PX + 104, yy, 11.5, MONO,
        SHELL if k in ("SOURCE", "BUILD") else alpha(SHELL, 0xC0), root)

rect(PX, D_T + 278, PWD, 1, [alpha(SLATE, 0x4A)], root)

txt("DEPENDS ON", PX, D_T + 290, 9, COND, alpha(MUTED, 0xAA), root, ls=2.0)
dx, dy = PX, D_T + 308
for d in DEPS:
    w = len(d) * 6.4 + 18
    if dx + w > PX + PWD:
        dx = PX
        dy += 24
    rect(dx, dy, w, 20, [alpha(SHELL, 0x08)], root, cornerRadius=10,
         strokes=[alpha(SLATE, 0x66)], strokeWeight=1)
    txt(d, dx + 9, dy + 4, 10, MONO, alpha(MUTED, 0xD0), root)
    dx += w + 6
w = len("+4 more") * 6.4 + 18
rect(dx, dy, w, 20, [alpha(SHELL, 0x08)], root, cornerRadius=10,
     strokes=[alpha(SLATE, 0x66)], strokeWeight=1)
txt("+4 more", dx + 9, dy + 4, 10, MONO, alpha(MUTED, 0xAA), root)

rect(PX, D_T + 366, PWD, 1, [alpha(SLATE, 0x4A)], root)

txt("FILES", PX, D_T + 378, 9, COND, alpha(MUTED, 0xAA), root, ls=2.0)
txt("2,318", PX + 110, D_T + 376, 11.5, MONO, SHELL, root)
txt("/usr/bin/ffmpeg", PX, D_T + 396, 10.5, MONO, alpha(MUTED, 0xAA), root)
txt("/usr/share/man/man1/ffmpeg.1", PX, D_T + 412, 10.5, MONO, alpha(MUTED, 0xAA), root)

rect(PX, D_T + 436, PWD, 1, [alpha(SLATE, 0x4A)], root)

txt("ACTIONS", PX, D_T + 448, 9, COND, alpha(MUTED, 0xAA), root, ls=2.0)
bx = PX
for lbl, w in [("PIN", 50), ("CHANGELOG", 92), ("REMOVE", 80)]:
    rect(bx, D_T + 464, w, 24, [alpha(O700, 0x16)], root, cornerRadius=4,
         strokes=[alpha(O700, 0x77)], strokeWeight=1)
    txt(lbl, bx + 12, D_T + 471, 10.5, COND, O300, root, ls=1.4)
    bx += w + 8

rect(PX, D_T + 500, PWD, 48, [alpha(ANTH, 0xDD)], root, cornerRadius=5,
     effects=inner("#000000B0", (0, 4), 10))
rect(PX, D_T + 500, 3, 48, [ORANGE], root, cornerRadius=2)
txt("COMMAND", PX + 14, D_T + 506, 8.5, COND, alpha(MUTED, 0xAA), root, ls=1.8)
txt("sudo pacman -Syu ffmpeg", PX + 14, D_T + 518, 12, MONO, SHELL, root)
txt("⌘⌥C", PX + 198, D_T + 520, 10, MONO, O300, root)
rect(R_X + R_W - 22 - 84, D_T + 506, 84, 36, grad([O300, ORANGE], [0, 0], [0, 1]),
     root, cornerRadius=5, effects=shadow(alpha(O700, 0xA0), (0, 5), 10))
ctxt("UPDATE", R_X + R_W - 22 - 42, D_T + 518, 11, COND_SB, SHELL, root, ls=1.6)

# ============================================================== the console ==
rect(0, CON_T, W, H - CON_T, wood(0x12, 0x03), root,
     effects=noise(1.0, 0.50, SLATE))
rect(0, CON_T, W, 3, [alpha(O700, 0x88)], root)
rect(0, CON_T - 1, W, 1, [alpha(O300, 0x44)], root)
rect(0, CON_T + 3, W, 24, grad([alpha(ANTH, 0xB0), alpha(ANTH, 0x00)],
                               [0, 0], [0, 1]), root)

txt("CONSOLE", 24, CON_T + 14, 10, COND, MUTED, root, ls=2.4)
txt("operations in flight", 104, CON_T + 14, 10.5, MONO, alpha(MUTED, 0x99), root)
rtxt("3 running · 1 queued · 0 failed", 1380, CON_T + 14, 10.5, MONO, MUTED, root)
rtxt("esc to cancel · ⌘L full log", 1576, CON_T + 14, 10.5, MONO, alpha(O300, 0xBB), root)

rect(24, CON_T + 40, 6, 6, [ORANGE], root, cornerRadius=1)
txt("ffmpeg", 42, CON_T + 37, 11.5, MONO, SHELL, root)
txt("7.1", 116, CON_T + 37, 11.5, MONO, alpha(MUTED, 0xAA), root)
txt("→", 152, CON_T + 37, 11.5, MONO, alpha(O300, 0xAA), root)
txt("8.0", 176, CON_T + 37, 11.5, MONO, O300, root)
rect(240, CON_T + 39, 300, 8, [alpha(SLATE, 0x33)], root, cornerRadius=4)
rect(240, CON_T + 39, 186, 8, [ORANGE], root, cornerRadius=4)
txt("62%", 552, CON_T + 37, 11, MONO, O300, root)
txt("14.2 / 22.9 MiB", 600, CON_T + 37, 11, MONO, alpha(MUTED, 0xB0), root)
rtxt("signing · pacman -Syu", 1576, CON_T + 37, 10.5, MONO, alpha(MUTED, 0xAA), root)

rect(24, CON_T + 62, 6, 6, [alpha(SLATE, 0x99)], root, cornerRadius=1)
txt("linux", 42, CON_T + 59, 11.5, MONO, alpha(SHELL, 0xC0), root)
txt("6.11.5 → 6.12.1", 116, CON_T + 59, 11.5, MONO, alpha(MUTED, 0xAA), root)
rtxt("queued · 124 MiB · needs restart", 1576, CON_T + 59, 10.5, MONO,
     alpha(MUTED, 0xAA), root)

txt("✓", 24, CON_T + 81, 11.5, MONO, alpha(MUTED, 0xAA), root)
txt("ripgrep", 42, CON_T + 79, 11.5, MONO, alpha(MUTED, 0xB0), root)
txt("14.1.0 → 14.1.1", 116, CON_T + 79, 11.5, MONO, alpha(MUTED, 0xAA), root)
rtxt("done · 2 min ago", 1576, CON_T + 79, 10.5, MONO, alpha(MUTED, 0x99), root)

# ------------------------------------------------------------------- write --
with open(OUT_JSONL, "w") as fh:
    for st in steps:
        fh.write(json.dumps(st) + "\n")

print("%d steps -> %s" % (len(steps), OUT_JSONL))
