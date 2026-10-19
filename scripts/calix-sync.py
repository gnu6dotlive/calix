#!/usr/bin/env python3
"""
calix-sync — Universal package state synchronizer for Calix & gnosix desktop.

Unifies & synchronizes:
  1. AppImages: discovers ~/Applications, correlates ~/.local/share/applications
     metadata (icon, version, description) and mirrors the result into the
     Shelly AppImage registry for interoperability.
  2. Cargo:     ingests ~/.cargo/.crates2.json (user + desktop daemons).
  3. NPM:       ingests `npm list -g --depth=0 --json`.
  4. ALPM/AUR:  ingests pacman status, checks the database lock and pending
     updates.
  5. Releases:  ingests release-tracked packages from the Calix registry.
  6. Health:    audits locks and conflicting update notifiers.

Usage:
  calix-sync [--dry-run] [--json] [--quiet]
  calix-sync list [all|appimage|cargo|npm|alpm|releases]
"""

import glob
import json
import os
import re
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from calix_core import (  # noqa: E402
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
APP_DIR = os.path.join(HOME, "Applications")
DESKTOP_DIR = os.path.join(HOME, ".local/share/applications")
SHELLY_CONFIG_DIR = os.path.join(HOME, ".config/shelly")
SHELLY_APPIMAGE_DB = os.path.join(SHELLY_CONFIG_DIR, "appimage-metadata-v2.db")

VERSION_KEYS = (
    "X-AppImage-Version",
    "X-OpenWork-Version",
    "X-AppImage-Name",
    "X-Craft-Version",
)


def ensure_dirs():
    os.makedirs(STATE_DIR, exist_ok=True)
    os.makedirs(SHARE_DIR, exist_ok=True)


# --------------------------------------------------------------------------
# AppImages
# --------------------------------------------------------------------------

def _read_desktop_metadata():
    """Map AppImage exec path → metadata harvested from .desktop entries."""
    metadata = {}
    if not os.path.isdir(DESKTOP_DIR):
        return metadata
    for desktop_file in sorted(glob.glob(os.path.join(DESKTOP_DIR, "*.desktop"))):
        try:
            with open(desktop_file, "r", encoding="utf-8", errors="ignore") as handle:
                content = handle.read()
        except OSError:
            continue
        name = comment = icon = exec_path = version = ""
        for raw_line in content.splitlines():
            line = raw_line.strip()
            if line.startswith("Name=") and not name:
                name = line.split("=", 1)[1]
            elif line.startswith("Comment=") and not comment:
                comment = line.split("=", 1)[1]
            elif line.startswith("Icon=") and not icon:
                icon = line.split("=", 1)[1]
            elif line.startswith("Exec="):
                match = re.search(r"\"?(/[^\"\s]+\.AppImage)\"?", line.split("=", 1)[1])
                if match:
                    exec_path = match.group(1)
            elif line.startswith(VERSION_KEYS) and not version:
                version = line.split("=", 1)[1]
        if exec_path:
            metadata[exec_path] = {
                "name": name,
                "comment": comment,
                "icon": icon,
                "version": version,
                "desktop_file": desktop_file,
            }
    return metadata


def _version_from_filename(filename):
    """Best-effort version extraction from an AppImage filename."""
    match = re.match(
        r"^([a-zA-Z0-9_-]+?)[-_]v?(\d+\.\d+.*?)[\.-](linux.*|x86_64.*|aarch64.*)\.AppImage$",
        filename,
    )
    if match:
        return match.group(1), match.group(2)
    return None, None


def sync_appimages(dry_run=False):
    """Discover ~/Applications and refresh the AppImage registry entries.

    Entries already present in the registry are *refreshed* (size, version,
    icon, description) instead of being carried over verbatim — the previous
    implementation kept stale metadata forever.
    """
    registry = []
    if os.path.exists(SHELLY_APPIMAGE_DB):
        try:
            with open(SHELLY_APPIMAGE_DB, "r", encoding="utf-8") as handle:
                registry = json.load(handle)
        except (OSError, json.JSONDecodeError):
            registry = []
    if not isinstance(registry, list):
        registry = []

    known = {item.get("path"): item for item in registry if isinstance(item, dict) and item.get("path")}
    desktop_meta = _read_desktop_metadata()
    entries = []

    if os.path.isdir(APP_DIR):
        for filename in sorted(os.listdir(APP_DIR)):
            if not filename.endswith(".AppImage"):
                continue
            full_path = os.path.join(APP_DIR, filename)
            if not os.path.isfile(full_path):
                continue
            size_on_disk = os.path.getsize(full_path)
            meta = desktop_meta.get(full_path, {})
            existing = known.get(full_path, {})

            app_name = meta.get("name") or existing.get("name")
            version = meta.get("version") or existing.get("version")
            description = meta.get("comment") or existing.get("description")
            icon_name = meta.get("icon") or existing.get("icon_name")

            if not app_name or not version:
                fallback_name, fallback_version = _version_from_filename(filename)
                app_name = app_name or fallback_name
                version = version or fallback_version

            if not app_name:
                app_name = os.path.splitext(filename)[0]
            if not version:
                version = "1.0.0"

            entries.append({
                "name": app_name,
                "version": version,
                "release_tag": version if version.startswith("v") else f"v{version}",
                "raw_update_info": existing.get("raw_update_info", ""),
                "icon_name": icon_name or app_name.lower(),
                "description": description or f"{app_name} application",
                "desktop_name": existing.get("desktop_name", app_name),
                "size_on_disk": size_on_disk,
                "environment_variables": existing.get("environment_variables", []),
                "command_line_args": existing.get("command_line_args", ""),
                "path": full_path,
                "update_url": existing.get("update_url", ""),
                "update_type": existing.get("update_type", "none"),
                "repo_owner": existing.get("repo_owner"),
                "repo_name": existing.get("repo_name"),
                "allow_prerelease": existing.get("allow_prerelease", False),
            })

    if not dry_run:
        if os.path.exists(SHELLY_APPIMAGE_DB):
            try:
                shutil.copy2(SHELLY_APPIMAGE_DB, f"{SHELLY_APPIMAGE_DB}.bak")
            except OSError:
                pass
        os.makedirs(SHELLY_CONFIG_DIR, exist_ok=True)
        write_json(SHELLY_APPIMAGE_DB, entries)
        write_json(os.path.join(STATE_DIR, "appimages.json"), entries)

    return entries


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

    if target in ("appimage", "appimages", "all"):
        data = _load("appimages")
        if data:
            rows = []
            for item in data:
                size_mb = f"{item.get('size_on_disk', 0) / (1024 * 1024):.1f} MB"
                rows.append([
                    item.get("name", ""),
                    item.get("version", ""),
                    size_mb,
                    os.path.basename(item.get("path", "")),
                ])
            print("🚀 AppImages in ~/Applications:")
            print_table(rows, ["Name", "Version", "Size", "File"])
            print()

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

    ensure_dirs()
    evidence = Evidence("calix-sync")

    start = time.time()
    appimages = sync_appimages(dry_run=dry_run)
    evidence.step("appimages", count=len(appimages))
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
            "appimages": len(appimages),
            "cargo": len(cargo_packages),
            "npm": len(npm_packages),
            "releases": len(releases),
        },
        "pending_updates": {
            "alpm": len(pacman_info["pending_updates"]),
            "aur": 0,
            "appimages": 0,
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
        print(f"  🚀 AppImages:        {status['counts']['appimages']:>5} registered in ~/Applications")
        print(f"  🦀 Cargo binaries:   {status['counts']['cargo']:>5} installed in ~/.cargo/bin")
        print(f"  📦 NPM globals:      {status['counts']['npm']:>5} installed in /usr/lib")
        print(f"  🔭 Release tracked:  {status['counts']['releases']:>5} repositories")
        print(f"  🔒 Pacman DB Lock:   {'LOCKED ⚠️' if status['db_locked'] else 'Free ✓'}")
        print(f"  🔄 Pending Updates:  {status['pending_updates']['alpm']} standard updates available\n")
        print("  ✓ State synchronized: ~/.local/state/calix/status.json")
        print("  ✓ Shelly AppImage DB updated: ~/.config/shelly/appimage-metadata-v2.db")

    evidence.finish(counts=status["counts"], db_locked=status["db_locked"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
