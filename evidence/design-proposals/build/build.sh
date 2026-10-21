#!/usr/bin/env bash
# Build and export the Le Comptoir proposal.
set -e
cd "$(dirname "$0")"
python3 01-comptoir.py
figcraft-cli run 01-comptoir.jsonl --out ../01-comptoir.figcraft > /tmp/opencode/build.log 2>&1 || {
  echo "BUILD FAILED:"; head -c 1200 /tmp/opencode/build.log; echo; exit 1; }
tail -1 /tmp/opencode/build.log
figcraft-cli export ../01-comptoir.figcraft -o ../01-comptoir.png --scale 1 > /dev/null
figcraft-cli export ../01-comptoir.figcraft -o /tmp/opencode/01-comptoir@2x.png --scale 2 > /dev/null
echo "exported"
