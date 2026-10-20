# Calix v0.2.4

Calix v0.2.4 is the brand release. The application wears the new mark — a
chalice whose cup is a scallop shell — and the user chooses its colour, in the
settings or from the control surface. The colour follows the accent of the
interface, so the mark and the application speak the same colour.

It also carries the v0.2.3 correctness fixes, for the accounts that have not
taken them yet.

## The mark, and the colour the user chooses

The rail header now shows the shell in a chalice, painted by the accent of the
interface. Two colourways, one geometry, and the choice is the user's:

- **Signal Orange** — `#FF6A00`, hover `#FF8E40` (the default);
- **Royal Blue** — `#4169E1`, hover `#87A0F4`.

The setting is in the appearance section of the settings, and on the control
surface:

```bash
calix-gpui settings set brand-color royal
calix-gpui settings get brand-color
```

The choice changes the mark, the accent (buttons, focus ring, active row) and the
focus ring itself. It does not change the source badges — ALPM, AUR, Flatpak,
AppImage, Cargo, NPM and Release each keep the colour that names their ecosystem.

The launcher, the taskbar and the window switcher read a system file, so they
cannot follow a per-user setting: the package ships the Signal Orange colour
master under the application id, with the raster sizes and the single-colour
variant the menus use.

The colours are design system tokens, taken as they are — no re-light, no
re-tint. The rules the application follows are written in
[`docs/brand.md`](docs/brand.md).

## What v0.2.3 fixed, and what v0.2.2 fixed before it

### The Updates scope never worked

One of the five entries in the rail — **Updates** — answered
*Failed to check for updates: Désérialisation de la liste des mises à jour*
in every released version, with a Retry button that could never succeed.

The cause was a disagreement between the two halves of the product, on the
shape of one payload. The frontend asked the CLI for every backend's updates
and read the answer as a flat list:

```rust
// the frontend, before
serde_json::from_str::<Vec<PackageUpdateItem>>(trimmed)
```

The CLI's `all` variant answers with an **object**, one array per backend:

```json
{"Packages": [], "Aur": [], "AppImage": [], "Flatpak": []}
```

The flat array is what the per-backend variants return
(`calix list-updates standard -j`, `list-updates aur -j`). The frontend asked
for the grouped one and parsed it as the flat one, so the deserialisation
always failed.

The frontend now reads the grouped answer and flattens it. The scope lists
the pending updates, the rail entry carries the count badge, and the two
sides of the agreement are pinned by tests: the CLI's own test already fixed
the shape of its answer, the frontend now parses that exact string, and a
third test parses the answer of the real CLI binary when it is present.

## The layout follows the mapped window

The workstation computed its widths from the size the application *requested*
at launch, not from the size the compositor grants it. A tiling compositor
resizes the window whenever the focus changes, and the request stays the one
from the launch. The result was an inspector of **137px** — below its 320px
floor — in a window of 1056px, and of **1622px** — above its 520px cap — in a
window of 2542px, in the same session.

The workstation now measures the mapped window (`viewport_size`), which is
what the compositor really granted, and the inspector stays inside
`[320, min(520, 3/7·w_c)]` at every width.

## What is in this release

- `calix-gpui`, the GPUI desktop application.
- `calix`, the Zig package-management CLI.
- `calix-daemon`, the per-user inventory service, packaged with a systemd user unit.
- `calix-notify`, the session-bus notification bridge, packaged with its own user unit.
- `libcalix-flatpak-backend.so.1`, the optional Flatpak backend, packaged and shipped.
- The desktop entry, icon and polkit policy under the `live.gnu6.calix` namespace.
- Packaging through `PKGBUILD-calix`; the package is `calix-gpui-git`.
- `pkexec` as the only elevation path. `sudo`, `doas` and `su` are rejected.
- A single-profile CI gate: the Rust tests and release binary share one release build.

## The front-end requalification has started

The navigation of the application is being requalified before it is redrawn.
The contract is published in `docs/gpui/slice-07/`: the current state with
code pointers, the information architecture, the wireframes at three widths,
fifty-three numbered requirements and the acceptance matrix.

It is a contract, not a change: no screen moves in this release. It records
what the next slices are measured against, and the three arbitration points
it leaves open (the shape of the rail, the meaning of the search box in each
scope, and when the next release lands) are decisions for the operator.

Two of the findings that shaped it are the defects fixed in this release: the
Updates scope that could not load, and the layout that ignored the window it
was given.

## Desktop notifications

`calix` already emits `live.gnu6.calix.Refresh` on the session bus after an
operation that changed installed state. `calix-notify` is a small bridge that
subscribes to that signal and turns it into one `org.freedesktop.Notifications`
notification. It links libdbus directly: there is no `dbus-monitor` output to
parse and no `notify-send` process in the path.

The bridge is installed but not enabled, like the daemon. Enable it per account:

```bash
systemctl --user enable --now calix-notify.service
systemctl --user status calix-notify.service
```

Two things it deliberately does not do:

- **It never autolaunches a session bus.** It takes `DBUS_SESSION_BUS_ADDRESS`,
  or `XDG_RUNTIME_DIR` when that is absolute, and exits otherwise. A machine with no
  desktop session must not grow a bus because a package was installed.
- **It never fails a package operation.** A missing notification daemon means the
  notification is dropped, not that the install is rolled back. Only a dropped bus
  stops the service, and systemd restarts it.

## Migration from Shelly

Calix v0.2.0 was a hard cut from the Shelly name. There is no runtime
compatibility layer, and Calix does not read the old Shelly paths, environment
variables or Flatpak ABI names. If you are coming from Shelly, the migration
below still applies unchanged.

### 1. Remove the old package

Find the package that owns the old executable. If it is not installed, skip this step:

```bash
pacman -Qo /usr/bin/shelly
```

Remove that package and its unneeded dependencies through polkit:

```bash
pkexec pacman -Rns <package-name>
```

Do this before installing `calix-gpui-git` so the two packages do not compete for the
same paths.

### 2. Move the user state

Calix uses new paths. Move the old state once; there is no fallback reader. Skip any
path that does not exist:

```bash
mv ~/.config/shelly ~/.config/calix
mv ~/.cache/Shelly ~/.cache/calix
```

The AppImage metadata database lives inside that config directory, so it moves with it;
there is no separate copy step. The system cache is moved through polkit, not `sudo`:

```bash
pkexec mv /var/cache/shelly /var/cache/calix
```

### 3. Install Calix

From a source checkout:

```bash
make package
pkexec pacman -U --noconfirm /tmp/calix-gpui-pkg/*.pkg.tar.zst
```

The package installs both user services but does not enable them. Enabling a user unit
is a per-account decision:

```bash
systemctl --user enable --now calix-daemon.service
systemctl --user status calix-daemon.service
```

### 4. Replace the old environment variables

`SHELLY_BIN`, `SHELLY_ELEVATOR`, `SHELLY_CONFIG_DIR` and `SHELLY_APPIMAGE_DB` are now
`CALIX_BIN`, `CALIX_ELEVATOR`, `CALIX_CONFIG_DIR` and `CALIX_APPIMAGE_DB`. The old names
are not read.

### 5. Rebuild Flatpak integration

The optional Flatpak backend ABI changed with no aliases:

- `libshelly-flatpak-backend.so.1` is now `libcalix-flatpak-backend.so.1`;
- `shelly_flatpak_backend_get_api` is now `calix_flatpak_backend_get_api`.

Any external consumer must be rebuilt against the new library.

## Release artifact

The GitHub release contains `calix-gpui-linux-x86_64.tar.gz`, its
`.sha256` checksum, and a `usr/` tree with:

- `usr/lib/calix/calix`
- `usr/lib/calix/calix-daemon`
- `usr/lib/calix/calix-notify`
- `usr/lib/calix/libcalix-flatpak-backend.so.1`
- `usr/lib/calix/calix-gpui`
- `usr/bin/calix`, `usr/bin/calix-daemon` and `usr/bin/calix-notify` symlinks
- `usr/bin/calix-gpui` wrapper
- the desktop entry and the polkit policy
- `usr/share/icons/hicolor/scalable/apps/live.gnu6.calix.svg`, its single-colour
  variant, and the raster sizes from 16 to 512
- `usr/lib/systemd/user/calix-daemon.service`
- `usr/lib/systemd/user/calix-notify.service`

It is not an installer. Use the package recipe or copy the tree deliberately.

## Known limitations

- `calix-daemon` serves inventory and release state on demand; it has no scheduler yet.
- The tracked-release observation file is still written by the frozen Python bridge.
- UI mutations cover ALPM, AUR and Flatpak.
- The navigation is being requalified: the contract is published, the
  implementation is not. The rail still mixes a scope switch with page
  navigation, and the current scope is still not named on screen.
- The list pane's width is clamped to the window it is drawn in, and the
  clamped value is kept: narrowing the window and widening it back does not
  restore the width the user had chosen.
- `calix-notify` is desktop-only: it needs a session bus and a notification daemon.
  Neither is required for the package manager itself.
- The bridge was validated against a private D-Bus and then against a live session
  bus; the rendered toast was not asserted programmatically.
- Calix is pre-1.0; the backend reports the inherited `3.1.9-dev` string, not the
  product version.

## Verification

The release commit passed `scripts/verify` and `scripts/ci-replay` locally before the
tag was pushed, and the published tarball was unpacked and run on the machine that
found the defects. The same gate runs in `ci.yml`.
