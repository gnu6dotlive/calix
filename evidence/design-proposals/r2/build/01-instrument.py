#!/usr/bin/env python3
"""L'Instrument — a Calix package manager as a precision instrument.

Builds the FigCraft document `01-instrument.figcraft` holding two frames:
  · 01-instrument-loaded  the working state
  · 01-instrument-quiet   the state right after opening, before any search

Nothing invents a colour: every fill is a design-system token.  The instrument's
metal is the anthracite ground with the slate token laid over it; the phosphor
is the signal-orange token, used only where a reading is live.
"""

import json
import math
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
OUT_JSONL = os.path.join(HERE, os.path.splitext(os.path.basename(__file__))[0] + ".jsonl")

# ---------------------------------------------------------------- tokens ----
ANTH = "#111418"     # anthracite, the ground
INK = "#F7F3ED"      # shell white, the ink
MUTED = "#A1A6AD"    # secondary text
SLATE = "#667085"    # hairlines, the stem
O100 = "#FFE5D2"     # signal orange 100, rim light
O300 = "#FF8E40"     # signal orange 300
ORANGE = "#FF6A00"   # signal orange, the phosphor
O700 = "#C95400"     # signal orange 700, the shadow side
BLUE = "#4169E1"
BLUE300 = "#87A0F4"

# ----------------------------------------------------------------- fonts ----
GROTESK = "IBM Plex Sans"
GROTESK_SB = "IBM Plex Sans SemiBold"
COND = "IBM Plex Sans Condensed"
COND_SB = "IBM Plex Sans Condensed SemiBold"
MONO = "IBM Plex Mono"

# ----------------------------------------------------------------- layout ---
W, H = 1600, 1000
M = 40                    # the plate's working margin
BAND = [                  # the five horizontal bands of the faceplate
    (0, 84),              # header
    (84, 262),            # source bank  |  control bay
    (262, 384),           # control strip | function rotary
    (384, 768),           # the readout window
    (768, 884),           # the specification band
    (884, 1000),          # the tape
]
BANK_X0, BANK_X1 = M, 1298       # the seven apertures
APERTURES = [
    ("ALPM", "BIN", "2,481"),
    ("AUR", "SRC", "61,204"),
    ("FLATPAK", "SBX", "187"),
    ("APPIMAGE", "ONE", "42"),
    ("CARGO", "CRATE", "9,318"),
    ("NPM", "NODE", "3,706"),
    ("TRACKED", "GIT", "27"),
]
FUNCTIONS = ["FIND", "INVENTORY", "DELTA"]

steps = []


def emit(cmd, params):
    steps.append({"command": cmd, "params": params})
    return len(steps) - 1


def R(i):
    return {"$step": i}


def alpha(col, a):
    return "%s%02X" % (col, a)


def grad(stops, frm=None, to=None):
    if frm is None:
        frm, to = [0, 0], [0, 1]
    return [{"type": "linearGradient", "stops": stops, "from": frm, "to": to}]


def radial(stops, frm, to, width=None):
    p = {"type": "radialGradient", "stops": stops, "from": frm, "to": to}
    if width:
        p["width"] = width
    return [p]


# --------------------------------------------------------------- primitives --
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


def txt(s, x, y, size=12, family=MONO, fill=INK, parent=None, width=None,
        align=None, ls=None, style=None, opacity=None, caps=None, **extra):
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
    if caps is not None:
        p["textCase"] = caps
    p.update(extra)
    params = {"text": s, "x": x, "y": y, "props": p}
    if width is not None:
        params["width"] = width
    if parent is not None:
        params["parent"] = parent
    return emit("text.create", params)


def rtxt(s, right_x, y, size=12, family=MONO, fill=INK, parent=None, ls=None,
         **extra):
    w = 500
    return txt(s, right_x - w, y, size, family, fill, parent, width=w,
               align="right", ls=ls, **extra)


def ctxt(s, centre_x, y, size=12, family=MONO, fill=INK, parent=None, ls=None,
         **extra):
    w = 500
    return txt(s, centre_x - w / 2, y, size, family, fill, parent, width=w,
               align="center", ls=ls, **extra)


def shadow(color="#000000B0", off=(0, 6), radius=14, spread=0):
    return [{"type": "dropShadow", "color": color, "offset": list(off),
             "radius": radius, "spread": spread}]


def inner(color="#000000C0", off=(0, 5), radius=12, spread=0):
    return [{"type": "innerShadow", "color": color, "offset": list(off),
             "radius": radius, "spread": spread}]


def noise(size=1.0, density=0.5, color=SLATE, blend="overlay"):
    return [{"type": "noise", "size": size, "density": density, "color": color,
             "blend": blend}]


# ------------------------------------------------------------ the identity ---
# A scallop shell as the cup of a chalice: a fan of ribs in a metal cup on a
# brushed aluminium stem.  Drawn in a 120 x 140 local box.

def mark_geometry():
    cx, cy, rx, ry = 60.0, 52.0, 48.0, 38.0
    n, depth = 7, 9.0

    def pt(t):
        return (cx + rx * math.cos(t), cy - ry * math.sin(t))

    pts = [pt(math.pi * i / n) for i in range(n + 1)]
    crests = []
    for i in range(n):
        tm = math.pi * (i + 0.5) / n
        p = pt(tm)
        crests.append((p[0] + depth * math.cos(tm), p[1] - depth * math.sin(tm)))

    d = ["M %.2f %.2f" % pts[0]]
    for i in range(n):
        c, e = crests[i], pts[i + 1]
        d.append("Q %.2f %.2f %.2f %.2f" % (c[0], c[1], e[0], e[1]))
    d.append("C 24.00 74.00 42.00 88.00 60.00 92.00")
    d.append("C 78.00 88.00 96.00 74.00 %.2f %.2f" % pts[0])
    d.append("Z")
    shell = " ".join(d)

    ribs = []
    for i in range(n):
        tm = math.pi * (i + 0.5) / n
        p = pt(tm)
        ribs.append("M 59.00 89.00 L %.2f %.2f" % (
            p[0] + 0.9 * math.cos(tm), p[1] - 0.9 * math.sin(tm)))

    hl = []
    for i in range(n // 2, n):
        tm = math.pi * (i + 0.5) / n
        p = pt(tm)
        hl.append("M %.2f %.2f L %.2f %.2f" % (
            p[0] + 4.3 * math.cos(tm), p[1] - 4.3 * math.sin(tm),
            p[0] + 1.2 * math.cos(tm), p[1] - 1.2 * math.sin(tm)))

    cup = ("M 11.00 64.00 C 11.00 71.50 34.00 75.50 60.00 75.50 "
           "C 86.00 75.50 109.00 71.50 109.00 64.00 "
           "C 109.00 81.00 87.00 94.00 60.00 96.00 "
           "C 33.00 94.00 11.00 81.00 11.00 64.00 Z")
    lip = ("M 11.00 64.00 C 11.00 71.50 34.00 75.50 60.00 75.50 "
           "C 86.00 75.50 109.00 71.50 109.00 64.00")
    shade = ("M 13.00 60.00 C 13.00 67.50 35.00 71.50 60.00 71.50 "
             "C 85.00 71.50 107.00 67.50 107.00 60.00")
    knop = ("M 46.00 92.00 C 46.00 99.00 51.00 102.00 60.00 102.00 "
            "C 69.00 102.00 74.00 99.00 74.00 92.00 Z")
    shaft = "M 53.00 99.00 L 67.00 99.00 L 68.00 127.00 L 52.00 127.00 Z"
    foot = ("M 44.00 125.00 L 76.00 125.00 C 78.50 125.00 78.50 137.00 76.00 137.00 "
            "L 44.00 137.00 C 41.50 137.00 41.50 125.00 44.00 125.00 Z")
    return {"shell": shell, "ribs": " ".join(ribs), "hl": " ".join(hl),
            "cup": cup, "lip": lip, "shade": shade, "knop": knop,
            "shaft": shaft, "foot": foot}


MARK = mark_geometry()
_NUM = re.compile(r"-?\d+(?:\.\d+)?")


def draw_mark(parent, x, y, height):
    """The badge, applied to the faceplate: an orange shell in an aluminium cup
    on an aluminium stem."""
    s = height / 140.0

    def path_of(key):
        nums = _NUM.sub(lambda m: "%.2f" % (float(m.group(0)) * s), MARK[key])
        out = []
        for tok in nums.split():
            out.append(tok if tok in ("M", "C", "Q", "L", "Z") else float(tok))
        res, k = [], 0
        while k < len(out):
            t = out[k]
            if isinstance(t, str):
                res.append(t)
                k += 1
                continue
            res.append("%.2f %.2f" % (t + x, out[k + 1] + y))
            k += 2
        return " ".join(res)

    vec(path_of("shell"), parent,
        fills=grad([O300, ORANGE, O700], [0.12, 0.0], [0.68, 1.0]),
        name="mark-shell")
    vec(path_of("ribs"), parent, fills=[],
        strokes=[alpha(O700, 0xE6)], strokeWeight=4.2 * s, strokeCap="round",
        name="mark-ribs")
    vec(path_of("hl"), parent, fills=[], strokes=[alpha(O100, 0xB0)],
        strokeWeight=2.4 * s, strokeCap="round", name="mark-rimlight")
    vec(path_of("shade"), parent, fills=[], strokes=[alpha(ANTH, 0x55)],
        strokeWeight=3.0 * s, strokeCap="round", name="mark-cup-shadow")
    vec(path_of("cup"), parent,
        fills=grad([MUTED, SLATE, ANTH], [0.0, 0.0], [1.0, 0.35]),
        name="mark-cup")
    vec(path_of("lip"), parent, fills=[], strokes=[alpha(O100, 0x96)],
        strokeWeight=2.2 * s, strokeCap="round", name="mark-lip")
    for key in ("knop", "shaft", "foot"):
        vec(path_of(key), parent,
            fills=grad([SLATE, MUTED, SLATE, ANTH], [0.0, 0.0], [1.0, 0.0]),
            name="mark-stem")


# ---------------------------------------------------------------- the data ---
DELTA_ROWS = [
    ("ripgrep", "14.1.0", "14.1.1", "core · x86_64", 1.2, "OUTDATED",
     "Fast recursive grep that respects .gitignore."),
    ("linux", "6.11.5", "6.12.1", "core · x86_64", 124.0, "OUTDATED",
     "The Linux kernel and modules."),
    ("ffmpeg", "7.1", "8.0", "extra · x86_64", 37.8, "OUTDATED",
     "Tools for transcoding, streaming and playback of multimedia."),
    ("postgresql", "17.4", "17.5", "extra · x86_64", 4.1, "OUTDATED",
     "Object-relational database system."),
    ("nvidia-dkms", "550.144", "555.58", "multilib · x86_64", 92.0, "HELD",
     "NVIDIA driver module built for the running kernel."),
    ("foot", "0.16.2", "0.17.0", "extra · x86_64", 0.6, "OUTDATED",
     "A fast GPU-driven terminal emulator."),
]
SCALE_MAX = 128.0          # MiB
SELECTED = 0               # ripgrep


# ----------------------------------------------------------------- plates ----
def milled(x, y, w, h, parent, sheen=0x26, deep=0x08):
    """A raised plate: the anodised surface, a light above, a shadow below."""
    rect(x, y, w, h, grad([alpha(SLATE, sheen), alpha(SLATE, deep)], [0, 0], [0, 1]),
         parent, effects=[{"type": "noise", "size": 1.0, "density": 0.42,
                           "color": SLATE, "blend": "overlay"},
                          {"type": "dropShadow", "color": "#000000A0",
                           "offset": [0, 4], "radius": 10, "spread": 0}])
    rect(x, y, w, 1, [alpha(INK, 0x26)], parent)
    rect(x, y + h - 1, w, 1, [alpha(ANTH, 0xE0)], parent)
    rect(x, y + h, w, 1, [alpha(ANTH, 0x88)], parent)


def recess(x, y, w, h, parent, fill=None):
    """A window cut into the plate: shadow at the top, light at the bottom."""
    rect(x, y, w, h, fill or [ANTH], parent,
         effects=inner("#000000E0", (0, 9), 22))
    rect(x, y, w, 3, [alpha(ANTH, 0xFF)], parent)
    rect(x, y + h - 3, w, 3, [alpha(INK, 0x1C)], parent)
    rect(x, y, 3, h, [alpha(ANTH, 0xAA)], parent)
    rect(x + w - 3, y, 3, h, [alpha(INK, 0x0A)], parent)


def rule(x1, y, x2, col=SLATE, a=0x40, thick=1):
    return rect(x1, y, x2 - x1, thick, [alpha(col, a)])


def vrule(x, y1, y2, col=SLATE, a=0x40, thick=1):
    return rect(x, y1, thick, y2 - y1, [alpha(col, a)])


def engrav(text, x, y, size=9.5, family=COND, fill=SLATE, ls=1.8, parent=None,
           align=None):
    """Engraved lettering: cut into the plate, so it reads as slate."""
    return txt(text, x, y, size, family, fill, parent, ls=ls, align=align)


# ============================================================== the screen ===
def screen(parent, loaded):
    ox = 0  # the frame's own origin

    # ---------------------------------------------------------- the plate --
    rect(0, 0, W, H, [ANTH], parent)
    for x in range(0, W + 1, 40):
        rect(x, 0, 1, H, [alpha(SLATE, 0x10)], parent)
    for y in range(0, H + 1, 40):
        rect(0, y, W, 1, [alpha(SLATE, 0x10)], parent)
    for x in range(0, W + 1, 200):
        rect(x, 0, 1, H, [alpha(SLATE, 0x1E)], parent)
    for y in range(0, H + 1, 200):
        rect(0, y, W, 1, [alpha(SLATE, 0x1E)], parent)

    # ------------------------------------------------------------- header --
    milled(0, 0, W, 84, parent, 0x1A, 0x04)
    draw_mark(parent, 38, 6, 74)
    txt("CALIX", 122, 24, 28, GROTESK_SB, INK, parent, ls=6)
    # the freshness of the reading: always here, always this typeface
    engrav("READ", 1180, 26, 9.5, COND, SLATE, 2.0, parent)
    txt("20:14:31Z", 1174, 40, 15, MONO, INK, parent)
    engrav("AGE", 1330, 26, 9.5, COND, SLATE, 2.0, parent)
    txt("4 MIN", 1324, 40, 15, MONO, INK, parent)
    # the help overlay lives off the surface; this is only the way in
    ellipse(1524, 24, 36, 36, [alpha(SLATE, 0x22)], parent,
            strokes=[alpha(SLATE, 0x66)], strokeWeight=1)
    ctxt("?", 1542, 32, 16, MONO, MUTED, parent)

    # ------------------------------------------------------- source bank ---
    engrav("SOURCE", M, 96, 9.5, COND, SLATE, 2.4, parent)
    aw = (BANK_X1 - BANK_X0 - 6 * 8) // 7
    for i, (name, kind, count) in enumerate(APERTURES):
        x = BANK_X0 + i * (aw + 8)
        y = 112
        engaged = (loaded and i == 0) or (not loaded and i == 0)
        milled(x, y, aw, 134, parent, 0x2A if engaged else 0x1C, 0x08)
        txt(name, x + 14, y + 16, 13, COND_SB, INK if engaged else MUTED,
            parent, ls=1.8)
        txt(kind, x + 14, y + 42, 10.5, MONO, SLATE, parent, ls=1.2)
        rule(x + 14, y + 62, x + 46, SLATE, 0x50)
        txt(count, x + 14, y + 76, 21, MONO, INK if engaged else MUTED, parent)
        txt("PKGS", x + 14 + len(count) * 12.6 + 10, y + 88, 9, MONO, SLATE, parent)
        if engaged:
            rect(x, y + 131, aw, 3, [ORANGE], parent,
                 effects=shadow(alpha(ORANGE, 0x99), (0, 0), 8))
    # the control bay, divided from the bank
    vrule(1336, 84, 384, SLATE, 0x44)

    # ----------------------------------------------------- control strip ---
    # one line of scope, one row of the actions this source and function
    # permit, and the state of the list being read
    if loaded:
        engrav("SCOPE", M, 276, 9.5, COND, SLATE, 2.0, parent)
        txt("ALPM · DELTA", 96, 272, 13, MONO, INK, parent, ls=1.0)
        engrav("PERMITS", 226, 276, 9.5, COND, SLATE, 2.0, parent)
        txt("UPDATE · PIN · IGNORE · HOLD", 300, 272, 13, MONO, MUTED, parent)
        engrav("FORBIDS", 548, 276, 9.5, COND, SLATE, 2.0, parent)
        txt("BUILD — NOTHING TO BUILD", 622, 272, 13, MONO, alpha(SLATE, 0xE0), parent)
        bx = M
        for label, w, hot in (("UPDATE ALL (8)", 122, True),
                              ("UPDATE SELECTED", 132, False),
                              ("PIN", 54, False),
                              ("IGNORE", 68, False),
                              ("REFRESH", 80, False)):
            if hot:
                rect(bx, 308, w, 30, [alpha(ORANGE, 0x18)], parent, cornerRadius=3,
                     strokes=[alpha(ORANGE, 0x99)], strokeWeight=1)
                txt(label, bx + 12, 314, 10.5, COND_SB, O300, parent, ls=1.2)
            else:
                rect(bx, 312, w, 26, [alpha(SLATE, 0x14)], parent, cornerRadius=3,
                     strokes=[alpha(SLATE, 0x55)], strokeWeight=1)
                txt(label, bx + 12, 318, 10.5, COND, MUTED, parent, ls=1.2)
            bx += w + 10
        engrav("SHOWN", 620, 318, 9.5, COND, SLATE, 1.8, parent)
        txt("8 OF 812", 686, 314, 13, MONO, INK, parent)
        engrav("FILTER", 800, 318, 9.5, COND, SLATE, 1.8, parent)
        txt("OUTDATED", 866, 314, 13, MONO, MUTED, parent)
    else:
        engrav("SCOPE", M, 276, 9.5, COND, SLATE, 2.0, parent)
        txt("ALPM · FIND", 96, 272, 13, MONO, INK, parent, ls=1.0)
        engrav("PERMITS", 226, 276, 9.5, COND, SLATE, 2.0, parent)
        txt("FIND · INSTALL · REMOVE", 300, 272, 13, MONO, MUTED, parent)
        engrav("FORBIDS", 520, 276, 9.5, COND, SLATE, 2.0, parent)
        txt("BUILD — NOTHING TO BUILD", 594, 272, 13, MONO, alpha(SLATE, 0xE0), parent)
        bx = M
        for label, w in (("UPDATE ALL (8)", 122), ("UPDATE SELECTED", 132),
                         ("PIN", 54), ("IGNORE", 68), ("REFRESH", 80)):
            rect(bx, 312, w, 26, [alpha(SLATE, 0x08)], parent, cornerRadius=3,
                 strokes=[alpha(SLATE, 0x2A)], strokeWeight=1)
            txt(label, bx + 12, 318, 10.5, COND, alpha(SLATE, 0xAA), parent, ls=1.2)
            bx += w + 10
        engrav("SHOWN", 620, 318, 9.5, COND, SLATE, 1.8, parent)
        txt("— OF 2,481", 686, 314, 13, MONO, alpha(MUTED, 0xAA), parent)
        engrav("FILTER", 800, 318, 9.5, COND, SLATE, 1.8, parent)
        txt("NONE", 866, 314, 13, MONO, alpha(MUTED, 0xAA), parent)

    # the density legend, engraved in the control bay under the knob: both
    # row shapes, so the other density is legible as a possibility
    engrav("ROWS", 1358, 302, 9.5, COND, SLATE, 1.8, parent)
    rect(1410, 306, 26, 4, [alpha(INK, 0x99)], parent)
    rect(1410, 313, 17, 3, [alpha(INK, 0x99)], parent)
    engrav("DENSE", 1444, 302, 9.5, COND, INK if loaded else SLATE, 1.4, parent)
    rect(1410, 342, 26, 4, [alpha(SLATE, 0x88)], parent)
    engrav("COMPACT", 1444, 338, 9.5, COND, SLATE, 1.4, parent)

    # the totaliser, in the bay above the knob: the size of what is pending
    engrav("Δ TOTAL", 1360, 96, 9.5, COND, SLATE, 1.8, parent)
    if loaded:
        txt("164.5", 1358, 110, 29, MONO, INK, parent)
        txt("MiB", 1452, 124, 12, MONO, MUTED, parent)
        txt("8 PACKAGES PENDING", 1358, 146, 10, MONO, alpha(SLATE, 0xE0), parent)
    else:
        txt("—", 1358, 110, 30, MONO, alpha(MUTED, 0xAA), parent)
        txt("NOTHING READ", 1358, 146, 10, MONO, alpha(SLATE, 0xC0), parent)

    # ------------------------------------------------- the function rotary --
    knob_cx, knob_cy, knob_r = 1458, 248, 33
    for i, ang in enumerate((215, 270, 325)):
        a = math.radians(ang)
        # the engraved scale arc, drawn as a polyline so it never depends on
        # arc support
        seg = []
        for k in range(13):
            t = math.radians(215 + (325 - 215) * k / 12.0)
            seg.append("%.2f %.2f" % (knob_cx + 48 * math.cos(t),
                                       knob_cy + 48 * math.sin(t)))
        if i == 0:
            vec("M " + " L ".join(seg), parent, fills=[],
                strokes=[alpha(SLATE, 0xAA)], strokeWeight=1.4)
        # a radial detent tick on the scale
        vec("M %.2f %.2f L %.2f %.2f" % (
            knob_cx + 44 * math.cos(a), knob_cy + 44 * math.sin(a),
            knob_cx + 51 * math.cos(a), knob_cy + 51 * math.sin(a)),
            parent, fills=[], strokes=[alpha(SLATE, 0xAA)], strokeWeight=1.4)
        tx = knob_cx + 70 * math.cos(a)
        ty = knob_cy + 70 * math.sin(a)
        ctxt(FUNCTIONS[i], tx, ty - 6, 10.5, COND,
             INK if (loaded and i == 2) or (not loaded and i == 0) else alpha(SLATE, 0xDD),
             parent, ls=1.6)
    # the knob: a machined disc standing proud of the plate
    ellipse(knob_cx - knob_r, knob_cy - knob_r, knob_r * 2, knob_r * 2,
            grad([alpha(SLATE, 0xFF), alpha(ANTH, 0xFF)], [0.05, 0.0], [0.95, 1.0]),
            parent, effects=shadow("#000000C0", (0, 6), 14))
    kf = knob_r - 5
    ellipse(knob_cx - kf, knob_cy - kf, kf * 2, kf * 2,
            grad([MUTED, SLATE, alpha(ANTH, 0xF0)], [0.12, 0.0], [0.82, 1.0]),
            parent)
    knurl = []
    for k in range(28):
        t = math.radians(k * 360.0 / 28)
        knurl.append("M %.2f %.2f L %.2f %.2f" % (
            knob_cx + (knob_r - 5) * math.cos(t),
            knob_cy + (knob_r - 5) * math.sin(t),
            knob_cx + (knob_r - 1) * math.cos(t),
            knob_cy + (knob_r - 1) * math.sin(t)))
    vec(" ".join(knurl), parent, fills=[], strokes=[alpha(INK, 0x22)],
        strokeWeight=1.3, name="knob-knurl")
    # a bright crescent on the upper-left of the cap
    cres = []
    for k in range(11):
        t = math.radians(168 + 150.0 * k / 10.0)
        cres.append("%.2f %.2f" % (knob_cx + (kf - 1) * math.cos(t),
                                   knob_cy + (kf - 1) * math.sin(t)))
    vec("M " + " L ".join(cres), parent, fills=[],
        strokes=[alpha(INK, 0x38)], strokeWeight=1.6, name="knob-crescent")
    ellipse(knob_cx - 5, knob_cy - 5, 10, 10, [alpha(ANTH, 0xC0)], parent,
            strokes=[alpha(INK, 0x22)], strokeWeight=1)
    # the pointer, thrown to the engaged detent
    ang = 325 if loaded else 215
    a = math.radians(ang)
    vec("M %.2f %.2f L %.2f %.2f" % (knob_cx, knob_cy,
                                     knob_cx + (knob_r - 8) * math.cos(a),
                                     knob_cy + (knob_r - 8) * math.sin(a)),
        parent, fills=[], strokes=[ORANGE], strokeWeight=2.6, strokeCap="round",
        effects=shadow(alpha(ORANGE, 0xAA), (0, 0), 7))
    ellipse(knob_cx + (knob_r - 8) * math.cos(a) - 3,
            knob_cy + (knob_r - 8) * math.sin(a) - 3, 6, 6, [O100], parent,
            effects=shadow(alpha(O100, 0x99), (0, 0), 5))

    # -------------------------------------------------- the readout window --
    recess(M, 384, W - 2 * M, 376, parent)
    for x in range(M + 20, W - M, 40):
        rect(x, 386, 1, 372, [alpha(SLATE, 0x0E)], parent)
    for y in range(404, 758, 52):
        rect(M + 2, y, W - 2 * M - 4, 1, [alpha(SLATE, 0x16)], parent)


    if loaded:
        # column headings, engraved on the glass
        engrav("PACKAGE", 68, 396, 9.5, COND, SLATE, 1.8, parent)
        rtxt("INSTALLED", 510, 396, 9.5, COND, SLATE, parent, ls=1.8)
        rtxt("AVAILABLE", 640, 396, 9.5, COND, SLATE, parent, ls=1.8)
        engrav("PROVENANCE", 680, 396, 9.5, COND, SLATE, 1.8, parent)
        engrav("Δ  MiB", 910, 396, 9.5, COND, SLATE, 1.8, parent)
        rtxt("SIZE", 1400, 396, 9.5, COND, SLATE, parent, ls=1.8)
        engrav("STATE", 1425, 396, 9.5, COND, SLATE, 1.8, parent)
        # the scale the delta is measured against
        track_x0, track_w = 910, 400
        for i in range(5):
            tx = track_x0 + i * track_w / 4
            rect(tx, 414, 1, 7, [alpha(SLATE, 0x77)], parent)
            ctxt(str(int(SCALE_MAX * i / 4)), tx, 424, 9, MONO, SLATE, parent)
        rule(track_x0, 412, track_x0 + track_w, SLATE, 0x50)
        # the rows
        for i, (name, inst, avail, prov, mib, state, desc) in enumerate(DELTA_ROWS):
            y = 446 + i * 52
            sel = (i == SELECTED)
            if sel:
                rect(52, y, 1496, 48, [alpha(SLATE, 0x10)], parent)
                rect(52, y, 3, 48, [ORANGE], parent,
                     effects=shadow(alpha(ORANGE, 0x99), (0, 0), 8))
            txt(name, 68, y + 6, 16, GROTESK_SB, INK if sel else MUTED, parent)
            txt(desc, 68, y + 28, 11.5, GROTESK, alpha(MUTED, 0xB4), parent)
            rtxt(inst, 510, y + 8, 13.5, MONO, alpha(MUTED, 0xC4), parent)
            rtxt(avail, 640, y + 8, 13.5, MONO,
                 O300 if sel else alpha(INK, 0xD8), parent)
            txt(prov.upper(), 680, y + 10, 10, COND, alpha(MUTED, 0xC0), parent, ls=1.2)
            # the delta, measured
            bw = max(4.0, track_w * (mib / SCALE_MAX))
            rect(track_x0, y + 14, bw, 12, [alpha(ORANGE, 0x66)], parent,
                 cornerRadius=1)
            rect(track_x0 + bw - 2, y + 11, 2, 18, [ORANGE], parent)
            rtxt(("%.1f MiB" % mib) if mib < 10 else ("%.0f MiB" % mib),
                 1400, y + 8, 13.5, MONO, alpha(INK, 0xD8), parent)
            txt(state, 1425, y + 10, 10, COND,
                alpha(MUTED, 0xC0) if state == "HELD" else O300, parent, ls=1.4)
    else:
        # the quiet state: the window is empty.  The instrument is ready; it
        # has nothing to say yet.
        rect(M + 2, 386, W - 2 * M - 4, 372, [
            {"type": "linearGradient", "stops": [alpha(INK, 0x00), alpha(INK, 0x0A),
                                                 alpha(INK, 0x00)],
             "from": [0.10, 0.0], "to": [0.66, 1.0]}], parent)

    # ---------------------------------------------- the specification band --
    milled(M, 768, W - 2 * M, 110, parent, 0x20, 0x07)
    cells = [360, 540, 800, 1060, 1240]
    for cx in cells:
        rect(cx, 782, 1, 80, [alpha(SLATE, 0x55)], parent)
    if loaded:
        name, inst, avail, prov, mib, state, desc = DELTA_ROWS[SELECTED]
        txt(name, M + 18, 782, 24, GROTESK_SB, INK, parent)
        txt(state, M + 18, 818, 11, COND, O300, parent, ls=1.6)
        engrav("BINARY", M + 100, 822, 9.5, COND, SLATE, 1.8, parent)

        engrav("VERSION", 380, 782, 9, COND, SLATE, 1.8, parent)
        txt("14.1.0", 380, 796, 16, MONO, alpha(MUTED, 0xC4), parent)
        txt("→", 452, 796, 16, MONO, alpha(O300, 0xAA), parent)
        txt("14.1.1", 478, 796, 16, MONO, O300, parent)
        engrav("REL 30 SEP 26", 380, 822, 9, MONO, SLATE, 0.8, parent)
        engrav("INST 11 APR 26", 380, 840, 9, MONO, SLATE, 0.8, parent)

        engrav("PROVENANCE", 560, 782, 9, COND, SLATE, 1.8, parent)
        txt("alpm · core · x86_64", 560, 796, 13, MONO, INK, parent)
        txt("OFFICIAL BINARY — NO RECIPE", 560, 818, 11, MONO, SLATE, parent)
        txt("SIGNED · ALECTRA (PGP)", 560, 840, 11, MONO, SLATE, parent)

        engrav("DEPENDS ON", 820, 782, 9, COND, SLATE, 1.8, parent)
        txt("pcre2  zlib  gcc-libs", 820, 798, 13, MONO, MUTED, parent)
        engrav("+2 MORE", 820, 822, 9, MONO, SLATE, 0.8, parent)

        engrav("FILES", 1080, 782, 9, COND, SLATE, 1.8, parent)
        txt("1,284", 1080, 796, 13, MONO, INK, parent)
        engrav("/usr/bin/rg", 1080, 822, 10, MONO, SLATE, 0.8, parent)

        engrav("THE COMMAND", 1260, 782, 9, COND, SLATE, 1.8, parent)
        rect(1256, 796, 288, 34, [alpha(ANTH, 0xD8)], parent, cornerRadius=3,
             effects=inner("#000000A0", (0, 3), 8))
        rect(1256, 796, 3, 34, [ORANGE], parent, cornerRadius=2)
        txt("calix install standard ripgrep", 1268, 806, 13.5, MONO, INK, parent)
        engrav("THE PRODUCT'S OWN", 1260, 840, 9, COND, SLATE, 1.6, parent)
    else:
        txt("ALPM", M + 18, 782, 24, GROTESK_SB, INK, parent)
        txt("BINARY REPOSITORY", M + 18, 818, 11, COND, O300, parent, ls=1.6)
        engrav("THE ENGAGED CHANNEL", M + 18, 840, 9, COND, SLATE, 1.6, parent)

        engrav("REPOSITORIES", 380, 782, 9, COND, SLATE, 1.8, parent)
        txt("core · extra", 380, 796, 12.5, MONO, INK, parent)
        txt("multilib", 380, 814, 12.5, MONO, INK, parent)
        engrav("SIGNED · ALECTRA (PGP)", 380, 838, 9.5, MONO, SLATE, 0.6, parent)

        engrav("HOLDS", 560, 782, 9, COND, SLATE, 1.8, parent)
        txt("2,481 PACKAGES", 560, 796, 12.5, MONO, INK, parent)
        txt("BINARIES — NO RECIPES", 560, 818, 10.5, MONO, SLATE, parent)
        engrav("CORE 2,103 · EXTRA 361 · MULTILIB 17", 560, 838, 9.5, MONO,
               SLATE, 0.6, parent)

        engrav("OPERATIONS", 820, 782, 9, COND, SLATE, 1.8, parent)
        txt("INSTALL · UPDATE", 820, 796, 12.5, MONO, MUTED, parent)
        txt("REMOVE", 820, 814, 12.5, MONO, MUTED, parent)
        engrav("NO BUILD, NO RECIPE", 820, 838, 9.5, MONO, SLATE, 0.6, parent)

        engrav("NEEDS", 1080, 782, 9, COND, SLATE, 1.8, parent)
        txt("A QUERY", 1080, 796, 12.5, MONO, INK, parent)
        engrav("BEFORE IT READS", 1080, 818, 9.5, MONO, SLATE, 0.6, parent)

        engrav("THE COMMAND", 1260, 782, 9, COND, SLATE, 1.8, parent)
        rect(1256, 796, 288, 34, [alpha(ANTH, 0xD8)], parent, cornerRadius=3,
             effects=inner("#000000A0", (0, 3), 8))
        rect(1256, 796, 3, 34, [ORANGE], parent, cornerRadius=2)
        txt("calix install standard <package>", 1268, 806, 12.5, MONO, MUTED, parent)
        engrav("THE PRODUCT'S OWN", 1260, 838, 9, COND, SLATE, 1.6, parent)

    # ------------------------------------------------------------- the tape --
    # a strip of the machine's own record, coming out of a slot in the plate
    rect(0, 884, W, 14, [alpha(ANTH, 0xFF)], parent,
         effects=inner("#000000C0", (0, 6), 14))
    rect(0, 895, W, 1, [alpha(INK, 0x10)], parent)
    rect(0, 896, W, 104, grad([alpha(SLATE, 0x12), alpha(SLATE, 0x03)],
                              [0, 0], [0, 1]), parent)
    for y in range(906, 998, 18):
        ellipse(50, y, 8, 8, [alpha(ANTH, 0xFF)], parent,
                strokes=[alpha(SLATE, 0x44)], strokeWeight=1)
    engrav("RECORD", 74, 902, 9, COND, SLATE, 2.0, parent)
    if loaded:
        lines = [
            ("calix install standard ripgrep", INK, "RUNNING", O300, True),
            ("→ pacman -Syu ripgrep", MUTED, "ALPM · 1.2 MiB", alpha(MUTED, 0xAA), False),
            ("alpm: downloading ripgrep-14.1.1-1-x86_64.pkg.tar.zst", alpha(MUTED, 0xAA),
             "20:14:29", alpha(MUTED, 0xAA), False),
            ("calix install standard alacritty", alpha(MUTED, 0x88),
             "DONE 20:12:04", alpha(MUTED, 0x88), False),
        ]
        for i, (line, col, right, rcol, live) in enumerate(lines):
            y = 918 + i * 21
            txt(line, 74, y, 11.5, MONO, col, parent)
            rtxt(right, 1536, y, 11, MONO, rcol, parent)
            if live:
                # the print head: where the record is being made now
                rect(74 + len(line) * 6.9 + 8, y - 1, 7, 13, [ORANGE], parent,
                     effects=shadow(alpha(ORANGE, 0x99), (0, 0), 6))
                # a progress bar, read against the tape's own scale
                rect(1200, y, 240, 6, [alpha(SLATE, 0x33)], parent, cornerRadius=3)
                rect(1200, y, 149, 6, [ORANGE], parent, cornerRadius=3)
                rtxt("62%", 1190, y - 2, 11, MONO, O300, parent)
    else:
        txt("NO OPERATIONS RECORDED", 74, 918, 11.5, MONO, alpha(MUTED, 0xAA), parent)
        rtxt("IDLE", 1536, 918, 11, MONO, alpha(MUTED, 0xAA), parent)


# ================================================================ the build ===
def main():
    for name, loaded, x in (("01-instrument-loaded", True, 0),
                            ("01-instrument-quiet", False, 1700)):
        frame = R(emit("frame.create", {
            "name": name, "x": x, "y": 0, "width": W, "height": H,
            "props": {"fills": [ANTH], "clip": True}}))
        screen(frame, loaded)
    emit("page.setBackground", {"color": ANTH})
    with open(OUT_JSONL, "w") as fh:
        for st in steps:
            fh.write(json.dumps(st) + "\n")
    print("%d steps -> %s" % (len(steps), OUT_JSONL))


main()
