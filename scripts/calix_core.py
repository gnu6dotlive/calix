#!/usr/bin/env python3
"""
calix_core — Shared primitives for the Calix package-state engine.

This module is deliberately dependency-free (stdlib only) so it can be
imported by the CLI entry points *and* by the unit tests without side
effects.

Responsibilities:
  1. Version parsing and comparison (semver-tolerant, numeric-aware).
  2. Repository target parsing across GitHub / GitLab / Gitea.
  3. Release asset classification.
  4. Registry (tracking config) and state I/O.
  5. Evidence emission conforming to the repository AGENTS.md contract
     (`evidence/runs/<run-id>/run.json` + `evidence/logs/<run-id>.log`).
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
import platform
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request

HOME = os.path.expanduser("~")
XDG_CONFIG = os.environ.get("XDG_CONFIG_HOME", os.path.join(HOME, ".config"))
XDG_STATE = os.environ.get("XDG_STATE_HOME", os.path.join(HOME, ".local/state"))

CONFIG_DIR = os.path.join(XDG_CONFIG, "calix")
STATE_DIR = os.path.join(XDG_STATE, "calix")

REGISTRY_PATH = os.path.join(CONFIG_DIR, "releases.json")
RELEASE_STATE_PATH = os.path.join(STATE_DIR, "releases.json")

# Repository root = parent of the `scripts/` directory holding this file.
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EVIDENCE_RUNS = os.path.join(REPO_ROOT, "evidence", "runs")
EVIDENCE_LOGS = os.path.join(REPO_ROOT, "evidence", "logs")

TOOL_VERSION = "2.0.0"
USER_AGENT = f"Calix-Release-Prober/{TOOL_VERSION} (gnosix; linux)"

# Preference order when several asset kinds are published for a release.
ASSET_PREFERENCE = ["appimage", "flatpak", "tarball", "zip", "deb", "rpm"]


# --------------------------------------------------------------------------
# 1. Versions
# --------------------------------------------------------------------------

_VERSION_RE = re.compile(r"^[vV]?(\d+(?:\.\d+)*)(?:[-+._]?(.*))?$")


def parse_version(value):
    """Split a version string into (numeric_core, prerelease_identifiers).

    Returns None when the string cannot be interpreted as a version.
    """
    if value is None:
        return None
    text = str(value).strip()
    if not text:
        return None
    match = _VERSION_RE.match(text)
    if not match:
        return None
    core = match.group(1)
    tail = (match.group(2) or "").strip()
    identifiers = [p for p in re.split(r"[.\-_]", tail) if p] if tail else []
    numbers = tuple(int(part) for part in core.split("."))
    while len(numbers) > 1 and numbers[-1] == 0:
        numbers = numbers[:-1]
    return (numbers, identifiers)


def compare_versions(left, right):
    """Return -1 / 0 / 1 comparing two version strings.

    Numeric segments compare numerically (so 0.5.10 > 0.5.9, which plain
    string comparison gets wrong), and a release outranks its own
    prerelease (1.0.0 > 1.0.0-rc1).
    """
    parsed_left = parse_version(left)
    parsed_right = parse_version(right)
    if parsed_left is None or parsed_right is None:
        left_text, right_text = str(left), str(right)
        return (left_text > right_text) - (left_text < right_text)

    left_core, left_pre = parsed_left
    right_core, right_pre = parsed_right

    if left_core != right_core:
        return (left_core > right_core) - (left_core < right_core)

    if not left_pre and not right_pre:
        return 0
    if not left_pre:
        return 1
    if not right_pre:
        return -1

    for left_id, right_id in zip(left_pre, right_pre):
        left_numeric, right_numeric = left_id.isdigit(), right_id.isdigit()
        if left_numeric and right_numeric:
            left_value, right_value = int(left_id), int(right_id)
            if left_value != right_value:
                return (left_value > right_value) - (left_value < right_value)
        elif left_numeric != right_numeric:
            # SemVer: numeric identifiers have lower precedence.
            return -1 if left_numeric else 1
        elif left_id != right_id:
            return (left_id > right_id) - (left_id < right_id)

    return (len(left_pre) > len(right_pre)) - (len(left_pre) < len(right_pre))


def is_newer(candidate, current):
    """True when `candidate` is strictly newer than `current`."""
    if current in (None, "", "unknown"):
        return True
    return compare_versions(candidate, current) > 0


def strip_version_prefix(tag):
    """Remove a leading `v`/`V` from a tag name."""
    if not tag:
        return ""
    return tag[1:] if tag[:1] in ("v", "V") else tag


def split_pkgrel(package_version):
    """Split an Arch package version 'pkgver-pkgrel' into (pkgver, pkgrel).

    The split is right-most, so an upstream version that itself contains a
    dash ('1.0-beta-2') keeps its upstream part intact.
    """
    text = str(package_version or "").strip()
    if "-" in text:
        pkgver, pkgrel = text.rsplit("-", 1)
        if pkgrel.isdigit():
            return pkgver, pkgrel
    return text, ""


def pkgbuild_source_names(pkgbuild_path, variables=None):
    """Return the source filenames declared by a PKGBUILD.

    The source basename is the authoritative name of the downloaded
    artifact — the `pkgname` may differ from it (e.g. pkgname
    'artcraft-launcher-bin' ships 'artcraft-launcher-<ver>-linux-x86_64.tar.gz').

    PKGBUILD variables are expanded from `variables` so that a source line
    such as `.../v$pkgver/pkg-$pkgver-linux-$CARCH.tar.gz` resolves to the
    concrete filename on disk.
    """
    names = []
    try:
        with open(pkgbuild_path, "r", encoding="utf-8") as handle:
            content = handle.read()
    except OSError:
        return names
    match = re.search(r"^source=\((.*?)\)", content, re.MULTILINE | re.DOTALL)
    if not match:
        return names

    expansions = {
        "pkgver": "",
        "pkgrel": "",
        "CARCH": host_arch(),
        "arch": host_arch(),
    }
    if variables:
        expansions.update({key: str(value) for key, value in variables.items()})

    def expand(text):
        for _ in range(4):  # bounded expansion; PKGBUILDs do not recurse deeply
            expanded = re.sub(
                r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?",
                lambda m: expansions.get(m.group(1), m.group(0)),
                text,
            )
            if expanded == text:
                break
            text = expanded
        return text

    for token in re.findall(r"[\"']([^\"']+)[\"']", match.group(1)):
        if token.startswith(("http://", "https://")):
            names.append(expand(token).rsplit("/", 1)[-1])
    return names


# --------------------------------------------------------------------------
# 2. Repository targets
# --------------------------------------------------------------------------

def classify_host(host):
    """Infer the forge provider from a hostname."""
    lowered = (host or "").lower()
    if "github" in lowered:
        return "github"
    if "gitlab" in lowered:
        return "gitlab"
    if "gitea" in lowered or "codeberg" in lowered or "forgejo" in lowered:
        return "gitea"
    # Unknown self-hosted forges overwhelmingly speak the Gitea API.
    return "gitea"


def parse_repo_target(target):
    """Normalize a repository target into provider/host/owner/repo.

    Accepts: `owner/repo`, `https://github.com/owner/repo`,
    `git@github.com:owner/repo.git`, GitLab subgroups, and self-hosted
    Gitea/Forgejo instances.
    """
    if not target:
        return None
    text = str(target).strip().rstrip("/")
    if text.endswith(".git"):
        text = text[:-4]

    host = None
    path = None

    scp = re.match(r"^git@([^:]+):(.+)$", text)
    if scp:
        host, path = scp.group(1), scp.group(2)
    else:
        scheme = re.match(r"^(?:https?|git|ssh)://([^/]+)/(.+)$", text)
        if scheme:
            host, path = scheme.group(1), scheme.group(2)
        else:
            segments = [segment for segment in text.split("/") if segment]
            if len(segments) < 2:
                return None
            # A bare `owner/repo` (or `group/sub/repo`) is the GitHub
            # shorthand form. A leading segment is only treated as a host
            # when it is unambiguously one: it carries a dot *and* the path
            # has at least two further segments (`codeberg.org/owner/repo`).
            leading_is_host = (
                "." in segments[0] and len(segments) >= 3
            )
            if leading_is_host:
                host, path = segments[0], "/".join(segments[1:])
            else:
                host, path = "github.com", "/".join(segments)

    path = path.split("?")[0].split("#")[0]
    parts = [segment for segment in path.split("/") if segment]
    if len(parts) < 2:
        return None

    owner = "/".join(parts[:-1])
    repo = parts[-1]
    return {
        "provider": classify_host(host),
        "host": host,
        "owner": owner,
        "repo": repo,
        "path": f"{owner}/{repo}",
    }


# --------------------------------------------------------------------------
# 3. Assets
# --------------------------------------------------------------------------

def classify_asset(name):
    """Classify a release asset filename into an installable kind."""
    lowered = (name or "").lower()
    if lowered.endswith(".appimage"):
        return "appimage"
    if lowered.endswith(".flatpak") or lowered.endswith(".flatpakref"):
        return "flatpak"
    if lowered.endswith((".tar.gz", ".tar.xz", ".tar.zst", ".tgz", ".tar.bz2")):
        return "tarball"
    if lowered.endswith(".deb"):
        return "deb"
    if lowered.endswith(".rpm"):
        return "rpm"
    if lowered.endswith(".zip"):
        return "zip"
    return "other"


def host_arch():
    """Return the host architecture in the naming convention used by releases."""
    machine = platform.machine().lower()
    if machine in ("aarch64", "arm64"):
        return "aarch64"
    if machine.startswith("riscv"):
        return "riscv64"
    return "x86_64"


def pick_asset(assets, arch=None):
    """Choose the best asset for the host, honouring ASSET_PREFERENCE."""
    if not assets:
        return None
    arch = arch or host_arch()
    foreign = ("aarch64", "arm", "riscv", "freebsd", "windows", "macos", "darwin")
    for kind in ASSET_PREFERENCE:
        for asset in assets:
            if asset.get("kind") != kind:
                continue
            name = asset.get("name", "").lower()
            # The arch token alone is not enough: a FreeBSD build also
            # carries "x86_64" in its filename.
            if arch in name and not any(token in name for token in foreign):
                return asset
        for asset in assets:
            if asset.get("kind") != kind:
                continue
            name = asset.get("name", "").lower()
            if not any(token in name for token in foreign):
                return asset
    return None


def detect_canal(tag, is_prerelease=False):
    """Map a release tag onto a Calix channel name."""
    lowered = (tag or "").lower()
    for token, canal in (
        ("nightly", "nightly"),
        ("canary", "canary"),
        ("beta", "beta"),
        ("alpha", "beta"),
        ("rc", "beta"),
    ):
        if token in lowered:
            return canal
    return "beta" if is_prerelease else "stable"


# --------------------------------------------------------------------------
# 4. HTTP
# --------------------------------------------------------------------------

def fetch_json(url, timeout=15):
    """GET a JSON document, raising on transport or decode failure."""
    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "application/vnd.github+json, application/json",
        },
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def fetch_text(url, timeout=15):
    """GET a plain-text document (used for upstream SHA256SUMS manifests)."""
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return response.read().decode("utf-8", errors="replace")


# --------------------------------------------------------------------------
# 5. Registry & state
# --------------------------------------------------------------------------

def ensure_dirs():
    os.makedirs(CONFIG_DIR, exist_ok=True)
    os.makedirs(STATE_DIR, exist_ok=True)


def read_json(path, default=None):
    if not os.path.exists(path):
        return {} if default is None else default
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError):
        return {} if default is None else default


def write_json(path, payload):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp_path = f"{path}.tmp"
    with open(tmp_path, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2)
        handle.write("\n")
    os.replace(tmp_path, path)


def read_registry():
    """Return the release tracking registry (source of truth for tracking)."""
    return read_json(REGISTRY_PATH, default={})


def write_registry(registry):
    ensure_dirs()
    write_json(REGISTRY_PATH, registry)


def read_release_state():
    """Return the resolved release state (what we know about each release)."""
    return read_json(RELEASE_STATE_PATH, default={})


def write_release_state(state):
    ensure_dirs()
    write_json(RELEASE_STATE_PATH, state)


def merge_release_state(registry, checked):
    """Merge freshly checked results into the persisted state, preserving keys.

    The previous implementation rewrote `releases.json` wholesale from a
    hardcoded literal, which silently discarded the `installed_version`
    recorded by a successful update. State is now merged, never clobbered.
    """
    state = read_release_state()
    for name, entry in registry.items():
        current = dict(state.get(name, {}))
        current.update({
            "repo": entry.get("repo"),
            "provider": entry.get("provider"),
            "canal": entry.get("canal", "stable"),
            "pkgname": entry.get("pkgname"),
            "pkgbuild_path": entry.get("pkgbuild_path"),
            "status": entry.get("status", "managed"),
        })
        if name in checked:
            current.update(checked[name])
        current.setdefault("installed_version", entry.get("installed_version"))
        state[name] = current
    return state


# --------------------------------------------------------------------------
# 6. Evidence
# --------------------------------------------------------------------------

#: Evidence destinations.
#:
#: ``repo``     — the repository's ``evidence/`` tree. This is the
#:                *development* contract from AGENTS.md: a run performed by an
#:                operator or an agent, whose record is meant to be reviewed
#:                and committed.
#: ``runtime``  — the engine's own state directory. A run performed
#:                autonomously (the periodic sync timer) is a product
#:                operation, not a development artefact, and must not append
#:                to the repository's git tree every few hours.
DEST_REPO = "repo"
DEST_RUNTIME = "runtime"

#: Where autonomous runs keep their operational journal.
RUNTIME_JOURNAL_DIR = os.path.join(STATE_DIR, "journal")


class Evidence:
    """Machine-readable run record + raw log.

    Two destinations, one format. A supervised run lands in the repository's
    ``evidence/`` tree; an autonomous run lands in the engine's state
    directory. The shape of the record is identical either way.
    """

    def __init__(self, kind, run_id=None, repo_root=None, destination=DEST_REPO):
        self.kind = kind
        self.destination = destination
        stamp = time.strftime("%Y%m%d-%H%M%S", time.localtime())
        self.run_id = run_id or f"{kind}-{stamp}"
        self.started_at = time.time()
        self.steps = []
        self.artifacts = []
        self.known_gaps = []
        self.status = "ok"

        if destination == DEST_RUNTIME:
            base = RUNTIME_JOURNAL_DIR
        else:
            self.repo_root = repo_root or REPO_ROOT
            base = os.path.join(self.repo_root, "evidence")

        self.log_path = os.path.join(base, "logs", f"{self.run_id}.log")
        self.run_dir = os.path.join(base, "runs", self.run_id)
        self.run_json = os.path.join(self.run_dir, "run.json")
        os.makedirs(os.path.dirname(self.log_path), exist_ok=True)
        os.makedirs(self.run_dir, exist_ok=True)
        self._log_handle = open(self.log_path, "a", encoding="utf-8")

    def log(self, message):
        line = f"[{time.strftime('%Y-%m-%dT%H:%M:%S%z')}] {message}"
        self._log_handle.write(line + "\n")
        self._log_handle.flush()
        return line

    def step(self, name, status="ok", **details):
        record = {"name": name, "status": status, "at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        record.update(details)
        self.steps.append(record)
        self.log(f"step {name} -> {status} {details if details else ''}".rstrip())
        return record

    def artifact(self, path, sha256=None):
        if not os.path.exists(path):
            return
        if self.destination == DEST_RUNTIME:
            entry = {"path": path}
        else:
            entry = {"path": os.path.relpath(path, self.repo_root)}
        if sha256:
            entry["sha256"] = sha256
        self.artifacts.append(entry)

    def gap(self, description):
        self.known_gaps.append(description)
        self.log(f"known gap: {description}")

    def fail(self, reason):
        self.status = "failed"
        self.log(f"FAILED: {reason}")

    def finish(self, **extra):
        self._log_handle.close()
        payload = {
            "run_id": self.run_id,
            "kind": self.kind,
            "status": self.status,
            "destination": self.destination,
            "started_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(self.started_at)),
            "finished_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "duration_ms": int((time.time() - self.started_at) * 1000),
            "tool_version": TOOL_VERSION,
            "host": {
                "kernel": platform.release(),
                "arch": platform.machine(),
                "python": platform.python_version(),
            },
            "steps": self.steps,
            "artifacts": self.artifacts,
            "known_gaps": self.known_gaps,
        }
        payload.update(extra)
        write_json(self.run_json, payload)
        return payload


def sha256_file(path):
    import hashlib
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run_command(command, cwd=None, timeout=600, check=False):
    """Run a command, capturing output. Returns (returncode, stdout, stderr)."""
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout.decode() if isinstance(exc.stdout, bytes) else (exc.stdout or "")
        stderr = exc.stderr.decode() if isinstance(exc.stderr, bytes) else (exc.stderr or "")
        return 124, stdout, stderr
    except FileNotFoundError as exc:
        return 127, "", str(exc)
    if check and completed.returncode != 0:
        raise subprocess.CalledProcessError(
            completed.returncode, command, output=completed.stdout, stderr=completed.stderr
        )
    return completed.returncode, completed.stdout, completed.stderr


def print_table(rows, headers):
    """Render a simple aligned table to stdout."""
    widths = [len(header) for header in headers]
    for row in rows:
        for index, cell in enumerate(row):
            widths[index] = max(widths[index], len(str(cell)))
    separator = "┌─" + "─┬─".join("─" * width for width in widths) + "─┐"
    print(separator)
    print("│ " + " │ ".join(header.ljust(widths[index]) for index, header in enumerate(headers)) + " │")
    print("├─" + "─┼─".join("─" * width for width in widths) + "─┤")
    for row in rows:
        print("│ " + " │ ".join(str(cell).ljust(widths[index]) for index, cell in enumerate(row)) + " │")
    print("└─" + "─┴─".join("─" * width for width in widths) + "─┘")


def eprint(*args):
    print(*args, file=sys.stderr)
