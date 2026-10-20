# Legacy references and naming

Calix is historically forked from **Shelly**, the Arch Linux package manager developed
by [Seafoam Labs](https://github.com/Seafoam-Labs/Shelly-ALPM). That lineage is recorded
in [the README](../README.md#history--lineage) and in the credits; it is not a claim
about the current code.

The rename was completed on 2026-10-20. No live code, build input, packaging asset,
script, path, environment variable or ABI symbol uses the old product name. The old name
now appears only in three places:

- the README's history and credits sections;
- `docs/**`, including this mapping and the dated records (`docs/shelly-*.md`,
  `docs/gpui/slice-*`, `docs/gpui/ux/`) that describe the tree as it was when they
  were written;
- `evidence/` and the git history, which are immutable records.

`scripts/check-naming` enforces that boundary: it fails if the old name appears in a
tracked file or path outside `README.md`, `docs/**` and `evidence/**`.

---

## Naming map

| Thing | Current name |
|---|---|
| Project / product | **Calix** |
| Desktop application | `calix-gpui` |
| Desktop app id / icon / polkit namespace | `live.gnu6.calix` |
| Inventory daemon | `calix-daemon` |
| Backend CLI | `calix` |
| Backend library | `Calix.PackageManager` (imported as `Zigalpm`) |
| Backend CLI source | `Calix.Cli.Zig/` |
| Frontend source | `Calix.Ui.Gpui/` |
| Optional Flatpak backend | `Calix.Flatpak.Backend/`, `libcalix-flatpak-backend.so.1` |
| Flatpak entry point | `calix_flatpak_backend_get_api` |
| Installed CLI | `/usr/lib/calix/calix`, with `/usr/bin/calix` as a symlink |
| Installed frontend | `/usr/lib/calix/calix-gpui` |
| User config | `~/.config/calix/` |
| User cache | `~/.cache/calix/` |
| System cache | `/var/cache/calix/` |
| Daemon state | `$XDG_STATE_HOME/calix` (default `~/.local/state/calix`) |
| Operator elevation override | `CALIX_ELEVATOR` |
| CLI override | `CALIX_BIN` |
| AUR build configuration | `calixbuild.conf` |
| D-Bus refresh signal | `live.gnu6.calix.Refresh` |
| D-Bus object path | `/live/gnu6/calix/Notifications` |
| Zig package | `.calix`, fingerprint `0x30067142801312fc` |

### Why `live.gnu6.calix`

The project owns the domain `gnu6.live`, so the reverse-DNS namespace is `live.gnu6`.
`live.gnu6.calix` is therefore derived from a domain the project actually controls.

Two alternatives were rejected:

- `com.calix.*` implies control of `calix.com`, which belongs to Calix Inc., an
  unrelated publicly traded company;
- `in.gnu.*` implies control of `gnu.in`, which the project does not own. The company
  name `gnu.in.labs inc.` is not a domain claim.

The D-Bus signal keeps the product segment once: `live.gnu6.calix.Refresh`, not a
repeated `calix.calix`.

---

## Hard cut for user state

There is no runtime migration shim and no fallback that reads the old paths. The old
name is not read at startup. Existing installations must move their state once:

```bash
mv ~/.config/shelly          ~/.config/calix
mv ~/.cache/Shelly           ~/.cache/calix
mv /var/cache/shelly         /var/cache/calix   # elevated via pkexec, not sudo
```

The AppImage metadata database moves with `~/.config/shelly/appimage-metadata-v2.db` to
`~/.config/calix/appimage-metadata-v2.db`. The frozen Python bridge was renamed to read
and write the new paths; it does not know about the old ones.

This is deliberate. A compatibility branch would keep the old name in live code, which
is exactly what the rename removes. The project is pre-1.0 and accepts the one-time
manual move.

---

## Breaking contracts

The rename is a hard cut for every public surface:

- **Flatpak ABI.** `libshelly-flatpak-backend.so.1` and
  `shelly_flatpak_backend_get_api` became `libcalix-flatpak-backend.so.1` and
  `calix_flatpak_backend_get_api`. No alias or compatibility symbol is exported.
  `Calix.Flatpak.Backend` and its only in-tree consumer are rebuilt together.
- **Zig package identity.** `.name = .calix` with the new fingerprint
  `0x30067142801312fc`. Zig caches are invalidated; the old fingerprint is gone.
- **Desktop, icon and polkit identity.** `live.gnu6.calix.desktop`,
  `live.gnu6.calix.svg`, `live.gnu6.calix.policy`, and action ids
  `live.gnu6.calix.{install,remove,upgrade}`.
- **Environment variables.** `SHELLY_BIN`, `SHELLY_ELEVATOR`, `SHELLY_CONFIG_DIR`,
  `SHELLY_APPIMAGE_DB` and the Flatpak test overrides are now `CALIX_*`. The old names
  are not read.

`PKGBUILD-calix` installs the CLI at the canonical polkit path `/usr/lib/calix/calix`
and exposes `/usr/bin/calix` as a symlink. The frontend wrapper sets `CALIX_BIN`.

---

## Removed surfaces

The rename did not preserve unbuilt code under a new name. The following were removed
because no Calix build path produced them and no live component imported them:

- `Shelly.Ui.Gtk/`, `Shelly.Tui/`, `Shelly.Notifications.Zig/`, `Shelly.Key/`;
- the legacy root `PKGBUILD`, `PKGBUILD-bin`, `PKGBUILD-git` and `PKGBUILD-cli`;
- the orphan `com.shellyorg.*.desktop` entries;
- the root icon and banner assets installed only by those recipes;
- `.github/FUNDING.yml`, which routed donations to Shelly's author from the Calix
  repository.

The tray refresh module is kept because it is wired into update and upgrade, but it now
signals `live.gnu6.calix.Refresh` on `/live/gnu6/calix/Notifications`. Its consumer is a
future Calix notifications service; until then the signal is a no-op for listeners that
are not installed.

---

## Historical documents

`docs/shelly-flatpak-backend-plan.md` and
`docs/shelly-libalpm-compatible-dependency-resolution-gaps.md` are upstream-era design
records. They keep their names and content because rewriting them would destroy the
record they preserve. The same applies to the dated `docs/gpui/slice-*` reports and the
files under `evidence/`.

For current behavior, use the README, `docs/calix-architecture.md` (historical context),
`docs/isolated-builds.md` and `docs/flatpak-backend-abi.md`.
