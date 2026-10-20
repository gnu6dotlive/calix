# Legacy references and naming

Calix is historically forked from Shelly and has since been refactored into a distinct
project. The rename is deliberately incomplete: several binaries, directories, paths and
ABI names still say `shelly`, and some of them are load-bearing (on-disk state, polkit
rules, environment contracts). This note records what was left alone in the documentation
and metadata pass of 2026-10-20, why, and what a future rename would touch.

The short version: **Calix is the product; `shelly` is the name of the transitional Zig
backend and of several inherited paths.** A `shelly` reference in this repository is not
a claim that Calix is Shelly.

Historical references remain for two different reasons. Some are **live contracts** —
binary names, environment variables, config paths and ABI symbols that existing
installations and packaging depend on; renaming them is a coordinated change, not a
documentation edit. Others are **dated records** — slice reports and evidence that
describe the tree as it was when they were written; rewriting them would destroy their
value as evidence. Neither category means the project still considers itself Shelly.

---

## Naming map

| Thing | Current name | Notes |
|---|---|---|
| Project / product | **Calix** | The name used in the UI, desktop entry, README and packaging |
| Desktop application | `calix-gpui` | Rust crate and binary; `app_id = "calix-gpui"` |
| Inventory daemon | `calix-daemon` | Separate Zig executable; socket `$XDG_RUNTIME_DIR/calix/daemon.sock` |
| Backend CLI | `shelly` | Zig binary, built by `Shelly.Cli.Zig/build.zig`; transitional |
| Backend library | `Shelly.PackageManager` | Zig module imported as `Zigalpm` |
| Backend source directory | `Shelly.Cli.Zig/` | Also contains `calix-daemon` |
| Frontend source directory | `Shelly.Ui.Gpui/` | Builds the `calix-gpui` crate |
| Installed CLI path | `/usr/lib/shelly/shelly` | `PKGBUILD-calix`; wrapper sets `SHELLY_BIN` |
| Frontend binary path | `/usr/lib/calix/calix-gpui` | `PKGBUILD-calix` |
| User config | `~/.config/shelly/` | `config.json`, `settings.json`, `gpui-ui.json` |
| User cache | `~/.cache/Shelly/` | Note the capital `S` in `xdg.shellyCache` |
| Daemon state | `$XDG_STATE_HOME/calix` | Defaults to `~/.local/state/calix` |
| Elevation override | `SHELLY_ELEVATOR` | Explicit operator choice; default is `pkexec` |
| CLI override | `SHELLY_BIN` | Used by the UI, Makefile and wrapper |
| Flatpak backend | `libshelly-flatpak-backend.so.1` | C ABI symbol `shelly_flatpak_backend_get_api` |

---

## Deliberately unchanged

These are code identifiers, build inputs, packaging assets or dated records. Changing
them in a documentation-only pass would either break behavior or make historical records
lie. They are listed here so the next person does not have to rediscover why they remain.

### 1. The Zig CLI binary is still `shelly`

`Shelly.Cli.Zig/build.zig` sets `.name = "shelly"`; the installed path is
`/usr/lib/shelly/shelly`; the UI finds it through `SHELLY_BIN`, with `/usr/bin/shelly`
and a relative development path as fallbacks. The polkit policy names the same path in
`org.freedesktop.policykit.exec.path`.

Renaming the binary is not a text substitution. It touches the build graph, the UI
client, the Makefile, `PKGBUILD-calix` and its wrapper, the polkit policy, shell
completions, tests, and any user script that invokes `shelly`. It should happen as one
coordinated change with a release note, not incrementally.

### 2. The Zig package name and fingerprint

`Shelly.Cli.Zig/build.zig.zon` still declares `.name = .shelly` and the upstream
fingerprint `0x38923a06e716d68e`. The file's own comment explains that changing the
fingerprint has security and trust implications and should be deliberate. Keeping it is
also an honest acknowledgement of lineage: this package is descended from Shelly's.

The version string is `3.1.9-dev`, inherited from the upstream tree this backend came
from. It is not the Calix product version.

### 3. Directory and module names

`Shelly.Ui.Gpui/`, `Shelly.Cli.Zig/`, `Shelly.PackageManager/`,
`Shelly.Flatpak.Backend/`, `Shelly.Http/`, `Shelly.Key/`, `Shelly.Diagnostics/`,
`Shelly.Notifications.Zig/`, `Shelly.Tui/` and `Shelly.Ui.Gtk/` retain the old prefix.
The Zig root module is `Shelly_Cli_Zig`; the backend library is imported as `Zigalpm`.
Renaming directories would touch every build file, import path, CI path filter and
documentation link in the tree. It is mechanical but wide, and it is not worth mixing
into a behavioral change.

### 4. Inherited on-disk paths

The backend reads `~/.config/shelly/config.json` and writes cache data under
`~/.cache/Shelly/`. The GPUI frontend stores `settings.json` and `gpui-ui.json` under
`~/.config/shelly/`. These paths are user state: renaming them without a migration
strands existing configuration.

The daemon already uses `$XDG_STATE_HOME/calix` for its own state, which is why the tree
has both names at once. A future rename needs either a compatibility fallback (read old,
write new) or an explicit migration step.

### 5. Environment variables

`SHELLY_BIN` is part of the contract between the wrapper, the Makefile and the UI.
`SHELLY_ELEVATOR` is an explicit operator override for the privilege elevator; the
default is `pkexec` and the implicit `sudo`/`doas` scan was removed. Renaming either
variable breaks existing launchers and scripts.

### 6. Packaging assets

`Shelly.Ui.Gpui/assets/com.calix.calix-gpui.policy` still has
`vendor_url = https://github.com/Seafoam-Labs/Shelly-ALPM` and
`exec.path = /usr/lib/shelly/shelly`. The first is an attribution/URL fix; the second is
coupled to the binary rename. Both are packaging assets, not documentation, and were
left for the packaging change that moves the binary.

Two upstream desktop entries remain in the repository root:
`com.shellyorg.shelly.desktop` and `com.shellyorg.shelly-notifications.desktop`. They are
**not installed by `PKGBUILD-calix`**, which installs
`Shelly.Ui.Gpui/assets/com.calix.calix-gpui.desktop` instead. They are dead upstream
artifacts and should be removed in a dedicated cleanup, not silently edited.

### 7. Flatpak ABI

The optional backend library is `libshelly-flatpak-backend.so.1` with the exported symbol
`shelly_flatpak_backend_get_api`. Renaming either is an ABI break for any separately
packaged backend. Do it with a SONAME/ABI version change, not as a string edit.

### 8. Frozen Python engine

`scripts/calix_core.py`, `scripts/calix-release.py`, `scripts/calix-sync.py` and the
`scripts/calix` dispatcher are frozen by project policy and planned for removal once the
Zig release watcher lands. They already use the Calix name; renaming their internals
would create churn in code that is meant to disappear. The daemon reads one file they
produce (`releases.json`) as a bounded bridge.

### 9. Upstream packaging files

`PKGBUILD`, `PKGBUILD-bin`, `PKGBUILD-cli` and `PKGBUILD-git` are inherited upstream
package definitions. `PKGBUILD-calix` is the current Calix package. The older files are
kept for reference; they are not the documented install path.

### 10. Historical records

`docs/gpui/**` contains dated slice records, acceptance reports, render-lab evidence and
screenshots. They describe the state of the tree at the time they were written, often
with commit hashes and captured output. They were not rewritten: editing a dated record
to use a newer product name would falsify evidence. The same applies to `evidence/**`.

`MANUAL_TESTING.md` keeps its literal `shelly` commands because the binary is still named
that; a note at the top explains the naming.

`docs/calix-architecture.md` is an internal working plan written in French before the
current state was settled. It received a reading note and a corrected lineage sentence,
but its historical plan text and old names were left in place.

---

## Documentation cleanup in this pass

| File | Classification | Action |
|---|---|---|
| `README.md` | Obsolete / confusing | Fully rewritten around Calix, with an explicit lineage section and current-state table |
| `CONTRIBUTING.md` | Obsolete / confusing | Rewritten for Calix and the current stack; removed stale `shelly-gpui` clone path and Catppuccin claim |
| `SECURITY.md` | Obsolete / confusing | Product name corrected to Calix |
| `MANUAL_TESTING.md` | Historical + user-visible | Title and a naming note added; literal `shelly` commands kept |
| `AGENTS.md` | Obsolete / confusing | Title corrected; a lineage/naming pointer added; engineering policy untouched |
| `.github/ISSUE_TEMPLATE/bug_report.yml` | Obsolete / confusing | Product and component names corrected |
| `.github/ISSUE_TEMPLATE/feature_request.yml` | Obsolete / confusing | Product name corrected |
| `docs/calix-architecture.md` | Historical | Reading note and corrected lineage sentence; plan text otherwise preserved |
| `docs/isolated-builds.md` | Technical (backend) | Naming note added; content preserved |
| `docs/flatpak-backend-abi.md` | Technical (backend) | Naming note added; content preserved |
| `docs/shellybuild.conf.md` | Technical (backend) | Naming note added; content preserved |
| `docs/shelly-flatpak-backend-plan.md` | Historical technical plan | Naming note added; content preserved |
| `docs/shelly-libalpm-compatible-dependency-resolution-gaps.md` | Historical technical analysis | Naming note added; content preserved |
| `docs/gpui/**`, `evidence/**` | Historical records | Not changed |

Not touched in this pass: `.github/workflows/ci.yml`, `scripts/verify`,
`scripts/ci-setup`, `scripts/ci-replay`, `scripts/install-hooks`, `.githooks/` and
`docs/ci.md` belong to the CI and verification work in flight and are outside this
documentation slice.

---

## What a future rename would touch

In dependency order:

1. **Decide the compatibility policy** for `~/.config/shelly` and `~/.cache/Shelly`
   (read-old/write-new, or migrate).
2. **Rename the Zig package** in `build.zig.zon`, including a deliberate decision about
   the fingerprint. Do this at a release boundary.
3. **Rename the CLI executable** in `Shelly.Cli.Zig/build.zig`, then update the daemon's
   imports, completions and tests.
4. **Update the UI contract**: `SHELLY_BIN` (with a fallback period for the old name),
   the `/usr/bin/shelly` and relative fallbacks in `backend/client.rs`, and the control
   CLI's default path.
5. **Update packaging in one commit**: `PKGBUILD-calix`, the `/usr/bin/calix-gpui`
   wrapper, the polkit `exec.path`, the AUR symlink and the systemd units.
6. **Update the Flatpak library name and exported symbol** with a SONAME/ABI bump, if the
   backend is to be renamed too.
7. **Rename directories and Zig modules** last, when the binary and packaging names are
   stable; this is the widest but least semantically risky change.
8. **Sweep documentation** with the command below, and keep the historical records as
   they are.

---

## Verification

The inventory command used for this pass, excluding build caches, vendored toolchains
and git:

```bash
grep -rn "Shelly\|shelly" \
  --include=*.md --include=*.txt --include=*.toml \
  --include=*.desktop --include=*.policy . \
  --exclude-dir=vendor --exclude-dir=.git --exclude-dir=target \
  --exclude-dir=zig-pkg --exclude-dir=.zig-cache
```

Remaining hits fall into the categories above: historical slice records and evidence,
literal backend commands, code identifiers, packaging paths, and the upstream artifacts
listed in §6 and §9.

A useful distinction when reading the results:

- `Shelly.PackageManager`, `Shelly.Cli.Zig`, `Shelly.Ui.Gpui`, `SHELLY_BIN`,
  `SHELLY_ELEVATOR`, `/usr/lib/shelly`, `libshelly-flatpak-backend` — **code or runtime
  contract**, leave until the coordinated rename.
- `docs/gpui/**`, `evidence/**` — **dated record**, do not rewrite.
- `com.shellyorg.*`, `PKGBUILD-git`, `shelly_banner.png`, `assets/shelly*` — **inherited
  upstream artifact**, remove in a dedicated cleanup.
- `shelly` as a command in `MANUAL_TESTING.md` — **literal current command**, correct
  today.
