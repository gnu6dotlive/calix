#!/usr/bin/env python3
"""
calix-release — Prober and manager for GitHub, GitLab, and Gitea releases.

Commands:
  probe <repo_or_url> [--canal stable|beta|canary|nightly|all] [--json]
      Probe a repository: releases, channels, assets, trunk commit.

  check [--json]
      Compare every tracked release package against its upstream latest.

  update <name>
      Build and install a newer release through its PKGBUILD.
      Privilege elevation uses pkexec (never sudo).

  registry list|add|remove
      Manage the set of release-tracked packages.

The registry lives in ~/.config/calix/releases.json (tracking intent) and the
resolved state in ~/.local/state/calix/releases.json (observations). The two
are separate so that a sync can never erase the recorded installed version.
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
# Ce qui reste à écrire en Zig : backends Cargo et NPM, découverte
# ~/Applications, registre de releases, vérification de provenance amont,
# flux d'application, helper root + polkit, inventaire unifié, daemon.
#
# L'UI ne lit plus ce moteur : elle lit ~/.local/state/calix/*.json, qu'il
# écrit. Ce couplage est transitoire et disparaît avec le daemon.
# =============================================================================

import json
import os
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from calix_core import (  # noqa: E402
    Evidence,
    REGISTRY_PATH,
    classify_asset,
    compare_versions,
    detect_canal,
    eprint,
    fetch_json,
    fetch_text,
    host_arch,
    is_newer,
    parse_repo_target,
    pick_asset,
    pkgbuild_source_names,
    print_table,
    read_registry,
    read_release_state,
    run_command,
    sha256_file,
    split_pkgrel,
    strip_version_prefix,
    write_registry,
    write_release_state,
    merge_release_state,
)


# --------------------------------------------------------------------------
# Provider probes
# --------------------------------------------------------------------------

def _normalized_release(tag, name, is_prerelease, published_at, assets, tarball, zipball):
    return {
        "name": name or tag,
        "tag": tag,
        "canal": detect_canal(tag, is_prerelease),
        "is_prerelease": bool(is_prerelease),
        "published_at": published_at,
        "assets": assets,
        "tarball_url": tarball,
        "zipball_url": zipball,
    }


def probe_github(owner, repo, canal="stable"):
    api_base = f"https://api.github.com/repos/{owner}/{repo}"
    data = {
        "provider": "github",
        "repo": f"{owner}/{repo}",
        "host": "github.com",
        "releases": [],
        "trunk_commit": None,
        "selected_release": None,
    }

    try:
        raw_releases = fetch_json(f"{api_base}/releases?per_page=30")
        for release in raw_releases:
            assets = [
                {
                    "name": asset.get("name", ""),
                    "kind": classify_asset(asset.get("name", "")),
                    "size": asset.get("size", 0),
                    "download_url": asset.get("browser_download_url", ""),
                }
                for asset in release.get("assets", [])
            ]
            data["releases"].append(_normalized_release(
                release.get("tag_name", ""),
                release.get("name"),
                release.get("prerelease", False),
                release.get("published_at"),
                assets,
                release.get("tarball_url", ""),
                release.get("zipball_url", ""),
            ))
    except Exception as exc:  # noqa: BLE001 - surfaced to the operator
        data["releases_error"] = f"GitHub releases API: {exc}"

    try:
        for branch in fetch_json(f"{api_base}/branches"):
            if branch.get("name") in ("main", "master"):
                sha = branch.get("commit", {}).get("sha", "")
                data["trunk_commit"] = {
                    "branch": branch.get("name"),
                    "sha": sha[:8],
                    "full_sha": sha,
                }
                break
    except Exception:
        pass

    return _select_canal(data, canal)


def probe_gitlab(host, owner, repo, canal="stable"):
    project = urllib_quote(f"{owner}/{repo}")
    api_base = f"https://{host}/api/v4/projects/{project}"
    data = {
        "provider": "gitlab",
        "repo": f"{owner}/{repo}",
        "host": host,
        "releases": [],
        "trunk_commit": None,
        "selected_release": None,
    }

    try:
        raw_releases = fetch_json(f"{api_base}/releases?per_page=30")
        for release in raw_releases:
            tag = release.get("tag_name", "")
            assets = []
            for link in release.get("assets", {}).get("links", []) or []:
                url = link.get("direct_asset_url") or link.get("url", "")
                assets.append({
                    "name": link.get("name", ""),
                    "kind": classify_asset(link.get("name", "")),
                    "size": 0,
                    "download_url": url,
                })
            for source in release.get("assets", {}).get("sources", []) or []:
                assets.append({
                    "name": source.get("format", "tarball"),
                    "kind": "tarball",
                    "size": 0,
                    "download_url": source.get("url", ""),
                })
            data["releases"].append(_normalized_release(
                tag,
                release.get("name"),
                release.get("upcoming_release", False),
                release.get("released_at"),
                assets,
                f"https://{host}/{owner}/{repo}/-/archive/{tag}/{repo}-{tag}.tar.gz",
                "",
            ))
    except Exception as exc:  # noqa: BLE001
        data["releases_error"] = f"GitLab releases API: {exc}"

    try:
        for branch in fetch_json(f"{api_base}/repository/branches?per_page=100"):
            if branch.get("name") in ("main", "master"):
                data["trunk_commit"] = {
                    "branch": branch.get("name"),
                    "sha": (branch.get("commit", {}).get("id") or "")[:8],
                    "full_sha": branch.get("commit", {}).get("id", ""),
                }
                break
    except Exception:
        pass

    return _select_canal(data, canal)


def probe_gitea(host, owner, repo, canal="stable"):
    api_base = f"https://{host}/api/v1/repos/{owner}/{repo}"
    data = {
        "provider": "gitea",
        "repo": f"{owner}/{repo}",
        "host": host,
        "releases": [],
        "trunk_commit": None,
        "selected_release": None,
    }

    try:
        raw_releases = fetch_json(f"{api_base}/releases?limit=30")
        for release in raw_releases:
            assets = [
                {
                    "name": asset.get("name", ""),
                    "kind": classify_asset(asset.get("name", "")),
                    "size": asset.get("size", 0),
                    "download_url": asset.get("browser_download_url", ""),
                }
                for asset in release.get("assets", []) or []
            ]
            data["releases"].append(_normalized_release(
                release.get("tag_name", ""),
                release.get("name"),
                release.get("prerelease", False),
                release.get("published_at"),
                assets,
                f"https://{host}/{owner}/{repo}/archive/{release.get('tag_name', '')}.tar.gz",
                "",
            ))
    except Exception as exc:  # noqa: BLE001
        data["releases_error"] = f"Gitea releases API: {exc}"

    try:
        for branch in fetch_json(f"{api_base}/branches?limit=50"):
            if branch.get("name") in ("main", "master"):
                data["trunk_commit"] = {
                    "branch": branch.get("name"),
                    "sha": (branch.get("commit", {}).get("id") or "")[:8],
                    "full_sha": branch.get("commit", {}).get("id", ""),
                }
                break
    except Exception:
        pass

    return _select_canal(data, canal)


def urllib_quote(value):
    import urllib.parse
    return urllib.parse.quote(value, safe="")


def _select_canal(data, canal):
    """Pick the release matching the requested channel."""
    releases = data.get("releases") or []
    if not releases:
        return data
    if canal == "all":
        data["selected_release"] = releases[0]
        return data
    if canal == "stable":
        matches = [release for release in releases if not release["is_prerelease"]]
    else:
        matches = [release for release in releases if release["canal"] == canal]
    data["selected_release"] = matches[0] if matches else releases[0]
    data["canal_requested"] = canal
    data["canal_fallback"] = not matches
    return data


def probe_repo(target, canal="stable"):
    """Dispatch to the correct provider probe for a repository target."""
    parsed = parse_repo_target(target)
    if not parsed:
        raise ValueError(f"Cannot parse repository target '{target}'")
    provider = parsed["provider"]
    if provider == "gitlab":
        return probe_gitlab(parsed["host"], parsed["owner"], parsed["repo"], canal)
    if provider == "gitea":
        return probe_gitea(parsed["host"], parsed["owner"], parsed["repo"], canal)
    return probe_github(parsed["owner"], parsed["repo"], canal)


# --------------------------------------------------------------------------
# Upstream checksum manifest
# --------------------------------------------------------------------------

def fetch_checksum_manifest(release):
    """Look for a SHA256SUMS manifest published alongside the release assets."""
    for asset in release.get("assets", []) or []:
        name = (asset.get("name") or "").lower()
        if "sha256" in name and name.endswith((".txt", ".sum", ".sha256")):
            url = asset.get("download_url")
            if not url:
                continue
            try:
                text = fetch_text(url)
            except Exception:
                continue
            manifest = {}
            for line in text.splitlines():
                parts = line.split()
                if len(parts) == 2 and len(parts[0]) == 64:
                    manifest[parts[1].lstrip("*")] = parts[0]
            if manifest:
                return manifest
    return {}


# --------------------------------------------------------------------------
# Commands
# --------------------------------------------------------------------------

def _parse_flags(args):
    """Split argv into (positional, flags) with --key value support."""
    positional = []
    flags = {}
    index = 0
    while index < len(args):
        token = args[index]
        if token.startswith("--"):
            key = token[2:]
            if "=" in key:
                key, value = key.split("=", 1)
                flags[key] = value
            elif index + 1 < len(args) and not args[index + 1].startswith("--"):
                flags[key] = args[index + 1]
                index += 1
            else:
                flags[key] = True
        else:
            positional.append(token)
        index += 1
    return positional, flags


def cmd_probe(args):
    positional, flags = _parse_flags(args)
    if not positional:
        eprint("Usage: calix-release probe <repo_or_url> [--canal stable|beta|canary|nightly|all] [--json]")
        return 1
    canal = flags.get("canal", "stable")
    emit_json = "json" in flags

    try:
        result = probe_repo(positional[0], canal)
    except ValueError as exc:
        eprint(f"Error: {exc}")
        return 1

    if emit_json:
        print(json.dumps(result, indent=2))
        return 0

    print(f"🔭 Calix Release Probe: {result['repo']} ({result['provider']} @ {result.get('host', '?')})")
    print(f"  Channel filter: {canal}" + ("  (no exact match — fell back)" if result.get("canal_fallback") else ""))
    if result.get("trunk_commit"):
        trunk = result["trunk_commit"]
        print(f"  Trunk branch:   {trunk['branch']} @ commit {trunk['sha']}")

    releases = result.get("releases") or []
    if not releases:
        print(f"  No releases found ({result.get('releases_error', 'empty')}).")
        print("  -> Build method: resolve to origin/main and compile from source.")
        return 0

    print(f"\n  Found {len(releases)} release(s):")
    for release in releases[:5]:
        marker = f"[{release['canal'].upper()}]" if release["is_prerelease"] else "[STABLE]"
        published = (release.get("published_at") or "")[:10]
        print(f"    * {release['tag']} {marker} — {published}")
        grouped = {}
        for asset in release["assets"]:
            grouped.setdefault(asset["kind"], []).append(asset["name"])
        for kind, names in grouped.items():
            print(f"        • {kind:8}: {', '.join(names)}")

    selected = result.get("selected_release")
    if selected:
        print(f"\n  🎯 Recommended Release: {selected['tag']} ({selected['canal']})")
        best = pick_asset(selected["assets"])
        if best:
            size_mb = best.get("size", 0) / (1024 * 1024)
            print(f"     Optimal asset: {best['name']} ({best['kind']}, {size_mb:.1f} MB)")
            print(f"     Download URL:  {best['download_url']}")
        else:
            print(f"     Source tarball: {selected.get('tarball_url')}")
    return 0


def cmd_check(args):
    _positional, flags = _parse_flags(args)
    emit_json = "json" in flags
    registry = read_registry()

    if not registry:
        print("No release-managed packages registered yet.")
        print(f"Register one with: calix registry add <name> --repo <owner/repo> --pkgbuild <path>")
        return 0

    print("🔭 Checking Release-Tracked Packages...")
    results = {}
    rows = []
    for name, item in registry.items():
        repo = item.get("repo")
        if not repo:
            continue
        try:
            probe = probe_repo(repo, item.get("canal", "stable"))
        except Exception as exc:  # noqa: BLE001
            print(f"  * {name:<20} probe error: {exc}")
            results[name] = {"error": str(exc)}
            continue

        selected = probe.get("selected_release")
        if not selected:
            print(f"  * {name:<20} no release found ({probe.get('releases_error', 'empty')})")
            results[name] = {"error": probe.get("releases_error", "no release")}
            continue

        remote_version = strip_version_prefix(selected["tag"])
        current_version = item.get("installed_version") or detect_installed_version(item.get("pkgname"))
        update_available = is_newer(remote_version, current_version)

        best = pick_asset(selected["assets"])
        results[name] = {
            "repo": repo,
            "provider": probe["provider"],
            "installed_version": current_version,
            "latest_version": remote_version,
            "update_available": update_available,
            "published_at": selected.get("published_at"),
            "asset": best["name"] if best else None,
            "download_url": best["download_url"] if best else None,
        }
        marker = "🔄 UPDATE AVAILABLE" if update_available else "✓ Up-to-date"
        rows.append([name, current_version, remote_version, marker])

    if rows:
        print()
        print_table(rows, ["Package", "Installed", "Latest", "Status"])

    if emit_json:
        print(json.dumps(results, indent=2))

    state = merge_release_state(registry, results)
    write_release_state(state)
    return 0


def detect_installed_version(pkgname):
    """Read the installed version of an Arch package via pacman.

    Returns the upstream version without the Arch packaging release
    suffix, so that '0.4.1-1' compares as '0.4.1' against upstream tags.
    """
    if not pkgname:
        return "unknown"
    code, out, _err = run_command(["pacman", "-Q", pkgname], timeout=10)
    if code == 0 and out.strip():
        return split_pkgrel(out.strip().split()[1])[0]
    return "unknown"


def _bump_pkgver(pkgbuild_path, version):
    """Rewrite pkgver/pkgrel in a PKGBUILD, returning the previous content."""
    with open(pkgbuild_path, "r", encoding="utf-8") as handle:
        original = handle.read()
    lines = original.splitlines(keepends=True)
    updated = []
    seen_ver = False
    seen_rel = False
    for line in lines:
        if line.startswith("pkgver="):
            updated.append(f"pkgver={version}\n")
            seen_ver = True
        elif line.startswith("pkgrel="):
            updated.append("pkgrel=1\n")
            seen_rel = True
        else:
            updated.append(line)
    if not seen_ver:
        eprint("Error: PKGBUILD has no pkgver= line.")
        raise ValueError("PKGBUILD has no pkgver= line")
    if not seen_rel:
        eprint("Warning: PKGBUILD has no pkgrel= line.")
    with open(pkgbuild_path, "w", encoding="utf-8") as handle:
        handle.writelines(updated)
    return original


def cmd_update(args):
    positional, flags = _parse_flags(args)
    if not positional:
        eprint("Usage: calix-release update <name> [--dry-run]")
        return 1
    name = positional[0]
    dry_run = "dry-run" in flags

    registry = read_registry()
    if name not in registry:
        eprint(f"Error: '{name}' is not tracked. Known: {', '.join(registry) or '(none)'}")
        eprint(f"Register it with: calix registry add {name} --repo <owner/repo> --pkgbuild <path>")
        return 1

    item = registry[name]
    pkgbuild_path = item.get("pkgbuild_path")
    pkgname = item.get("pkgname")
    canal = item.get("canal", "stable")
    repo = item.get("repo")

    evidence = Evidence("calix-update")
    evidence.log(f"update requested for '{name}' (repo={repo}, canal={canal})")

    probe = probe_repo(repo, canal)
    evidence.step("probe", releases=len(probe.get("releases") or []), provider=probe["provider"])

    selected = probe.get("selected_release")
    if not selected:
        evidence.fail("no release resolved from remote")
        evidence.finish()
        eprint(f"Error: could not retrieve a release for '{name}'.")
        return 1

    remote_version = strip_version_prefix(selected["tag"])
    current_version = item.get("installed_version") or detect_installed_version(pkgname)
    evidence.step("resolve", installed=current_version, latest=remote_version)

    print(f"🔄 Preparing update for '{name}' from {repo}...")
    print(f"  Current version: {current_version}")
    print(f"  Latest release:  {remote_version}")

    if not is_newer(remote_version, current_version):
        print(f"✓ '{name}' is already at the latest release ({remote_version}).")
        evidence.step("noop", reason="already up-to-date")
        evidence.finish()
        return 0

    if not pkgbuild_path or not os.path.exists(pkgbuild_path):
        evidence.fail("pkgbuild_path missing or not found")
        evidence.finish()
        eprint(f"Error: PKGBUILD path not configured or not found for '{name}'.")
        return 1

    pkg_dir = os.path.dirname(pkgbuild_path)
    backup_path = f"{pkgbuild_path}.calix-bak"
    shutil.copy2(pkgbuild_path, backup_path)
    evidence.artifact(backup_path, sha256_file(backup_path))

    def restore_pkgbuild():
        shutil.copy2(backup_path, pkgbuild_path)
        evidence.log("PKGBUILD restored from backup")

    try:
        print(f"  Bumping PKGBUILD to {remote_version}...")
        _bump_pkgver(pkgbuild_path, remote_version)

        print("  Regenerating checksums via updpkgsums...")
        code, out, err = run_command(["updpkgsums"], cwd=pkg_dir, timeout=300)
        if code != 0:
            raise RuntimeError(f"updpkgsums failed ({code}): {err or out}")
        evidence.step("updpkgsums", returncode=code)

        # Cross-verify the downloaded tarball against the upstream manifest.
        manifest = fetch_checksum_manifest(selected)
        if manifest:
            verified = False
            for source_name in pkgbuild_source_names(
                    pkgbuild_path, {"pkgver": remote_version}):
                local_artifact = os.path.join(pkg_dir, source_name)
                if not os.path.exists(local_artifact):
                    continue
                local_sum = sha256_file(local_artifact)
                expected = manifest.get(source_name)
                if expected and expected != local_sum:
                    raise RuntimeError(
                        f"checksum mismatch for {source_name}: "
                        f"upstream {expected} != local {local_sum}")
                verified = verified or bool(expected)
                evidence.step("checksum", artifact=source_name, verified=bool(expected))
            if not verified:
                evidence.gap("upstream SHA256SUMS manifest found but no local artifact "
                             "matched a declared PKGBUILD source name")
        else:
            evidence.gap("upstream publish no SHA256SUMS manifest for this release")

        if dry_run:
            restore_pkgbuild()
            evidence.step("dry-run", pkgver=remote_version, pkgbuild_restored=True)
            evidence.finish(dry_run=True, target_version=remote_version)
            print("  --dry-run: source resolved and checksums regenerated; "
                  "PKGBUILD restored, stopping before build.")
            return 0

        print("  Compiling package via makepkg...")
        # No --syncdeps: dependency resolution must not require a root shell.
        code, out, err = run_command(
            ["makepkg", "--noconfirm", "--force"], cwd=pkg_dir, timeout=1800)
        if code != 0:
            raise RuntimeError(f"makepkg failed ({code}):\n{err or out}")
        evidence.step("makepkg", returncode=code)

        import glob
        built = glob.glob(os.path.join(pkg_dir, f"{pkgname}-{remote_version}-*.pkg.tar.*"))
        if not built:
            built = sorted(
                glob.glob(os.path.join(pkg_dir, "*.pkg.tar.*")),
                key=os.path.getmtime,
                reverse=True,
            )
        if not built:
            raise RuntimeError("no built package artifact found")
        pkg_file = built[0]
        evidence.artifact(pkg_file, sha256_file(pkg_file))

        print(f"  Installing {os.path.basename(pkg_file)} via pkexec pacman -U...")
        code, out, err = run_command(
            ["pkexec", "pacman", "-U", "--noconfirm", pkg_file], timeout=900)
        if code != 0:
            raise RuntimeError(f"pacman -U failed ({code}): {err or out}")
        evidence.step("pkexec-pacman", returncode=code, artifact=os.path.basename(pkg_file))

        installed = detect_installed_version(pkgname)
        evidence.step("verify", installed_version=installed)
        if installed != remote_version:
            raise RuntimeError(
                f"post-install verification failed: expected {remote_version}, pacman reports {installed}")

        # Record the new version in the registry *and* the state file.
        registry[name]["installed_version"] = remote_version
        write_registry(registry)
        state = read_release_state()
        state.setdefault(name, {})
        state[name]["installed_version"] = remote_version
        state[name]["latest_version"] = remote_version
        state[name]["update_available"] = False
        write_release_state(state)

        print(f"✓ Successfully updated and installed '{name}' {remote_version}.")
        evidence.finish(package=os.path.basename(pkg_file), version=remote_version)
        return 0

    except Exception as exc:  # noqa: BLE001
        restore_pkgbuild()
        evidence.fail(str(exc))
        evidence.finish(error=str(exc))
        eprint(f"✗ Update failed: {exc}")
        eprint("  The PKGBUILD was restored to its previous state.")
        return 1


def cmd_verify(args):
    """Audit a tracked package against its upstream release.

    Checks that the installed version matches the latest release and that
    the local build artifact still matches the upstream checksum manifest.
    """
    positional, flags = _parse_flags(args)
    if not positional:
        eprint("Usage: calix-release verify <name>")
        return 1
    name = positional[0]
    registry = read_registry()
    if name not in registry:
        eprint(f"Error: '{name}' is not tracked.")
        return 1
    item = registry[name]
    pkgbuild_path = item.get("pkgbuild_path")
    pkgname = item.get("pkgname")

    evidence = Evidence("calix-verify")
    installed = detect_installed_version(pkgname)
    probe = probe_repo(item.get("repo"), item.get("canal", "stable"))
    selected = probe.get("selected_release")
    latest = strip_version_prefix(selected["tag"]) if selected else "unknown"

    print(f"🔎 Verifying '{name}'")
    print(f"  Installed: {installed}")
    print(f"  Latest:    {latest}")
    version_ok = installed == latest
    print(f"  Version:   {'✓ match' if version_ok else '✗ mismatch'}")

    checksum_ok = None
    if pkgbuild_path and os.path.exists(pkgbuild_path) and selected:
        manifest = fetch_checksum_manifest(selected)
        pkg_dir = os.path.dirname(pkgbuild_path)
        for source_name in pkgbuild_source_names(pkgbuild_path, {"pkgver": latest}):
            local_artifact = os.path.join(pkg_dir, source_name)
            if not os.path.exists(local_artifact):
                continue
            expected = manifest.get(source_name)
            if not expected:
                continue
            actual = sha256_file(local_artifact)
            checksum_ok = actual == expected
            print(f"  Checksum:  {'✓ match' if checksum_ok else '✗ MISMATCH'} "
                  f"({source_name})")
            print(f"            expected {expected}")
            print(f"            actual   {actual}")
            break

    evidence.step("version", installed=installed, latest=latest, ok=version_ok)
    if checksum_ok is not None:
        evidence.step("checksum", ok=checksum_ok)
    else:
        evidence.gap("no local artifact matched the upstream checksum manifest")
    ok = version_ok and checksum_ok is not False
    evidence.finish(ok=ok)
    print(f"  Result:    {'✓ verified' if ok else '✗ verification failed'}")
    return 0 if ok else 1


def cmd_registry(args):
    positional, flags = _parse_flags(args)
    action = positional[0] if positional else "list"
    registry = read_registry()

    if action == "list":
        if not registry:
            print("Registry is empty.")
            return 0
        rows = [
            [
                name,
                entry.get("repo", ""),
                entry.get("canal", "stable"),
                entry.get("pkgname", ""),
                entry.get("installed_version", "unknown"),
            ]
            for name, entry in sorted(registry.items())
        ]
        print_table(rows, ["Name", "Repo", "Canal", "Pkgname", "Installed"])
        print(f"\nRegistry file: {REGISTRY_PATH}")
        return 0

    if action == "add":
        if not positional[1:]:
            eprint("Usage: calix registry add <name> --repo <owner/repo> [--canal stable] "
                   "[--pkgname pkg] [--pkgbuild /path/PKGBUILD]")
            return 1
        name = positional[1]
        repo = flags.get("repo")
        if not repo:
            eprint("Error: --repo <owner/repo> is required.")
            return 1
        parsed = parse_repo_target(repo)
        if not parsed:
            eprint(f"Error: cannot parse repo '{repo}'.")
            return 1
        entry = {
            "repo": f"{parsed['owner']}/{parsed['repo']}",
            "provider": parsed["provider"],
            "host": parsed["host"],
            "canal": flags.get("canal", "stable"),
            "pkgname": flags.get("pkgname", name),
            "pkgbuild_path": flags.get("pkgbuild", ""),
            "status": "managed",
        }
        registry[name] = entry
        write_registry(registry)
        print(f"✓ Registered '{name}' → {entry['repo']} ({entry['provider']}).")
        return 0

    if action == "remove":
        if len(positional) < 2:
            eprint("Usage: calix registry remove <name>")
            return 1
        name = positional[1]
        if name not in registry:
            eprint(f"Error: '{name}' is not tracked.")
            return 1
        del registry[name]
        write_registry(registry)
        state = read_release_state()
        state.pop(name, None)
        write_release_state(state)
        print(f"✓ Untracked '{name}'.")
        return 0

    eprint(f"Unknown registry action '{action}'. Use list|add|remove.")
    return 1


def main():
    if len(sys.argv) < 2 or sys.argv[1] in ("-h", "--help", "help"):
        print(__doc__)
        return 0
    command = sys.argv[1]
    if command == "probe":
        return cmd_probe(sys.argv[2:])
    if command == "check":
        return cmd_check(sys.argv[2:])
    if command in ("update", "upgrade"):
        return cmd_update(sys.argv[2:])
    if command == "registry":
        return cmd_registry(sys.argv[2:])
    if command == "verify":
        return cmd_verify(sys.argv[2:])
    eprint(f"Unknown command '{command}'. Use 'probe', 'check', 'update', or 'registry'.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
