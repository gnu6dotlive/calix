#!/usr/bin/env bash
# Build and export the Le Comptoir proposal (author 01).
#
#   ./build.sh
#
# writes evidence/design-proposals/01-comptoir.figcraft and 01-comptoir.png,
# and keeps the 2x render and the run record inside the repository.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/.."
RUN="$OUT/../runs/design-01-comptoir-20261020"

python3 "$HERE/01-comptoir.py"
figcraft-cli run "$HERE/01-comptoir.jsonl" --out "$OUT/01-comptoir.figcraft" \
  > "$RUN/build.log" 2>&1 || { echo "BUILD FAILED:"; head -c 1200 "$RUN/build.log"; echo; exit 1; }
tail -1 "$RUN/build.log"
figcraft-cli export "$OUT/01-comptoir.figcraft" -o "$OUT/01-comptoir.png" --scale 1 > /dev/null
figcraft-cli export "$OUT/01-comptoir.figcraft" -o "$RUN/01-comptoir@2x.png" --scale 2 > /dev/null
cp "$OUT/01-comptoir.png" "$RUN/01-comptoir.png"
echo "exported"
