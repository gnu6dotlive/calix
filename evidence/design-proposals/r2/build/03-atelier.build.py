#!/usr/bin/env python3
"""Calix — L'Atelier: a package manager as a workbench.

Builds 03-atelier.figcraft with two frames:
    03-atelier-loaded   the bench with work on it
    03-atelier-quiet    the bench the moment it is opened, empty

    python3 evidence/design-proposals/r2/build/03-atelier.build.py --out .../03-atelier.figcraft
    figcraft-cli run 03-atelier.build.jsonl --out 03-atelier.figcraft
    figcraft-cli export 03-atelier.figcraft -o out/ --scale 2
"""
import argparse
import json
import math
import os

# ---------------------------------------------------------------- tokens ----
GROUND = "#111418"   # anthracite — the ground
INK    = "#F7F3ED"   # shell white — the ink, and the warmth of every surface
MUTED  = "#A1A6AD"   # secondary text
SLATE  = "#667085"   # hairlines, the stem
O100   = "#FFE5D2"   # signal orange 100 — rim light
O300   = "#FF8E40"   # signal orange 300 — hover
ORANGE = "#FF6A00"   # signal orange — the accent
O700   = "#C95400"   # signal orange 700 — the shadow side
B300   = "#87A0F4"   # royal blue 300
BLUE   = "#4169E1"   # royal blue

SANS = "IBM Plex Sans"          # the workhorse
MONO = "IBM Plex Mono"          # figures, commands, stencilled names
CLEAR = "#FFFFFF00"

W, H = 1600, 1000
BACK_Y, BACK_H = 0, 64        # the backboard: identity and freshness
RACK_Y, RACK_H = 64, 134      # the rack of seven trays
MAT_Y, MAT_H = 210, 620       # the mat — the work surface
RAIL_Y, RAIL_H = 844, 148     # the near rail — the log
MARGIN = 20


def a(col, alpha):
    return col + "%02X" % alpha


# --------------------------------------------------------------- engine ----
CMDS = []


def add(cmd, params):
    CMDS.append({"command": cmd, "params": params})
    return len(CMDS) - 1


def S(i):
    return {"$step": i}


def create(kind, parent=None, x=0, y=0, w=None, h=None, name=None, props=None,
           **kw):
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
        setp(idx, fills=[{"type": "solid", "color": CLEAR}])
    return idx


def setp(idx, **kw):
    add("node.set", {"ids": [S(idx)], **kw})


def paint(idx, fill=None, stroke=None, sw=1.0, effects=None, opacity=None,
          radius=None, sides=None, dashes=None, rotation=None):
    kw = {}
    if fill is not None:
        kw["fills"] = [{"type": "solid", "color": fill}]
    if stroke is not None:
        kw["strokes"] = [{"type": "solid", "color": stroke}]
        kw["strokeWeight"] = sw
        if sides:
            kw["strokeSides"] = sides
        if dashes:
            kw["dashes"] = dashes
    if effects:
        kw["effects"] = effects
    if opacity is not None:
        kw["opacity"] = opacity
    if radius is not None:
        kw["cornerRadius"] = radius
    if rotation is not None:
        kw["rotation"] = rotation
    setp(idx, **kw)


def grad(idx, stops, kind="linearGradient", frm=None, to=None, **kw):
    pd = {"type": kind, "stops": stops}
    if frm is not None:
        pd["from"] = frm
    if to is not None:
        pd["to"] = to
    kw["fills"] = [pd]
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
        align=None, w=None, lh=None, case=None, opacity=None, rotation=None,
        name=None):
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
    if rotation is not None:
        props["rotation"] = rotation
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
        if k in ("M", "L"):
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


def vpath(parent, cmds, x=0, y=0, fill=None, stroke=None, sw=1.0, opacity=None,
          rotation=None, dashes=None):
    bx, by, _, _ = path_bbox(cmds)
    norm = []
    for c in cmds:
        k = c[0]
        if k in ("M", "L"):
            norm.append((k, c[1] - bx, c[2] - by))
        elif k == "C":
            norm.append((k, c[1] - bx, c[2] - by, c[3] - bx, c[4] - by,
                         c[5] - bx, c[6] - by))
        elif k == "Q":
            norm.append((k, c[1] - bx, c[2] - by, c[3] - bx, c[4] - by))
        elif k == "A":
            norm.append((k, c[1], c[2], c[3], c[4], c[5], c[5] - bx, c[6] - by))
        else:
            norm.append(c)
    p = {"type": "vector", "parent": S(parent), "x": x + bx, "y": y + by,
         "path": path_of(norm)}
    idx = add("node.create", p)
    kw = {"fills": [{"type": "solid", "color": fill if fill else CLEAR}]}
    if stroke:
        kw["strokes"] = [{"type": "solid", "color": stroke}]
        kw["strokeWeight"] = sw
        if dashes:
            kw["dashes"] = dashes
    if opacity is not None:
        kw["opacity"] = opacity
    if rotation is not None:
        kw["rotation"] = rotation
    setp(idx, **kw)
    return idx


def seg(parent, x1, y1, x2, y2, stroke, sw=1.0, opacity=None, dashes=None):
    return vpath(parent, [("M", x1, y1), ("L", x2, y2)], 0, 0, stroke=stroke,
                 sw=sw, opacity=opacity, dashes=dashes)


def ring(parent, cx, cy, r, stroke, sw=1.0, fill=None, opacity=None,
         rotation=None, dashes=None):
    idx = create("ellipse", parent=parent, x=cx - r, y=cy - r, w=r * 2, h=r * 2)
    kw = {"fills": [{"type": "solid", "color": fill if fill else CLEAR}],
          "strokes": [{"type": "solid", "color": stroke}], "strokeWeight": sw}
    if opacity is not None:
        kw["opacity"] = opacity
    if rotation is not None:
        kw["rotation"] = rotation
    if dashes:
        kw["dashes"] = dashes
    setp(idx, **kw)
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


def shell(parent, x, y, s=1.0, body=ORANGE, outline=O100, ribs=O700, cup=True,
          opacity=None):
    """The scallop shell as the cup of a chalice."""
    f = create("frame", parent=parent, x=x, y=y, w=72 * s, h=72 * s,
               name="ShellMark")
    if opacity is not None:
        setp(f, opacity=opacity)
    sh = scale_cmds(SHELL_BASE, s, 0, 0)
    sid = vpath(f, sh, 0, 0, fill=body)
    if body != CLEAR:
        grad(sid, [a(O300, 255), ORANGE, O700], frm=[0, 0], to=[1, 1])
    setp(sid, strokes=[{"type": "solid", "color": a(outline, 120)
                        if outline != CLEAR else outline}], strokeWeight=0.9 * s)
    hx, hy = HINGE[0] * s, HINGE[1] * s
    for (rx, ry) in RIB_BASE:
        ex, ey = rx * s, ry * s
        ex = hx + (ex - hx) * 0.84
        ey = hy + (ey - hy) * 0.84
        vpath(f, [("M", hx, hy), ("L", ex, ey)], 0, 0, stroke=ribs, sw=1.0 * s,
              opacity=0.8)
    vpath(f, scale_cmds([("M", 22, 34), ("Q", 36, 21, 50, 34)], s, 0, 0), 0, 0,
          stroke=ribs, sw=0.8 * s, opacity=0.4)
    if cup:
        cup_path = scale_cmds([
            ("M", 14, 33), ("L", 58, 33),
            ("C", 56, 42, 52, 47, 45, 48), ("L", 27, 48),
            ("C", 20, 47, 16, 42, 14, 33), ("Z",)], s, 0, 0)
        cid = vpath(f, cup_path, 0, 0, fill=SLATE)
        grad(cid, [a(INK, 155), MUTED, SLATE, a(SLATE, 205)], frm=[0, 0],
             to=[0, 1])
        setp(cid, effects=[inner(a(GROUND, 150), [0, -3], 5)])
        vpath(f, scale_cmds([("M", 14, 33.4), ("L", 58, 33.4)], s, 0, 0), 0, 0,
              stroke=INK, sw=1.3 * s, opacity=0.55)
        stem = scale_cmds([("M", 29, 47), ("L", 43, 47), ("L", 40, 61),
                            ("L", 32, 61), ("Z",)], s, 0, 0)
        stid = vpath(f, stem, 0, 0, fill=SLATE)
        grad(stid, [SLATE, MUTED, a(INK, 215), MUTED, SLATE], frm=[0, 0],
             to=[1, 0])
        foot = scale_cmds([("M", 20, 67), ("L", 52, 67), ("L", 48, 60),
                           ("L", 24, 60), ("Z",)], s, 0, 0)
        fid = vpath(f, foot, 0, 0, fill=SLATE)
        grad(fid, [SLATE, MUTED, a(INK, 235), MUTED, SLATE], frm=[0, 0],
             to=[1, 0])
        sh_id = create("ellipse", parent=f, x=24 * s, y=66 * s, w=24 * s,
                       h=3.4 * s)
        paint(sh_id, fill=a(GROUND, 190))
        setp(sh_id, effects=[blur(3)])
    return f


# -------------------------------------------------------------- symbols ----
def symbol(parent, x, y, kind, size=13, color=INK, opacity=1.0, rotation=None):
    """The engraved mark of a source, cut into the tray."""
    s = size
    if kind == "square":
        i = create("rectangle", parent=parent, x=x, y=y, w=s, h=s)
        paint(i, fill=color, opacity=opacity, rotation=rotation)
    elif kind == "roundsquare":
        i = create("rectangle", parent=parent, x=x, y=y, w=s, h=s)
        paint(i, fill=color, opacity=opacity, radius=2.5, rotation=rotation)
    elif kind == "diamond":
        pts = [("M", s / 2, 0), ("L", s, s / 2), ("L", s / 2, s), ("L", 0, s / 2),
               ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity, rotation=rotation)
    elif kind == "triangle":
        pts = [("M", s / 2, 0), ("L", s, s), ("L", 0, s), ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity, rotation=rotation)
    elif kind == "hexagon":
        r = s / 2
        pts = [("M", r, 0), ("L", s, r * 0.55), ("L", s, r * 1.45),
               ("L", r, s), ("L", 0, r * 1.45), ("L", 0, r * 0.55), ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity, rotation=rotation)
    elif kind == "ring":
        ring(parent, x + s / 2, y + s / 2, s / 2, color, 1.6, opacity=opacity)
    elif kind == "pennant":
        pts = [("M", 0, s), ("L", 0, 0), ("L", s, s * 0.32), ("L", 0, s * 0.6),
               ("Z",)]
        vpath(parent, pts, x, y, fill=color, opacity=opacity, rotation=rotation)
    return None


def state_mark(parent, x, y, kind, size=12, color=INK, opacity=1.0):
    """The mark of a tool's state, cut into the bench. Returns the layer."""
    s = size
    last = None
    if kind == "onhand":
        last = create("ellipse", parent=parent, x=x, y=y, w=s, h=s)
        paint(last, fill=color, opacity=opacity)
    elif kind == "due":
        ring(parent, x + s / 2, y + s / 2, s / 2 - 0.7, color, 1.5,
             opacity=opacity)
        last = create("ellipse", parent=parent, x=x + s * 0.28, y=y + s * 0.28,
                      w=s * 0.44, h=s * 0.44)
        paint(last, fill=color, opacity=opacity)
    elif kind == "pending":
        last = ring(parent, x + s / 2, y + s / 2, s / 2 - 0.7, color, 1.3,
                    opacity=opacity, dashes=[3, 2.4])
    elif kind == "held":
        ring(parent, x + s / 2, y + s / 2, s / 2 - 0.8, color, 1.1,
             opacity=opacity)
        last = vpath(parent, [("M", x + 1.6, y + s - 1.6),
                              ("L", x + s - 1.6, y + 1.6)],
                     0, 0, stroke=color, sw=1.1, opacity=opacity)
    return last


# ============================================================== the bench ===
TRAYS = [
    # name, symbol, count, unit, on hand, due, current, note
    ("STANDARD", "square", "9,812", "TOOLS", "1,204 ON HAND", "37 DUE", True, ""),
    ("AUR", "diamond", "92,431", "RECIPES", "1,860 ON HAND", "412 DUE", False,
     "RECIPES — NO BINARIES"),
    ("FLATPAK", "ring", "2,014", "APPS", "68 ON HAND", "6 DUE", False, ""),
    ("APPIMAGE", "roundsquare", "1,873", "BINARIES", "12 ON HAND", "9 DUE", False,
     "NO DEPENDENCIES"),
    ("CARGO", "hexagon", "148,206", "CRATES", "34 ON HAND", "21 DUE", False, ""),
    ("NPM", "triangle", "3,290,441", "PACKAGES", "187 ON HAND", "143 DUE", False,
     "GLOBAL INSTALLS ONLY"),
    ("TRACKED", "pennant", "62", "RELEASES", "21 ON HAND", "4 DUE", False,
     "FOLLOWED FROM GIT HOSTS"),
]

ROWS = [
    # name, provenance, size, state, description, age
    ("ripgrep", "extra/14.1.0-1", "4.2 MB", "onhand",
     "Fast line-oriented search that walks directories for a pattern",
     "PLACED 1.2 s AGO"),
    ("rust", "extra/1.80.0-1", "142 MB", "onhand",
     "Rust toolchain, Cargo the crate manager, rustc the compiler", "3 d AGO"),
    ("python", "core/3.12.7-1", "52.1 MB", "due",
     "Python 3.12 interpreter — the system default for scripts", "11 d AGO"),
    ("nvim", "extra/0.10.2-1", "24.9 MB", "onhand",
     "Fork of Vim, built for extensibility — the same key map", "3 d AGO"),
    ("gimp", "extra/2.10.38-1", "96.3 MB", "due",
     "GNU image manipulation program — raster editor", "26 d AGO"),
    ("postgresql", "extra/17.2-1", "68.7 MB", "onhand",
     "Relational database server, version 17.2", "11 d AGO"),
    ("kitty", "extra/0.37.0-1", "12.4 MB", "onhand",
     "Fast, GPU accelerated terminal emulator", "3 d AGO"),
    ("blender", "extra/4.1.1-3", "312 MB", "pending",
     "3D creation suite — modelling, animation and render", "—"),
    ("docker", "extra/27.1.1-1", "41.2 MB", "onhand",
     "Container runtime for the Docker platform", "9 d AGO"),
    ("man-db", "core/2.12.0-1", "4.8 MB", "onhand",
     "Displays the manual page for any command on the system", "3 d AGO"),
]


def bench_surface(root):
    """The anthracite ground and the warm bench the objects sit on."""
    ground = create("rectangle", parent=root, x=0, y=0, w=W, h=H, name="Ground")
    paint(ground, fill=GROUND)
    bench = create("rectangle", parent=root, x=0, y=0, w=W, h=H, name="Bench")
    grad(bench, [a(INK, 30), a(INK, 9)], kind="radialGradient")
    return bench


def backboard(root, state):
    """Identity on the left, the age of the answer on the right. Always."""
    b = create("frame", parent=root, x=0, y=0, w=W, h=BACK_H, name="Backboard")
    paint(b, fill=a(GROUND, 120), stroke=SLATE, sw=1.0, sides=[0, 0, 1, 0])
    shell(b, 22, 6, s=0.66)
    txt(b, "CALIX", 78, 16, size=25, style="SemiBold", color=INK, ls=6.5)
    txt(b, "CHARTED 11:42 · STANDARD SYNCED 4 MIN AGO", 1000, 27, size=9.5,
        family=MONO, color=MUTED, align="right", w=576)
    return b


def rack(root, state):
    """Seven trays, seen from above. One is pulled out and open."""
    r = create("frame", parent=root, x=0, y=RACK_Y, w=W, h=RACK_H,
               name="Rack")
    tw, gap = 210, 14
    x0 = 23
    for i, (name, sym, count, unit, onhand, due, current, note) in enumerate(
            TRAYS):
        tx = x0 + i * (tw + gap)
        tray = create("frame", parent=r, x=tx, y=0, w=tw, h=RACK_H,
                      name="Tray " + name)
        paint(tray, fill=a(INK, 20), stroke=a(SLATE, 110), sw=1.0, radius=5,
              effects=[shadow(a(GROUND, 200), [0, 6], 16)])
        # the well — the inside of the box
        well = create("frame", parent=tray, x=10, y=8, w=tw - 20, h=88,
                      name="Well")
        paint(well, fill=a(GROUND, 120), radius=3,
              effects=[inner(a(GROUND, 200), [0, 4], 7)])
        seg(well, 0, 0, tw - 20, 0, a(INK, 26), 1.0)
        symbol(well, 12, 12, sym, 13, INK if current else MUTED,
               opacity=1.0 if current else 0.75)
        txt(well, count, 60, 10, size=19, family=MONO,
            color=INK if current else a(INK, 200), align="right", w=118)
        txt(well, unit, 60, 32, size=7.5, color=MUTED, ls="16%", align="right",
            w=118)
        if current:
            # the open tray: two tools lying in it
            for (ox, oy, ow, orot, ocol) in [(24, 56, 34, -5, a(INK, 190)),
                                             (66, 62, 28, 4, a(MUTED, 170)),
                                             (102, 54, 22, -2, a(O300, 150))]:
                t = create("rectangle", parent=well, x=ox, y=oy, w=ow, h=9)
                paint(t, fill=ocol, radius=2, rotation=orot)
        txt(well, onhand, 12, 76, size=8, family=MONO, color=MUTED)
        txt(well, due, 78, 76, size=8, family=MONO,
            color=O300 if not current else a(O300, 220), align="right", w=100)
        # the lip — the front of the box, where the label is stencilled
        lip = create("frame", parent=tray, x=0, y=96, w=tw, h=38, name="Lip")
        grad(lip, [a(INK, 48 if current else 34), a(SLATE, 150 if current
                                                      else 110)],
             frm=[0, 0], to=[0, 1])
        paint(lip, stroke=SLATE, sw=1.0, sides=[0, 0, 1, 0])
        seg(lip, 0, 0, tw, 0, a(INK, 46 if current else 30), 1.0)
        if current:
            tick = create("rectangle", parent=lip, x=10, y=12, w=3, h=15)
            paint(tick, fill=ORANGE)
        txt(lip, name, 20, 8, size=12, family=MONO,
            color=ORANGE if current else a(INK, 205), ls="14%")
        if note:
            txt(lip, note, 20, 24, size=7, color=MUTED, ls="6%")
    return r


def mat(root, state):
    """The cutting mat: the work surface, ruled and worn."""
    m = create("frame", parent=root, x=MARGIN, y=MAT_Y, w=W - 2 * MARGIN,
               h=MAT_H, name="Mat")
    paint(m, fill=a(GROUND, 150), stroke=SLATE, sw=1.0, sides=[0, 1, 0, 0],
          radius=6, effects=[shadow(a(GROUND, 225), [0, 10], 28)])
    # the ruled grid of the mat
    gx = 0
    while gx <= W - 2 * MARGIN:
        seg(m, gx, 6, gx, MAT_H - 6, a(SLATE, 32), 1.0)
        gx += 40
    gy = 6
    while gy <= MAT_H - 6:
        seg(m, 6, gy, W - 2 * MARGIN - 6, gy, a(SLATE, 26), 1.0)
        gy += 40
    # cut marks, and a ring left by a cup
    for (cx, cy, dx, dy) in [(792, 372, 42, -9), (838, 352, 24, 11),
                              (470, 96, 28, 3), (250, 512, 20, -2),
                              (612, 560, 34, 7)]:
        seg(m, cx, cy, cx + dx, cy + dy, a(INK, 34), 1.0)
    return m


def job_chalk(m, state):
    """The job the bench is set up for, written in chalk on the mat."""
    active = "FIND" if state == "quiet" else "STOCK"
    txt(m, "JOB", 24, 16, size=8, color=MUTED, ls="18%")
    words = [("FIND", 24), ("·", 68), ("STOCK", 84), ("·", 140), ("DELTA", 158)]
    for (word, wx) in words:
        on = (word == active)
        txt(m, word, wx, 32, size=13, family=MONO,
            color=ORANGE if on else a(INK, 185), ls="10%", rotation=-0.8)
    # the chalk ring, drawn twice, around the job in hand
    cx = 24 + 20 if active == "FIND" else 84 + 24
    e1 = create("ellipse", parent=m, x=cx - 26, y=40 - 14, w=52, h=28)
    paint(e1, fill=CLEAR, stroke=a(ORANGE, 140), sw=1.3, rotation=3.5)
    e2 = create("ellipse", parent=m, x=cx - 25, y=39 - 15, w=50, h=30)
    paint(e2, fill=CLEAR, stroke=a(ORANGE, 60), sw=1.0, rotation=-6)
    return None


def find_slot(m, state):
    """A slot cut into the mat, where a name is written."""
    s = create("frame", parent=m, x=440, y=16, w=340, h=30, name="FindSlot")
    paint(s, fill=a(GROUND, 170), radius=5,
          effects=[inner(a(GROUND, 220), [0, 3], 6)])
    seg(s, 2, 0.5, 338, 0.5, a(INK, 34), 1.0)
    ring(s, 20, 15, 5, MUTED, 1.2, opacity=0.85)
    vpath(s, [("M", 23.6, 18.6), ("L", 27.5, 22.5)], 0, 0, stroke=MUTED, sw=1.2,
          opacity=0.85)
    if state == "quiet":
        txt(s, "FIND A TOOL", 36, 10, size=10, color=MUTED, ls="8%")
        caret = create("rectangle", parent=s, x=112, y=9, w=1.4, h=13)
        paint(caret, fill=ORANGE)
    else:
        txt(s, "ripgrep", 36, 10, size=10, color=a(INK, 210), ls="4%")
        txt(s, "LAST FIND", 96, 11, size=7.5, color=MUTED, ls="10%")
    return s


def scale_model(m, state):
    """The other density, drawn as a scale model beside the switch."""
    g = create("frame", parent=m, x=1276, y=10, w=280, h=44, name="Scale")
    txt(g, "SCALE", 0, 4, size=8, color=MUTED, ls="18%")
    # the detailed model
    d = create("frame", parent=g, x=40, y=0, w=66, h=30)
    paint(d, fill=CLEAR, stroke=a(SLATE, 130), sw=0.9, radius=3)
    for i in range(3):
        seg(d, 6, 7 + i * 8, 30, 7 + i * 8, a(INK, 160), 2.0)
        seg(d, 34, 7 + i * 8, 50, 7 + i * 8, a(INK, 95), 1.3)
        seg(d, 54, 7 + i * 8, 60, 7 + i * 8, a(O300, 110), 1.3)
    txt(g, "DETAILED", 40, 33, size=7, color=ORANGE, ls="10%")
    # the compact model
    c = create("frame", parent=g, x=134, y=0, w=66, h=30)
    paint(c, fill=CLEAR, stroke=a(SLATE, 130), sw=0.9, radius=3)
    for i in range(7):
        seg(c, 5, 4 + i * 3.4, 26, 4 + i * 3.4, a(INK, 130), 1.3)
        seg(c, 30, 4 + i * 3.4, 44, 4 + i * 3.4, a(INK, 85), 1.0)
        seg(c, 48, 4 + i * 3.4, 56, 4 + i * 3.4, a(O300, 110), 1.0)
    txt(g, "COMPACT", 134, 33, size=7, color=MUTED, ls="10%")
    return g


def list_header(m, state):
    if state == "quiet":
        txt(m, "1,204 IN THE STANDARD TRAY · 37 UPDATE DUE · REPOS ALL", 860, 28,
            size=8.5, family=MONO, color=MUTED, align="right", w=420)
    else:
        txt(m, "1,204 ON THE BENCH · 37 UPDATE DUE · REPOS ALL", 860, 28,
            size=8.5, family=MONO, color=MUTED, align="right", w=420)
    seg(m, 24, 64, W - 2 * MARGIN - 24, 64, a(SLATE, 80), 1.0)
    txt(m, "TOOL", 76, 78, size=8, color=MUTED, ls="18%")
    txt(m, "PROVENANCE", 514, 78, size=8, color=MUTED, ls="18%")
    txt(m, "SIZE", 634, 78, size=8, color=MUTED, ls="18%", align="right", w=90)
    txt(m, "STATE", 744, 78, size=8, color=MUTED, ls="18%")
    txt(m, "PLACED", 874, 78, size=8, color=MUTED, ls="18%")


def row(m, i, data, picked):
    name, prov, size, state, desc, age = data
    y = 96 + i * 62
    r = create("frame", parent=m, x=24, y=y, w=966, h=62, name="Row " + name)
    paint(r, fill=CLEAR)
    if picked:
        band = create("rectangle", parent=r, x=0, y=4, w=966, h=54)
        grad(band, [a(ORANGE, 26), a(ORANGE, 0)], frm=[0, 0], to=[1, 0])
        # the tool, picked up: it tilts, and its shadow comes off the mat
        gl = state_mark(r, 4, 12, state, 12, a(INK, 210), opacity=1.0)
        setp(gl, effects=[shadow(a(GROUND, 230), [4, 7], 11)])
        setp(gl, rotation=1.7)
        nm = txt(r, name, 32, 4, size=15.5, style="SemiBold", color=INK)
        setp(nm, effects=[shadow(a(GROUND, 225), [4, 7], 12)])
        setp(nm, rotation=1.7)
    else:
        state_mark(r, 4, 12, state, 12,
                   a(INK, 190) if state == "onhand" else O300)
        txt(r, name, 32, 4, size=15, style="SemiBold", color=a(INK, 225))
    txt(r, desc, 32, 29, size=11.5, color=MUTED)
    txt(r, prov, 470, 12, size=10.5, family=MONO, color=MUTED)
    txt(r, size, 544, 12, size=10.5, family=MONO, color=MUTED, align="right",
        w=90)
    chip = create("frame", parent=r, x=700, y=8, w=104, h=20)
    if state == "due":
        paint(chip, fill=a(ORANGE, 22), stroke=a(ORANGE, 150), sw=0.9, radius=4)
        txt(chip, "UPDATE DUE", 0, 5, size=7.5, color=O300, ls="8%",
            align="center", w=104, lh="100%")
    elif state == "pending":
        paint(chip, fill=CLEAR, stroke=a(O300, 110), sw=0.9, radius=4,
              dashes=[3, 2.4])
        txt(chip, "PENDING", 0, 5, size=7.5, color=O300, ls="8%",
            align="center", w=104, lh="100%")
    else:
        paint(chip, fill=a(INK, 12), stroke=a(SLATE, 110), sw=0.9, radius=4)
        txt(chip, "ON HAND", 0, 5, size=7.5, color=a(INK, 180), ls="8%",
            align="center", w=104, lh="100%")
    txt(r, age, 830, 12, size=8.5, family=MONO, color=a(MUTED, 180), ls="2%")
    return r


def empty_slot(m):
    """The slot on an empty bench, saying what would fit in it."""
    slot = create("frame", parent=m, x=24, y=96, w=966, h=54, name="EmptySlot")
    paint(slot, fill=a(INK, 6), stroke=a(ORANGE, 90), sw=1.0, radius=4,
          dashes=[7, 5])
    txt(slot, "A PACKAGE FROM THE STANDARD TRAY FITS HERE", 32, 12, size=10,
        color=a(INK, 190), ls="8%")
    txt(slot, "WRITE A NAME IN THE SLOT TO FILL THE BENCH", 32, 30, size=8,
        color=MUTED, ls="8%")
    return slot


def card_slot(m):
    """Where the inspection card is laid down, when a tool is picked up."""
    slot = create("frame", parent=m, x=1030, y=90, w=460, h=510,
                  name="CardSlot")
    paint(slot, fill=a(INK, 4), stroke=a(SLATE, 90), sw=1.0, radius=4,
          dashes=[6, 5])
    txt(slot, "INSPECTION CARD", 16, 12, size=8, color=MUTED, ls="18%")
    txt(slot, "LAID BESIDE THE TOOL YOU PICK UP", 16, 28, size=8,
        color=a(MUTED, 170), ls="6%")
    return slot


def card(m, state):
    """The inspection card, laid on the mat beside the picked-up tool."""
    c = create("frame", parent=m, x=1030, y=96, w=460, h=510, name="Card")
    paint(c, fill=a(INK, 30), stroke=a(SLATE, 120), sw=1.0, radius=5,
          effects=[shadow(a(GROUND, 235), [6, 12], 26)])
    # a steel clip on the top edge
    clip = create("frame", parent=c, x=196, y=-7, w=68, h=16, name="Clip")
    paint(clip, fill=SLATE, radius=3)
    grad(clip, [SLATE, MUTED, SLATE])
    seg(c, 0, 0.5, 460, 0.5, a(INK, 40), 1.0)
    seg(c, 0, 509.5, 460, 509.5, a(GROUND, 160), 1.0)

    symbol(c, 20, 24, "square", 12, a(INK, 200))
    txt(c, "STANDARD ▸ EXTRA", 40, 25, size=8, color=MUTED, ls="14%")
    txt(c, "ripgrep", 20, 46, size=26, style="SemiBold", color=INK)
    txt(c, "14.1.0-1", 20, 84, size=15, family=MONO, color=INK)
    chip = create("frame", parent=c, x=330, y=44, w=110, h=21)
    paint(chip, fill=a(INK, 12), stroke=a(SLATE, 130), sw=0.9, radius=4)
    txt(chip, "ON HAND", 0, 5, size=7.5, color=a(INK, 190), ls="8%",
        align="center", w=110, lh="100%")
    txt(c, "PLACED ON THE BENCH 1.2 s AGO", 20, 106, size=8.5, family=MONO,
        color=MUTED)

    seg(c, 20, 128, 440, 128, a(SLATE, 90), 1.0)
    txt(c, "PROVENANCE", 20, 140, size=8, color=MUTED, ls="18%")
    txt(c, "standard ▸ extra ▸ 14.1.0-1", 20, 156, size=11.5, family=MONO,
        color=INK)
    txt(c, "SIGNED BY THE ALPM KEY · SHA256 9F4C…B21", 20, 176, size=9,
        family=MONO, color=MUTED)
    txt(c, "A BINARY FROM THE OFFICIAL REPOSITORIES — NO RECIPE HERE", 20, 194,
        size=9, color=a(MUTED, 220), ls="2%")

    seg(c, 20, 216, 440, 216, a(SLATE, 90), 1.0)
    txt(c, "DEPENDENCIES", 20, 228, size=8, color=MUTED, ls="18%")
    txt(c, "4 RESOLVED", 330, 229, size=8, family=MONO, color=MUTED,
        align="right", w=110)
    for i, (d, dv) in enumerate([("pcre2", "10.43"), ("gcc-libs", "14.1.1"),
                                  ("glib2", "2.80"), ("libintl", "0.22.5")]):
        yy = 246 + i * 18
        txt(c, d, 20, yy, size=10, family=MONO, color=a(INK, 200))
        txt(c, dv, 330, yy, size=10, family=MONO, color=MUTED, align="right",
            w=110)

    seg(c, 20, 328, 440, 328, a(SLATE, 90), 1.0)
    txt(c, "FILES", 20, 340, size=8, color=MUTED, ls="18%")
    txt(c, "14 FILES", 330, 341, size=8, family=MONO, color=MUTED,
        align="right", w=110)
    for i, fl in enumerate(["/usr/bin/rg", "/usr/share/man/man1/rg.1",
                            "/usr/share/licenses/ripgrep/LICENSE"]):
        txt(c, fl, 20, 358 + i * 17, size=9.5, family=MONO, color=MUTED)

    seg(c, 20, 414, 440, 414, a(SLATE, 90), 1.0)
    txt(c, "COMMAND", 20, 426, size=8, color=MUTED, ls="18%")
    txt(c, "STANDARD ▸ PACMAN", 330, 427, size=8, family=MONO, color=MUTED,
        align="right", w=110)
    cmd = create("frame", parent=c, x=20, y=444, w=420, h=32)
    paint(cmd, fill=a(GROUND, 190), stroke=a(SLATE, 120), sw=1.0, radius=4)
    txt(cmd, "$ calix install standard ripgrep", 12, 9, size=12, family=MONO,
        color=ORANGE)

    for (bx, label, col) in [(20, "RE-INSTALL", a(INK, 210)),
                              (130, "REMOVE", MUTED)]:
        b2 = create("frame", parent=c, x=bx, y=484, w=100, h=24)
        paint(b2, fill=a(INK, 10), stroke=a(SLATE, 110), sw=0.9, radius=4)
        txt(b2, label, 0, 7, size=8, color=col, ls="8%", align="center", w=100,
            lh="100%")
    return c


def rail(root, state):
    """The near rail: a cork strip with the backend's own log on it."""
    r = create("frame", parent=root, x=MARGIN, y=RAIL_Y, w=W - 2 * MARGIN,
               h=RAIL_H, name="Rail")
    paint(r, fill=a(INK, 30), stroke=SLATE, sw=1.0, sides=[1, 0, 0, 0],
          effects=[shadow(a(GROUND, 210), [0, -8], 22)])
    # the steel edge along the top of the strip
    edge = create("rectangle", parent=r, x=0, y=0, w=W - 2 * MARGIN, h=3)
    grad(edge, [SLATE, MUTED, SLATE], frm=[0, 0], to=[1, 0])
    seg(r, 0, 3.5, W - 2 * MARGIN, 3.5, a(INK, 50), 1.0)
    # cork
    tile = create("frame", parent=root, x=1660, y=0, w=27, h=27,
                  name="CorkTile")
    paint(tile, fill=CLEAR)
    cork = [(2.1, 3.4, 1.4, a(MUTED, 95)), (8.6, 1.2, 0.9, a(MUTED, 70)),
            (5.3, 8.7, 1.6, a(O700, 95)), (13.2, 6.4, 1.0, a(MUTED, 80)),
            (19.4, 2.6, 1.3, a(O700, 78)), (23.8, 9.1, 0.9, a(MUTED, 70)),
            (1.6, 14.2, 1.1, a(MUTED, 72)), (9.4, 13.1, 1.5, a(O700, 85)),
            (16.1, 12.7, 0.8, a(MUTED, 62)), (21.7, 15.4, 1.2, a(MUTED, 76)),
            (4.7, 19.6, 1.0, a(O700, 70)), (12.8, 20.4, 1.3, a(MUTED, 84)),
            (18.3, 21.8, 0.9, a(O700, 62)), (24.4, 23.1, 1.1, a(MUTED, 68)),
            (2.4, 24.2, 0.8, a(MUTED, 58)), (25.1, 18.2, 0.7, a(MUTED, 55)),
            (7.1, 25.4, 0.9, a(O700, 58)), (14.9, 25.6, 0.7, a(MUTED, 52))]
    for (dx, dy, r_, col) in cork:
        d = create("ellipse", parent=tile, x=dx, y=dy, w=r_ * 2, h=r_ * 2)
        paint(d, fill=col)
    setp(r, fills=[{"type": "solid", "color": a(INK, 30)},
                    {"type": "solid", "color": a(INK, 30)}])
    add("fill.pattern", {"source": S(tile), "ids": [S(r)], "scale": 1,
                          "spacing": [27, 27], "opacity": 0.42})

    ring(r, 690, 74, 32, a(O700, 38), 1.2)
    ring(r, 690, 74, 32, a(O700, 18), 1.0, rotation=8)
    txt(r, "BACKEND LOG · PACMAN", 24, 20, size=8, color=MUTED, ls="18%")
    if state == "quiet":
        txt(r, "NOTHING RUN YET", 24, 42, size=11, family=MONO,
            color=a(MUTED, 190), ls="6%")
    else:
        txt(r, "sudo pacman -S --needed ripgrep", 24, 40, size=11.5, family=MONO,
            color=a(INK, 200))
        txt(r, "resolving dependencies…", 24, 62, size=10, family=MONO,
            color=MUTED)
        txt(r, "✔ ripgrep 14.1.0-1 installed · 4.2 MB · 1.2 s", 24, 82, size=10,
            family=MONO, color=a(MUTED, 220))
        # the operation in flight
        txt(r, "IN FLIGHT", 980, 20, size=8, color=MUTED, ls="18%")
        txt(r, "UPDATING", 980, 40, size=8.5, color=O300, ls="10%")
        txt(r, "standard rust 1.79.0 → 1.80.0", 1050, 38, size=12, color=INK)
        bar_bg = create("rectangle", parent=r, x=1050, y=64, w=260, h=3)
        paint(bar_bg, fill=a(INK, 20))
        bar = create("rectangle", parent=r, x=1050, y=64, w=161, h=3)
        paint(bar, fill=ORANGE)
        head = create("rectangle", parent=r, x=1200, y=62, w=16, h=7)
        paint(head, fill=O300, effects=[blur(3)])
        txt(r, "12.4 / 142 MB", 1330, 58, size=9.5, family=MONO, color=MUTED)
        txt(r, "0:41", 1330, 78, size=9.5, family=MONO, color=a(MUTED, 190))
    return r


def build(state):
    root = create("frame", x=0, y=0, w=W, h=H,
                  name="03-atelier-" + state)
    paint(root, fill=GROUND)
    bench_surface(root)
    backboard(root, state)
    rack(root, state)
    m = mat(root, state)
    job_chalk(m, state)
    find_slot(m, state)
    scale_model(m, state)
    list_header(m, state)
    if state == "quiet":
        empty_slot(m)
        card_slot(m)
        txt(m, "THE BENCH IS CLEAR", 60, 430, size=22, family=MONO,
            color=a(INK, 105), ls="18%", rotation=-0.6)
        txt(m, "THE JOB AND THE TRAY DECIDE WHAT YOU CAN PICK UP", 60, 466,
            size=10, color=a(MUTED, 190), ls="6%")
    else:
        for i, data in enumerate(ROWS):
            row(m, i, data, picked=(i == 0))
        card(m, state)
    rail(root, state)
    return root


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    build("loaded")
    build("quiet")
    out = os.path.abspath(args.out)
    with open(out + ".jsonl", "w") as fh:
        for c in CMDS:
            fh.write(json.dumps(c) + "\n")
    print("%d steps -> %s.jsonl" % (len(CMDS), out))


if __name__ == "__main__":
    main()
