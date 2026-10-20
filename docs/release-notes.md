# Calix v0.2.1

Calix v0.2.1 adds the desktop notification bridge on top of the v0.2.0 rename. The
rename itself is a **hard cut**: there is no runtime compatibility layer, and Calix does
not read the old Shelly paths, environment variables or Flatpak ABI names. If you are
coming from Shelly, the migration below still applies unchanged.

## What is in this release

- `calix-gpui`, the GPUI desktop application.
- `calix`, the Zig package-management CLI.
- `calix-daemon`, the per-user inventory service, packaged with a systemd user unit.
- `calix-notify`, the session-bus notification bridge, packaged with its own user unit.
- The desktop entry, icon and polkit policy under the `live.gnu6.calix` namespace.
- Packaging through `PKGBUILD-calix`; the package is `calix-gpui-git` and now depends
  on `dbus`, which the bridge links.
- `pkexec` as the only elevation path. `sudo`, `doas` and `su` are rejected.
- A single-profile CI gate: the Rust tests and release binary share one release build.

## Desktop notifications

`calix` already emits `live.gnu6.calix.Refresh` on the session bus after an operation
that changed installed state. `calix-notify` is a small bridge that subscribes to that
signal and turns it into one `org.freedesktop.Notifications` notification. It links
libdbus directly: there is no `dbus-monitor` output to parse and no `notify-send`
process in the path.

The bridge is installed but not enabled, like the daemon. Enable it per account:

```bash
systemctl --user enable --now calix-notify.service
systemctl --user status calix-notify.service
```

Two things it deliberately does not do:

- **It never autolaunches a session bus.** It takes `DBUS_SESSION_BUS_ADDRESS`, or
  `XDG_RUNTIME_DIR` when that is absolute, and exits otherwise. A machine with no
  desktop session must not grow a bus because a package was installed.
- **It never fails a package operation.** A missing notification daemon means the
  notification is dropped, not that the install is rolled back. Only a dropped bus
  stops the service, and systemd restarts it.

## Migration from Shelly

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
pkexec pacman -U /tmp/calix-gpui-pkg/*.pkg.tar.zst
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
- `usr/lib/calix/calix-gpui`
- `usr/bin/calix`, `usr/bin/calix-daemon` and `usr/bin/calix-notify` symlinks
- `usr/bin/calix-gpui` wrapper
- the desktop entry, icon and polkit policy
- `usr/lib/systemd/user/calix-daemon.service`
- `usr/lib/systemd/user/calix-notify.service`

It is not an installer. Use the package recipe or copy the tree deliberately.

## Known limitations

- `calix-daemon` serves inventory and release state on demand; it has no scheduler yet.
- The tracked-release observation file is still written by the frozen Python bridge.
- UI mutations cover ALPM, AUR and Flatpak.
- `calix-notify` is desktop-only: it needs a session bus and a notification daemon.
  Neither is required for the package manager itself.
- The bridge was validated against a private D-Bus, not against a live notification
  daemon, and the packaged unit was not enabled on a live session.
- Calix is pre-1.0; the backend reports the inherited `3.1.9-dev` string, not the
  product version.

## Verification

The release commit passed `scripts/verify` and `scripts/ci-replay` locally before the
tag was pushed. The same gate runs in `ci.yml`.
