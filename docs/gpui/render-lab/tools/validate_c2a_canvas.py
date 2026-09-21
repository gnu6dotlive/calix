#!/usr/bin/env python3
"""Validate the atlas authority for the RENDER-03-C2A Board G canvas."""

import json
from pathlib import Path


EXPECTED_BOARD = "G"
EXPECTED_FIXTURE_COUNT = 8
EXPECTED_SIZE = (427, 87)


def main() -> None:
    atlas_path = Path(__file__).resolve().parents[1] / "references/atlas/reference-atlas.json"
    atlas = json.loads(atlas_path.read_text())
    fixtures = [fixture for fixture in atlas["fixtures"].values() if fixture["board"] == EXPECTED_BOARD]

    if len(fixtures) != EXPECTED_FIXTURE_COUNT:
        raise SystemExit(
            f"C2A atlas fixture count mismatch: expected {EXPECTED_FIXTURE_COUNT}, got {len(fixtures)}"
        )

    sizes = {tuple(fixture["crop_px"][2:]) for fixture in fixtures}
    if sizes != {EXPECTED_SIZE}:
        raise SystemExit(f"C2A atlas dimensions mismatch: expected {EXPECTED_SIZE}, got {sorted(sizes)}")

    print(f"C2A atlas authority: {len(fixtures)}/{EXPECTED_FIXTURE_COUNT} {EXPECTED_BOARD} fixtures at {EXPECTED_SIZE[0]}x{EXPECTED_SIZE[1]}")


if __name__ == "__main__":
    main()
