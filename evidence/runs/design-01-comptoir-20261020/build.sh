#!/usr/bin/env bash
# Build and export the Le Comptoir proposal from this directory.
#   ./build.sh
# writes evidence/design-proposals/01-comptoir.figcraft and 01-comptoir.png
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/../../design-proposals"

python3 "$HERE/build.py"
figcraft-cli run "$HERE/build.jsonl" --out "$OUT/01-comptoir.figcraft" \
  > "$HERE/build.log" 2>&1 || { echo "BUILD FAILED:"; head -c 1200 "$HERE/build.log"; echo; exit 1; }
tail -1 "$HERE/build.log"
figcraft-cli export "$OUT/01-comptoir.figcraft" -o "$OUT/01-comptoir.png" --scale 1 > /dev/null
cp "$OUT/01-comptoir.png" "$HERE/01-comptoir.png"
echo "exported"
