#!/usr/bin/env python3
"""
Calix engine unit tests.

Covers the primitives that previously caused silent, late-failure bugs:

  * version comparison (lexicographic comparison silently mis-ranked
    0.5.10 below 0.5.9);
  * repository target parsing (the `owner/repo` shorthand was parsed as
    `host/path` and rejected);
  * provider dispatch (GitLab/Gitea targets were probed against the GitHub
    API);
  * asset selection (architecture filtering);
  * release-state merging (a sync used to clobber the recorded installed
    version);
  * undefined-name detection (a missing `import glob` made the update
    command raise NameError *after* a successful build).

Run with:
    python3 -m unittest discover -s tests/calix -v
"""

import ast
import builtins
import os
import sys
import unittest

SCRIPTS_DIR = os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
    "scripts",
)
sys.path.insert(0, SCRIPTS_DIR)

from calix_core import (  # noqa: E402
    classify_asset,
    compare_versions,
    detect_canal,
    is_newer,
    merge_release_state,
    parse_repo_target,
    parse_version,
    pick_asset,
    pkgbuild_source_names,
    split_pkgrel,
    strip_version_prefix,
)

RELEASE_SCRIPT = os.path.join(SCRIPTS_DIR, "calix-release.py")
SYNC_SCRIPT = os.path.join(SCRIPTS_DIR, "calix-sync.py")


def find_undefined_names(source):
    """Return module-level names that are read but never bound.

    This is the defect class that produced the `glob` NameError: a name
    used inside a function body without a matching import or assignment.
    """
    tree = ast.parse(source)

    bound = set(dir(builtins)) | {"__name__", "__file__", "__doc__"}
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                bound.add((alias.asname or alias.name).split(".")[0])
        elif isinstance(node, ast.ImportFrom):
            for alias in node.names:
                bound.add(alias.asname or alias.name)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            bound.add(node.name)
        elif isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            bound.add(node.id)
        elif isinstance(node, ast.arg):
            bound.add(node.arg)
        elif isinstance(node, ast.ExceptHandler) and node.name:
            bound.add(node.name)
        elif isinstance(node, (ast.comprehension,)):
            for inner in ast.walk(node.target):
                if isinstance(inner, ast.Name):
                    bound.add(inner.id)
        elif isinstance(node, ast.Global):
            bound.update(node.names)

    undefined = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Load):
            if node.id not in bound:
                undefined.add(node.id)
    return undefined


class TestVersionComparison(unittest.TestCase):
    def test_numeric_segments_compare_numerically(self):
        # The regression that mattered: string compare says "0.5.10" < "0.5.9".
        self.assertGreater(compare_versions("0.5.10", "0.5.9"), 0)
        self.assertGreater(compare_versions("0.10.0", "0.9.0"), 0)
        self.assertLess(compare_versions("0.4.1", "0.5.1"), 0)

    def test_trailing_zeros_are_equivalent(self):
        self.assertEqual(compare_versions("1.0", "1.0.0"), 0)
        self.assertEqual(compare_versions("1.2.0", "1.2"), 0)

    def test_v_prefix_is_ignored(self):
        self.assertEqual(compare_versions("v0.5.1", "0.5.1"), 0)
        self.assertEqual(strip_version_prefix("v0.5.1"), "0.5.1")
        self.assertEqual(strip_version_prefix("0.5.1"), "0.5.1")

    def test_release_outranks_its_own_prerelease(self):
        self.assertGreater(compare_versions("1.0.0", "1.0.0-rc1"), 0)
        self.assertGreater(compare_versions("1.0.0-rc2", "1.0.0-rc1"), 0)
        self.assertLess(compare_versions("1.0.0-1", "1.0.0-alpha"), 0)

    def test_is_newer(self):
        self.assertTrue(is_newer("0.5.1", "0.4.1"))
        self.assertFalse(is_newer("0.4.1", "0.4.1"))
        self.assertFalse(is_newer("0.4.1", "0.5.1"))
        self.assertTrue(is_newer("1.0.0", "unknown"))

    def test_parse_version_handles_garbage(self):
        self.assertIsNone(parse_version("not-a-version"))
        self.assertIsNone(parse_version(""))
        self.assertIsNone(parse_version(None))


class TestRepoTargetParsing(unittest.TestCase):
    def test_bare_owner_repo_shorthand(self):
        parsed = parse_repo_target("storytold/craft-launcher")
        self.assertIsNotNone(parsed)
        self.assertEqual(parsed["provider"], "github")
        self.assertEqual(parsed["host"], "github.com")
        self.assertEqual(parsed["path"], "storytold/craft-launcher")

    def test_https_github_url(self):
        parsed = parse_repo_target("https://github.com/storytold/craft-launcher")
        self.assertEqual(parsed["provider"], "github")
        self.assertEqual(parsed["path"], "storytold/craft-launcher")

    def test_scp_style_ssh_url(self):
        parsed = parse_repo_target("git@github.com:storytold/craft-launcher.git")
        self.assertEqual(parsed["provider"], "github")
        self.assertEqual(parsed["path"], "storytold/craft-launcher")

    def test_gitlab_subgroups(self):
        parsed = parse_repo_target("https://gitlab.com/group/subgroup/project")
        self.assertEqual(parsed["provider"], "gitlab")
        self.assertEqual(parsed["host"], "gitlab.com")
        self.assertEqual(parsed["owner"], "group/subgroup")
        self.assertEqual(parsed["repo"], "project")

    def test_gitea_instance(self):
        parsed = parse_repo_target("https://codeberg.org/owner/repo")
        self.assertEqual(parsed["provider"], "gitea")
        self.assertEqual(parsed["host"], "codeberg.org")

    def test_bare_host_with_dot_is_a_host(self):
        parsed = parse_repo_target("codeberg.org/owner/repo")
        self.assertEqual(parsed["provider"], "gitea")
        self.assertEqual(parsed["host"], "codeberg.org")

    def test_invalid_targets_are_rejected(self):
        self.assertIsNone(parse_repo_target("owner"))
        self.assertIsNone(parse_repo_target(""))
        self.assertIsNone(parse_repo_target(None))


class TestAssetHandling(unittest.TestCase):
    def test_classification(self):
        self.assertEqual(classify_asset("app.AppImage"), "appimage")
        self.assertEqual(classify_asset("app.flatpak"), "flatpak")
        self.assertEqual(classify_asset("app.tar.gz"), "tarball")
        self.assertEqual(classify_asset("app.deb"), "deb")
        self.assertEqual(classify_asset("app.rpm"), "rpm")
        self.assertEqual(classify_asset("app.zip"), "zip")
        self.assertEqual(classify_asset("SHA256SUMS.txt"), "other")

    def test_pick_asset_prefers_appimage_and_filters_arch(self):
        assets = [
            {"name": "app-1.0-linux-aarch64.AppImage", "kind": "appimage", "size": 1, "download_url": "a"},
            {"name": "app-1.0-linux-x86_64.AppImage", "kind": "appimage", "size": 2, "download_url": "b"},
            {"name": "app-1.0-linux-x86_64.tar.gz", "kind": "tarball", "size": 3, "download_url": "c"},
        ]
        best = pick_asset(assets, arch="x86_64")
        self.assertEqual(best["name"], "app-1.0-linux-x86_64.AppImage")

    def test_pick_asset_never_returns_a_foreign_arch(self):
        assets = [
            {"name": "app-1.0-linux-aarch64.AppImage", "kind": "appimage", "size": 1, "download_url": "a"},
            {"name": "app-1.0-freebsd-x86_64.tar.gz", "kind": "tarball", "size": 2, "download_url": "b"},
        ]
        best = pick_asset(assets, arch="x86_64")
        self.assertIsNone(best)

    def test_canal_detection(self):
        self.assertEqual(detect_canal("v0.5.1", False), "stable")
        self.assertEqual(detect_canal("v0.6.0-beta.1", True), "beta")
        self.assertEqual(detect_canal("nightly-2026-10-19", True), "nightly")
        self.assertEqual(detect_canal("canary-1", True), "canary")
        self.assertEqual(detect_canal("v1.0.0-rc1", True), "beta")


class TestReleaseStateMerging(unittest.TestCase):
    def test_merge_preserves_installed_version(self):
        registry = {"pkg": {"repo": "o/r", "pkgname": "pkg", "canal": "stable"}}
        merged = merge_release_state(registry, {"pkg": {"latest_version": "2.0.0"}})
        self.assertEqual(merged["pkg"]["latest_version"], "2.0.0")
        self.assertIn("installed_version", merged["pkg"])

    def test_merge_keeps_entries_absent_from_the_check(self):
        registry = {"pkg": {"repo": "o/r"}}
        merged = merge_release_state(registry, {})
        self.assertIn("pkg", merged)


class TestPkgbuildSources(unittest.TestCase):
    """The PKGBUILD source line carries shell variables that must be expanded.

    Unexpanded, the resolved filename ('pkg-$pkgver-linux-$CARCH.tar.gz')
    never matches the artifact on disk, and the upstream checksum
    cross-verification is silently skipped.
    """

    def _write(self, content):
        import tempfile
        handle = tempfile.NamedTemporaryFile("w", suffix="PKGBUILD", delete=False)
        handle.write(content)
        handle.close()
        self.addCleanup(os.unlink, handle.name)
        return handle.name

    def test_variables_are_expanded(self):
        path = self._write(
            'pkgname=pkg\n'
            'pkgver=1.2.3\n'
            'source=("https://example.com/v$pkgver/pkg-$pkgver-linux-$CARCH.tar.gz")\n'
            'sha256sums=(\'deadbeef\')\n'
        )
        names = pkgbuild_source_names(path, {"pkgver": "1.2.3"})
        self.assertEqual(len(names), 1)
        self.assertNotIn("$", names[0])
        self.assertIn("1.2.3", names[0])
        self.assertIn("x86_64", names[0])

    def test_multiple_sources(self):
        path = self._write(
            'source=("https://example.com/a.tar.gz" "https://example.com/b.zip")\n'
        )
        self.assertEqual(pkgbuild_source_names(path), ["a.tar.gz", "b.zip"])

    def test_missing_file_returns_empty(self):
        self.assertEqual(pkgbuild_source_names("/nonexistent/PKGBUILD"), [])

    def test_non_url_sources_are_ignored(self):
        path = self._write('source=("local.patch")\n')
        self.assertEqual(pkgbuild_source_names(path), [])


class TestPkgrelSplitting(unittest.TestCase):
    def test_pkgrel_is_stripped_for_comparison(self):
        self.assertEqual(split_pkgrel("0.4.1-1"), ("0.4.1", "1"))
        self.assertEqual(split_pkgrel("1.0.0-beta-3"), ("1.0.0-beta", "3"))
        self.assertEqual(split_pkgrel("0.5.1"), ("0.5.1", ""))


class TestStaticAnalysis(unittest.TestCase):
    """Guards against the late NameError class of defect."""

    def test_release_script_has_no_undefined_names(self):
        with open(RELEASE_SCRIPT, "r", encoding="utf-8") as handle:
            source = handle.read()
        undefined = find_undefined_names(source)
        self.assertEqual(
            undefined, set(),
            f"calix-release.py reads undefined names: {sorted(undefined)}")

    def test_sync_script_has_no_undefined_names(self):
        with open(SYNC_SCRIPT, "r", encoding="utf-8") as handle:
            source = handle.read()
        undefined = find_undefined_names(source)
        self.assertEqual(
            undefined, set(),
            f"calix-sync.py reads undefined names: {sorted(undefined)}")

    def test_core_module_has_no_undefined_names(self):
        core = os.path.join(SCRIPTS_DIR, "calix_core.py")
        with open(core, "r", encoding="utf-8") as handle:
            source = handle.read()
        undefined = find_undefined_names(source)
        self.assertEqual(undefined, set(),
                         f"calix_core.py reads undefined names: {sorted(undefined)}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
