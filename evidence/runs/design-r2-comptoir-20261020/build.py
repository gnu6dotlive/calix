#!/usr/bin/env python3
"""Le Comptoir — Calix package manager as a shop counter at night. Round 2, proposal 04.

Two states of one arrangement, in one FigCraft file:
  * "loaded"  — the counter after a search
  * "quiet"   — the counter the moment the shop opens
"""
import json, math, subprocess, os

OUT_DIR = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------- tokens
GROUND = "#111418"; WHITE = "#F7F3ED"; MUTED = "#A1A6AD"; SLATE = "#667085"
O100 = "#FFE5D2"; O300 = "#FF8E40"; ORANGE = "#FF6A00"; O700 = "#C95400"
B300 = "#87A0F4"; BLUE = "#4169E1"

def hx(c):
    c = c.lstrip('#')
    return tuple(int(c[i:i+2], 16) for i in (0, 2, 4))

def mix(c1, c2, t):
    a, b = hx(c1), hx(c2)
    return "#%02X%02X%02X" % tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))

def al(c, a):
    r, g, b = hx(c)
    return "#%02X%02X%02X%02X" % (r, g, b, round(max(0.0, min(1.0, a)) * 255))

# the shop's material, all of it derived from the tokens
WOOD_DEEP  = mix(GROUND, O700, 0.050)   # the wall, unlit
WOOD_DARK  = mix(GROUND, O700, 0.095)   # the shelf
WOOD_MID   = mix(GROUND, O700, 0.150)   # the counter
WOOD_LIT   = mix(GROUND, O700, 0.200)   # wood under a lamp
BRASS_DEEP = mix(SLATE, GROUND, 0.72)   # blackened brass
BRASS_MID  = mix(SLATE, GROUND, 0.42)
BRASS_LIT  = mix(SLATE, WHITE, 0.42)
BOARD      = mix(SLATE, GROUND, 0.82)   # the counter board
CARD       = mix(WHITE, GROUND, 0.78)   # a package's card
LABEL      = mix(WHITE, GROUND, 0.60)   # its printed label
PAPER_HOT  = mix(WHITE, GROUND, 0.34)   # the till receipt
INK        = GROUND                      # ink on paper

steps = []
CUR = None
def S(cmd, params):
    steps.append({"command": cmd, "params": params})
    return len(steps) - 1

# ---------------------------------------------------------------- primitives
def rect(x, y, w, h, fill=None, radius=0, stroke=None, sw=1, opacity=None,
         rotation=None, dashes=None, parent=None, name=None):
    p = {}
    if fill is not None: p["fill"] = fill
    if radius: p["cornerRadius"] = radius
    if stroke is not None:
        p["stroke"] = stroke
        p["strokeWeight"] = sw
    if dashes: p["dashes"] = dashes
    if opacity is not None: p["opacity"] = opacity
    if rotation is not None: p["rotation"] = rotation
    if name: p["name"] = name
    return S("shape.rectangle", {"x": x, "y": y, "width": w, "height": h,
                                 "parent": {"$step": parent if parent is not None else CUR},
                                 "props": p})

def ellipse(x, y, w, h, fill=None, stroke=None, sw=1, rotation=None, opacity=None, parent=None):
    p = {"fill": fill if fill is not None else "#00000000"}
    if stroke is not None:
        p["stroke"] = stroke; p["strokeWeight"] = sw
    if rotation is not None: p["rotation"] = rotation
    if opacity is not None: p["opacity"] = opacity
    return S("shape.ellipse", {"x": x, "y": y, "width": w, "height": h,
                               "parent": {"$step": parent if parent is not None else CUR},
                               "props": p})

def T(s, x, cap, size=12, fam="IBM Plex Sans", weight=400, color=WHITE, ls=None,
      width=None, align=None, opacity=None, parent=None):
    props = {"fontSize": size, "fontFamily": fam, "fontWeight": weight, "fill": color}
    if ls is not None: props["letterSpacing"] = ls
    if width is not None: props["width"] = width
    if align is not None: props["textAlign"] = align
    if opacity is not None: props["opacity"] = opacity
    return S("text.create", {"text": s, "x": x, "y": cap - 0.33 * size,
                             "parent": {"$step": parent if parent is not None else CUR},
                             "props": props})

def shadow(ids, opacity=0.5, x=0, y=8, radius=16, spread=0, color="#000000"):
    S("effect.add", {"ids": [{"$step": i} for i in ids],
                      "effect": {"type": "dropShadow", "color": color, "opacity": opacity,
                                 "offset": [x, y], "radius": radius, "spread": spread}})

def inner(ids, opacity=0.6, x=0, y=4, radius=12, color="#000000"):
    S("effect.add", {"ids": [{"$step": i} for i in ids],
                      "effect": {"type": "innerShadow", "color": color, "opacity": opacity,
                                 "offset": [x, y], "radius": radius}})

def grain(ids, opacity=0.055):
    S("effect.add", {"ids": [{"$step": i} for i in ids],
                      "effect": {"type": "noise", "opacity": opacity}})

def blur(ids, radius=8):
    S("effect.add", {"ids": [{"$step": i} for i in ids],
                      "effect": {"type": "layerBlur", "radius": radius}})

def grad(ids, stops, frm=None, to=None):
    paint = {"type": "linearGradient", "stops": stops}
    if frm is not None: paint["from"] = frm
    if to is not None: paint["to"] = to
    S("node.set", {"ids": [{"$step": i} for i in ids], "props": {"fills": [paint]}})

def radial(ids, stops):
    S("node.set", {"ids": [{"$step": i} for i in ids],
                   "props": {"fills": [{"type": "radialGradient", "stops": stops}]}})

def mkpath(cmds):
    out = []
    for c in cmds:
        k = c[0]; nums = c[1:]
        out.append("Z" if k == "Z" else k + " " + " ".join(("%.2f" % n) for n in nums))
    return " ".join(out)

def vpath(d, fill=None, stroke=None, sw=1, opacity=None, parent=None):
    p = {}
    if fill is not None: p["fill"] = fill
    if stroke is not None:
        p["stroke"] = stroke; p["strokeWeight"] = sw
    if opacity is not None: p["opacity"] = opacity
    return S("vector.create", {"path": d, "x": 0, "y": 0, "closed": True,
                                "parent": {"$step": parent if parent is not None else CUR},
                                "props": p})

# ---------------------------------------------------------------- the mark
def scallop_mark(ox, oy, sc, parent):
    """A scallop shell as the cup of a chalice.  Local box: x 0..120, y 24..153."""
    HX, HY = 60.0, 96.0
    R_TIP, R_VAL = 70.0, 67.5
    A0, A1 = -52.0, 52.0
    N = 9
    ang = [math.radians(A0 + (A1 - A0) * i / (N - 1)) for i in range(N)]

    def pol(a, r):
        return (HX + r * math.sin(a), HY - r * math.cos(a))

    def P(x, y):
        return (ox + x * sc, oy + (y - 24) * sc)

    def path(cmds):
        out = []
        for c in cmds:
            k = c[0]; pts = []
            for i in range(1, len(c), 2):
                pts.extend(P(c[i], c[i + 1]))
            out.append(tuple([k] + pts))
        return mkpath(out)

    vpath(path([("M", 8, 56), ("C", 30, 68, 90, 68, 112, 56),
                ("L", 112, 68), ("C", 90, 82, 30, 82, 8, 68), ("Z",)]),
          fill=mix(SLATE, GROUND, 0.55), parent=parent)
    d = [("M", HX, HY)]
    t0 = pol(ang[0], R_TIP)
    d.append(("Q", HX - 6, HY - 12, t0[0], t0[1]))
    margin = [("M", t0[0], t0[1])]
    for i in range(N - 1):
        am = (ang[i] + ang[i + 1]) / 2
        val = pol(am, R_VAL)
        c1 = pol(am, R_TIP + 0.5); c2 = pol(am, R_VAL - 2)
        d.append(("C", c1[0], c1[1], c2[0], c2[1], val[0], val[1]))
        margin.append(("C", c1[0], c1[1], c2[0], c2[1], val[0], val[1]))
        c3 = pol(am, R_VAL - 2); c4 = pol(am, R_TIP + 0.5)
        d.append(("C", c3[0], c3[1], c4[0], c4[1], pol(ang[i+1], R_TIP)[0], pol(ang[i+1], R_TIP)[1]))
        margin.append(("C", c3[0], c3[1], c4[0], c4[1], pol(ang[i+1], R_TIP)[0], pol(ang[i+1], R_TIP)[1]))
    t6 = pol(ang[N - 1], R_TIP)
    d.append(("Q", HX + 6, HY - 12, t6[0], t6[1]))
    d.append(("Q", HX + 4, HY + 3, HX, HY)); d.append(("Z",))
    shell = vpath(path(d), fill=ORANGE, parent=parent)
    grad([shell], [O300, ORANGE, O700], [0.05, 0], [0.95, 1])
    vpath(path(margin), stroke=O100, sw=1.4 * sc, opacity=0.38, parent=parent)
    for a in ang:
        w = math.radians(1.7)
        p1 = pol(a - w, R_TIP - 4); p2 = pol(a + w, R_TIP - 4)
        vpath(path([("M", HX, HY), ("L", p1[0], p1[1]), ("L", p2[0], p2[1]), ("Z",)]),
              fill=O100, opacity=0.13, parent=parent)
    for i in range(N - 1):
        am = (ang[i] + ang[i + 1]) / 2
        g0 = pol(am, 7); g1 = pol(am, R_VAL - 2)
        vpath(path([("M", g0[0], g0[1]), ("L", g1[0], g1[1])]), stroke=O700,
              sw=1.3 * sc, opacity=0.34, parent=parent)
    wall = path([("M", 8, 68), ("C", 30, 82, 90, 82, 112, 68),
                 ("C", 110, 102, 92, 115, 72, 116), ("L", 48, 116),
                 ("C", 28, 115, 10, 102, 8, 68), ("Z",)])
    wl = vpath(wall, fill=SLATE, parent=parent)
    grad([wl], [BRASS_LIT, BRASS_MID, BRASS_DEEP], [0.05, 0], [0.25, 1])
    vpath(path([("M", 8, 68), ("C", 30, 82, 90, 82, 112, 68)]), stroke=BRASS_LIT,
          sw=2.8 * sc, opacity=0.9, parent=parent)
    stem = rect(P(55, 115)[0], P(55, 115)[1], 10 * sc, 27 * sc, fill=SLATE, radius=2 * sc, parent=parent)
    grad([stem], [BRASS_LIT, BRASS_DEEP, BRASS_LIT], [0, 0], [1, 0])
    foot = vpath(path([("M", 55, 141), ("L", 65, 141), ("L", 74, 147), ("L", 46, 147), ("Z",)]),
                 fill=SLATE, parent=parent)
    grad([foot], [BRASS_LIT, BRASS_DEEP], [0, 0], [0, 1])
    base = ellipse(P(38, 144)[0], P(38, 144)[1], 44 * sc, 10 * sc, fill=SLATE, parent=parent)
    grad([base], [BRASS_LIT, BRASS_DEEP], [0, 0], [0, 1])

# ================================================================ content
GOODS = [
    ("rindexer", "0.15.1-1", "extra", "2.3 MB", "INSTALLED", None,
     "indexes directories with ripgrep,", "for search engines"),
    ("ripgrep", "14.1.0-1", "extra", "5.1 MB", "UPDATE", "14.1.1",
     "recursively search directories", "for a regex pattern"),
    ("ripgrep-all", "0.10.9-1", "extra", "1.2 MB", "AVAILABLE", None,
     "every ripgrep feature in one", "binary, no optional features"),
    ("ripgrep-git", "14.1.0.r6-1", "extra", "4.8 MB", "AVAILABLE", None,
     "the git master, built from", "source every time"),
    ("rizin", "0.6.0-1", "extra", "38.2 MB", "AVAILABLE", None,
     "reverse engineering framework", "and binary editor"),
]

COUNTERS = [
    ("ALPM", "OFFICIAL REPOS", "1,284", ""),
    ("AUR", "BUILD RECIPES", "8,431", ""),
    ("FLATPAK", "SANDBOXES", "96", ""),
    ("APPIMAGE", "SINGLE BINARIES", "41", ""),
    ("CARGO", "RUST CRATES", "612", ""),
    ("NPM", "NODE PACKAGES", "1,308", ""),
    ("TRACKED RELEASES", "FOLLOWED BINARIES", "0", "NOTHING IN STOCK"),
]

# ---------------------------------------------------------------- geometry
BOARD_X, BOARD_Y, BOARD_W, BOARD_H = 262, 26, 1300, 168
SHELF_TOP = 452            # the plank the goods stand on
LIP_TOP = 492              # the lip of the shelf
COUNTER_Y = 556            # the counter surface begins
GOODS_TOP = 282
BOX_W, BOX_H = 200, 170
SLOTS = [286 + i * 246 for i in range(5)]


def build_screen(x0, loaded, name):
    global CUR
    scr = S("frame.create", {"x": x0, "y": 0, "width": 1600, "height": 1000,
                             "name": name, "props": {"fill": GROUND, "clip": True}})
    CUR = scr

    # ---------------------------------------------------------- the wall
    wall = rect(0, 0, 1600, COUNTER_Y, fill=WOOD_DEEP)
    grad([wall], [mix(GROUND, O700, 0.03), WOOD_DEEP, mix(GROUND, O700, 0.085)], [0, 0], [0, 1])
    grain([wall], 0.07)
    # panelled wainscot: a moulding down each stile
    for px in range(0, 1600, 128):
        rect(px, 0, 1, COUNTER_Y, fill=al(GROUND, 0.40))
        rect(px + 1, 0, 1, COUNTER_Y, fill=al(O100, 0.045))
    # the chair rail
    rect(0, 250, 1600, 3, fill=al(O100, 0.13))
    rect(0, 253, 1600, 2, fill=al(GROUND, 0.5))
    # a warm wash falling from the board onto the wall
    ww = ellipse(240, 120, 1200, 300)
    radial([ww], [al(O100, 0.12), al(O100, 0.0)])
    for px in range(0, 1600, 256):
        rect(px, 0, 4, COUNTER_Y, fill=al(GROUND, 0.35))

    # the lamp over the ALPM counter washing the wall
    pool = ellipse(90, 30, 460, 400)
    radial([pool], [al(O100, 0.15), al(O100, 0.0)])
    # the sign's own lamp, so the empty corner of the wall is lit, not dead
    sp = ellipse(-60, 40, 420, 480)
    radial([sp], [al(O100, 0.10), al(O100, 0.0)])

    # ---------------------------------------------------------- the board
    bx, by, bw, bh = BOARD_X, BOARD_Y, BOARD_W, BOARD_H
    board = rect(bx, by, bw, bh, fill=BOARD, radius=4)
    shadow([board], 0.6, 0, 12, 26)
    inner([board], 0.5, 0, 3, 10)
    rect(bx, by, bw, 2, fill=al(BRASS_LIT, 0.45))
    rail = rect(bx, by, bw, 30, fill=BRASS_DEEP)
    grad([rail], [BRASS_MID, BRASS_DEEP], [0, 0], [0, 1])
    rect(bx, by + 29, bw, 1, fill=al(GROUND, 0.6))
    T("THE SEVEN COUNTERS", bx + 20, 47, size=9.5, fam="IBM Plex Sans Condensed",
      weight=700, color=al(MUTED, 0.8), ls="0.22")
    ellipse(bx + bw - 224, 36, 8, 8, fill=O300)
    T("ALPM INDEX · 4 MIN OLD", bx + bw - 210, 45, size=10, fam="IBM Plex Mono",
      color=al(MUTED, 0.95), ls="0.04", align="right", width=190)

    cellw = bw / 7.0
    for i, (nm, kind, count, extra) in enumerate(COUNTERS):
        cx = bx + i * cellw
        lit = (i == 0)
        if i:
            rect(cx, by + 34, 1, bh - 38, fill=al(SLATE, 0.20))
        if lit:
            p = ellipse(cx + 26, by + 44, cellw - 52, 108)
            radial([p], [al(O100, 0.20), al(O100, 0.0)])
        lx = cx + cellw / 2
        shade = vpath(mkpath([("M", lx - 13, 90), ("L", lx + 13, 90),
                               ("L", lx + 7, 72), ("L", lx - 7, 72), ("Z",)]), fill=BRASS_MID)
        grad([shade], [BRASS_LIT, BRASS_DEEP], [0, 0], [1, 0])
        rect(lx - 1.5, 60, 3, 12, fill=BRASS_DEEP)
        if lit:
            ellipse(lx - 6, 84, 12, 10, fill=O100)
            glow = ellipse(lx - 20, 76, 40, 36)
            radial([glow], [al(O100, 0.45), al(O100, 0.0)])
        else:
            ellipse(lx - 4, 86, 8, 7, fill=al(SLATE, 0.7))
        T(nm, cx, 132, size=12, fam="IBM Plex Sans Condensed", weight=700,
          color=WHITE if lit else al(MUTED, 0.66), ls="0.10", align="center", width=cellw)
        T(kind, cx, 152, size=8.5, fam="IBM Plex Mono", color=al(MUTED, 0.5),
          ls="0.04", align="center", width=cellw)
        T(count, cx, 176, size=12, fam="IBM Plex Mono",
          color=ORANGE if lit else al(SLATE, 0.85), align="center", width=cellw)
        if extra:
            T(extra, cx, 190, size=8, fam="IBM Plex Mono", color=al(SLATE, 0.9),
              ls="0.02", align="center", width=cellw)

    # ---- the counter's pendant lamp, and the pool it makes
    rect(800, 188, 2, 44, fill=al(SLATE, 0.7))
    pshade = vpath(mkpath([("M", 760, 232), ("L", 840, 232), ("L", 822, 264), ("L", 778, 264), ("Z",)]),
                    fill=BRASS_MID)
    grad([pshade], [BRASS_LIT, BRASS_DEEP], [0, 0], [1, 0])
    rect(768, 228, 64, 4, fill=BRASS_DEEP)
    ellipse(788, 256, 26, 22, fill=O100)
    pglow = ellipse(756, 242, 88, 76)
    radial([pglow], [al(O100, 0.45), al(O100, 0.0)])
    lamp_pool = ellipse(600, 290, 560, 320)
    radial([lamp_pool], [al(O100, 0.20), al(O100, 0.0)])

    # ---------------------------------------------------------- the shelf
    # the shop's own overhead light washes the goods
    wash = ellipse(180, 210, 1240, 300)
    radial([wash], [al(O100, 0.19), al(O100, 0.0)])
    wash2 = ellipse(420, 300, 900, 220)
    radial([wash2], [al(O100, 0.10), al(O100, 0.0)])
    plank = rect(0, SHELF_TOP, 1600, LIP_TOP - SHELF_TOP, fill=WOOD_DARK)
    grad([plank], [mix(GROUND, O700, 0.13), WOOD_LIT], [0, 0], [0, 1])
    grain([plank], 0.09)
    lip = rect(0, LIP_TOP, 1600, COUNTER_Y - LIP_TOP, fill=WOOD_DARK)
    grad([lip], [WOOD_LIT, mix(GROUND, O700, 0.075)], [0, 0], [0, 1])
    grain([lip], 0.09)
    rect(0, LIP_TOP, 1600, 2, fill=al(O100, 0.38))
    rect(0, LIP_TOP + 2, 1600, 1, fill=al(GROUND, 0.5))

    # ---------------------------------------------------------- the goods
    for i, (nm, ver, prov, size, state, nxt, d1, d2) in enumerate(GOODS):
        gx = SLOTS[i]
        lifted = (nm == "ripgrep")
        if loaded:
            cs = rect(gx + 8, SHELF_TOP - 8, BOX_W - 16, 14, fill=al(GROUND, 0.8), radius=7)
            blur([cs], 6)
            if lifted:
                fp = rect(gx + 12, SHELF_TOP - 2, BOX_W - 24, 32, fill=al(WOOD_LIT, 0.5), radius=4)
                inner([fp], 0.35, 0, 2, 5)
            box = S("frame.create", {"x": gx + (-12 if lifted else 0),
                                     "y": GOODS_TOP - (28 if lifted else 0),
                                     "width": BOX_W, "height": BOX_H,
                                     "parent": {"$step": scr},
                                     "name": "good " + nm + (" (lifted)" if lifted else ""),
                                     "props": {"cornerRadius": 3,
                                               "rotation": -2.6 if lifted else 0,
                                               "fills": [{"type": "linearGradient",
                                                          "stops": [mix(WHITE, GROUND, 0.70), CARD],
                                                          "from": [0, 0], "to": [0.2, 1]}],
                                               "strokes": [al(GROUND, 0.75)], "strokeWeight": 1}})
            if lifted:
                shadow([box], 0.72, 18, 32, 28)
            else:
                shadow([box], 0.5, 5, 9, 12)
            vpath(mkpath([("M", 0, 0), ("L", BOX_W, 0), ("L", BOX_W + 18, -14), ("L", 18, -14), ("Z",)]),
                  fill=mix(WHITE, GROUND, 0.64), parent=box)
            vpath(mkpath([("M", BOX_W, 0), ("L", BOX_W + 18, -14), ("L", BOX_W + 18, BOX_H - 14),
                          ("L", BOX_W, BOX_H), ("Z",)]), fill=mix(WHITE, GROUND, 0.86), parent=box)
            rect(0, 0, BOX_W, 1.5, fill=al(WHITE, 0.18), parent=box)
            # the shop's own label, pinned to the package
            lw = max(72, len(nm) * 7.4 + 22)
            lab = rect(20, 20, lw, 30, fill=mix(WHITE, GROUND, 0.52), radius=2, parent=box)
            inner([lab], 0.3, 0, 2, 4)
            ellipse(26, 22, 7, 7, fill=BRASS_LIT, parent=box)
            T(nm, 29, 40, size=12.5, fam="IBM Plex Sans", weight=600, color=al(GROUND, 0.92), parent=box)
            T((ver + " → " + nxt + " · " + prov) if nxt else (ver + " · " + prov),
              20, 78, size=9.5, fam="IBM Plex Mono",
              color=al(MUTED, 0.85) if not nxt else al(O300, 0.95), parent=box)
            T(d1, 20, 106, size=9.5, fam="IBM Plex Sans", color=al(MUTED, 0.66), parent=box)
            T(d2, 20, 122, size=9.5, fam="IBM Plex Sans", color=al(MUTED, 0.66), parent=box)
            # the price tag, hooked on the lip
            rect(gx + 29, LIP_TOP - 6, 2, 12, fill=al(O100, 0.30))
            update = (state == "UPDATE")
            tag = rect(gx + 14, LIP_TOP + 6, 150, 25,
                       fill=ORANGE if update else mix(WHITE, GROUND, 0.46), radius=2)
            shadow([tag], 0.45, 0, 4, 8)
            ellipse(gx + 24, LIP_TOP + 15, 5, 5, fill=al(GROUND if not update else O700, 0.55))
            T(state, gx + 36, LIP_TOP + 22, size=7.5, fam="IBM Plex Sans Condensed",
              weight=700, color=WHITE if update else al(GROUND, 0.82), ls="0.12")
            T(size, gx + 100, LIP_TOP + 23, size=10, fam="IBM Plex Mono",
              color=WHITE if update else al(GROUND, 0.9), align="right", width=58)

    if not loaded:
        for i in range(5):
            gx = SLOTS[i]
            ellipse(gx + 30, LIP_TOP + 8, 7, 7, fill=al(BRASS_LIT, 0.32))
            rect(gx + 33, LIP_TOP + 11, 1, 9, fill=al(O100, 0.14))
    # the shelf label, engraved on the lip
    T("5 ON THE SHELF · 1,284 AT THE ALPM COUNTER" if loaded else "0 ON THE SHELF · 1,284 AT THE ALPM COUNTER",
      44, LIP_TOP + 20, size=8.5, fam="IBM Plex Mono", color=al(O100, 0.4))
    T("SORTED BY NAME ↑" if loaded else "NOTHING ASKED FOR YET",
      44, LIP_TOP + 33, size=8.5, fam="IBM Plex Mono", color=al(O100, 0.26))
    # the density switch, at the other end of the lip
    dsw = rect(1470, LIP_TOP + 4, 92, 36, fill=BRASS_DEEP, radius=3)
    grad([dsw], [BRASS_MID, BRASS_DEEP], [0, 0], [0, 1])
    T("DETAILED", 1478, LIP_TOP + 19, size=7.5, fam="IBM Plex Mono", color=al(O100, 0.5))
    T("COMPACT", 1478, LIP_TOP + 32, size=7.5, fam="IBM Plex Mono", color=al(O100, 0.22))
    rect(1470, LIP_TOP + 4, 40, 2, fill=ORANGE)

    # ---------------------------------------------------------- the counter
    counter = rect(0, COUNTER_Y, 1600, 1000 - COUNTER_Y, fill=WOOD_MID)
    grad([counter], [mix(GROUND, O700, 0.10), mix(GROUND, O700, 0.075), mix(GROUND, O700, 0.045)],
          [0, 0], [0.8, 1])
    grain([counter], 0.085)
    cp = ellipse(200, 540, 1250, 500)
    radial([cp], [al(O100, 0.11), al(O100, 0.0)])
    # a pool of light on the near right of the counter
    cp2 = ellipse(1180, 700, 620, 420)
    radial([cp2], [al(O100, 0.05), al(O100, 0.0)])

    tp = ellipse(-30, 600, 420, 320)
    radial([tp], [al(O100, 0.075), al(O100, 0.0)])
    rect(0, 992, 1600, 2, fill=al(O100, 0.16))
    rect(0, 990, 1600, 2, fill=al(GROUND, 0.5))
    # ---- the till: a physical switch, caught between positions
    till = rect(40, 636, 292, 176, fill=BRASS_DEEP, radius=10)
    grad([till], [BRASS_MID, mix(SLATE, GROUND, 0.6)], [0, 0], [0.35, 1])
    rect(41, 637, 290, 2, fill=al(BRASS_LIT, 0.35), radius=1)
    shadow([till], 0.65, 6, 18, 26)
    rect(40, 798, 292, 14, fill=WOOD_DARK)
    T("THE TILL", 60, 668, size=9.5, fam="IBM Plex Sans Condensed", weight=700,
      color=al(O100, 0.35), ls="0.22")
    chan = rect(64, 682, 244, 54, fill=mix(SLATE, GROUND, 0.9), radius=8)
    inner([chan], 0.9, 0, 4, 10)
    detents = [105, 186, 267]
    for j, dx in enumerate(detents):
        lit = (j == 0)
        if lit:
            g = ellipse(dx - 11, 716, 22, 22)
            radial([g], [al(O100, 0.5), al(O100, 0.0)])
        rect(dx - 1.5, 724, 3, 9, fill=ORANGE if lit else al(SLATE, 0.5), radius=1)
    hx_ = 150 if loaded else detents[0]
    if loaded:
        for dx, op in ((38, 0.06), (24, 0.11)):
            g = rect(hx_ + dx - 31, 690, 62, 32, fill=BRASS_LIT, radius=5, opacity=op)
            grad([g], [BRASS_LIT, BRASS_MID], [0, 0], [0, 1])
    handle = rect(hx_ - 33, 686, 66, 38, fill=BRASS_LIT, radius=5, rotation=-5 if loaded else 0)
    grad([handle], [BRASS_LIT, BRASS_MID, BRASS_DEEP], [0, 0], [0.3, 1])
    shadow([handle], 0.6, 4, 10, 12)
    for k in range(4):
        rect(hx_ - 20 + k * 12, 694, 2, 22, fill=al(BRASS_DEEP, 0.5), radius=1)
    for j, (nm, dx) in enumerate(zip(["FIND", "INVENTORY", "DELTA"], detents)):
        lit = (j == 0)
        T(nm, dx - 45, 764, size=9, fam="IBM Plex Sans Condensed", weight=700,
          color=ORANGE if lit else al(O100, 0.4), ls="0.12", align="center", width=90)

    # ---- the placard: the selected object's ticket, propped on the counter
    if loaded:
        pl = rect(392, 606, 600, 230, fill=BOARD, radius=5)
        grad([pl], [mix(SLATE, GROUND, 0.74), mix(SLATE, GROUND, 0.88)], [0, 0], [0.4, 1])
        shadow([pl], 0.68, 8, 22, 30)
        rect(392, 606, 600, 2, fill=al(BRASS_LIT, 0.4))
        inner([pl], 0.45, 0, 3, 8)
        T("ripgrep", 418, 644, size=25, fam="IBM Plex Sans", weight=600, color=WHITE)
        T("14.1.0-1", 860, 642, size=12, fam="IBM Plex Mono", color=MUTED,
          align="right", width=106)
        rect(418, 660, 548, 1, fill=al(SLATE, 0.28))
        for lab, y, val in [("PROVENANCE", 686, "extra · official repository"),
                            ("SIZE", 736, "5.1 MB · 21.4 MB installed"),
                            ("DEPENDS", 764, "libpcre2-8 · gcc-libs · zlib"),
                            ("FILES", 792, "/usr/bin/rg · man1/rg.1.gz")]:
            T(lab, 418, y, size=9, fam="IBM Plex Sans Condensed", weight=700,
              color=al(MUTED, 0.72), ls="0.18")
            T(val, 512, y, size=10.5, fam="IBM Plex Mono", color=al(WHITE, 0.9))
        T("signed by the packagers — no build recipe", 512, 710, size=8.5,
          fam="IBM Plex Mono", color=al(MUTED, 0.8))
        T("CALIX COMMAND", 730, 682, size=9, fam="IBM Plex Sans Condensed", weight=700,
          color=al(MUTED, 0.72), ls="0.18")
        cmd = rect(730, 692, 236, 38, fill=mix(SLATE, GROUND, 0.92), radius=4)
        inner([cmd], 0.8, 0, 3, 8)
        T("calix install standard ripgrep", 742, 716, size=10.5, fam="IBM Plex Mono", color=O300)
        T("STATE", 730, 756, size=9, fam="IBM Plex Sans Condensed", weight=700,
          color=al(MUTED, 0.72), ls="0.18")
        T("INSTALLED 14.1.0", 730, 778, size=10.5, fam="IBM Plex Mono", color=al(WHITE, 0.9))
        up = rect(730, 788, 142, 22, fill=ORANGE, radius=4)
        shadow([up], 0.4, 0, 3, 7)
        T("UPDATE 14.1.1", 730, 803, size=9, fam="IBM Plex Mono", color=WHITE,
          ls="0.08", align="center", width=142)
    else:
        cp2 = ellipse(300, 580, 860, 400)
        radial([cp2], [al(O100, 0.13), al(O100, 0.0)])
        cp3 = ellipse(520, 620, 520, 300)
        radial([cp3], [al(O100, 0.08), al(O100, 0.0)])

    # ---- the slate: the order, written in chalk
    sframe = rect(1164, 614, 354, 210, fill=WOOD_DARK, radius=7, rotation=-1.6)
    grad([sframe], [WOOD_MID, mix(GROUND, O700, 0.06)], [0, 0], [0.4, 1])
    shadow([sframe], 0.62, 6, 16, 24)
    slate = rect(1178, 628, 326, 182, fill=mix(SLATE, GROUND, 0.9), radius=4, rotation=-1.6)
    grad([slate], [mix(SLATE, GROUND, 0.8), mix(SLATE, GROUND, 0.93)], [0, 0], [0.2, 1])
    shadow([slate], 0.6, 6, 16, 24)
    inner([slate], 0.6, 0, 3, 9)
    rect(1176, 626, 330, 2, fill=al(BRASS_LIT, 0.32))
    if loaded:
        T("REQUEST", 1200, 664, size=9, fam="IBM Plex Mono", color=al(MUTED, 0.6), ls="0.16")
        T("ri", 1200, 714, size=28, fam="IBM Plex Mono", color=WHITE)
        rect(1248, 684, 3, 34, fill=ORANGE)
        rect(1200, 756, 230, 1, fill=al(SLATE, 0.28))
        T("5 ON THE SHELF", 1200, 780, size=9.5, fam="IBM Plex Mono", color=al(SLATE, 1.0))
    else:
        T("WHAT ARE YOU", 1200, 692, size=21, fam="IBM Plex Sans Condensed", weight=700,
          color=al(WHITE, 0.9), ls="0.04")
        T("LOOKING FOR?", 1200, 720, size=21, fam="IBM Plex Sans Condensed", weight=700,
          color=al(WHITE, 0.9), ls="0.04")
        rect(1200, 736, 3, 28, fill=ORANGE)
        rect(1200, 756, 230, 1, fill=al(SLATE, 0.28))
        T("TYPE A NAME · 1,284 ON THIS COUNTER", 1200, 780, size=9.5, fam="IBM Plex Mono",
          color=al(SLATE, 1.0))

    # ---- the receipt: what the backend actually ran
    if loaded:
        rec = vpath(mkpath([("M", 52, 872), ("L", 880, 872), ("L", 880, 1012), ("L", 52, 1012), ("Z",)]),
                    fill=PAPER_HOT)
        shadow([rec], 0.5, 4, 12, 18)
        rect(52, 872, 828, 1, fill=al(WHITE, 0.4))
        for i, line in enumerate([
                "20:14:02   pacman -Sy --needed ripgrep",
                "20:14:02   resolving: libpcre2-8 gcc-libs zlib",
                "20:14:03   downloading ripgrep-14.1.1-1-x86_64.pkg.tar.zst",
                "20:14:04   verifying signature ... ok",
                "20:14:05   installed ripgrep 14.1.1"]):
            T(line, 72, 898 + i * 19, size=10.5, fam="IBM Plex Mono",
              color=al(INK, 0.82) if i != 3 else al(O700, 0.95))

    # ---------------------------------------------------------- the sign
    for cx_ in (60, 172):
        rect(cx_, 0, 2, 40, fill=al(SLATE, 0.6))
    sign = rect(30, 40, 176, 196, fill=BOARD, radius=6)
    grad([sign], [mix(SLATE, GROUND, 0.74), mix(SLATE, GROUND, 0.9)], [0, 0], [0.3, 1])
    shadow([sign], 0.7, 8, 20, 28)
    rect(30, 40, 176, 2, fill=al(BRASS_LIT, 0.4))
    inner([sign], 0.4, 0, 3, 8)
    scallop_mark(78, 50, 0.60, scr)
    T("CALIX", 30, 196, size=21, fam="IBM Plex Sans Condensed", weight=700,
      color=WHITE, ls="0.18", align="center", width=176)
    rect(92, 208, 52, 2, fill=al(O300, 0.75))
    return scr


loaded_frame = build_screen(0, True, "Le Comptoir — loaded")
quiet_frame = build_screen(1700, False, "Le Comptoir — quiet")

# ================================================================ build
with open(os.path.join(OUT_DIR, "build.jsonl"), "w") as f:
    for s in steps:
        f.write(json.dumps(s) + "\n")
print("steps:", len(steps))

out = os.path.join(OUT_DIR, "04-comptoir.figcraft")
r = subprocess.run(["figcraft-cli", "run", os.path.join(OUT_DIR, "build.jsonl"), "--out", out],
                   capture_output=True, text=True)
lines = [l for l in (r.stdout + r.stderr).strip().splitlines() if '"ok":false' in l or l.startswith("error")]
print("\n".join(lines[-8:]) if lines else "build ok")

if r.returncode == 0:
    for nm, tag in (("Le Comptoir — loaded", "loaded"), ("Le Comptoir — quiet", "quiet")):
        png = os.path.join(OUT_DIR, f"04-comptoir-{tag}.png")
        r2 = subprocess.run(["figcraft-cli", "export", out, "-o", png, "--scale", "1", "--node", nm],
                            capture_output=True, text=True)
        print(tag, (r2.stdout + r2.stderr)[-160:].strip())
        from PIL import Image
        print("  size:", Image.open(png).size)
