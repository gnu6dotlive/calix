# Manual testing checklist

This document tracks manual testing procedures for Calix components that require human
verification beyond automated tests.

> **Naming note.** The Zig backend executable is named `calix`, so the commands below
> are literal. The desktop application is Calix (`calix-gpui`). Some sections were
> inherited from the upstream project and describe components that are not part of the
> current Calix UI; check them against the current tree before treating a missing
> feature as a regression. See [`docs/legacy-references.md`](docs/legacy-references.md).

## Tray refresh after CLI updates (#1591)

- [ ] With pending updates visible in the tray, run `calix upgrade` in a terminal. After completion, verify the tray rechecks and updates its icon and package submenus without restarting it.
- [ ] Repeat for individual `upgrade standard`, `upgrade aur`, `upgrade flatpak`, and `update` commands. Packages still awaiting updates should remain listed.
- [ ] Run an update through `pkexec` or the configured privilege elevator; verify the original user's tray refreshes.
- [ ] Complete a CLI update while the tray is already checking. Verify it checks again after the in-progress check finishes.
- [ ] Run a combined upgrade where one backend fails after another succeeds; verify the tray reflects the remaining updates and the command still returns failure.
- [ ] Repeat with the tray stopped or the session bus unavailable; verify no notification errors appear and the CLI retains its normal exit status.

## Terminal PKGBUILD review (#1385)

- [ ] With a configuration missing `CollapsePkgbuildDiff`, verify `calix config get CollapsePkgbuildDiff` returns `True`.
- [ ] Review an AUR update with a long PKGBUILD: each change has three unchanged context lines on either side, nearby sections merge, and omitted sections show their line counts.
- [ ] Run `calix config set CollapsePkgbuildDiff false`, then repeat the review and verify the full file appears. Restore `true` and verify the collapsed view returns.
- [ ] Review a package without a previous PKGBUILD and verify the entire new file appears.
- [ ] Confirm security warnings and related source files remain visible, and declining approval still cancels the operation.
- [ ] Repeat with color disabled and redirected output; omission markers and changes remain readable.
- [ ] Verify GUI PKGBUILD reviews still receive the complete diff regardless of this setting.

## AppImage checks during combined upgrades

- [ ] Verify `calix config get DisableAppImageUpdateCheck` returns `False` for a configuration without the new key.
- [ ] Set `DisableAppImageUpdateCheck` to `true` and run `calix upgrade all`: the preview omits AppImages, and execution performs no AppImage update check or upgrade.
- [ ] Repeat with bare `calix`, `calix -U`, and `calix upgrade standard --all`, including privilege elevation.
- [ ] Verify `calix upgrade appimage` and `calix list-updates appimage` still check AppImages explicitly.
- [ ] Restore `false` and verify combined upgrades include AppImages again; `--no-appimage` still skips them for one run.

## Flatpak checks during combined upgrades

- [ ] Verify `calix config get DisableFlatpakUpdateCheck` returns `False` for a configuration without the new key.
- [ ] Set `DisableFlatpakUpdateCheck` to `true` and run `calix upgrade all`: the preview omits Flatpak, and execution performs no Flatpak update check or upgrade.
- [ ] Repeat with bare `calix`, `calix -U`, and `calix upgrade standard --all`, including privilege elevation.
- [ ] Enable both `DisableFlatpakUpdateCheck` and `DisableAppImageUpdateCheck`; verify combined upgrades skip both backends.
- [ ] Verify `calix upgrade flatpak` and `calix list-updates flatpak` still check Flatpak explicitly.
- [ ] Restore `false` and verify combined upgrades include Flatpak again when the backend is installed; `--no-flatpak` still skips it for one run.

## UI Testing (Calix-UI)

### Installation & Startup

- [ ] Application launches successfully on first run
- [ ] Application icon displays correctly in system tray/taskbar
- [ ] Window opens at appropriate size and position
- [ ] Application responds to window resize operations
- [ ] Application can be minimized/maximized/closed properly

### AppImage Environment Variables (#1719)

- [ ] Open an installed AppImage and enter `WEBKIT_DISABLE_DMABUF_RENDERER=1` in Environment Variables, then select Save Environment.
- [ ] Launch from the desktop menu and `calix run appimage <name>`; confirm the variable reaches the application.
- [ ] Restart Calix, sync one/all AppImages, update, and replace the AppImage with a newer filename; confirm the setting survives.
- [ ] Add/edit/remove multiple lines, including an empty value (`KEY=`); verify duplicate or invalid keys prevent saving.
- [ ] Navigate away with unsaved changes and return; confirm the draft remains. Back prompts to discard it.
- [ ] Verify a failed save keeps the draft and the previously saved launcher/settings.
- [ ] Remove the variable's line and save; confirm the normal inherited/bundled environment is restored.
- [ ] On an NVIDIA system affected by #1719, verify YARC launches successfully with the workaround.

CLI equivalents (use the exact installed name):

```sh
calix config appimage YARC --set-env WEBKIT_DISABLE_DMABUF_RENDERER=1
calix config appimage YARC --unset-env WEBKIT_DISABLE_DMABUF_RENDERER
calix config appimage YARC --clear-env
```

### Package Search & Display

- [ ] Search functionality returns relevant results
- [ ] Package list displays correctly with all metadata
- [ ] Package details view shows complete information
- [ ] Package icons/images load properly
- [ ] Scrolling through large package lists is smooth
- [ ] Filtering and sorting options work as expected

### Package Installation

- [ ] Installing a package shows progress correctly
- [ ] Installation completes successfully

### Error Messages

- [ ] In a disposable package database, trigger a database lock failure. CLI, GTK, and TUI show the configured `db.lck` path and the matching, quoted removal command. The message says to remove the file only when no package manager is running.
- [ ] Trigger missing dependencies and file conflicts in a disposable environment. The explanation names the affected packages or paths and provides a next step; technical details follow it.
- [ ] Fail an AUR build step. The message includes the package name and failed stage, and build output remains available.
- [ ] Deny a GTK authorization request. The failure explains that permission was not granted.
- [ ] Verify GTK retains the specific failure in its final status and TUI displays the full explanation, including multiline lock instructions.
- [ ] Verify recoverable cleanup failures appear as warnings and cancellation does not claim that an unexpected error occurred.

### Dynamic AUR Sources

- [ ] `calix install aur gpu-screen-recorder-ui-git` resolves its `$(sed ...)` source URL and reaches normal Git source acquisition
- [ ] The PKGBUILD review warns before a `source=()` command substitution executes
- [ ] Declining the PKGBUILD review executes no source command
- [ ] A command substitution that resolves to a local source shows a supplemental related-file review before the file is copied
- [ ] Declining the supplemental review prevents all package lifecycle functions from running
- [ ] Command substitution in non-source arrays remains a clear unsupported-PKGBUILD failure
- [ ] Success/failure notifications display appropriately
- [ ] Dependencies are shown and handled correctly
- [ ] Disk space warnings appear when needed
- [ ] Installation can be cancelled mid-process

### Package Updates

- [ ] Available updates are detected and listed
- [ ] Update all functionality works correctly
- [ ] Individual package updates work
- [ ] Update progress is displayed accurately
- [ ] Post-update notifications are shown

### Package Removal

- [ ] Package removal shows dependency warnings
- [ ] Removal process completes successfully
- [ ] Orphaned packages are identified
- [ ] Confirmation dialogs appear before removal

### AUR Integration

- [ ] AUR packages can be searched
- [ ] AUR package information displays correctly
- [ ] AUR packages can be installed
- [ ] AUR packages can be updated
- [ ] Build process output is visible
- [ ] AUR package removal works

### Flatpak Integration

- [ ] A base-only installation does not install `flatpak` or
  `calix-flatpak-backend` as a required dependency
- [ ] `calix --help`, `calix --version`, and completion generation work with
  no backend installed
- [ ] A direct Flatpak command explains that `calix-flatpak-backend` and
  Flatpak must be installed
- [ ] `list-updates all` warns, skips Flatpak, and continues other backends when
  the backend is missing
- [ ] `upgrade all` silently skips Flatpak and continues other selected
  backends when the backend is missing
- [ ] `upgrade all` still warns for an incompatible Flatpak backend and reports
  a broken Flatpak backend as a failed upgrade step
- [ ] Backup warns and exports standard/AUR records without Flatpak records
  when the backend is missing
- [ ] Installing `calix-flatpak-backend` enables Flatpak without rebuilding
  or reinstalling the base package
- [ ] An ABI-incompatible backend is rejected with an upgrade-together message
- [ ] Flatpak packages are listed separately
- [ ] Flatpak search works correctly
- [ ] Flatpak installation succeeds
- [ ] Flatpak updates work
- [ ] Flatpak removal works
- [ ] Status and percentage events reach both terminal and `--ui-mode`
- [ ] Cancelling an in-progress Flatpak transaction stops its native
  `GCancellable` and reports cancellation once

### Repository Management

- [ ] Repository list displays correctly
- [ ] Repository synchronization works
- [ ] Repository status indicators are accurate
- [ ] Custom repositories can be added (when implemented)

### UI/UX Elements

- [ ] All buttons are clickable and responsive
- [ ] Tooltips display helpful information
- [ ] Keyboard shortcuts work as expected
- [ ] Tab navigation works properly
- [ ] Theme/styling is consistent throughout
- [ ] Text is readable and properly sized
- [ ] Icons are clear and meaningful

### Error Handling

- [ ] Network errors are handled gracefully
- [ ] Permission errors show appropriate messages
- [ ] Invalid input is rejected with clear feedback
- [ ] Application doesn't crash on unexpected errors

## CLI Testing (Calix-CLI)

### Basic Commands

- [ ] `calix --help` displays help information
- [ ] `calix --version` shows correct version
- [ ] Command syntax errors show helpful messages
- [ ] Running `calix` with no arguments still displays and confirms the combined upgrade plan
- [ ] `calix firefox` searches standard repositories and the AUR, while recognized commands and shortcodes retain their
      normal behavior
- [ ] Bare-value results count down toward `1`, with the closest match displayed last as `1` and selected by Enter
- [ ] Entering `0` cancels SearchInstall without starting a transaction
- [ ] Selecting a standard or AUR result enters the corresponding normal install workflow

### Package Operations

- [ ] `calix search <package>` returns results
- [ ] `calix install <package>` installs successfully
- [ ] `calix remove <package>` removes successfully
- [ ] `calix update` updates all packages
- [ ] `calix info <package>` shows package details

### Removal Confirmation (#1845)

Use disposable packages in a VM/container with an unused dependency and an
unused optional dependency; keep another optional dependency required by a
separate installed package.

- [ ] `calix -Rso <package>` lists the complete removal set, versions, and
  total removed size before `Proceed with package removal? (Y/n)`.
- [ ] The unused dependencies appear; the dependency still required elsewhere
  stays installed and does not appear in the removal plan.
- [ ] `n`, Ctrl-D/EOF, and Ctrl-C cancel without removing packages or associated
  configuration. A subsequent transaction can acquire the database lock.
- [ ] Enter and `y` remove the displayed packages.
- [ ] `calix remove standard --opt-deps <package>` behaves identically.
- [ ] `-n`/`--no-confirm` displays the plan and removes packages without prompting.
- [ ] AUR removal confirms before removal and cache cleanup; declining preserves
  both. GTK removal shows the plan and honors accept/cancel.
- [ ] `purify` and automatic build-dependency cleanup retain their existing
  approval flow without a second removal prompt.

Automated coverage: `(cd Calix.PackageManager && zig build alpm-removal-test)`
and `(cd Calix.Cli.Zig && zig build test)`. Run the focused removal target as
root in the disposable environment to exercise DB-only commits; those cases
are skipped when unprivileged.

### Repository Operations

- [ ] `calix sync` synchronizes repositories
- [ ] Repository refresh works correctly
- [ ] Database updates complete successfully

### AUR Commands

- [ ] AUR search works from CLI
- [ ] AUR installation works from CLI
- [ ] AUR updates work from CLI

### Custom AUR Base URL (Atoll Compatibility)

- [ ] Configure custom base URL: `calix config set AurUrl https://atoll.seafoam-labs.org`
- [ ] Verify RPC search and package info queries against the configured base
- [ ] Verify `calix search aur --pkgbuild <pkg>` retrieves `PKGBUILD` via Git checkout instead of cgit
- [ ] Install normal package and split package from the custom base
- [ ] Verify AUR dependency and provider resolution work against custom base
- [ ] Verify list, list-updates, update, remove, and upgrade operations work with custom base
- [ ] Verify aggregate commands (`upgrade all`, `list-updates all`) use custom base for their AUR branch
- [ ] Verify elevated operations still use the invoking user's `AurUrl` setting
- [ ] Verify `--aur-url` overrides the persistent `AurUrl` setting per command
- [ ] Switch back to official base (`calix config set AurUrl https://aur.archlinux.org`) and verify checkout origin replacement safeguard

### Sandboxed AUR Builds (Landlock)
Requires a kernel with Landlock enabled (check `cat /sys/kernel/security/lsm`).
- [ ] `calix build --makesrcinfo --reviewed PKGBUILD > .SRCINFO` matches
  `makepkg --printsrcinfo`, including split-package and architecture-specific
  fields
- [ ] `--makesrcinfo` writes progress/review output to stderr and only SRCINFO
  to stdout
- [ ] `--makesrcinfo` does not run `pkgver()`, `verify()`, `prepare()`,
  `build()`, `check()`, or `package()`, and does not create package artifacts
- [ ] A split PKGBUILD that conditionally appends an enabled member to
  `pkgname` builds that member when no `--package` selection is supplied
- [ ] `calix build --package <dynamic-member> PKGBUILD` builds only that
  enabled member and rejects it when its condition evaluates false
- [ ] With `[sandbox] enabled = true` in `calixbuild.conf`, a PKGBUILD whose
  `build()` runs `ls "$HOME"` fails that listing with "Permission denied" in
  the step output while the build itself completes
- [ ] The same PKGBUILD lists `$HOME` normally with the sandbox disabled
- [ ] A step reading a file outside the allow-list (for example
  `cat "$HOME/.ssh/id_rsa"`) is denied
- [ ] Steps still have working `/tmp`, `/proc`, `/usr`, and `/etc` access
  (`nproc`, redirects to `/dev/null`, and compiler invocations succeed)
- [ ] Package signing still works with the sandbox enabled
- [ ] On a kernel without Landlock, an enabled sandbox fails the build before
  the first step instead of running unprotected
- [ ] A build that needs extra user paths succeeds after adding them to
  `[sandbox] extra_read` / `extra_write`
- [ ] A failing sandboxed step leaves a `[sandbox]` hint line in its build log

### Keyring Management

- [ ] `calix keyring init` initializes keyring
- [ ] `calix keyring populate` populates keys
- [ ] Keyring operations complete without errors

### Output Formatting

- [ ] CLI output is properly formatted
- [ ] Colors/markup display correctly in terminal
- [ ] Progress indicators work
- [ ] Error messages are clear and actionable
- [ ] Verbose mode provides additional details

### UI Mode Integration

- [ ] CLI can be called from UI successfully
- [ ] Output is properly captured and displayed in UI
- [ ] Error codes are correctly propagated

### Shell Completions

- [ ] `calix utility --completions bash|fish|zsh` writes each script
- [ ] Shortcode tokens expand in a live shell (`calix -Sa<tab>` offers
  `-Sap` and `-Sad`; `calix -N<tab>` offers `-N` and `-Na`)
- [ ] Options complete after a shortcode (`calix -U -<tab>` offers the
  combined-upgrade options; `calix -B -<tab>` offers the backup options)

**Note:** isolate the generated script from completions installed by an
older package build before live-testing it. Fish lazily loads every
`calix.fish` found on `fish_complete_path` on the first completion request,
so a stale `/usr/share/fish/vendor_completions.d/calix.fish` mixes silently
with the script under test and produces confusing candidates. Point
`fish_complete_path` at an empty directory while testing (or reinstall the
package first); for bash and zsh use a clean shell, and regenerate
`~/.zcompdump` if zsh keeps serving stale entries.

## Installation & Deployment

### PKGBUILD Installation

- [ ] `makepkg -si` builds and installs successfully
- [ ] `calix` and `calix-flatpak-backend` can be packaged and installed
  independently
- [ ] `/usr/lib/calix/libcalix-flatpak-backend.so.1` belongs only to the
  backend package
- [ ] `readelf -d /usr/bin/calix` has no `libflatpak.so.0` or
  `libostree-1.so.1` `NEEDED` entry
- [ ] The backend has SONAME `libcalix-flatpak-backend.so.1` and a
  `libflatpak.so.0` `NEEDED` entry
- [ ] All dependencies are correctly specified
- [ ] Post-install scripts execute properly
- [ ] Files are installed to correct locations

### Web Installer

- [ ] Web install script downloads correctly
- [ ] Installation completes without errors
- [ ] All components are installed
- [ ] Desktop entries are created

### AUR Installation

- [ ] Package builds via yay/paru
- [ ] All dependencies resolve correctly
- [ ] Installation completes successfully

### Uninstallation

- [ ] Uninstall script removes all components
- [ ] Configuration files are handled appropriately
- [ ] No orphaned files remain

## System Integration

### Permissions

- [ ] Root operations work correctly
- [ ] Permission errors are handled gracefully
- [ ] User is prompted for elevation when needed
- [ ] `zig build --build-file Calix.Cli.Zig/build.zig isolated-build-test`
  passes under umasks `0022`, `0007`, `0027`, and `0077`, preserving reviewed
  file modes, guest traversal permissions, readable configuration, and the
  private host operation boundary
- [ ] From an authenticated normal-user polkit session,
  `Calix.Cli.Zig/scripts/test-isolated-build.sh` passes all four coordinator
  umasks through real nspawn, checks UID/GID 1000 and guest configuration,
  exports artifacts to the invoking user, and removes operation roots;
  exit `77` means skipped, not passed
- [ ] Build the pinned endcord PKGBUILD with `--isolated` using the rebuilt
  CLI under coordinator umasks `0022` and `0077`; validate the exported
  archives without installing them and record the recipe revision/source hash
- [ ] `Calix.Cli.Zig/scripts/test-elevation-cancellation.sh` passes without
  privileges for both SIGINT and SIGTERM
- [ ] From a normal user session with a working elevator,
  `Calix.Cli.Zig/scripts/test-isolated-cancellation.sh` exits successfully and
  leaves neither nspawn descendants nor an isolated operation directory

### File System

- [ ] Package cache is managed correctly
- [ ] Temporary files are cleaned up
- [ ] Configuration files are preserved on update

### System State

- [ ] System package database remains consistent
- [ ] No conflicts with pacman operations
- [ ] Lock files are handled correctly

### Desktop Integration

- [ ] Desktop file launches application
- [ ] Application appears in application menu
- [ ] File associations work (if applicable)
- [ ] System notifications display correctly

## Performance Testing

### Responsiveness

- [ ] UI remains responsive during operations
- [ ] Large package lists load quickly
- [ ] Search results appear promptly
- [ ] No UI freezing during background operations

### Resource Usage

- [ ] Memory usage is reasonable
- [ ] CPU usage is acceptable during operations
- [ ] Disk I/O is efficient
- [ ] Network bandwidth usage is appropriate

## Cross-Platform Testing (Arch-based distros)

### Arch Linux

- [ ] All features work on vanilla Arch
- [ ] Integration with pacman is seamless

### Manjaro

- [ ] Compatible with Manjaro repositories
- [ ] Manjaro-specific packages work

### EndeavourOS

- [ ] Works with EndeavourOS setup
- [ ] No conflicts with EndeavourOS tools

### Other Arch-based

- [ ] Test on other Arch derivatives as available

## Regression Testing

### After Updates

- [ ] Previously working features still work
- [ ] No new crashes or errors introduced
- [ ] Performance hasn't degraded
- [ ] UI/UX remains consistent

## Notes

- Date of last full manual test: _____________
- Tester: _____________
- Version tested: _____________
- Issues found: _____________

## Known Issues

Document any known issues that are being tracked:

1.
2.
3.

## Native build PATH (issue #1931)

- [ ] Run a minimal reviewed PKGBUILD that launches an intentionally missing
  executable using Rust `Command::spawn`. With an inaccessible temporary
  directory appended to the invoking shell's PATH, confirm the build sees only
  the configured build PATH and the lookup returns `NotFound`, not
  `PermissionDenied`. Restore the temporary directory's permissions afterward.
- [ ] Repeat through a GUI AUR operation launched with pkexec and through CLI
  dependency synchronization. Capture the non-root build PATH; it must not
  contain the elevated coordinator's private directories.
- [ ] Put a custom tool in an absolute directory named in `[build] extra_path`.
  Confirm metadata review, `--makesrcinfo`, lifecycle functions, and native
  packaging helpers can use it, with configured tools preceding system tools.
- [ ] Check system/user overrides and `extra_path = []`. A configured missing,
  non-directory, or inaccessible entry must report its path before PKGBUILD
  execution. Confirm Perl tools and ccache/distcc precedence remain correct.
- [ ] With Landlock enabled, confirm PATH additions alone do not grant access
  outside the sandbox allow-list; add explicit sandbox grants and retry.
- [ ] In an isolated build, confirm extra directories are interpreted inside
  the guest; host-only directories must produce a clear error, without mounts.
- [ ] Rebuild `scx-scheds-git` through the GUI on the affected system and verify
  the intentionally missing formatter no longer causes errno 13.
