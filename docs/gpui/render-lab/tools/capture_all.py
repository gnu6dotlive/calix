#!/usr/bin/env python3
"""Batch capture harness for all 46 canonical fixtures in RENDER-01.

Ensures determinism by:
1. Freezing the clock at t = 0.500s
2. Selecting each fixture via the canonical socket control protocol
3. Capturing back-to-back frames with grim
4. Locating the fiducial magenta marker bounding box
5. Comparing normalized decoded RGBA buffers for bit-exact identity
6. Emitting .roi.png, .manifest.json, and a consolidated EVIDENCE_INDEX.json
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from PIL import Image

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE_DIR = ROOT / "docs/gpui/render-lab/evidence/render01"
CALIX_BIN = os.environ.get("CALIX_BIN", "/mnt/workbench/target/release/calix-gpui")
WIN_GEOM = os.environ.get("WIN_GEOM", "-621,653 1263x1302")
RUNTIME_DIR = os.environ.get("XDG_RUNTIME_DIR", "/run/user/1000")

MARKER = (255, 0, 255)
INSET = 2

FIXTURES = [
    # Group G: Fields (8)
    ("field.zero-positive-scalar", "g", "NATIVE", "g01-linear-scalar-gradient/r1", "Direct 1D linear scalar field natively supported by GPUI linear_gradient. 2D non-linear gradients require composition."),
    ("field.signed-voltage", "g", "COMPOSABLE", "g02-bipolar-split/r1", "Zero-crossing threshold composed from two adjacent opposite-polarity gradients. Sharp boundary or nonlinear transition requires quad tessellation or texture."),
    ("field.current-magnitude", "g", "COMPOSABLE", "g03-magnitude-contour-stops/r1", "Magnitude ramp composable via multi-stop linear gradient; discrete isoline steps achieved by stepped color stops."),
    ("field.current-direction", "g", "COMPOSABLE", "g04-streamline-vector-field/r1", "Directional vector field expressed via composed vector paths; continuous dense vector direction field requires texture."),
    ("field.overload-heat", "g", "COMPOSABLE", "g05-thermal-multistop-gradient/r1", "Thermal palette composed from layered linear gradients and centered radiant oval."),
    ("field.diagnostic-confidence", "g", "NATIVE", "g06-probabilistic-confidence-band/r1", "1D probability density cleanly maps to single stock linear_gradient."),
    ("field.component-stress", "g", "COMPOSABLE", "g07-stress-concentration-contour/r1", "Stress concentration composed from concentric stress boundary shells."),
    ("field.selection-density", "g", "COMPOSABLE", "g08-continuous-density-plane/r1", "2D bilinear density plane approximated by cross-fading orthogonal linear gradients."),

    # Group H: Depth (8)
    ("depth.contact-shadow", "h", "NATIVE", "h01-single-box-shadow/r1", "Tight contact occlusion cleanly produced by stock BoxShadow with low blur radius."),
    ("depth.component-lift-shadow", "h", "NATIVE", "h02-diffused-elevation-shadow/r1", "Elevation penumbra produced by stock BoxShadow with larger blur and vertical offset."),
    ("depth.recessed-socket-shadow", "h", "COMPOSABLE", "h03-cavity-occlusion-bevel/r1", "GPUI has no native inner shadow; cavity depth is composed from directional 1px/2px inner bevel borders and linear cavity gradient."),
    ("depth.inset-panel-inner-shadow", "h", "COMPOSABLE", "h04-inset-panel-directional-bevel/r1", "Inner shadow illusion composed from directional border strokes and linear falloff child."),
    ("depth.raised-instrument-subtle-bevel", "h", "COMPOSABLE", "h05-dual-tone-linear-bevel/r1", "Physical bevel profile composed from opposing edge highlights, outer shadow, and subtle body gradient."),
    ("depth.soft-edge-top-highlight", "h", "NATIVE", "h06-ambient-top-highlight-rim/r1", "Single top-edge specular line directly expressible via stock border_t_1 and border_color."),
    ("depth.rim-highlight-outline", "h", "NATIVE", "h07-continuous-perimeter-rim/r1", "Uniform perimeter highlight directly expressible via stock border_1 and border_color."),
    ("depth.shallow-bevel-3d-edge", "h", "COMPOSABLE", "h08-prismatic-shallow-bevel/r1", "Multi-stage prismatic bevel composed of concentric nested borders and contrasting elevation shadows."),

    # Group I: Optical (8)
    ("optical.radial-fade", "i", "COMPOSABLE", "i01-concentric-falloff/r1", "Stock GPUI lacks radial gradient primitive; radial fade is approximated by concentric geometric discs. High ring count has tessellation cost."),
    ("optical.directional-fade", "i", "NATIVE", "i02-linear-directional-fade/r1", "Linear optical fade directly supported by stock linear_gradient with alpha color stops."),
    ("optical.soft-rectangle-rounded", "i", "NATIVE", "i03-soft-rounded-box-glow/r1", "Soft rounded glow boundary cleanly produced by stock BoxShadow with zero offset and positive spread."),
    ("optical.soft-circle-radial", "i", "COMPOSABLE", "i04-radial-disc-glow-boundary/r1", "Two-scale glow halo composed from nested circular disc and multi-lobe BoxShadow."),
    ("optical.edge-vignette", "i", "COMPOSABLE", "i05-corner-darkening-vignette/r1", "True radial vignette composed of peripheral linear gradients and corner shadow anchors."),
    ("optical.local-focus", "i", "COMPOSABLE", "i06-central-luminance-boost/r1", "Focus spotlight composed of centered illuminated disc and ambient darkening mask."),
    ("optical.energized-wire-glow", "i", "COMPOSABLE", "i07-filament-core-bloom/r1", "Bloom illusion composed of crisp 2px central filament and dual-stage soft BoxShadow glow."),
    ("optical.heat-region-glow", "i", "COMPOSABLE", "i08-thermal-emission-envelope/r1", "Volumetric heat glow approximated by layered concentric thermal discs with nonlinear opacity falloff."),

    # Group J: Materials (8)
    ("material.painted-metal.matte-anthracite", "j", "NATIVE", "j01-matte-anthracite-coating/r1", "Diffuse matte industrial finish cleanly represented by stock solid fill with subtle structural edge highlight."),
    ("material.painted-metal.signal-orange", "j", "NATIVE", "j02-signal-orange-coating/r1", "Vibrant painted metal finish cleanly represented by stock solid fill, 1px highlight border and subtle top sheen."),
    ("material.painted-metal.beret-green", "j", "NATIVE", "j03-beret-green-coating/r1", "Muted tactical paint cleanly represented by stock solid fill, subtle gradient, and edge border."),
    ("material.painted-metal.royal-blue", "j", "NATIVE", "j04-royal-blue-coating/r1", "Precision instrument enamel cleanly represented by stock solid fill, subtle gradient, and edge border."),
    ("material.brushed-aluminum", "j", "TEXTURE_PROOF", "j05-brushed-sphere-texture/r1", "Directional anisotropic grain and micro-streaks cannot be synthesized via vector primitives; proven through deterministic immutable memory texture."),
    ("material.dark-anodized-aluminum", "j", "COMPOSABLE", "j06-dark-anodized-satin/r1", "Satin anodized sheen composed of subtle multi-stage linear gradient, low-contrast specular rim, and dark metallic fill."),
    ("material.warm-paper", "j", "TEXTURE_PROOF", "j07-warm-paper-fibrous-texture/r1", "Paper fiber microstructure and stochastic pulp variation require texture synthesis; verified via immutable memory texture."),
    ("material.smoked-plastic", "j", "COMPOSABLE", "j08-smoked-polycarbonate-translucent/r1", "Optical transmission and surface reflection composed of translucent tinted fill, inner highlight line, and backdrop contrast."),

    # Group K: Microstructure (14)
    ("micro.k01", "k", "TEXTURE_PROOF", "k01-uniform-noise-texture/r1", "Uniform high-frequency stochastic noise synthesized into deterministic immutable texture buffer."),
    ("micro.k02", "k", "TEXTURE_PROOF", "k02-stratified-jitter-texture/r1", "Stratified jitter grid marks synthesized into deterministic immutable texture buffer."),
    ("micro.k03", "k", "TEXTURE_PROOF", "k03-fine-grit-texture/r1", "Fine multi-scale grit with stochastic particle points synthesized into deterministic immutable texture buffer."),
    ("micro.k04", "k", "TEXTURE_PROOF", "k04-brushed-micro-texture/r1", "Dense directional micro-scratch field synthesized into deterministic immutable texture buffer."),
    ("micro.k05", "k", "TEXTURE_PROOF", "k05-anisotropic-grain-texture/r1", "Anisotropic directional noise field synthesized into deterministic immutable texture buffer."),
    ("micro.k06", "k", "TEXTURE_PROOF", "k06-cellular-pattern-texture/r1", "Cellular Voronoi distance field synthesized into deterministic immutable texture buffer."),
    ("micro.k07", "k", "TEXTURE_PROOF", "k07-stochastic-stipple-texture/r1", "Stochastic stipple point density synthesized into deterministic immutable texture buffer."),
    ("micro.k08", "k", "TEXTURE_PROOF", "k08-value-noise-lattice-texture/r1", "Bilinear value noise lattice synthesized into deterministic immutable texture buffer."),
    ("micro.k09", "k", "TEXTURE_PROOF", "k09-halftone-mesh-texture/r1", "Geometric halftone dot mesh synthesized into deterministic immutable texture buffer."),
    ("micro.k10", "k", "TEXTURE_PROOF", "k10-woven-matrix-texture/r1", "Interleaved woven matrix threads synthesized into deterministic immutable texture buffer."),
    ("micro.k11", "k", "TEXTURE_PROOF", "k11-crater-relief-texture/r1", "Crater pore depressions with directional lighting synthesized into deterministic immutable texture buffer."),
    ("micro.k12", "k", "TEXTURE_PROOF", "k12-etched-fiber-texture/r1", "Curved etched fiber segments synthesized into deterministic immutable texture buffer."),
    ("micro.k13", "k", "TEXTURE_PROOF", "k13-fine-dither-texture/r1", "Ordered Bayer matrix dither synthesized into deterministic immutable texture buffer."),
    ("micro.k14", "k", "TEXTURE_PROOF", "k14-coarse-grain-texture/r1", "Coarse block grain clusters synthesized into deterministic immutable texture buffer."),
]


def cli(*args):
    env = dict(os.environ)
    env["XDG_RUNTIME_DIR"] = RUNTIME_DIR
    res = subprocess.run([CALIX_BIN, *args], env=env, capture_output=True, text=True)
    return res.stdout.strip()


def locate_marker(im):
    px = im.load()
    w, h = im.size
    xs0, ys0 = w, h
    xs1, ys1 = -1, -1
    for y in range(h):
        for x in range(w):
            r, g, b = px[x, y][:3]
            if r > 200 and g < 80 and b > 200:
                if x < xs0:
                    xs0 = x
                if y < ys0:
                    ys0 = y
                if x > xs1:
                    xs1 = x
                if y > ys1:
                    ys1 = y
    if xs1 < 0:
        raise RuntimeError("Fiducial marker #FF00FF not found")
    return (xs0 + INSET, ys0 + INSET, xs1 - xs0 + 1 - 2 * INSET, ys1 - ys0 + 1 - 2 * INSET)


def hash_crop(crop):
    raw = crop.tobytes()
    return hashlib.sha256(raw).hexdigest()


def main():
    print(f"=== Starting batch capture for {len(FIXTURES)} fixtures ===")
    for grp in ["g", "h", "i", "j", "k"]:
        (EVIDENCE_DIR / grp).mkdir(parents=True, exist_ok=True)

    cli("render-lab", "time", "0.500")
    results = []

    for idx, (fid, grp, verdict, recipe, caveats) in enumerate(FIXTURES, 1):
        print(f"[{idx:02d}/46] {fid} ({recipe}) ... ", end="", flush=True)
        cli("render-lab", "fixture", fid)
        time.sleep(0.4)

        out_prefix = EVIDENCE_DIR / grp / fid
        a_path = str(out_prefix) + ".tmp_a.png"
        b_path = str(out_prefix) + ".tmp_b.png"

        # Capture a
        subprocess.run(["grim", "-g", WIN_GEOM, a_path], check=True)
        time.sleep(0.2)
        # Capture b
        subprocess.run(["grim", "-g", WIN_GEOM, b_path], check=True)

        im_a = Image.open(a_path).convert("RGBA")
        im_b = Image.open(b_path).convert("RGBA")

        roi_a = locate_marker(im_a)
        roi_b = locate_marker(im_b)
        if roi_a != roi_b:
            raise RuntimeError(f"ROI mismatch for {fid}: a={roi_a} b={roi_b}")

        x, y, w, h = roi_a
        crop_a = im_a.crop((x, y, x + w, y + h))
        crop_b = im_b.crop((x, y, x + w, y + h))

        ha = hash_crop(crop_a)
        hb = hash_crop(crop_b)
        if ha != hb:
            raise RuntimeError(f"Pixel hash mismatch for {fid}: a={ha} b={hb}")

        # Save ROI image
        roi_img_path = str(out_prefix) + ".roi.png"
        crop_a.save(roi_img_path)

        # Build manifest
        manifest = {
            "schema_version": 2,
            "gui_running": True,
            "active_fixture": fid,
            "recipe": recipe,
            "backend": "gpui-stock-0.2.2",
            "verdict": verdict,
            "clock_mode": "frozen",
            "clock_time": 0.5,
            "roi": {"x": x, "y": y, "w": w, "h": h},
            "pixel_sha256": ha,
            "runs_identical": True,
            "window_geom": WIN_GEOM,
            "compositor_scale": 1,
            "caveats": caveats,
        }
        manifest_path = str(out_prefix) + ".manifest.json"
        with open(manifest_path, "w") as mf:
            json.dump(manifest, mf, indent=2)

        # Cleanup tmp
        if os.path.exists(a_path):
            os.unlink(a_path)
        if os.path.exists(b_path):
            os.unlink(b_path)

        evidence_rel = f"evidence/render01/{grp}/{fid}.roi.png"
        results.append({
            "fixture": fid,
            "backend": "gpui-stock-0.2.2",
            "verdict": verdict,
            "recipe": recipe,
            "evidence": evidence_rel,
            "manifest_hash": ha,
            "caveats": caveats,
        })
        print(f"OK ({ha[:16]}...)")

    index_path = EVIDENCE_DIR / "EVIDENCE_INDEX.json"
    with open(index_path, "w") as idx_f:
        json.dump(results, idx_f, indent=2)
    print(f"=== Finished all 46 fixtures. Index written to {index_path} ===")


if __name__ == "__main__":
    main()
