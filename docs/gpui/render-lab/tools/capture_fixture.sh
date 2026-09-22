#!/usr/bin/env bash
# RENDER-01 evidence capture for one fixture.
#
# Passive repeat protocol (shared-display safe):
#   1. select fixture and prepare the exact native window once
#   2. capture A, then make read-only assertions only
#   3. capture B, locate fiducial ROI in EACH capture
#   4. require unchanged fixture/window/geometry and identical pixel hashes
#   5. emit <out>.roi.png + <out>.manifest.json
#
# Env:
#   SHELLY_BIN       shelly-gpui binary (required)
#   RUNTIME_DIR      XDG_RUNTIME_DIR of the lab instance (required)
#   EXPECTED_ROI     optional exact ROI dimensions "WxH" (fails on mismatch)
#   RUNTIME_COMMIT   exact runtime source commit recorded in the manifest
#   PACKAGE_VERSION  exact package version recorded in the manifest
#
# Usage: capture_fixture.sh <fixture-id> <out-prefix>
set -u
FIXTURE="$1"
OUT="$2"
TOOLS="$(cd "$(dirname "$0")" && pwd)"

cli() { XDG_RUNTIME_DIR="$RUNTIME_DIR" "$SHELLY_BIN" "$@"; }
window_address() {
  hyprctl clients -j | jq -r 'map(select(.class=="shelly-gpui")) | if length == 1 then .[0].address else empty end'
}
assert_target_client() {
  local address="$1" client pid exe expected
  client="$(hyprctl clients -j | jq -c --arg address "$address" '.[] | select(.address==$address)')"
  [ -n "$client" ] || { echo "TARGET_CLIENT_MISSING address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.class')" = "shelly-gpui" ] || { echo "TARGET_CLASS_CHANGED address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.mapped')" = "true" ] || { echo "TARGET_NOT_MAPPED address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.hidden')" = "false" ] || { echo "TARGET_HIDDEN address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.visible')" = "true" ] || { echo "TARGET_NOT_VISIBLE address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.xwayland')" = "false" ] || { echo "TARGET_NOT_NATIVE_WAYLAND address=$address"; exit 1; }
  pid="$(printf '%s' "$client" | jq -r '.pid')"
  exe="$(readlink -f "/proc/$pid/exe")"
  expected="$(readlink -f "$SHELLY_BIN")"
  [ "$exe" = "$expected" ] || {
    echo "RUNTIME_EXECUTABLE_MISMATCH pid=$pid expected=$expected actual=$exe"
    exit 1
  }
  STABLE_ID="$(printf '%s' "$client" | jq -r '.stableId')"
  [ -n "$STABLE_ID" ] && [ "$STABLE_ID" != "null" ] || { echo "SHELLY_STABLE_ID_ABSENT address=$address"; exit 1; }
  TARGET_PID="$pid"
}
assert_same_target() {
  local address="$1" current pid exe stable_id
  current="$(hyprctl clients -j | jq -c --arg address "$address" '.[] | select(.address==$address)')"
  [ -n "$current" ] || { echo "TARGET_CLIENT_MISSING address=$address"; exit 1; }
  [ "$(printf '%s' "$current" | jq -r '.pid')" = "$PID" ] || { echo "TARGET_PID_CHANGED address=$address"; exit 1; }
  [ "$(printf '%s' "$current" | jq -r '.stableId')" = "$STABLE_ID" ] || { echo "TARGET_STABLE_ID_CHANGED address=$address"; exit 1; }
  assert_target_client "$address" >/dev/null
}

active_of() {
  cli render-lab status --json 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin)['data'].get('active_fixture') or '')"
}

echo "--- capture: $FIXTURE ---"
cli render-lab open >/dev/null 2>&1 || { echo "OPEN-FAILED"; exit 1; }
sleep 2
cli render-lab fixture "$FIXTURE" >/dev/null 2>&1 || { echo "SELECT-FAILED $FIXTURE"; exit 1; }
sleep 2

A0="$(active_of)"; [ "$A0" = "$FIXTURE" ] || { echo "ASSERT-PRE-FAILED want=$FIXTURE got=$A0"; exit 1; }
ADDRESS="$(window_address)"; [ -n "$ADDRESS" ] || { echo "SHELLY_WINDOW_NOT_UNIQUE"; exit 1; }
assert_target_client "$ADDRESS"
PID="$TARGET_PID"
grim -T "$STABLE_ID" "${OUT}.a.png" || exit 1

if ! RA="$(python3 "$TOOLS/roi_hash.py" locate "${OUT}.a.png" 2>/dev/null)"; then
  echo "VISIBLE_RENDER_LAB_NOT_ESTABLISHED capture=${OUT}.a.png target_window=$ADDRESS stable_id=$STABLE_ID"
  exit 1
fi
ROI_SIZE="$(echo "$RA" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f\"{d['w']}x{d['h']}\")")"
[ -z "${EXPECTED_ROI:-}" ] || [ "$ROI_SIZE" = "$EXPECTED_ROI" ] || { echo "ROI-DIMENSION-MISMATCH expected=$EXPECTED_ROI actual=$ROI_SIZE"; exit 1; }
A1="$(active_of)"; [ "$A1" = "$FIXTURE" ] || { echo "ASSERT-MID-FAILED want=$FIXTURE got=$A1"; exit 1; }
assert_same_target "$ADDRESS"
sleep 1

grim -T "$STABLE_ID" "${OUT}.b.png" || exit 1
A2="$(active_of)"; [ "$A2" = "$FIXTURE" ] || { echo "ASSERT-POST-FAILED want=$FIXTURE got=$A2"; exit 1; }
assert_same_target "$ADDRESS"

if ! RB="$(python3 "$TOOLS/roi_hash.py" locate "${OUT}.b.png" 2>/dev/null)"; then
  echo "VISIBLE_RENDER_LAB_NOT_ESTABLISHED capture=${OUT}.b.png target_window=$ADDRESS stable_id=$STABLE_ID"
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
manifest['compositor_scale'] = 1
manifest['runtime_commit'] = '${RUNTIME_COMMIT:-unknown}'
manifest['capture_harness_commit'] = '$(git -C "$TOOLS/../../../../" rev-parse HEAD 2>/dev/null || printf unknown)'
manifest['package_version'] = '${PACKAGE_VERSION:-unknown}'
manifest['window_address'] = '$ADDRESS'
manifest['window_pid'] = int('$PID')
manifest['stable_id'] = '$STABLE_ID'
manifest['capture_backend'] = 'grim-toplevel'
manifest['xwayland'] = False
json.dump(manifest, open('$OUT.manifest.json', 'w'), indent=2)
"
rm -f "${OUT}.a.png" "${OUT}.b.png" "${OUT}.compare.json"
echo "OK $FIXTURE roi=$ROI hash=$(python3 -c "import json; print(json.load(open('$OUT.manifest.json'))['pixel_sha256'][:16])")"
