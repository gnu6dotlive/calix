#!/usr/bin/env python3
"""L'Atelier — Calix package manager, design proposal 04.

Builds a 1600x1000 dark desktop screen with figcraft-cli.
Every colour is derived from the design-system tokens.
"""
import json, math, subprocess, os, sys

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

SURF1 = mix(GROUND, WHITE, 0.035)
SURF2 = mix(GROUND, WHITE, 0.065)
SURF3 = mix(GROUND, WHITE, 0.11)
HAIR = al(SLATE, 0.34)
HAIR2 = al(SLATE, 0.18)
STEEL_HI = mix(SLATE, WHITE, 0.55)
STEEL_MD = SLATE
STEEL_LO = mix(SLATE, GROUND, 0.45)

# ---------------------------------------------------------------- steps
steps = []
def S(cmd, params):
    steps.append({"command": cmd, "params": params})
    return len(steps) - 1

ROOT = S("frame.create", {"x": 0, "y": 0, "width": 1600, "height": 1000,
                           "name": "CALIX — L'Atelier", "props": {"fill": GROUND}})

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
                                 "parent": {"$step": parent if parent is not None else ROOT},
                                 "props": p})

def ellipse(x, y, w, h, fill=None, stroke=None, sw=1, rotation=None, opacity=None, parent=None):
    p = {"fill": fill if fill is not None else "#00000000"}
    if stroke is not None:
        p["stroke"] = stroke; p["strokeWeight"] = sw
    if rotation is not None: p["rotation"] = rotation
    if opacity is not None: p["opacity"] = opacity
    return S("shape.ellipse", {"x": x, "y": y, "width": w, "height": h,
                               "parent": {"$step": parent if parent is not None else ROOT},
                               "props": p})

def T(s, x, cap, size=12, fam="IBM Plex Sans", weight=400, color=WHITE, ls=None,
      width=None, align=None, opacity=None, lineh=None, parent=None):
    props = {"fontSize": size, "fontFamily": fam, "fontWeight": weight, "fill": color}
    if ls is not None: props["letterSpacing"] = ls
    if width is not None: props["width"] = width
    if align is not None: props["textAlign"] = align
    if opacity is not None: props["opacity"] = opacity
    if lineh is not None: props["lineHeight"] = lineh
    return S("text.create", {"text": s, "x": x, "y": cap - 0.33 * size,
                             "parent": {"$step": parent if parent is not None else ROOT},
                             "props": props})

def tw(s, size, fam="IBM Plex Sans", ls=0.0):
    if fam == "IBM Plex Mono": base = 0.602
    elif "Condensed" in fam: base = 0.50
    else: base = 0.512
    return len(s) * (base + ls) * size

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

def grad(ids, stops, frm=None, to=None):
    paint = {"type": "linearGradient", "stops": stops}
    if frm is not None: paint["from"] = frm
    if to is not None: paint["to"] = to
    S("node.set", {"ids": [{"$step": i} for i in ids], "props": {"fills": [paint]}})

def mkpath(cmds):
    out = []
    for k, nums in cmds:
        if k == "Z":
            out.append("Z")
        else:
            out.append(k + " " + " ".join(("%.2f" % n) for n in nums))
    return " ".join(out)

def vpath(d, fill=None, stroke=None, sw=1, opacity=None, parent=None):
    p = {}
    if fill is not None: p["fill"] = fill
    if stroke is not None:
        p["stroke"] = stroke; p["strokeWeight"] = sw
    if opacity is not None: p["opacity"] = opacity
    return S("vector.create", {"path": d, "x": 0, "y": 0, "closed": True,
                                "parent": {"$step": parent if parent is not None else ROOT},
                                "props": p})

# ================================================================ THE MARK
def scallop_mark(ox, oy, sc, parent=None):
    """A scallop shell as the cup of a chalice: a fan of ribs sitting in a
    metal cup on a brushed aluminium stem.  Local box: x 0..120, y 24..153."""
    ids = []
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
            k = c[0]
            pts = []
            for i in range(1, len(c), 2):
                pts.extend(P(c[i], c[i + 1]))
            out.append((k, pts))
        return mkpath(out)

    # 1. the cup's interior, seen behind the shell
    it = vpath(path([("M", 8, 56), ("C", 30, 68, 90, 68, 112, 56),
                     ("L", 112, 68), ("C", 90, 82, 30, 82, 8, 68), ("Z",)]),
               fill=mix(SLATE, GROUND, 0.32), parent=parent)
    ids.append(it)

    # 2. the shell: a fan of ribs with a gently scalloped margin
    d = [("M", HX, HY)]
    t0 = pol(ang[0], R_TIP)
    d.append(("Q", HX - 6, HY - 12, t0[0], t0[1]))
    margin = [("M", t0[0], t0[1])]
    for i in range(N - 1):
        am = (ang[i] + ang[i + 1]) / 2
        val = pol(am, R_VAL)
        c1 = pol(am, R_TIP + 0.5)
        c2 = pol(am, R_VAL - 2)
        d.append(("C", c1[0], c1[1], c2[0], c2[1], val[0], val[1]))
        margin.append(("C", c1[0], c1[1], c2[0], c2[1], val[0], val[1]))
        c3 = pol(am, R_VAL - 2)
        c4 = pol(am, R_TIP + 0.5)
        d.append(("C", c3[0], c3[1], c4[0], c4[1], pol(ang[i + 1], R_TIP)[0], pol(ang[i + 1], R_TIP)[1]))
        margin.append(("C", c3[0], c3[1], c4[0], c4[1], pol(ang[i + 1], R_TIP)[0], pol(ang[i + 1], R_TIP)[1]))
    t6 = pol(ang[N - 1], R_TIP)
    d.append(("Q", HX + 6, HY - 12, t6[0], t6[1]))
    d.append(("Q", HX + 4, HY + 3, HX, HY))
    d.append(("Z",))
    shell = vpath(path(d), fill=ORANGE, parent=parent)
    ids.append(shell)
    grad([shell], [O300, ORANGE, O700], [0.05, 0], [0.95, 1])

    # 3. rim light along the margin
    ml = vpath(path(margin), stroke=O100, sw=1.4 * sc, opacity=0.38, parent=parent)
    ids.append(ml)

    # 4. ribs: a lit ridge on each rib, a groove in each valley
    for a in ang:
        w = math.radians(1.7)
        p1 = pol(a - w, R_TIP - 4)
        p2 = pol(a + w, R_TIP - 4)
        ridge = vpath(path([("M", HX, HY), ("L", p1[0], p1[1]), ("L", p2[0], p2[1]), ("Z",)]),
                      fill=O100, opacity=0.13, parent=parent)
        ids.append(ridge)
    for i in range(N - 1):
        am = (ang[i] + ang[i + 1]) / 2
        g0 = pol(am, 7)
        g1 = pol(am, R_VAL - 2)
        groove = vpath(path([("M", g0[0], g0[1]), ("L", g1[0], g1[1])]), stroke=O700,
                        sw=1.3 * sc, opacity=0.34, parent=parent)
        ids.append(groove)

    # 5. the cup's front wall, drawn over the shell's foot
    wall = path([("M", 8, 68), ("C", 30, 82, 90, 82, 112, 68),
                 ("C", 110, 102, 92, 115, 72, 116),
                 ("L", 48, 116),
                 ("C", 28, 115, 10, 102, 8, 68), ("Z",)])
    wl = vpath(wall, fill=STEEL_MD, parent=parent)
    ids.append(wl)
    grad([wl], [STEEL_HI, STEEL_MD, STEEL_LO], [0.05, 0], [0.25, 1])

    rim = vpath(path([("M", 8, 68), ("C", 30, 82, 90, 82, 112, 68)]), stroke=STEEL_HI,
                sw=2.8 * sc, opacity=0.9, parent=parent)
    ids.append(rim)

    # 6. stem
    stem = rect(P(55, 115)[0], P(55, 115)[1], 10 * sc, 27 * sc, fill=STEEL_MD,
                radius=2 * sc, parent=parent)
    ids.append(stem)
    grad([stem], [STEEL_HI, STEEL_LO, STEEL_HI], [0, 0], [1, 0])

    # 7. foot and base
    foot = vpath(path([("M", 55, 141), ("L", 65, 141), ("L", 74, 147), ("L", 46, 147), ("Z",)]),
                 fill=STEEL_MD, parent=parent)
    ids.append(foot)
    grad([foot], [STEEL_HI, STEEL_LO], [0, 0], [0, 1])
    base = ellipse(P(38, 144)[0], P(38, 144)[1], 44 * sc, 10 * sc, fill=STEEL_MD, parent=parent)
    ids.append(base)
    grad([base], [STEEL_HI, STEEL_LO], [0, 0], [0, 1])
    return ids

# ================================================================ 1. HEADER
hdr = rect(0, 0, 1600, 92, fill=SURF1)
grad([hdr], [mix(GROUND, WHITE, 0.08), mix(GROUND, WHITE, 0.02)], [0, 0], [0, 1])
grain([hdr], 0.05)
shadow([hdr], 0.55, 0, 10, 22)
rect(0, 90, 1600, 1, fill=al(SLATE, 0.45))
rect(0, 91, 1600, 1, fill=al(WHITE, 0.06))

scallop_mark(26, 6, 0.55)

T("CALIX", 112, 30, size=33, fam="IBM Plex Sans Condensed", weight=700, color=WHITE, ls="0.16")
rect(113, 62, 46, 3, fill=ORANGE, radius=1.5)

rect(266, 22, 1, 48, fill=al(SLATE, 0.5))

# where am I — tray › tool › filter, labels baseline-aligned with the values
T("TRAY", 296, 35.5, size=10.5, fam="IBM Plex Mono", color=MUTED, ls="0.14")
rect(332, 31, 8, 8, fill=ORANGE)
T("ALPM", 348, 28, size=21, fam="IBM Plex Sans Condensed", weight=700, color=WHITE, ls="0.04")
T("›", 406, 28, size=19, fam="IBM Plex Mono", color=al(MUTED, 0.7))
T("TOOL", 430, 35.5, size=10.5, fam="IBM Plex Mono", color=MUTED, ls="0.14")
T("FIND", 464, 28, size=21, fam="IBM Plex Sans Condensed", weight=700, color=ORANGE, ls="0.04")
T("›", 520, 28, size=19, fam="IBM Plex Mono", color=al(MUTED, 0.7))
T("FILTER", 544, 35.5, size=10.5, fam="IBM Plex Mono", color=MUTED, ls="0.14")
T("name ~ z", 596, 30, size=16, fam="IBM Plex Mono", color=WHITE)

T("INDEX FRESHNESS", 1290, 26, size=10, fam="IBM Plex Mono", color=MUTED, ls="0.14")
ellipse(1290, 40, 8, 8, fill=O300)
T("4 MIN OLD", 1306, 42, size=15, fam="IBM Plex Sans Condensed", weight=700, color=WHITE, ls="0.04")
T("·  20:14:09", 1404, 42, size=13, fam="IBM Plex Mono", color=MUTED)

# ================================================================ 2. TRAY COLUMN
col = rect(0, 92, 292, 764, fill=mix(GROUND, WHITE, 0.018))
grain([col], 0.06)
rect(291, 92, 1, 764, fill=al(SLATE, 0.28))

T("TRAYS", 24, 122, size=14, fam="IBM Plex Sans Condensed", weight=700, color=WHITE, ls="0.18")
T("7 SOURCES", 148, 121, size=10, fam="IBM Plex Mono", color=MUTED, ls="0.12", align="right", width=120)
rect(24, 132, 244, 1, fill=HAIR2)

trays = [
    ("ALPM", "1,284", "core · extra · multilib", True, False),
    ("AUR", "8,431", "recipes, not binaries", False, False),
    ("FLATPAK", "96", "sandboxed applications", False, False),
    ("APPIMAGE", "41", "self-contained binaries", False, False),
    ("CARGO", "612", "crates.io · rust", False, False),
    ("NPM", "1,308", "global node packages", False, False),
    ("TRACKED RELEASES", "0", "nothing followed yet", False, True),
]
ty = 148
for name, count, holds, sel, empty in trays:
    h = 62
    if empty:
        card = rect(24, ty, 244, h, fill=mix(GROUND, WHITE, 0.012), radius=6,
                    stroke=al(SLATE, 0.42), sw=1, dashes=[5, 4])
        inner([card], 0.5, 0, 3, 7)
        T(name, 40, ty + 24, size=12.5, fam="IBM Plex Sans Condensed", weight=700,
          color=al(MUTED, 0.85), ls="0.14")
        T("⌘F TO FOLLOW A RELEASE", 40, ty + 46, size=10, fam="IBM Plex Mono",
          color=al(ORANGE, 0.75), ls="0.08")
    else:
        card = rect(24, ty, 244, h, fill=SURF2 if sel else mix(GROUND, WHITE, 0.035),
                    radius=6, stroke=al(ORANGE, 0.5) if sel else HAIR2)
        shadow([card], 0.55 if sel else 0.4, 6 if sel else 0, 8 if sel else 3, 16 if sel else 8)
        grain([card], 0.05)
        bar = rect(24, ty + 8, 3, h - 16, fill=ORANGE if sel else al(SLATE, 0.6), radius=1.5)
        if sel:
            shadow([bar], 0.6, 0, 0, 6)
        T(name, 40, ty + 24, size=14, fam="IBM Plex Sans Condensed", weight=700,
          color=WHITE if sel else mix(MUTED, WHITE, 0.15), ls="0.14")
        T(count, 212, ty + 24, size=11, fam="IBM Plex Mono",
          color=ORANGE if sel else MUTED, align="right", width=56)
        T(holds, 40, ty + 46, size=10.5, fam="IBM Plex Mono", color=al(MUTED, 0.78))
    ty += h + 10

rules = rect(24, 660, 244, 88, fill=mix(GROUND, WHITE, 0.022), radius=6, stroke=HAIR2)
T("THE RULE OF THE BENCH", 40, 680, size=10, fam="IBM Plex Sans Condensed", weight=700,
  color=al(MUTED, 0.9), ls="0.16")
T("A TRAY HOLDS ONLY ITS OWN KIND.", 40, 700, size=10, fam="IBM Plex Mono", color=al(MUTED, 0.85))
T("A FLATPAK IS NEVER UPDATED", 40, 718, size=10, fam="IBM Plex Mono", color=al(MUTED, 0.85))
T("FROM A CARGO TRAY.", 40, 736, size=10, fam="IBM Plex Mono", color=al(MUTED, 0.85))

T("⌥←  ⌥→   MOVE BETWEEN TRAYS", 24, 772, size=10, fam="IBM Plex Mono", color=al(SLATE, 0.95))
T("⌘1 ⌘2 ⌘3   PICK A TOOL", 24, 792, size=10, fam="IBM Plex Mono", color=al(SLATE, 0.95))

# ================================================================ 3. TOOL RAIL
rail = rect(316, 104, 900, 80, fill=mix(GROUND, WHITE, 0.05), radius=8, stroke=HAIR2)
grad([rail], [mix(GROUND, WHITE, 0.10), mix(GROUND, WHITE, 0.03)], [0, 0], [0, 1])
grain([rail], 0.07)
inner([rail], 0.45, 0, 3, 10)
rect(317, 105, 898, 1, fill=al(WHITE, 0.08))

tools = [("FIND", "⌘1", "1,284", True), ("INVENTORY", "⌘2", "612", False), ("DELTA", "⌘3", "18", False)]
tx = 332
for name, key, tally, sel in tools:
    blk = rect(tx, 112 if sel else 116, 158, 56, fill=SURF3 if sel else SURF2, radius=8,
               stroke=al(ORANGE, 0.55) if sel else HAIR, rotation=-2.2 if sel else 0)
    if sel:
        shadow([blk], 0.65, 8, 20, 24)
        # the picked-up tool carries an orange edge along its raised side
        rect(tx + 7, 110.5, 142, 3, fill=ORANGE, radius=1.5, rotation=-2.2)
    else:
        shadow([blk], 0.45, 0, 5, 10)
    T(name, tx + 16, 112 + 28 if sel else 116 + 24, size=15.5, fam="IBM Plex Sans Condensed",
      weight=700, color=WHITE if sel else al(MUTED, 0.92), ls="0.14")
    T(key, tx + 114, 112 + 28 if sel else 116 + 24, size=11, fam="IBM Plex Mono",
      color=al(ORANGE, 0.9) if sel else al(MUTED, 0.8), align="right", width=30)
    T(tally, tx + 114, 112 + 44 if sel else 116 + 40, size=9.5, fam="IBM Plex Mono",
      color=al(ORANGE, 0.75) if sel else al(SLATE, 1.0), align="right", width=30)
    tx += 168

sf = rect(852, 116, 348, 56, fill=mix(GROUND, WHITE, 0.02), radius=8, stroke=HAIR)
inner([sf], 0.75, 0, 4, 10)
ellipse(868, 134, 14, 14, stroke=al(MUTED, 0.85), sw=1.7)
rect(879, 148, 8, 1.8, fill=al(MUTED, 0.85), rotation=45)
T("z", 900, 133, size=19, fam="IBM Plex Mono", color=WHITE)
rect(913, 129, 2, 20, fill=ORANGE)
T("SEARCH ALPM", 900, 162, size=9.5, fam="IBM Plex Mono", color=al(MUTED, 0.8), ls="0.12")
T("⌘F", 1160, 138, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.85), align="right", width=36)

# ================================================================ 4. SURFACE
surf = rect(316, 200, 900, 656, fill=mix(GROUND, WHITE, 0.045), radius=10)
grad([surf], [mix(GROUND, WHITE, 0.055), mix(GROUND, WHITE, 0.03)], [0, 0], [0, 1])
grain([surf], 0.07)
shadow([surf], 0.5, 0, 14, 26)
rect(317, 201, 898, 1, fill=al(WHITE, 0.07))

for gx in range(376, 1216, 60):
    rect(gx, 252, 1, 604, fill=al(SLATE, 0.13))
for gy in range(262, 856, 52):
    rect(316, gy, 900, 1, fill=al(SLATE, 0.13))

# a bench rule along the mat's edge — a texture you notice only when you look
for i in range(43):
    rx = 352 + i * 20
    tall = (i % 5 == 0)
    rect(rx, 826 if tall else 830, 1, 10 if tall else 5, fill=al(SLATE, 0.5 if tall else 0.32))

T("6 SHOWN", 336, 222, size=12, fam="IBM Plex Mono", color=WHITE)
T("·  1,284 IN ALPM  ·  FILTER name~z  ·  SORT NAME ↑", 392, 222, size=12,
  fam="IBM Plex Mono", color=al(MUTED, 0.95))
# density switch
d1 = rect(1030, 206, 82, 24, fill=SURF3, radius=5, stroke=al(ORANGE, 0.5))
shadow([d1], 0.5, 0, 4, 8)
T("DETAILED", 1030, 222, size=9, fam="IBM Plex Mono", color=WHITE, ls="0.06", align="center", width=82)
d2 = rect(1114, 206, 82, 24, fill=mix(GROUND, WHITE, 0.012), radius=5, stroke=HAIR2, dashes=[4, 3])
T("COMPACT", 1114, 222, size=9, fam="IBM Plex Mono", color=al(MUTED, 0.85), ls="0.06", align="center", width=82)

for lx, lab in ((352, "PACKAGE"), (680, "PROVENANCE"), (776, "VERSION"), (868, "SIZE"), (952, "STATE")):
    T(lab, lx, 248, size=9.5, fam="IBM Plex Mono", color=al(MUTED, 0.75), ls="0.16")
rect(336, 258, 860, 1, fill=HAIR2)

VPX, VPY = 316, 258
vp = S("frame.create", {"x": VPX, "y": VPY, "width": 900, "height": 402, "parent": {"$step": ROOT},
                        "name": "list viewport", "props": {"clip": True, "fill": "#00000000"}})

rows = [
    ("zbar", "0.23-1", "extra", "384 KB", "AVAILABLE", "reads barcodes from images and video streams", None),
    ("zenity", "3.0.0-2", "extra", "1.1 MB", "UPDATE", "graphical dialog boxes for shell scripts", "3.0.0-3"),
    ("zeromq", "4.3.5-1", "extra", "2.4 MB", "AVAILABLE", "high-performance messaging library", None),
    ("zram-generator", "1.5.1-2", "extra", "24 KB", "INSTALLED", "compressed swap devices backed by zstd", None),
    ("zsh", "5.9-3", "extra", "5.8 MB", "UPDATE", "powerful shell with scriptable completion", "5.9-4"),
    ("zstd", "1.5.6-1", "core", "712 KB", "INSTALLED", "zstandard compression library", None),
]
RH = 74
for i, (name, ver, prov, size, state, desc, nxt) in enumerate(rows):
    y0 = i * RH
    selected = (name == "zsh")
    if selected:
        f = S("frame.create", {"x": 20, "y": y0 + 3, "width": 860, "height": 66,
                               "parent": {"$step": vp}, "name": "row zsh (picked up)",
                               "props": {"cornerRadius": 8, "rotation": -0.55,
                                         "fills": [{"type": "linearGradient",
                                                    "stops": [SURF3, mix(GROUND, WHITE, 0.075)],
                                                    "from": [0, 0], "to": [0, 1]}],
                                         "strokes": [al(ORANGE, 0.5)], "strokeWeight": 1}})
        shadow([f], 0.68, 5, 16, 22)
        rect(8, 1, 844, 1, fill=al(WHITE, 0.08), parent=f)
        rect(0, 8, 3.5, 50, fill=ORANGE, radius=1.8, parent=f)
        T(name, 20, 22, size=15.5, fam="IBM Plex Sans", weight=600, color=WHITE, parent=f)
        T(desc, 20, 44, size=12, fam="IBM Plex Sans", color=al(MUTED, 0.92), parent=f)
        T(prov, 344, 26, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.95), parent=f)
        T(ver, 440, 26, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.95), parent=f)
        T("→ " + nxt, 440 + tw(ver, 11, "IBM Plex Mono") + 10, 26, size=11, fam="IBM Plex Mono",
          color=O300, parent=f)
        T(size, 566, 26, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.95), parent=f)
        chip = rect(646, 12, 92, 23, fill=ORANGE, radius=4, parent=f)
        shadow([chip], 0.45, 0, 3, 6)
        T(state, 646, 26, size=9.5, fam="IBM Plex Mono", color=WHITE, ls="0.10",
          align="center", width=92, parent=f)
    else:
        T(name, 36, y0 + 22, size=15.5, fam="IBM Plex Sans", weight=600,
          color=mix(WHITE, GROUND, 0.12), parent=vp)
        T(desc, 36, y0 + 44, size=12, fam="IBM Plex Sans", color=al(MUTED, 0.8), parent=vp)
        T(prov, 364, y0 + 26, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.8), parent=vp)
        T(ver, 460, y0 + 26, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.8), parent=vp)
        T(size, 552, y0 + 26, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.8), parent=vp)
        if state == "INSTALLED":
            rect(636, y0 + 12, 92, 23, fill=mix(GROUND, WHITE, 0.06), radius=4,
                 stroke=al(SLATE, 0.35), parent=vp)
            T(state, 636, y0 + 26, size=9.5, fam="IBM Plex Mono", color=al(MUTED, 0.95),
              ls="0.10", align="center", width=92, parent=vp)
        elif state == "UPDATE":
            chip = rect(636, y0 + 12, 92, 23, fill=ORANGE, radius=4, parent=vp)
            shadow([chip], 0.4, 0, 3, 6)
            T(state, 636, y0 + 26, size=9.5, fam="IBM Plex Mono", color=WHITE,
              ls="0.10", align="center", width=92, parent=vp)
            T(nxt, 736, y0 + 26, size=11, fam="IBM Plex Mono", color=O300, parent=vp)
        else:
            rect(636, y0 + 12, 92, 23, fill=mix(GROUND, WHITE, 0.01), radius=4,
                 stroke=al(SLATE, 0.3), dashes=[4, 3], parent=vp)
            T(state, 636, y0 + 26, size=9.5, fam="IBM Plex Mono", color=al(MUTED, 0.85),
              ls="0.10", align="center", width=92, parent=vp)
        if i < len(rows) - 1:
            rect(20, y0 + RH - 1, 860, 1, fill=al(SLATE, 0.12), parent=vp)

fade = rect(316, 588, 900, 72, fill=mix(GROUND, WHITE, 0.04))
grad([fade], [al(mix(GROUND, WHITE, 0.04), 0.0), al(mix(GROUND, WHITE, 0.04), 1.0)], [0, 0], [0.62, 1])
rect(1206, 300, 3, 180, fill=al(SLATE, 0.5), radius=1.5)

slot = rect(352, 726, 500, 80, fill=mix(GROUND, WHITE, 0.012), radius=8,
            stroke=al(SLATE, 0.45), sw=1, dashes=[6, 5])
inner([slot], 0.55, 0, 4, 9)
T("EMPTY SLOT", 372, 756, size=12, fam="IBM Plex Sans Condensed", weight=700,
  color=al(ORANGE, 0.85), ls="0.18")
T("AN OFFICIAL-REPOSITORY PACKAGE WOULD SIT HERE.", 372, 780, size=10.5,
  fam="IBM Plex Mono", color=al(MUTED, 0.9))
T("⌘F TO PLACE ONE", 700, 780, size=10.5, fam="IBM Plex Mono", color=al(MUTED, 0.9))

# ================================================================ 5. INSPECTION CARD
card = rect(1240, 104, 336, 752, fill=SURF1, radius=12)
grad([card], [mix(GROUND, WHITE, 0.07), mix(GROUND, WHITE, 0.035)], [0, 0], [0.3, 1])
grain([card], 0.06)
shadow([card], 0.6, 0, 20, 34)
rect(1241, 105, 334, 1, fill=al(WHITE, 0.09))

T("INSPECTION CARD", 1264, 136, size=11.5, fam="IBM Plex Sans Condensed", weight=700,
  color=ORANGE, ls="0.18")
T("LIFTED FROM THE BENCH", 1264, 156, size=9, fam="IBM Plex Mono", color=al(SLATE, 1.0), ls="0.10")
T("⌘4", 1512, 135, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.8), align="right", width=40)

T("zsh", 1264, 186, size=38, fam="IBM Plex Sans Condensed", weight=700, color=WHITE, ls="0.02")
T("5.9-3", 1472, 186, size=15, fam="IBM Plex Mono", color=MUTED, align="right", width=80)
rect(1264, 204, 288, 1, fill=HAIR2)

def card_section(y, label):
    T(label, 1264, y, size=10, fam="IBM Plex Sans Condensed", weight=700,
      color=al(MUTED, 0.95), ls="0.18")
    rect(1264, y + 10, 288, 1, fill=al(SLATE, 0.22))

card_section(236, "PROVENANCE")
T("extra — official repository", 1264, 262, size=14.5, fam="IBM Plex Sans", weight=600, color=WHITE)
T("SIGNED BY THE PACKAGERS. NO BUILD RECIPE —", 1264, 284, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.9))
T("OFFICIAL PACKAGES ARE ALREADY BUILT.", 1264, 300, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.9))

card_section(336, "STATE")
T("INSTALLED", 1264, 360, size=10, fam="IBM Plex Mono", color=al(MUTED, 0.95), ls="0.12")
T("5.9-3", 1350, 360, size=12, fam="IBM Plex Mono", color=WHITE)
upd = rect(1414, 345, 138, 22, fill=ORANGE, radius=4)
shadow([upd], 0.45, 0, 3, 7)
T("UPDATE 5.9-4", 1414, 360, size=9.5, fam="IBM Plex Mono", color=WHITE, ls="0.08",
  align="center", width=138)
T("5.8 MB DOWNLOAD  ·  18.4 MB INSTALLED", 1264, 382, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.9))

card_section(416, "DEPENDENCIES")
for i, dep in enumerate(["libreadline 8.2", "ncurses 6.4", "pcre2 10.42", "zlib 1.3", "gmp 6.2"]):
    T(dep, 1264, 442 + i * 20, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.95))
    T("●", 1528, 441 + i * 20, size=9, fam="IBM Plex Mono", color=al(O300, 0.8), align="right", width=12)

card_section(556, "FILES")
for i, fpath in enumerate(["/bin/zsh", "/usr/bin/zsh", "/usr/share/man/man1/zsh.1.gz", "/usr/share/zsh/functions/…"]):
    T(fpath, 1264, 582 + i * 20, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.95))

card_section(676, "THE COMMAND")
cmd = rect(1264, 692, 288, 42, fill=mix(GROUND, WHITE, 0.015), radius=6, stroke=HAIR)
inner([cmd], 0.75, 0, 4, 9)
T("pacman -S zsh", 1280, 719, size=14, fam="IBM Plex Mono", color=O300)
T("⌘⌥C", 1516, 718, size=10.5, fam="IBM Plex Mono", color=al(MUTED, 0.85), align="right", width=36)

b1 = rect(1264, 752, 150, 40, fill=ORANGE, radius=6)
shadow([b1], 0.5, 0, 6, 12)
T("INSTALL 5.9-4", 1264, 777, size=11, fam="IBM Plex Mono", color=WHITE, ls="0.10",
  align="center", width=150)
b2 = rect(1424, 752, 62, 40, fill=mix(GROUND, WHITE, 0.02), radius=6, stroke=HAIR)
T("REMOVE", 1424, 777, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.95), ls="0.10",
  align="center", width=62)
b3 = rect(1496, 752, 56, 40, fill=mix(GROUND, WHITE, 0.02), radius=6, stroke=HAIR)
T("FILES", 1496, 777, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.95), ls="0.10",
  align="center", width=56)
T("THESE ACTIONS BELONG TO THE ALPM TRAY:",
  1264, 818, size=10, fam="IBM Plex Mono", color=al(SLATE, 1.0))
T("NO BUILD, NO RECIPE.", 1264, 834, size=10, fam="IBM Plex Mono", color=al(SLATE, 1.0))

# ================================================================ 6. CONSOLE
con = rect(0, 856, 1600, 144, fill=SURF1)
grad([con], [mix(GROUND, WHITE, 0.06), mix(GROUND, WHITE, 0.015)], [0, 0], [0, 1])
grain([con], 0.06)
shadow([con], 0.5, 0, -8, 18)
rect(0, 856, 1600, 1, fill=al(SLATE, 0.42))
rect(0, 857, 1600, 1, fill=al(WHITE, 0.05))

T("OPERATIONS IN FLIGHT", 24, 880, size=10, fam="IBM Plex Sans Condensed", weight=700,
  color=al(MUTED, 0.95), ls="0.18")

ellipse(26, 896, 9, 9, fill=ORANGE)
T("SYNCING ALPM INDEX", 44, 904, size=12, fam="IBM Plex Mono", color=WHITE)
T("68%", 476, 904, size=11, fam="IBM Plex Mono", color=ORANGE, align="right", width=40)
track = rect(24, 918, 656, 7, fill=mix(GROUND, WHITE, 0.015), radius=3.5)
inner([track], 0.8, 0, 2, 5)
bar = rect(24, 918, 446, 7, fill=ORANGE, radius=3.5)
grad([bar], [O700, ORANGE, O100], [0, 0], [1, 0])
rect(466, 916, 3, 11, fill=O100, radius=1.5)

ellipse(26, 940, 9, 9, fill=al(SLATE, 0.9))
T("DOWNLOADING ZSH-5.9-4-X86_64.PKG.TAR.ZST", 44, 948, size=12, fam="IBM Plex Mono", color=al(MUTED, 0.95))
T("2.4 MB", 600, 948, size=11, fam="IBM Plex Mono", color=al(MUTED, 0.9), align="right", width=40)
track2 = rect(24, 962, 656, 5, fill=mix(GROUND, WHITE, 0.015), radius=2.5)
inner([track2], 0.8, 0, 2, 4)
rect(24, 962, 128, 5, fill=al(SLATE, 0.85), radius=2.5)

T("PACMAN -S --NEEDED ZSH", 24, 986, size=11, fam="IBM Plex Mono", color=al(SLATE, 1.0))

rect(716, 872, 1, 112, fill=al(SLATE, 0.3))
T("LOG", 740, 880, size=10, fam="IBM Plex Sans Condensed", weight=700, color=al(MUTED, 0.95), ls="0.18")
for i, (ts, msg, col) in enumerate([
        ("20:14:02", "RESOLVED 1284 PACKAGES IN 41 MS", MUTED),
        ("20:14:05", "ZSH 5.9-3 → 5.9-4", O300),
        ("20:14:09", "DOWNLOADING ZSH-5.9-4 (2.4 MB)", MUTED),
        ("20:14:11", "WAITING FOR PACKAGER SIGNATURE", al(MUTED, 0.8))]):
    T(ts, 740, 906 + i * 20, size=11, fam="IBM Plex Mono", color=al(SLATE, 1.0))
    T(msg, 812, 906 + i * 20, size=11, fam="IBM Plex Mono", color=col)

rect(1156, 872, 1, 112, fill=al(SLATE, 0.3))
T("KEYBOARD", 1180, 880, size=10, fam="IBM Plex Sans Condensed", weight=700, color=al(MUTED, 0.95), ls="0.18")
keys = [("⌘1", "FIND"), ("⌘2", "INVENTORY"), ("⌘3", "DELTA"), ("⌘4", "CARD"),
        ("⌘5", "CONSOLE"), ("⌥←→", "TRAY"), ("⌘F", "SEARCH"), ("⌘L", "LIST"),
        ("⌘⌥C", "COPY CMD"), ("ESC", "CLEAR"), ("⌘⇧D", "DENSITY"), ("⏎", "INSTALL")]
for i, (k, meaning) in enumerate(keys):
    cx = 1180 + (i % 4) * 100
    cy = 902 + (i // 4) * 30
    rect(cx, cy, 32, 20, fill=SURF2, radius=4, stroke=HAIR2)
    T(k, cx, cy + 14, size=10, fam="IBM Plex Mono", color=O300, align="center", width=32)
    T(meaning, cx + 38, cy + 14, size=9.5, fam="IBM Plex Mono", color=al(MUTED, 0.9))

# ================================================================ build
with open(os.path.join(OUT_DIR, "build.jsonl"), "w") as f:
    for s in steps:
        f.write(json.dumps(s) + "\n")
print("steps:", len(steps))

out = os.path.join(OUT_DIR, "04-atelier.figcraft")
png = os.path.join(OUT_DIR, "04-atelier.png")
r = subprocess.run(["figcraft-cli", "run", os.path.join(OUT_DIR, "build.jsonl"), "--out", out],
                   capture_output=True, text=True)
lines = [l for l in (r.stdout + r.stderr).strip().splitlines() if '"ok":false' in l or l.startswith("error")]
print("\n".join(lines[-6:]) if lines else "build ok")
if r.returncode == 0:
    r2 = subprocess.run(["figcraft-cli", "export", out, "-o", png, "--scale", "1"],
                        capture_output=True, text=True)
    print((r2.stdout + r2.stderr)[-300:])
    from PIL import Image
    print("image size:", Image.open(png).size)
