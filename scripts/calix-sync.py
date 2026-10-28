#!/usr/bin/env python3
"""
calix-sync — Universal package state synchronizer for Calix & gnosix desktop.

Unifies & synchronizes:
  1. Cargo:     ingests ~/.cargo/.crates2.json (user + desktop daemons).
  2. NPM:       ingests `npm list -g --depth=0 --json`.
  3. ALPM/AUR:  ingests pacman status, checks the database lock and pending
     updates.
  4. Releases:  ingests release-tracked packages from the Calix registry.
  5. Health:    audits locks and conflicting update notifiers.

AppImages are not here any more: `calix sync appimage` owns the metadata
database under `~/.config/calix`, on both conventions' roots, and the mirror
this script used to keep in the state directory had no reader anyway (F10,
F11).

Usage:
  calix-sync [--dry-run] [--json] [--quiet] [--runtime]
  calix-sync list [all|cargo|npm|alpm|releases]

  --runtime  Record the run in the engine's state directory instead of the
             repository's evidence/ tree. Used by the periodic timer, which
             must not append to the git tree.
"""


# =============================================================================
# STATUT : GELÉ — FONCTIONNALITÉ COMPLÈTE, CORRECTIFS UNIQUEMENT
# =============================================================================
# Ce module est un PONT vers le backend Zig. Il sera absorbé (tranche 1 de
# calix-architecture.md) et supprimé. Il ne doit plus recevoir de
# nouvelles fonctionnalités : une capacité ajoutée ici est une capacité à
# réécrire deux fois.
#
# Ce qui est DÉJÀ dans le Zig et ne doit PAS être porté : client HTTP,
# E/S JSON, parsing d'arguments, rendu de tableaux, lancement de processus,
# classification des forges et des assets, et le probing de releases lui-même
# (6 forges dans shared/ et appimage/update_manager.zig).
#
# Ce qui reste à écrire en Zig : backends Cargo et NPM, registre de releases,
# vérification de provenance amont, flux d'application, helper root + polkit,
# inventaire unifié, daemon.
#
# L'UI ne lit plus ce moteur : elle lit ~/.local/state/calix/*.json, qu'il
# écrit. Ce couplage est transitoire et disparaît avec le daemon.
# =============================================================================

import json
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from calix_core import (  # noqa: E402
    DEST_REPO,
    DEST_RUNTIME,
    Evidence,
    STATE_DIR,
    eprint,
    merge_release_state,
    print_table,
    read_registry,
    run_command,
    split_pkgrel,
    write_json,
    write_release_state,
)

HOME = os.path.expanduser("~")
SHARE_DIR = os.path.join(HOME, ".local/share/calix")


def ensure_dirs():
    os.makedirs(STATE_DIR, exist_ok=True)
    os.makedirs(SHARE_DIR, exist_ok=True)


# --------------------------------------------------------------------------
# Cargo / NPM / ALPM
# --------------------------------------------------------------------------

def sync_cargo():
    crates_file = os.path.join(HOME, ".cargo/.crates2.json")
    crates = {}
    if os.path.exists(crates_file):
        try:
            with open(crates_file, "r", encoding="utf-8") as handle:
                data = json.load(handle)
            for key, value in data.get("installs", {}).items():
                parts = key.split(" ", 2)
                name = parts[0]
                version = parts[1] if len(parts) > 1 else ""
                source = parts[2] if len(parts) > 2 else ""
                crates[name] = {
                    "version": version,
                    "source": source.strip("()"),
                    "bins": value.get("bins", []),
                }
        except (OSError, json.JSONDecodeError):
            pass
    write_json(os.path.join(STATE_DIR, "cargo.json"), crates)
    return crates


def sync_npm():
    packages = {}
    code, out, _err = run_command(
        ["npm", "list", "-g", "--depth=0", "--json"], timeout=45)
    if code == 0 and out.strip():
        try:
            data = json.loads(out)
            for name, value in data.get("dependencies", {}).items():
                packages[name] = {
                    "version": value.get("version", "unknown"),
                    "overridden": value.get("overridden", False),
                }
        except json.JSONDecodeError:
            pass
    write_json(os.path.join(STATE_DIR, "npm.json"), packages)
    return packages


def sync_pacman():
    info = {
        "installed_total": 0,
        "foreign_aur_total": 0,
        "pending_updates": [],
        "db_locked": os.path.exists("/var/lib/pacman/db.lck"),
    }

    code, out, _err = run_command(["pacman", "-Q"], timeout=30)
    if code == 0:
        info["installed_total"] = len(out.strip().splitlines())
        installed = {}
        for line in out.strip().splitlines():
            parts = line.split()
            if len(parts) >= 2:
                installed[parts[0]] = parts[1]

    code, out, _err = run_command(["pacman", "-Qm"], timeout=30)
    if code == 0:
        info["foreign_aur_total"] = len(out.strip().splitlines())

    code, out, _err = run_command(["checkupdates"], timeout=60)
    if code == 0 and out.strip():
        for line in out.strip().splitlines():
            parts = line.split()
            if len(parts) >= 4:
                info["pending_updates"].append({
                    "name": parts[0],
                    "old_version": parts[1],
                    "new_version": parts[3],
                })

    info["installed"] = installed if code == 0 else {}
    write_json(os.path.join(STATE_DIR, "pacman.json"), {
        key: value for key, value in info.items() if key != "installed"
    })
    return info


def sync_releases():
    """Refresh release state for every package in the Calix registry.

    The registry (~/.config/calix/releases.json) is the source of truth for
    *what* is tracked; this function only refreshes the *observed* state and
    never removes entries.
    """
    registry = read_registry()
    observed = {}
    for name, entry in registry.items():
        pkgname = entry.get("pkgname")
        installed_version = entry.get("installed_version")
        package_version = installed_version
        if pkgname:
            code, out, _err = run_command(["pacman", "-Q", pkgname], timeout=10)
            if code == 0 and out.strip():
                package_version = out.strip().split()[1]
                installed_version = split_pkgrel(package_version)[0]
        observed[name] = {
            "installed_version": installed_version or "unknown",
            "installed_package_version": package_version or "unknown",
            "checked_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        }
    state = merge_release_state(registry, observed)
    write_release_state(state)
    return state


# --------------------------------------------------------------------------
# Listing
# --------------------------------------------------------------------------

def _load(name):
    path = os.path.join(STATE_DIR, f"{name}.json")
    if not os.path.exists(path):
        return None
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError):
        return None


def cmd_list(target):
    target = (target or "all").lower()

    if target in ("cargo", "all"):
        data = _load("cargo")
        if data:
            rows = [
                [name, value.get("version", ""), ", ".join(value.get("bins", []))]
                for name, value in sorted(data.items())
            ]
            print("🦀 User & Desktop Cargo Crates:")
            print_table(rows, ["Crate", "Version", "Binaries"])
            print()

    if target in ("npm", "all"):
        data = _load("npm")
        if data:
            rows = [[name, value.get("version", "")] for name, value in sorted(data.items())]
            print("📦 NPM Global Packages:")
            print_table(rows, ["Package", "Version"])
            print()

    if target in ("alpm", "all"):
        data = _load("pacman")
        if data:
            print("📦 ALPM / Pacman:")
            print(f"  installed packages : {data.get('installed_total', 0)}")
            print(f"  foreign (AUR)      : {data.get('foreign_aur_total', 0)}")
            pending = data.get("pending_updates", [])
            print(f"  pending updates    : {len(pending)}")
            if pending:
                rows = [
                    [item.get("name", ""), item.get("old_version", ""), item.get("new_version", "")]
                    for item in pending[:25]
                ]
                print_table(rows, ["Package", "Installed", "Available"])
            print()

    if target in ("releases", "all"):
        data = _load("releases")
        if data:
            print("🔭 Release-Tracked Packages:")
            rows = [
                [
                    name,
                    entry.get("installed_version", "unknown"),
                    entry.get("latest_version", "-"),
                    "yes" if entry.get("update_available") else "no",
                    entry.get("repo", ""),
                ]
                for name, entry in sorted(data.items())
            ]
            print_table(rows, ["Package", "Installed", "Latest", "Update?", "Repo"])
            print()

    return 0


# --------------------------------------------------------------------------
# Main
# --------------------------------------------------------------------------

def main():
    if len(sys.argv) > 1 and sys.argv[1] == "list":
        target = sys.argv[2] if len(sys.argv) > 2 else "all"
        return cmd_list(target)

    dry_run = "--dry-run" in sys.argv
    emit_json = "--json" in sys.argv
    quiet = "--quiet" in sys.argv
    # An autonomous run (the periodic timer) is a product operation: its
    # record belongs in the engine's state directory, not in the repository's
    # git tree.
    runtime = "--runtime" in sys.argv

    ensure_dirs()
    evidence = Evidence(
        "calix-sync", destination=DEST_RUNTIME if runtime else DEST_REPO
    )

    start = time.time()
    cargo_packages = sync_cargo()
    evidence.step("cargo", count=len(cargo_packages))
    npm_packages = sync_npm()
    evidence.step("npm", count=len(npm_packages))
    pacman_info = sync_pacman()
    evidence.step("alpm", installed=pacman_info["installed_total"], aur=pacman_info["foreign_aur_total"])
    releases = sync_releases()
    evidence.step("releases", count=len(releases))
    duration = time.time() - start

    status = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "duration_ms": int(duration * 1000),
        "db_locked": pacman_info["db_locked"],
        "counts": {
            "alpm_total": pacman_info["installed_total"],
            "aur_total": pacman_info["foreign_aur_total"],
            "cargo": len(cargo_packages),
            "npm": len(npm_packages),
            "releases": len(releases),
        },
        "pending_updates": {
            "alpm": len(pacman_info["pending_updates"]),
            "aur": 0,
        },
    }
    write_json(os.path.join(STATE_DIR, "status.json"), status)

    if emit_json:
        print(json.dumps(status, indent=2))
        evidence.finish(counts=status["counts"])
        return 0

    if not quiet:
        print("🏺 Calix System Sync — Unified Package State")
        print(f"  Time: {status['timestamp']} (completed in {status['duration_ms']}ms)\n")
        print(f"  📦 Pacman / ALPM:   {status['counts']['alpm_total']:>5} installed ({status['counts']['aur_total']} AUR)")
        print(f"  🦀 Cargo binaries:   {status['counts']['cargo']:>5} installed in ~/.cargo/bin")
        print(f"  📦 NPM globals:      {status['counts']['npm']:>5} installed in /usr/lib")
        print(f"  🔭 Release tracked:  {status['counts']['releases']:>5} repositories")
        print(f"  🔒 Pacman DB Lock:   {'LOCKED ⚠️' if status['db_locked'] else 'Free ✓'}")
        print(f"  🔄 Pending Updates:  {status['pending_updates']['alpm']} standard updates available\n")
        print("  ✓ State synchronized: ~/.local/state/calix/status.json")

    evidence.finish(counts=status["counts"], db_locked=status["db_locked"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
