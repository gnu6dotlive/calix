#!/usr/bin/env bash
# Build and export L'Instrument: one FigCraft document holding two frames.
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/.."
RUN="$HERE/../../../runs/design-01-instrument-r2-20261020"

python3 "$HERE/01-instrument.py"
figcraft-cli run "$HERE/01-instrument.jsonl" --out "$OUT/01-instrument.figcraft" \
  > "$RUN/build.log" 2>&1 || { echo "BUILD FAILED:"; head -c 1500 "$RUN/build.log"; echo; exit 1; }
tail -1 "$RUN/build.log"
figcraft-cli export "$OUT/01-instrument.figcraft" -o "$OUT/01-instrument-loaded.png" \
  --node 01-instrument-loaded --scale 1 > /dev/null
figcraft-cli export "$OUT/01-instrument.figcraft" -o "$OUT/01-instrument-quiet.png" \
  --node 01-instrument-quiet --scale 1 > /dev/null
cp "$OUT/01-instrument-loaded.png" "$RUN/01-instrument-loaded.png"
cp "$OUT/01-instrument-quiet.png" "$RUN/01-instrument-quiet.png"
echo "exported"
