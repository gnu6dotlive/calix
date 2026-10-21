#!/usr/bin/env python3
"""Calix — "La Carte": a package manager as a chart.

Builds 03-carte.figcraft. Run:

    python3 evidence/design-proposals/build/03-carte.build.py \
        --out evidence/design-proposals/03-carte.figcraft

The generator is the source of truth; the .figcraft is its output.
"""
import argparse
import json
import math
import os

# ---------------------------------------------------------------- tokens ----
GROUND = "#111418"   # anthracite — the ground
INK    = "#F7F3ED"   # shell white — the ink
MUTED  = "#A1A6AD"   # secondary text
SLATE  = "#667085"   # hairlines, the stem
O100   = "#FFE5D2"   # signal orange 100 — rim light
O300   = "#FF8E40"   # signal orange 300 — hover
ORANGE = "#FF6A00"   # signal orange — the accent
O700   = "#C95400"   # signal orange 700 — shadow side
B300   = "#87A0F4"   # royal blue 300 — alternative accent, hover
BLUE   = "#4169E1"   # royal blue — the alternative accent

SANS = "IBM Plex Sans"
MONO = "IBM Plex Mono"
COND = "IBM Plex Sans Condensed"

CLEAR = "#FFFFFF00"  # no paint (alpha only, no hue)


def a(hexcol, alpha):
    """Token + alpha as #RRGGBBAA."""
    return hexcol + "%02X" % alpha


# ------------------------------------------------------------- geometry ----
W, H = 1600, 1000
HEADER_H = 96
RAIL_W = 316
SHEET_X, SHEET_Y, SHEET_W, SHEET_H = 316, 96, 884, 788
CARD_X, CARD_W, CARD_H = 1200, 400, 788
CONSOLE_Y, CONSOLE_H = 884, 116

# --------------------------------------------------------------- engine ----
CMDS = []


def add(cmd, params):
    CMDS.append({"command": cmd, "params": params})
    return len(CMDS) - 1


def S(i):
    return {"$step": i}


def create(kind, parent=None, x=0, y=0, w=None, h=None, name=None, props=None, **kw):
    p = {"type": kind}
    if parent is not None:
        p["parent"] = S(parent)
    p["x"] = x
    p["y"] = y
    if w is not None:
        p["width"] = w
    if h is not None:
        p["height"] = h
    if name:
        p["name"] = name
    p["props"] = props or {}
    p.update(kw)
    idx = add("node.create", p)
    if kind == "frame":
        # a frame is born white; every frame in this drawing states its own
        # paint, so start from nothing
        setp(idx, fills=[{"type": "solid", "color": CLEAR}])
    return idx


def setp(idx, **kw):
    add("node.set", {"ids": [S(idx)], **kw})


def paint(idx, fill=None, stroke=None, sw=1.0, effects=None, opacity=None,
          radius=None, sides=None, blend=None):
    kw = {}
    if fill is not None:
        kw["fills"] = [{"type": "solid", "color": fill}]
    if stroke is not None:
        kw["strokes"] = [{"type": "solid", "color": stroke}]
        kw["strokeWeight"] = sw
        if sides:
            kw["strokeSides"] = sides
    if effects:
        kw["effects"] = effects
    if opacity is not None:
        kw["opacity"] = opacity
    if radius is not None:
        kw["cornerRadius"] = radius
    if blend is not None:
        kw["blend"] = blend
    setp(idx, **kw)


def grad(idx, stops, kind="linearGradient", frm=None, to=None, **kw):
    paint_def = {"type": kind, "stops": stops}
    if frm is not None:
        paint_def["from"] = frm
    if to is not None:
        paint_def["to"] = to
    kw["fills"] = [paint_def]
    setp(idx, **kw)


def shadow(color, offset, radius, spread=0.0):
    return {"type": "dropShadow", "color": color, "offset": offset,
            "radius": radius, "spread": spread}


def inner(color, offset, radius, spread=0.0):
    return {"type": "innerShadow", "color": color, "offset": offset,
            "radius": radius, "spread": spread}


def blur(radius):
    return {"type": "layerBlur", "radius": radius}


def txt(parent, s, x, y, size=12, family=SANS, style=None, color=INK, ls=None,
        align=None, w=None, lh=None, case=None, opacity=None, name=None):
    props = {"fontFamily": family, "fontSize": size, "fill": color}
    if style:
        props["fontStyle"] = style
    if ls is not None:
        props["letterSpacing"] = ls
    if align:
        props["textAlign"] = align
    if lh is not None:
        props["lineHeight"] = lh
    if case:
        props["textCase"] = case
    if opacity is not None:
        props["opacity"] = opacity
    p = {"text": s, "parent": S(parent), "x": x, "y": y, "props": props}
    if w is not None:
        p["width"] = w
        props["textAutoResize"] = "height"
    if name:
        p["name"] = name
    return add("text.create", p)


# --------------------------------------------------------------- vectors ---
def path_of(cmds):
    out = []
    for c in cmds:
        k = c[0]
        if k == "M" or k == "L":
            out.append("%s %.2f %.2f" % (k, c[1], c[2]))
        elif k == "C":
            out.append("C %.2f %.2f %.2f %.2f %.2f %.2f" % c[1:])
        elif k == "Q":
            out.append("Q %.2f %.2f %.2f %.2f" % c[1:])
        elif k == "A":
            out.append("A %.2f %.2f %.2f %d %d %.2f %.2f" % c[1:])
        elif k == "Z":
            out.append("Z")
    return " ".join(out)


def path_bbox(cmds):
    xs, ys = [], []
    for c in cmds:
        k = c[0]
        if k in ("M", "L"):
            xs.append(c[1]); ys.append(c[2])
        elif k == "C":
            xs += [c[1], c[3], c[5]]; ys += [c[2], c[4], c[6]]
        elif k == "Q":
            xs += [c[1], c[3]]; ys += [c[2], c[4]]
        elif k == "A":
            xs += [c[5], c[6]]; ys += [c[6], c[7]]
    return min(xs), min(ys), max(xs), max(ys)


def normalize(cmds):
    bx, by, _, _ = path_bbox(cmds)
    out = []
    for c in cmds:
        k = c[0]
        if k in ("M", "L"):
            out.append((k, c[1] - bx, c[2] - by))
        elif k == "C":
            out.append((k, c[1] - bx, c[2] - by, c[3] - bx, c[4] - by,
                        c[5] - bx, c[6] - by))
        elif k == "Q":
            out.append((k, c[1] - bx, c[2] - by, c[3] - bx, c[4] - by))
        elif k == "A":
            out.append((k, c[1], c[2], c[3], c[4], c[5], c[5] - bx, c[6] - by))
        else:
            out.append(c)
    return out, bx, by


def vpath(parent, cmds, x=0, y=0, fill=None, stroke=None, sw=1.0,
          opacity=None, name=None):
    norm, bx, by = normalize(cmds)
    p = {"type": "vector", "parent": S(parent), "x": x + bx, "y": y + by,
         "path": path_of(norm)}
    if name:
        p["name"] = name
    idx = add("node.create", p)
    paint(idx, fill=fill if fill else CLEAR, stroke=stroke, sw=sw,
          opacity=opacity)
    return idx


def seg(parent, x1, y1, x2, y2, stroke, sw=1.0, opacity=None):
    """A straight rule drawn as a two-point vector."""
    return vpath(parent, [("M", x1, y1), ("L", x2, y2)], x=0, y=0,
                 stroke=stroke, sw=sw, opacity=opacity)


def arc(parent, cx, cy, r, start_deg, end_deg, stroke, sw=1.0, opacity=None):
    """Circular arc from start to end degrees (0 = up, clockwise)."""
    def pt(deg):
        a_ = math.radians(deg)
        return (cx + r * math.sin(a_), cy - r * math.cos(a_))
    x1, y1 = pt(start_deg)
    x2, y2 = pt(end_deg)
    large = 1 if (end_deg - start_deg) % 360 > 180 else 0
    sweep = 1 if end_deg > start_deg else 0
    return vpath(parent, [("M", x1, y1),
                          ("A", r, r, 0, large, sweep, x2, y2)],
                 x=0, y=0, stroke=stroke, sw=sw, opacity=opacity)


def ring(parent, cx, cy, r, stroke, sw=1.0, fill=None, opacity=None):
    idx = create("ellipse", parent=parent, x=cx - r, y=cy - r, w=r * 2, h=r * 2)
    paint(idx, fill=fill if fill else CLEAR, stroke=stroke, sw=sw,
          opacity=opacity)
    return idx


# ------------------------------------------------------- the shell mark ----
SHELL_BASE = [
    ("M", 36, 45),
    ("C", 22, 44, 12, 36, 10, 26),
    ("C", 11, 22, 12.5, 18, 15, 16),
    ("C", 18, 13, 19, 12, 22, 12),
    ("C", 24, 8, 26, 6, 29, 6),
    ("C", 32, 6, 34, 8, 36, 8),
    ("C", 38, 8, 40, 6, 43, 6),
    ("C", 46, 6, 48, 8, 50, 12),
    ("C", 53, 12, 54, 13, 57, 16),
    ("C", 59.5, 18, 61, 22, 62, 26),
    ("C", 60, 36, 50, 44, 36, 45),
    ("Z",),
]
RIB_BASE = [(12, 22), (17, 15), (24, 9), (36, 8), (48, 9), (55, 15),
            (60, 22)]
HINGE = (36, 45)


def scale_cmds(cmds, s, tx, ty):
    def sc(x, y):
        return (x * s + tx, y * s + ty)
    out = []
    for c in cmds:
        k = c[0]
        if k in ("M", "L"):
            x, y = sc(c[1], c[2]); out.append((k, x, y))
        elif k == "C":
            p1 = sc(c[1], c[2]); p2 = sc(c[3], c[4]); p3 = sc(c[5], c[6])
            out.append((k, p1[0], p1[1], p2[0], p2[1], p3[0], p3[1]))
        elif k == "Q":
            p1 = sc(c[1], c[2]); p2 = sc(c[3], c[4])
            out.append((k, p1[0], p1[1], p2[0], p2[1]))
        else:
            out.append(c)
    return out


def shell(parent, x, y, s=1.0, body=ORANGE, outline=O100, ribs=O700,
          cup=True, metal=True, opacity=None, mark_scale=1.0):
    """Draw the scallop-shell chalice at (x,y) in the parent's coordinates."""
    f = create("frame", parent=parent, x=x, y=y, w=72 * s, h=72 * s,
               name="ShellMark")
    if opacity is not None:
        setp(f, opacity=opacity)
    # --- the shell -------------------------------------------------------
    sh = scale_cmds(SHELL_BASE, s, 0, 0)
    shell_id = vpath(f, sh, 0, 0, fill=body)
    if body != CLEAR:
        grad(shell_id, [a(O300, 255), ORANGE, O700], kind="linearGradient",
             frm=[0, 0], to=[1, 1])
    setp(shell_id, strokes=[{"type": "solid", "color": a(outline, 120)
                              if outline != CLEAR else outline}],
         strokeWeight=0.9 * s)

    # ribs, radiating from the hinge
    hx, hy = HINGE[0] * s, HINGE[1] * s
    for (rx, ry) in RIB_BASE:
        ex, ey = rx * s, ry * s
        ex = hx + (ex - hx) * 0.84
        ey = hy + (ey - hy) * 0.84
        vpath(f, [("M", hx, hy), ("L", ex, ey)], 0, 0, stroke=ribs,
              sw=1.0 * s, opacity=0.8)
    # one growth arc across the fan
    arc_pts = scale_cmds([("M", 22, 34), ("Q", 36, 21, 50, 34)], s, 0, 0)
    vpath(f, arc_pts, 0, 0, stroke=ribs, sw=0.8 * s, opacity=0.4)

    if cup:
        # --- the cup: a metal bowl holding the shell ---------------------
        cup_path = scale_cmds([
            ("M", 14, 33), ("L", 58, 33),
            ("C", 56, 42, 52, 47, 45, 48),
            ("L", 27, 48),
            ("C", 20, 47, 16, 42, 14, 33), ("Z",),
        ], s, 0, 0)
        cup_id = vpath(f, cup_path, 0, 0, fill=SLATE)
        grad(cup_id, [a(INK, 155), MUTED, SLATE, a(SLATE, 205)],
             frm=[0, 0], to=[0, 1])
        setp(cup_id, effects=[inner(a(GROUND, 150), [0, -3], 5)])
        # the lip catches the light
        vpath(f, scale_cmds([("M", 14, 33.4), ("L", 58, 33.4)], s, 0, 0),
              0, 0, stroke=INK, sw=1.3 * s, opacity=0.55)

        # --- the stem: brushed aluminium --------------------------------
        stem = scale_cmds([("M", 29, 47), ("L", 43, 47), ("L", 40, 61),
                            ("L", 32, 61), ("Z",)], s, 0, 0)
        stem_id = vpath(f, stem, 0, 0, fill=SLATE)
        grad(stem_id, [SLATE, MUTED, a(INK, 215), MUTED, SLATE],
             frm=[0, 0], to=[1, 0])

        # --- the foot ----------------------------------------------------
        foot = scale_cmds([("M", 20, 67), ("L", 52, 67), ("L", 48, 60),
                           ("L", 24, 60), ("Z",)], s, 0, 0)
        foot_id = vpath(f, foot, 0, 0, fill=SLATE)
        grad(foot_id, [SLATE, MUTED, a(INK, 235), MUTED, SLATE],
             frm=[0, 0], to=[1, 0])
        # shadow the foot casts on the ground
        sh_id = create("ellipse", parent=f, x=24 * s, y=66 * s,
                       w=24 * s, h=3.4 * s)
        paint(sh_id, fill=a(GROUND, 190))
        setp(sh_id, effects=[blur(3)])
    return f


# -------------------------------------------------------------- symbols ---
def symbol(parent, x, y, kind, size=13, color=INK, opacity=1.0):
    """Chart symbols: one per source, drawn in a size x size box."""
    s = size
    if kind == "square":
        i = create("rectangle", parent=parent, x=x, y=y, w=s, h=s)
        paint(i, fill=color, opacity=opacity)
    elif kind == "roundsquare":
        i = create("rectangle", parent=parent, x=x, y=y, w=s, h=s)
        paint(i, fill=color, opacity=opacity, radius=2.5)
    elif kind == "diamond":
        pts = [("M", s / 2, 0), ("L", s, s / 2), ("L", s / 2, s), ("L", 0, s / 2),
               ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity)
    elif kind == "triangle":
        pts = [("M", s / 2, 0), ("L", s, s), ("L", 0, s), ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity)
    elif kind == "hexagon":
        r = s / 2
        pts = [("M", r, 0), ("L", s, r * 0.55), ("L", s, r * 1.45),
               ("L", r, s), ("L", 0, r * 1.45), ("L", 0, r * 0.55), ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity)
    elif kind == "ring":
        ring(parent, x + s / 2, y + s / 2, s / 2, color, 1.4, opacity=opacity)
    elif kind == "pennant":
        pts = [("M", 0, s), ("L", 0, 0), ("L", s, s * 0.32), ("L", 0, s * 0.6),
               ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity)
    elif kind == "disc":            # ported — installed
        i = create("ellipse", parent=parent, x=x, y=y, w=s, h=s)
        paint(i, fill=color, opacity=opacity)
    elif kind == "open":             # available
        ring(parent, x + s / 2, y + s / 2, s / 2 - 0.7, color, 1.3,
             opacity=opacity)
    elif kind == "stale":            # out of date
        ring(parent, x + s / 2, y + s / 2, s / 2 - 0.7, color, 1.5,
             opacity=opacity)
        i = create("ellipse", parent=parent, x=x + s * 0.28, y=y + s * 0.28,
                   w=s * 0.44, h=s * 0.44)
        paint(i, fill=color, opacity=opacity)
    elif kind == "held":              # held back by the bearing
        ring(parent, x + s / 2, y + s / 2, s / 2 - 0.8, color, 1.1,
             opacity=opacity)
        vpath(parent, [("M", x + 1.6, y + s - 1.6), ("L", x + s - 1.6, y + 1.6)],
              0, 0, stroke=color, sw=1.1, opacity=opacity)
    elif kind == "recipe":           # a recipe, not a binary
        pts = [("M", s / 2, 0), ("L", s, s / 2), ("L", s / 2, s), ("L", 0, s / 2),
               ("Z",)]
        vpath(parent, pts, x, y, stroke=color, sw=1.2, opacity=opacity)
    return None


def pill(parent, x, y, w, h, label, color=INK, border=SLATE, bg=None,
         size=8.5, ls="6%", family=SANS, align="center"):
    f = create("frame", parent=parent, x=x, y=y, w=w, h=h,
               props={"layout": "horizontal", "mainAlign": "center",
                      "crossAlign": "center", "padding": 0, "gap": 0})
    paint(f, fill=bg if bg else CLEAR, stroke=border, sw=0.9, radius=h / 2)
    txt(f, label, 0, 0, size=size, family=family, color=color, ls=ls,
        lh="100%")
    return f


# =========================================================== the drawing ===
def build():
    # ---------------------------------------------------------- ground ----
    root = create("frame", x=0, y=0, w=W, h=H, name="Calix — La Carte")
    paint(root, fill=GROUND)
    ground = create("rectangle", parent=root, x=0, y=0, w=W, h=H,
                    name="Ground")
    setp(ground, fills=[{"type": "solid", "color": GROUND},
                        {"type": "solid", "color": GROUND}])

    # a slow sea-light behind the chart, and its falloff
    halo = create("ellipse", parent=root, x=180, y=-260, w=1500, h=1100)
    grad(halo, [a(SLATE, 60), a(SLATE, 0)], kind="radialGradient")
    halo2 = create("ellipse", parent=root, x=980, y=520, w=900, h=700)
    grad(halo2, [a(BLUE, 26), a(BLUE, 0)], kind="radialGradient")

    # ---------------------------------------------------------- header ----
    head = create("frame", parent=root, x=0, y=0, w=W, h=HEADER_H,
                  name="Header")
    paint(head, fill=a(INK, 12), stroke=SLATE, sw=1.0, sides=[0,0,1,0],
          effects=[shadow(a(GROUND, 235), [0, 8], 26)])
    rim = create("rectangle", parent=head, x=0, y=0, w=W, h=1)
    paint(rim, fill=a(INK, 34))

    # the mark
    shell(head, 26, 12, s=1.0)

    txt(head, "CALIX", 112, 32, size=27, style="SemiBold", color=INK, ls=6.5)
    seg(head, 246, 28, 246, 68, a(SLATE, 150), 1.0)

    # --- the bearing: a compass rose that steers ------------------------
    rose = create("frame", parent=head, x=264, y=2, w=88, h=88, name="Compass")
    ring(rose, 44, 44, 34, a(SLATE, 150), 1.0)
    ring(rose, 44, 44, 28, a(SLATE, 70), 0.8)
    for i in range(16):
        ang = i * 22.5
        r0 = 30.5 if i % 4 else 27.5
        p1 = (44 + r0 * math.sin(math.radians(ang)),
              44 - r0 * math.cos(math.radians(ang)))
        p2 = (44 + 33 * math.sin(math.radians(ang)),
              44 - 33 * math.cos(math.radians(ang)))
        vpath(rose, [("M", p1[0], p1[1]), ("L", p2[0], p2[1])], 0, 0,
              stroke=a(SLATE, 170 if i % 4 == 0 else 90),
              sw=1.0 if i % 4 == 0 else 0.8)

    bearings = [(0, "1", True), (45, "5", False), (90, "2", False),
                (180, "3", False), (270, "4", False)]
    for ang, num, active in bearings:
        a_ = math.radians(ang)
        px = 44 + 20 * math.sin(a_)
        py = 44 - 20 * math.cos(a_)
        lx = 44 + 39 * math.sin(a_)
        ly = 44 - 39 * math.cos(a_)
        if active:
            ring(rose, px, py, 5.6, ORANGE, 1.3)
            i = create("ellipse", parent=rose, x=px - 2.8, y=py - 2.8, w=5.6,
                       h=5.6)
            paint(i, fill=ORANGE)
            txt(rose, num, lx - 5, ly - 5, size=8.5, family=MONO, color=O300,
                align="center", w=10)
        else:
            i = create("ellipse", parent=rose, x=px - 2.3, y=py - 2.3, w=4.6,
                       h=4.6)
            paint(i, fill=B300, opacity=0.9)
            txt(rose, num, lx - 5, ly - 5, size=8.5, family=MONO,
                color=a(B300, 160), align="center", w=10)

    # the turn, caught mid-flight: a ghost needle and the arc it swept
    arc(rose, 44, 44, 14, 0, 90, a(ORANGE, 110), 1.0)
    vpath(rose, [("M", 56, 44), ("L", 48.5, 42.2), ("L", 48.5, 45.8), ("Z",)],
          0, 0, fill=a(ORANGE, 95))

    # the needle itself — bearing 000
    vpath(rose, [("M", 44, 15), ("L", 40.7, 45), ("L", 47.3, 45), ("Z",)],
          0, 0, fill=ORANGE)
    hub = create("ellipse", parent=rose, x=40.6, y=40.6, w=6.8, h=6.8)
    paint(hub, fill=ORANGE)

    txt(head, "BEARING", 366, 24, size=9.5, color=MUTED, ls="14%")
    txt(head, "000°", 366, 38, size=23, family=MONO, color=INK)
    txt(head, "ALL LANDMARKS", 428, 45, size=14, color=INK, ls="8%")
    txt(head, "⌘1…⌘5 TURN THE CHART · ⌥← ⌥→ STEER THE TERRITORY",
        366, 70, size=9, color=MUTED, ls="4%")

    txt(head, "TERRITORY", 640, 24, size=9.5, color=MUTED, ls="14%")
    txt(head, "ALPM", 640, 36, size=23, color=INK, ls="10%")
    txt(head, "OFFICIAL REPOSITORIES", 734, 43, size=12, color=MUTED, ls="6%")
    txt(head, "core · extra · multilib", 640, 70, size=10.5, family=MONO,
        color=MUTED)

    txt(head, "9,812 LANDMARKS ON CHART", 1226, 24, size=12.5, color=INK,
        ls="5%", align="right", w=350)
    txt(head, "CHARTED 11:42 · ALPM SYNCED 4 MIN AGO", 1226, 44, size=9.5,
        family=MONO, color=MUTED, align="right", w=350)
    txt(head, "⌘1…⌘5 BEARINGS · ⌥← ⌥→ TERRITORIES · ESC BEARING RETURN",
        1226, 64, size=9, color=MUTED, ls="3%", align="right", w=350)

    # ------------------------------------------------------ the legend ----
    rail = create("frame", parent=root, x=0, y=HEADER_H, w=RAIL_W,
                  h=H - HEADER_H, name="Legend")
    paint(rail, fill=a(INK, 11), stroke=SLATE, sw=1.0, sides=[0,1,0,0],
          effects=[shadow(a(GROUND, 200), [12, 0], 26)])
    rail_lip = create("rectangle", parent=rail, x=0, y=0, w=RAIL_W, h=1)
    paint(rail_lip, fill=a(INK, 34))

    txt(rail, "CHART LEGEND", 22, 20, size=11, color=INK, ls="18%")
    txt(rail, "BEARING 000° REACHES EVERY TERRITORY — NOTHING HELD BACK",
        22, 38, size=8.5, color=MUTED, ls="1%")
    seg(rail, 22, 56, 294, 56, a(SLATE, 90), 1.0)

    territories = [
        ("ALPM", "square", "9,812", 1204, 37, "core · extra · multilib", True),
        ("AUR", "diamond", "92,431", 1860, 412,
         "user-contributed recipes — no binaries", False),
        ("FLATPAK", "ring", "2,014", 68, 6, "sandboxed applications", False),
        ("APPIMAGE", "roundsquare", "1,873", 12, 9,
         "self-contained binaries", False),
        ("CARGO", "hexagon", "148,206", 34, 21, "crates from crates.io", False),
        ("NPM", "triangle", "3,290,441", 187, 143, "global node packages", False),
        ("TRACKED", "pennant", "62", 21, 4, "binaries followed from git hosts",
         False),
    ]
    ty = 72
    for (nm, sym, total, ported, outdated, note, current) in territories:
        if current:
            band = create("rectangle", parent=rail, x=0, y=ty - 6, w=RAIL_W,
                          h=52)
            paint(band, fill=a(ORANGE, 16))
            bar = create("rectangle", parent=rail, x=0, y=ty - 6, w=2.5, h=52)
            paint(bar, fill=ORANGE)
        symbol(rail, 22, ty + 1, sym, 12, INK if current else MUTED,
               opacity=1.0 if current else 0.85)
        txt(rail, nm, 44, ty - 1, size=12 if current else 11.5,
            style="SemiBold" if current else None,
            color=INK if current else a(INK, 200), ls="6%")
        txt(rail, total, 196, ty - 1, size=10, family=MONO,
            color=INK if current else MUTED, align="right", w=98)
        txt(rail, note, 44, ty + 14, size=9, color=MUTED)
        # coverage: 18 cells, filled = the share that is ported
        cells = 18
        cw, ch, gap = 9, 4, 3
        share = ported / float(int(total.replace(",", "")))
        filled = share * cells
        for c in range(cells):
            cx = 44 + c * (cw + gap)
            cell = create("rectangle", parent=rail, x=cx, y=ty + 30, w=cw, h=ch)
            if c < int(filled):
                paint(cell, fill=ORANGE if current else a(MUTED, 235))
            elif c == int(filled) and filled - int(filled) > 0.02:
                part = max(1.2, cw * (filled - int(filled)))
                paint(cell, fill=CLEAR)
                seg_cell = create("rectangle", parent=rail, x=cx, y=ty + 30,
                                  w=part, h=ch)
                paint(seg_cell, fill=a(ORANGE, 170) if current
                      else a(MUTED, 130))
            else:
                paint(cell, fill=a(SLATE, 55))
        txt(rail, "%s PORTED · %s OUTDATED" % (format(ported, ","),
                                              format(outdated, ",")),
            44, ty + 38, size=8.5, family=MONO, color=MUTED)
        ty += 55

    seg(rail, 22, ty + 2, 294, ty + 2, a(SLATE, 90), 1.0)
    txt(rail, "CHART KEY", 22, ty + 14, size=9.5, color=MUTED, ls="16%")
    key = [("disc", "PORTED — ON THIS MACHINE", INK),
           ("open", "AVAILABLE — NOT INSTALLED", a(INK, 190)),
           ("stale", "OUTDATED — A NEWER BUILD IS CHARTED", O300),
           ("recipe", "RECIPE ONLY — NOTHING TO UPDATE", MUTED),
           ("held", "HELD BACK — OFF THIS BEARING (NONE AT 000°)", MUTED)]
    ky = ty + 32
    for (kind, label, col) in key:
        symbol(rail, 23, ky, kind, 11, col)
        txt(rail, label, 44, ky + 1, size=9, color=MUTED, ls="1%")
        ky += 18

    seg(rail, 22, ky + 4, 294, ky + 4, a(SLATE, 90), 1.0)
    txt(rail, "THE INKS", 22, ky + 16, size=9.5, color=MUTED, ls="16%")
    shell(rail, 22, ky + 28, s=0.44, body=ORANGE, outline=O100, ribs=O700)
    txt(rail, "SIGNAL ORANGE", 60, ky + 40, size=9, color=MUTED, ls="3%")
    shell(rail, 166, ky + 28, s=0.44, body=BLUE, outline=B300, ribs=BLUE)
    txt(rail, "ROYAL BLUE", 204, ky + 40, size=9, color=MUTED, ls="3%")

    seg(rail, 22, ky + 66, 294, ky + 66, a(SLATE, 90), 1.0)
    txt(rail, "HELM", 22, ky + 78, size=9.5, color=MUTED, ls="16%")
    txt(rail, "THE KEYBOARD IS THE HELM", 118, ky + 79, size=8, color=MUTED,
        align="right", w=176)
    helm = [("⌘1", "ALL LANDMARKS", True),
            ("⌘2", "INSTALLED PORTS", False),
            ("⌘3", "OUT OF DATE", False),
            ("⌘4", "FIND ON THE CHART", False),
            ("⌘5", "REMOVAL QUAY", False),
            ("⌥← ⌥→", "STEER THE TERRITORY", False),
            ("⌘F", "FIND", False),
            ("⌘L", "BACK TO THE LANDMARK LIST", False),
            ("⌘⌥C", "COPY THE INSTALL COMMAND", False),
            ("ESC", "BEARING RETURN", False)]
    hy = ky + 96
    for (key_t, act, active) in helm:
        row = create("frame", parent=rail, x=22, y=hy, w=272, h=17,
                     props={"layout": "horizontal", "gap": 8, "padding": 0,
                            "crossAlign": "center", "sizingH": "fixed",
                            "sizingV": "fixed"})
        txt(row, key_t, 0, 0, size=9.5, family=MONO,
            color=O300 if active else a(INK, 210), w=52)
        txt(row, act, 0, 0, size=9.5, color=MUTED if not active
            else a(INK, 230))
        hy += 19

    # ------------------------------------------------ the chart sheet ----
    sheet = create("frame", parent=root, x=SHEET_X, y=SHEET_Y, w=SHEET_W,
                   h=SHEET_H, name="ChartSheet")
    paint(sheet, fill=a(INK, 20), stroke=SLATE, sw=1.0, sides=[0,1,0,0],
          effects=[shadow(a(GROUND, 225), [0, 12], 32)])
    sheet_lip = create("rectangle", parent=sheet, x=0, y=0, w=SHEET_W, h=1)
    paint(sheet_lip, fill=a(INK, 40))

    # -- the ground of the chart: fog, graticule, contours, soundings ----
    #    (this layer is panned a little ahead of the landmarks: parallax)
    mapl = create("frame", parent=sheet, x=-7, y=3, w=SHEET_W, h=SHEET_H,
                  name="MapLayer")

    fog = create("rectangle", parent=mapl, x=-40, y=-80, w=980, h=200)
    grad(fog, [a(INK, 44), a(INK, 0)], frm=[0, 0], to=[0, 1])
    setp(fog, effects=[blur(34)])
    fog2 = create("ellipse", parent=mapl, x=430, y=-150, w=520, h=220)
    paint(fog2, fill=a(MUTED, 26), effects=[blur(40)])

    for gx in range(0, SHEET_W + 1, 147):
        seg(mapl, gx, 0, gx, SHEET_H, a(SLATE, 58), 1.0)
    for gy in range(0, SHEET_H + 1, 44):
        seg(mapl, 0, gy, SHEET_W, gy, a(SLATE, 38), 1.0)

    # two depth contours
    vpath(mapl, [("M", -20, 300), ("C", 180, 250, 300, 360, 520, 330),
                 ("C", 700, 305, 780, 380, 920, 350)], 0, 0,
          stroke=a(SLATE, 105), sw=1.1)
    vpath(mapl, [("M", 60, 620), ("C", 240, 560, 420, 660, 640, 610),
                 ("C", 780, 580, 840, 640, 940, 620)], 0, 0,
          stroke=a(SLATE, 80), sw=1.0)

    soundings = [(70, 120, "12.4"), (196, 238, "18"), (322, 96, "7.5"),
                 (430, 330, "24"), (556, 180, "9.2"), (668, 430, "31"),
                 (124, 500, "15"), (300, 560, "11"), (470, 640, "27"),
                 (612, 560, "19"), (790, 250, "22"), (700, 700, "13"),
                 (836, 620, "16")]
    for (sx, sy, sv) in soundings:
        txt(mapl, sv, sx, sy, size=9, family=MONO, color=a(MUTED, 70))

    # -- the strip: find, filters in force, count, scale -----------------
    search = create("frame", parent=sheet, x=24, y=14, w=230, h=28,
                    name="Find")
    paint(search, fill=a(INK, 14), stroke=a(SLATE, 120), sw=1.0, radius=6)
    ring(search, 17, 14, 4.6, MUTED, 1.2, opacity=0.8)
    vpath(search, [("M", 20.4, 17.4), ("L", 24, 21)], 0, 0, stroke=MUTED,
          sw=1.2, opacity=0.8)
    txt(search, "FIND A LANDMARK", 32, 9, size=9.5, color=MUTED, ls="7%")
    cap = create("frame", parent=search, x=184, y=5, w=36, h=18,
                 props={"layout": "horizontal", "mainAlign": "center",
                        "crossAlign": "center"})
    paint(cap, fill=a(INK, 12), stroke=a(SLATE, 110), sw=0.8, radius=4)
    txt(cap, "⌘F", 0, 0, size=9, family=MONO, color=MUTED, align="center",
        w=36, lh="100%")

    fx = 266
    for (label, active) in [("ALL REPOS", True), ("CORE", False),
                            ("EXTRA", False), ("MULTILIB", False)]:
        wch = 9 + len(label) * 5.6
        pill(sheet, fx, 17, wch, 20, label,
             color=O300 if active else MUTED,
             border=a(ORANGE, 170) if active else a(SLATE, 110),
             bg=a(ORANGE, 26) if active else CLEAR, size=8.5, ls="8%")
        fx += wch + 8

    txt(sheet, "9,812 ON CHART · 37 OUTDATED", 470, 18, size=9.5, family=MONO,
        color=MUTED, align="right", w=230)

    scale_ctl = create("frame", parent=sheet, x=712, y=14, w=140, h=24,
                       name="Scale")
    paint(scale_ctl, fill=a(INK, 12), stroke=a(SLATE, 110), sw=0.9, radius=5)
    seg_det = create("frame", parent=scale_ctl, x=2, y=2, w=78, h=20,
                     props={"layout": "horizontal", "mainAlign": "center",
                            "crossAlign": "center"})
    paint(seg_det, fill=a(ORANGE, 34), radius=4)
    txt(seg_det, "DETAILED", 0, 0, size=8.5, color=O300, ls="6%",
        align="center", w=78, lh="100%")
    txt(scale_ctl, "COMPACT", 84, 7, size=8.5, color=MUTED, ls="6%")

    # -- column headings -------------------------------------------------
    seg(sheet, 24, 82, 836, 82, a(SLATE, 90), 1.0)
    txt(sheet, "LANDMARK", 28, 60, size=8.5, color=MUTED, ls="16%")
    txt(sheet, "PROVENANCE", 350, 60, size=8.5, color=MUTED, ls="16%")
    txt(sheet, "SIZE", 500, 60, size=8.5, color=MUTED, ls="16%", align="right",
        w=90)
    txt(sheet, "STATE", 616, 60, size=8.5, color=MUTED, ls="16%")
    txt(sheet, "NOTE", 740, 60, size=8.5, color=MUTED, ls="16%")

    # -- the landmarks ---------------------------------------------------
    rows = [
        ("firefox", "extra/141.0-1", "78.4 MB", "OUTDATED",
         "Fast, private web browser from Mozilla — ESR build, sandboxed",
         "SIGNED", True),
        ("rust", "extra/1.80.0-1", "142 MB", "PORTED",
         "Rust language toolchain, Cargo package manager and rustc",
         "SIGNED", False),
        ("python", "core/3.12.7-1", "52.1 MB", "PORTED",
         "Python 3.12 interpreter — the system default for scripts", "SIGNED",
         False),
        ("nvim", "extra/0.10.2-1", "24.9 MB", "AVAILABLE",
         "Fork of Vim, built for extensibility — runs the same key map", "",
         False),
        ("gimp", "extra/2.10.38-1", "96.3 MB", "PORTED",
         "GNU image manipulation program — raster editor", "SIGNED", False),
        ("postgresql", "extra/17.2-1", "68.7 MB", "OUTDATED",
         "Relational database server — 17.4 is on the chart", "SIGNED", False),
        ("kitty", "extra/0.37.0-1", "12.4 MB", "PORTED",
         "Fast, GPU accelerated terminal emulator", "SIGNED", False),
        ("blender", "extra/4.1.1-3", "312 MB", "AVAILABLE",
         "3D creation suite — modelling, animation and render", "", False),
        ("docker", "extra/27.1.1-1", "41.2 MB", "AVAILABLE",
         "Container runtime for the Docker platform", "", False),
        ("wireguard-tools", "extra/1.0.20240722-1", "6.8 MB", "PORTED",
         "WireGuard network tunnel utilities", "SIGNED", False),
        ("man-db", "core/2.12.0-1", "4.8 MB", "PORTED",
         "Display the manual page for any command on the system", "SIGNED",
         False),
    ]
    ry = 90
    for (nm, prov, size, state, desc, tag, sel) in rows:
        row = create("frame", parent=sheet, x=24, y=ry, w=836, h=66,
                     name="Row " + nm)
        if sel:
            grad(row, [a(ORANGE, 40), a(ORANGE, 8)], frm=[0, 0], to=[1, 0])
            bar = create("rectangle", parent=row, x=0, y=0, w=2, h=66)
            paint(bar, fill=ORANGE)
            glow = create("ellipse", parent=row, x=-6, y=6, w=34, h=34)
            paint(glow, fill=a(ORANGE, 60), effects=[blur(13)])
            ring(row, 10, 20, 11, ORANGE, 1.5)
            ring(row, 8.6, 18.6, 14.5, a(ORANGE, 90), 1.0)
        else:
            paint(row, fill=CLEAR)
        glyph = {"PORTED": "disc", "OUTDATED": "stale",
                 "AVAILABLE": "open"}[state]
        symbol(row, 4, 14, glyph, 12,
               INK if state == "PORTED" else (O300 if state == "OUTDATED"
                                             else a(INK, 170)))
        txt(row, nm, 28, 9, size=15, style="SemiBold", color=INK)
        txt(row, desc, 28, 33, size=11.5, color=MUTED)
        txt(row, prov, 350, 14, size=10.5, family=MONO, color=MUTED)
        txt(row, size, 500, 14, size=10.5, family=MONO, color=MUTED,
            align="right", w=90)
        if state == "OUTDATED":
            pill(row, 616, 9, 112, 22, "OUTDATED", color=O300,
                 border=a(ORANGE, 180), bg=a(ORANGE, 30), size=8.5, ls="6%")
        elif state == "PORTED":
            pill(row, 616, 9, 112, 22, "PORTED", color=a(INK, 190),
                 border=a(SLATE, 130), bg=a(INK, 12), size=8.5, ls="6%")
        else:
            pill(row, 616, 9, 112, 22, "AVAILABLE", color=a(INK, 150),
                 border=a(SLATE, 90), bg=CLEAR, size=8.5, ls="6%")
        txt(row, tag, 740, 15, size=8.5, family=MONO, color=a(MUTED, 170),
            ls="4%")
        if state == "OUTDATED":
            txt(row, "140.0-1 → 141.0-1", 736, 31,
                size=9, family=MONO, color=O300, ls="1%")
        rule = create("rectangle", parent=row, x=28, y=64, w=808, h=1)
        paint(rule, fill=a(SLATE, 55))
        ry += 66


    # -- the scroll, still settling -------------------------------------
    track = create("rectangle", parent=sheet, x=871, y=96, w=3, h=660)
    paint(track, fill=a(INK, 22))
    trail = create("rectangle", parent=sheet, x=871, y=132, w=3, h=120)
    paint(trail, fill=a(INK, 38))
    thumb = create("rectangle", parent=sheet, x=871, y=104, w=3, h=120)
    paint(thumb, fill=a(INK, 120))

    # ---------------------------------------------------- the chart card ---
    card = create("frame", parent=root, x=CARD_X, y=SHEET_Y, w=CARD_W,
                  h=SHEET_H, name="ChartCard")
    paint(card, fill=a(INK, 22), stroke=SLATE, sw=1.0, sides=[1,0,0,0],
          effects=[shadow(a(GROUND, 230), [-14, 0], 34)])
    card_lip = create("rectangle", parent=card, x=0, y=0, w=CARD_W, h=1)
    paint(card_lip, fill=a(INK, 40))
    crest = create("rectangle", parent=card, x=0, y=0, w=2, h=CARD_H)
    paint(crest, fill=a(O100, 120))
    seg(card, 24, 40, 376, 40, a(SLATE, 110), 1.0)

    txt(card, "CHART CARD", 24, 20, size=9.5, color=MUTED, ls="18%")
    txt(card, "⌘⌥C COPY COMMAND", 226, 21, size=8.5, family=MONO,
        color=MUTED, align="right", w=150)
    txt(card, "firefox", 24, 44, size=27, style="SemiBold", color=INK, ls="-1%")
    symbol(card, 24, 82, "square", 11, a(INK, 190))
    txt(card, "MOZILLA FIREFOX · WEB BROWSER", 44, 83, size=9, color=MUTED,
        ls="8%")

    txt(card, "VERSION", 24, 116, size=8.5, color=MUTED, ls="16%")
    txt(card, "140.0-1", 24, 132, size=20, family=MONO, color=INK)
    txt(card, "→ 141.0-1", 116, 138, size=13, family=MONO, color=O300)
    pill(card, 264, 116, 112, 22, "OUTDATED", color=O300,
         border=a(ORANGE, 180), bg=a(ORANGE, 30), size=8.5, ls="6%")

    txt(card, "PROVENANCE", 24, 180, size=8.5, color=MUTED, ls="16%")
    txt(card, "ALPM  ▸  extra  ▸  141.0-1", 24, 196, size=11.5, family=MONO,
        color=INK)
    txt(card, "SIGNED BY THE ALPM OFFICIAL KEY (SHA256 9F4C…B21)", 24, 214,
        size=9.5, family=MONO, color=MUTED)
    txt(card, "BINARY FROM THE OFFICIAL REPOSITORY — NO RECIPE TO BUILD",
        24, 232, size=9, color=MUTED, ls="2%")

    # the actions this source and bearing allow
    up = create("frame", parent=card, x=24, y=256, w=134, h=30,
                props={"layout": "horizontal", "mainAlign": "center",
                       "crossAlign": "center"})
    paint(up, fill=ORANGE, radius=5)
    txt(up, "UPDATE TO 141.0", 0, 0, size=10, style="SemiBold", color=INK,
        ls="3%", align="center", w=134, lh="100%")
    rf = create("frame", parent=card, x=166, y=256, w=84, h=30,
                props={"layout": "horizontal", "mainAlign": "center",
                       "crossAlign": "center"})
    paint(rf, fill=a(INK, 10), stroke=a(SLATE, 140), sw=1.0, radius=5)
    txt(rf, "REFRESH", 0, 0, size=10, color=a(INK, 220), ls="4%",
        align="center", w=84, lh="100%")
    rm = create("frame", parent=card, x=258, y=256, w=82, h=30,
                props={"layout": "horizontal", "mainAlign": "center",
                       "crossAlign": "center"})
    paint(rm, fill=CLEAR, stroke=a(SLATE, 100), sw=1.0, radius=5)
    txt(rm, "REMOVE", 0, 0, size=10, color=MUTED, ls="4%", align="center",
        w=82, lh="100%")

    txt(card, "DEPENDENCIES", 24, 314, size=8.5, color=MUTED, ls="16%")
    txt(card, "4 RESOLVED", 226, 315, size=8.5, family=MONO, color=MUTED,
        align="right", w=150)
    deps = [("gtk3", "3.24.38"), ("glib2", "2.80"), ("dbus", "1.14"),
            ("ffmpeg6.0", "6.0.1")]
    dy = 332
    for (d, dv) in deps:
        txt(card, d, 24, dy, size=10.5, family=MONO, color=a(INK, 200))
        txt(card, dv, 226, dy, size=10.5, family=MONO, color=MUTED,
            align="right", w=150)
        dy += 18

    txt(card, "FILES", 24, 416, size=8.5, color=MUTED, ls="16%")
    txt(card, "1,842 FILES", 226, 417, size=8.5, family=MONO, color=MUTED,
        align="right", w=150)
    files = ["/usr/bin/firefox", "/usr/lib/firefox/libxul.so",
             "/usr/share/applications/firefox.desktop"]
    fy = 434
    for fl in files:
        txt(card, fl, 24, fy, size=10, family=MONO, color=MUTED)
        fy += 17

    txt(card, "INSTALL COMMAND", 24, 504, size=8.5, color=MUTED, ls="16%")
    txt(card, "ALPM ▸ PACMAN", 226, 505, size=8.5, family=MONO, color=MUTED,
        align="right", w=150)
    cmd_plate = create("frame", parent=card, x=24, y=522, w=352, h=34)
    paint(cmd_plate, fill=a(GROUND, 190), stroke=a(SLATE, 130), sw=1.0,
          radius=6)
    txt(cmd_plate, "$ pacman -Syu --needed firefox", 12, 10, size=11.5,
        family=MONO, color=INK)
    cap2 = create("frame", parent=cmd_plate, x=296, y=5, w=44, h=24,
                  props={"layout": "horizontal", "mainAlign": "center",
                         "crossAlign": "center"})
    paint(cap2, fill=a(ORANGE, 30), radius=4)
    txt(cap2, "⌘⌥C", 0, 0, size=9, family=MONO, color=O300, align="center",
        w=44, lh="100%")

    seg(card, 24, 574, 376, 574, a(SLATE, 90), 1.0)
    txt(card, "ALSO IN ALPM", 24, 588, size=8.5, color=MUTED, ls="16%")
    txt(card, "COMPACT SCALE", 226, 589, size=8.5, family=MONO, color=MUTED,
        align="right", w=150)
    compact = [("kitty", "12.4 MB", "PORTED"), ("nvim", "24.9 MB", "AVAILABLE"),
               ("zsh", "6.1 MB", "PORTED"), ("man-db", "4.8 MB", "PORTED")]
    cy = 608
    for (nm, sz, st) in compact:
        symbol(card, 24, cy + 2, {"PORTED": "disc",
                                  "AVAILABLE": "open",
                                  "OUTDATED": "stale"}[st], 9,
               INK if st == "PORTED" else a(INK, 170))
        txt(card, nm, 42, cy, size=11, color=a(INK, 225))
        txt(card, sz, 226, cy + 1, size=10, family=MONO, color=MUTED,
            align="right", w=150)
        cy += 22
        if cy < 700:
            seg(card, 24, cy - 4, 376, cy - 4, a(SLATE, 65), 1.0)

    txt(card, "COMPACT SCALE — THE SAME ROWS, THE SAME COLUMNS, LESS OF THEM",
        24, 692, size=8.5, color=MUTED, ls="2%")
    txt(card, "⌘L RETURNS TO THE LANDMARK LIST · ESC RETURNS THE BEARING",
        24, 708, size=8.5, color=MUTED, ls="2%")
    seg(card, 24, 730, 376, 730, a(SLATE, 90), 1.0)
    txt(card, "NOT IN THIS TERRITORY", 24, 740, size=8.5, color=MUTED,
        ls="16%")
    txt(card, "There is no build recipe here. The AUR is where recipes live —",
        24, 754, size=10, color=MUTED, lh="150%")
    txt(card, "steer there with ⌥← ⌥→, or turn the bearing with ⌘1…⌘5.",
        24, 768, size=10, color=MUTED, lh="150%")

    # ------------------------------------------------------- the console ---
    con = create("frame", parent=root, x=RAIL_W, y=CONSOLE_Y, w=W - RAIL_W,
                 h=CONSOLE_H, name="Console")
    paint(con, fill=a(INK, 12), stroke=SLATE, sw=1.0, sides=[1,0,0,0],
          effects=[shadow(a(GROUND, 200), [0, -10], 24)])
    seg(con, 24, 34, 1244, 34, a(SLATE, 90), 1.0)
    txt(con, "OPERATIONS IN FLIGHT", 24, 16, size=9.5, color=MUTED, ls="18%")
    txt(con, "1 IN FLIGHT · 1 QUEUED · 0 FAILED", 1014, 17, size=9.5,
        family=MONO, color=MUTED, align="right", w=230)

    # the seal on the margin
    shell(con, 1112, 22, s=1.15, body=CLEAR, outline=INK, ribs=INK,
          cup=False, opacity=0.1)

    spin = create("ellipse", parent=con, x=24, y=48, w=13, h=13)
    paint(spin, fill=CLEAR, stroke=ORANGE, sw=1.6)
    setp(spin, arcStart=40, arcSweep=250)
    txt(con, "INSTALLING", 48, 47, size=8.5, color=O300, ls="10%")
    txt(con, "firefox 141.0-1", 116, 44, size=12.5, color=INK)
    bar_bg = create("rectangle", parent=con, x=280, y=53, w=220, h=3)
    paint(bar_bg, fill=a(INK, 26))
    bar = create("rectangle", parent=con, x=280, y=53, w=135, h=3)
    paint(bar, fill=ORANGE)
    head2 = create("rectangle", parent=con, x=405, y=51, w=18, h=7)
    paint(head2, fill=O300, effects=[blur(3)])
    txt(con, "48.2 / 78.4 MB", 516, 46, size=10, family=MONO, color=MUTED)
    txt(con, "0:04", 620, 46, size=10, family=MONO, color=MUTED)

    txt(con, "QUEUED", 48, 78, size=8.5, color=MUTED, ls="10%")
    txt(con, "rust 1.79.0 → 1.80.0", 116, 75, size=12.5, color=a(INK, 210))
    txt(con, "waits for the install to finish", 280, 78, size=10,
        family=MONO, color=MUTED)

    # -- the weave of the ground -----------------------------------------
    # a 16 px ruled tile, laid over the whole ground, so the texture is
    # there for anyone who looks for it and invisible to anyone who does not
    weave_src = create("frame", parent=root, x=1660, y=0, w=16, h=16,
                       name="WeaveTile")
    paint(weave_src, fill=CLEAR)
    wv = create("rectangle", parent=weave_src, x=0, y=0, w=1, h=16)
    paint(wv, fill=INK)
    wh = create("rectangle", parent=weave_src, x=0, y=0, w=16, h=1)
    paint(wh, fill=INK)
    add("fill.pattern", {"source": S(weave_src), "ids": [S(ground)],
                          "scale": 1, "spacing": [16, 16], "opacity": 0.07})
    return root


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    build()
    out = os.path.abspath(args.out)
    with open(out + ".jsonl", "w") as fh:
        for c in CMDS:
            fh.write(json.dumps(c) + "\n")
    print("%d steps -> %s.jsonl" % (len(CMDS), out))


if __name__ == "__main__":
    main()
