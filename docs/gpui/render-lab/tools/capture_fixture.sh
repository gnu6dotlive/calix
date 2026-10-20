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
#   CALIX_BIN       calix-gpui binary (required)
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

cli() { XDG_RUNTIME_DIR="$RUNTIME_DIR" "$CALIX_BIN" "$@"; }
window_address() {
  hyprctl clients -j | jq -r 'map(select(.class=="calix-gpui")) | if length == 1 then .[0].address else empty end'
}
assert_target_client() {
  local address="$1" client pid exe expected
  client="$(hyprctl clients -j | jq -c --arg address "$address" '.[] | select(.address==$address)')"
  [ -n "$client" ] || { echo "TARGET_CLIENT_MISSING address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.class')" = "calix-gpui" ] || { echo "TARGET_CLASS_CHANGED address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.mapped')" = "true" ] || { echo "TARGET_NOT_MAPPED address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.hidden')" = "false" ] || { echo "TARGET_HIDDEN address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.visible')" = "true" ] || { echo "TARGET_NOT_VISIBLE address=$address"; exit 1; }
  [ "$(printf '%s' "$client" | jq -r '.xwayland')" = "false" ] || { echo "TARGET_NOT_NATIVE_WAYLAND address=$address"; exit 1; }
  pid="$(printf '%s' "$client" | jq -r '.pid')"
  exe="$(readlink -f "/proc/$pid/exe")"
  expected="$(readlink -f "$CALIX_BIN")"
  [ "$exe" = "$expected" ] || {
    echo "RUNTIME_EXECUTABLE_MISMATCH pid=$pid expected=$expected actual=$exe"
    exit 1
  }
  STABLE_ID="$(printf '%s' "$client" | jq -r '.stableId')"
  [ -n "$STABLE_ID" ] && [ "$STABLE_ID" != "null" ] || { echo "CALIX_STABLE_ID_ABSENT address=$address"; exit 1; }
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

status_fixture() {
  local phase="$1" stdout stderr exit_file fixture
  stdout="${OUT}.status-${phase}.stdout.json"
  stderr="${OUT}.status-${phase}.stderr.txt"
  exit_file="${OUT}.status-${phase}.exit"
  if cli render-lab status --json >"$stdout" 2>"$stderr"; then
    printf '0\n' >"$exit_file"
  else
    printf '%s\n' "$?" >"$exit_file"
    echo "CONTROL_STATUS_COMMAND_FAILED phase=$phase" >&2
    return 1
  fi
  if [ ! -s "$stdout" ]; then
    echo "CONTROL_STATUS_EMPTY phase=$phase" >&2
    return 1
  fi
  if ! fixture="$(python3 - "$stdout" "$FIXTURE" <<'PY'
import json
import sys

payload = json.load(open(sys.argv[1]))
expected = sys.argv[2]
if payload.get("ok") is not True:
    raise SystemExit("CONTROL_STATUS_NOT_OK")
data = payload.get("data")
if not isinstance(data, dict) or "active_fixture" not in data or data["active_fixture"] is None:
    raise SystemExit("CONTROL_STATUS_FIXTURE_MISSING")
if data["active_fixture"] != expected:
    raise SystemExit(f"CONTROL_STATUS_FIXTURE_MISMATCH actual={data['active_fixture']}")
print(data["active_fixture"])
PY
)"; then
    echo "CONTROL_STATUS_MALFORMED phase=$phase" >&2
    return 1
  fi
  printf '%s\n' "$fixture"
}

echo "--- capture: $FIXTURE ---"
cli render-lab open >/dev/null 2>&1 || { echo "OPEN-FAILED"; exit 1; }
sleep 2
cli render-lab fixture "$FIXTURE" >/dev/null 2>&1 || { echo "SELECT-FAILED $FIXTURE"; exit 1; }
sleep 2

PRE_FIXTURE="$(status_fixture pre)" || exit 1
[ "$PRE_FIXTURE" = "$FIXTURE" ] || { echo "ASSERT-PRE-FAILED want=$FIXTURE got=$PRE_FIXTURE"; exit 1; }
ADDRESS="$(window_address)"; [ -n "$ADDRESS" ] || { echo "CALIX_WINDOW_NOT_UNIQUE"; exit 1; }
assert_target_client "$ADDRESS"
PID="$TARGET_PID"
grim -T "$STABLE_ID" "${OUT}.a.png" || exit 1

sleep 1

grim -T "$STABLE_ID" "${OUT}.b.png" || exit 1
POST_STATUS=0
POST_FIXTURE="$(status_fixture post)" || POST_STATUS=$?
assert_same_target "$ADDRESS"

if python3 "$TOOLS/roi_hash.py" locate "${OUT}.a.png" >"${OUT}.locator-a.json" 2>"${OUT}.locator-a.error"; then
  RA="$(<"${OUT}.locator-a.json")"
else
  RA=""
  echo "VISIBLE_RENDER_LAB_NOT_ESTABLISHED capture=${OUT}.a.png target_window=$ADDRESS stable_id=$STABLE_ID"
fi
if [ -n "$RA" ]; then
  ROI_SIZE="$(printf '%s' "$RA" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f\"{d['w']}x{d['h']}\")")"
  [ -z "${EXPECTED_ROI:-}" ] || [ "$ROI_SIZE" = "$EXPECTED_ROI" ] || { echo "ROI-DIMENSION-MISMATCH expected=$EXPECTED_ROI actual=$ROI_SIZE"; exit 1; }
fi

if python3 "$TOOLS/roi_hash.py" locate "${OUT}.b.png" >"${OUT}.locator-b.json" 2>"${OUT}.locator-b.error"; then
  RB="$(<"${OUT}.locator-b.json")"
else
  RB=""
  echo "VISIBLE_RENDER_LAB_NOT_ESTABLISHED capture=${OUT}.b.png target_window=$ADDRESS stable_id=$STABLE_ID"
fi

ROI=""
COMPARE_STATUS=0
if [ -n "$RA" ] && [ -n "$RB" ]; then
  [ "$RA" = "$RB" ] || { echo "ROI-MISMATCH a=$RA b=$RB"; COMPARE_STATUS=1; }
  ROI="$(printf '%s' "$RA" | python3 -c "import json,sys; d=json.load(sys.stdin); print(f\"{d['x']},{d['y']},{d['w']},{d['h']}\")")"
  python3 "$TOOLS/roi_hash.py" compare "${OUT}.a.png" "${OUT}.b.png" "$ROI" > "${OUT}.compare.json" || COMPARE_STATUS=1
fi

if [ -n "$ROI" ]; then python3 - "$OUT" "$ROI" <<'EOF'
import json, sys
out, roi = sys.argv[1], sys.argv[2]
from PIL import Image
x, y, w, h = (int(v) for v in roi.split(","))
im = Image.open(out + ".a.png").convert("RGBA")
im.crop((x, y, x + w, y + h)).save(out + ".roi.png")
EOF
fi

if [ -f "${OUT}.compare.json" ]; then
  CMP="$(<"${OUT}.compare.json")"
else
  CMP='{"identical": false}'
fi
HARNESS_COMMIT="$(git -C "$TOOLS/../../../../" rev-parse HEAD 2>/dev/null || printf unknown)"
python3 - "$OUT" "$CMP" "$PRE_FIXTURE" "$POST_FIXTURE" "$POST_STATUS" "$ADDRESS" "$PID" "$STABLE_ID" "$HARNESS_COMMIT" <<'PY'
import json, sys
import os
out, cmp_text, pre_fixture, post_fixture, post_status, address, pid, stable_id, harness_commit = sys.argv[1:]
cmp = json.loads(cmp_text)
base = {}
manifest = dict(base)
manifest['roi'] = cmp.get('roi')
manifest['pixel_sha256'] = cmp.get('a')
manifest['runs_identical'] = cmp.get('identical', False)
manifest['compositor_scale'] = 1
manifest['runtime_commit'] = os.environ.get('RUNTIME_COMMIT', 'unknown')
manifest['capture_harness_commit'] = harness_commit
manifest['package_version'] = os.environ.get('PACKAGE_VERSION', 'unknown')
manifest['window_address_pre'] = address
manifest['window_address_post'] = address
manifest['window_pid_pre'] = int(pid)
manifest['window_pid_post'] = int(pid)
manifest['stable_id_pre'] = stable_id
manifest['stable_id_post'] = stable_id
manifest['fixture_pre'] = pre_fixture
manifest['fixture_post'] = post_fixture
manifest['post_status_exit'] = int(post_status)
manifest['capture_backend'] = 'grim-toplevel'
manifest['passive_interval_commands'] = ['grim-A', 'sleep-1s', 'grim-B']
json.dump(manifest, open(out + '.manifest.json', 'w'), indent=2)
PY

[ "$POST_STATUS" -eq 0 ] && [ -n "$RB" ] && [ "$COMPARE_STATUS" -eq 0 ] || exit 1
echo "OK $FIXTURE roi=$ROI"
