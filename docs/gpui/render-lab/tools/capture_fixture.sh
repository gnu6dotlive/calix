#!/usr/bin/env bash
# RENDER-01 evidence capture for one fixture.
#
# Hardened protocol (shared-display safe):
#   1. select fixture, assert active via status before AND after each capture
#   2. park cursor out of the way before each capture
#   3. capture twice back-to-back, locate fiducial ROI in EACH capture
#   4. require identical ROIs and identical pixel hashes, else FAIL loudly
#   5. emit <out>.roi.png + <out>.manifest.json
#
# Env:
#   SHELLY_BIN       shelly-gpui binary (required)
#   RUNTIME_DIR      XDG_RUNTIME_DIR of the lab instance (required)
#   WIN_GEOM         deprecated; observed geometry is always read from Hyprland
#   EXPECTED_ROI     optional exact ROI dimensions "WxH" (fails on mismatch)
#
# Usage: capture_fixture.sh <fixture-id> <out-prefix>
set -u
FIXTURE="$1"
OUT="$2"
TOOLS="$(cd "$(dirname "$0")" && pwd)"

cli() { XDG_RUNTIME_DIR="$RUNTIME_DIR" "$SHELLY_BIN" "$@"; }
park() { hyprctl eval 'return hl.dispatch(hl.dsp.cursor.move({x=50, y=50}))' >/dev/null 2>&1; }
window_address() {
  hyprctl clients -j | jq -r 'map(select(.class=="shelly-gpui" and .mapped == true and .xwayland == false)) | if length == 1 then .[0].address else empty end'
}
focus_window() {
  local address="$1"
  hyprctl dispatch "hl.dsp.focus({ window = \"address:$address\" })" >/dev/null || {
    echo "WINDOW_FOCUS_FAILED address=$address"
    exit 1
  }
  hyprctl dispatch "hl.dsp.window.alter_zorder({ window = \"address:$address\", mode = \"top\" })" >/dev/null || {
    echo "WINDOW_RAISE_FAILED address=$address"
    exit 1
  }
  sleep 0.2
  local active
  active="$(hyprctl activewindow -j)"
  local active_address active_class active_xwayland
  active_address="$(printf '%s' "$active" | jq -r '.address')"
  active_class="$(printf '%s' "$active" | jq -r '.class')"
  active_xwayland="$(printf '%s' "$active" | jq -r '.xwayland')"
  [ "$active_address" = "$address" ] && [ "$active_class" = "shelly-gpui" ] && [ "$active_xwayland" = "false" ] || {
    echo "WINDOW_FOCUS_ASSERTION_FAILED target=$address active=$active_address"
    exit 1
  }
}
window_geom() {
  hyprctl clients -j | jq -r 'map(select(.class=="shelly-gpui" and .mapped == true and .xwayland == false)) | if length == 1 then (.[0].at[0] | tostring) + "," + (.[0].at[1] | tostring) + " " + (.[0].size[0] | tostring) + "x" + (.[0].size[1] | tostring) else empty end'
}

active_of() {
  cli render-lab status --json 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin)['data'].get('active_fixture') or '')"
}

echo "--- capture: $FIXTURE ---"
cli render-lab fixture "$FIXTURE" >/dev/null 2>&1 || { echo "SELECT-FAILED $FIXTURE"; exit 1; }
sleep 4

A0="$(active_of)"; [ "$A0" = "$FIXTURE" ] || { echo "ASSERT-PRE-FAILED want=$FIXTURE got=$A0"; exit 1; }
ADDRESS="$(window_address)"; [ -n "$ADDRESS" ] || { echo "SHELLY_WINDOW_NOT_UNIQUE"; exit 1; }
park; sleep 1
focus_window "$ADDRESS"
GEOM="$(window_geom)"; [ -n "$GEOM" ] || { echo "WINDOW-GEOMETRY-FAILED address=$ADDRESS"; exit 1; }
grim -g "$GEOM" "${OUT}.a.png" || exit 1
A1="$(active_of)"; [ "$A1" = "$FIXTURE" ] || { echo "ASSERT-MID-FAILED want=$FIXTURE got=$A1"; exit 1; }

park; sleep 1
focus_window "$ADDRESS"
GEOM="$(window_geom)"; [ -n "$GEOM" ] || { echo "WINDOW-GEOMETRY-FAILED address=$ADDRESS"; exit 1; }
grim -g "$GEOM" "${OUT}.b.png" || exit 1
A2="$(active_of)"; [ "$A2" = "$FIXTURE" ] || { echo "ASSERT-POST-FAILED want=$FIXTURE got=$A2"; exit 1; }

if ! RA="$(python3 "$TOOLS/roi_hash.py" locate "${OUT}.a.png")"; then
  echo "FIDUCIAL_NOT_FOUND capture=${OUT}.a.png target_window=$ADDRESS geometry=$GEOM"
  exit 1
fi
if ! RB="$(python3 "$TOOLS/roi_hash.py" locate "${OUT}.b.png")"; then
  echo "FIDUCIAL_NOT_FOUND capture=${OUT}.b.png target_window=$ADDRESS geometry=$GEOM"
  exit 1
fi
[ "$RA" = "$RB" ] || { echo "ROI-MISMATCH a=$RA b=$RB"; exit 1; }
ROI="$(echo "$RA" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f\"{d['x']},{d['y']},{d['w']},{d['h']}\")")"
[ -z "${EXPECTED_ROI:-}" ] || [ "$(echo "$RA" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f\"{d['w']}x{d['h']}\")")" = "$EXPECTED_ROI" ] || { echo "ROI-DIMENSION-MISMATCH expected=$EXPECTED_ROI actual=$RA"; exit 1; }

python3 "$TOOLS/roi_hash.py" compare "${OUT}.a.png" "${OUT}.b.png" "$ROI" > "${OUT}.compare.json" || { echo "PIXEL-MISMATCH $FIXTURE"; exit 1; }

python3 - "$OUT" "$ROI" <<'EOF'
import json, sys
out, roi = sys.argv[1], sys.argv[2]
from PIL import Image
x, y, w, h = (int(v) for v in roi.split(","))
im = Image.open(out + ".a.png").convert("RGBA")
im.crop((x, y, x + w, y + h)).save(out + ".roi.png")
EOF

cli render-lab status --json 2>/dev/null | python3 -c "
import json, sys
resp = json.load(sys.stdin)
base = resp['data']
cmp = json.load(open('$OUT.compare.json'))
manifest = dict(base)
manifest['roi'] = cmp['roi']
manifest['pixel_sha256'] = cmp['a']
manifest['runs_identical'] = cmp['identical']
manifest['window_geom'] = '$GEOM'
manifest['compositor_scale'] = 1
json.dump(manifest, open('$OUT.manifest.json', 'w'), indent=2)
"
rm -f "${OUT}.a.png" "${OUT}.b.png" "${OUT}.compare.json"
echo "OK $FIXTURE roi=$ROI hash=$(python3 -c "import json; print(json.load(open('$OUT.manifest.json'))['pixel_sha256'][:16])")"
