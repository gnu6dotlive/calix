# Calix v0.2.2

Calix v0.2.2 is a correctness release. It fixes two defects that made the
v0.2.0 and v0.2.1 artifacts unusable on a normal Arch machine, and it ships
the optional Flatpak backend in the package.

## Read this first: v0.2.0 and v0.2.1 crash on Intel CPUs

The v0.2.0 and v0.2.1 release tarballs were built on an AMD machine with
Zig's native CPU model, so the binaries contain SSE4a instructions that exist
only on AMD processors. On an Intel CPU the backend CLI dies with `SIGILL` the
first time it opens a network connection: `calix news`, `calix search`,
anything that reaches the AUR or the Arch news feed. The frontend was not
affected, and neither was the Arch package, which is built on the machine
that installs it.

v0.2.2 is built for the x86_64 baseline, and the gate now checks every
shipped binary for instructions above that baseline, so it cannot ship again.

## What is in this release

- `calix-gpui`, the GPUI desktop application.
- `calix`, the Zig package-management CLI.
- `calix-daemon`, the per-user inventory service, packaged with a systemd user unit.
- `calix-notify`, the session-bus notification bridge, packaged with its own user unit.
- `libcalix-flatpak-backend.so.1`, the optional Flatpak backend, packaged and shipped.
- The desktop entry, icon and polkit policy under the `live.gnu6.calix` namespace.
- Packaging through `PKGBUILD-calix`; the package is `calix-gpui-git` and now depends
  on `dbus`, which the bridge links.
- `pkexec` as the only elevation path. `sudo`, `doas` and `su` are rejected.
- A single-profile CI gate: the Rust tests and release binary share one release build.

## What changed

### The optional Flatpak backend ships in the package

The package declared `flatpak` as an optional dependency but never installed
`/usr/lib/calix/libcalix-flatpak-backend.so.1`, the library the CLI loads at
runtime. The package now builds and installs it. The Flatpak runtime stays
optional: without it the library cannot be loaded, the backend reports itself
unavailable, and Calix runs with every other ecosystem.

### Commands are no longer swallowed by the inventory

The dispatcher asks each command in order whether it owns the invocation, and
`calix inventory` never checked. It built the whole inventory snapshot and
printed it, so every command listed after it in the dispatcher — list-updates,
mark, news, search, config, purify, remove, repo-db, run, utility, build and
resolve — ran the inventory instead of the command. On a machine without the
Flatpak backend they all failed with `FlatpakBackendUnavailable`; with the
backend installed they printed an inventory document instead of doing their
job.

`calix inventory` now claims only its own path, and the daemon serves the
same document it always did.

### The inventory document degrades instead of failing

An absent optional backend contributes no Flatpak rows, and the rest of the
document is still a true view of the machine. The service says so once per
process, so a missing ecosystem is never silently mistaken for a machine
without Flatpak.

### The binaries are portable

Zig's native CPU model bakes the build machine's instruction set into a
binary, and Zig emits it without a runtime check. Every Zig build in this
tree now resolves the x86_64 baseline CPU model, and the local gate fails a
build that uses an instruction above it.

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
- The bridge was validated against a private D-Bus and then against a live session
  bus; the rendered toast was not asserted programmatically.
- Calix is pre-1.0; the backend reports the inherited `3.1.9-dev` string, not the
  product version.

## Verification

The release commit passed `scripts/verify` and `scripts/ci-replay` locally before the
tag was pushed, and the published tarball was unpacked and run on a machine whose CPU
does not have the instructions the old artifacts used. The same gate runs in `ci.yml`.
