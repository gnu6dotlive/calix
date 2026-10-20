# Contributing to Calix

Thank you for your interest in Calix. Bug reports, design discussion, documentation
fixes and code contributions are welcome.

Calix is pre-1.0 and changes quickly. Before starting a large change, open an issue or a
discussion so the approach can be agreed on first.

---

## Project architecture

Calix is split into a native frontend, a state service and a package-management backend:

- **`Calix.Ui.Gpui/` (Rust)** — the `calix-gpui` desktop application.
  Built on [GPUI 0.2](https://github.com/zed-industries/zed/tree/main/crates/gpui), the
  GPU framework behind Zed. It owns rendering, layout, search orchestration, the package
  inspector and the operation console.
- **`Calix.Cli.Zig/src/daemon/` (Zig)** — `calix-daemon`. Serves a versioned JSON-lines
  inventory over a Unix socket. The frontend reads the daemon rather than opening state
  files.
- **`Calix.Cli.Zig/` (Zig)** — the `calix` CLI. Package operations (search, install,
  remove, upgrade, builds, keyring) run here. The rename is complete; see
  [`docs/legacy-references.md`](docs/legacy-references.md) for the old-name boundary.
- **`Calix.PackageManager/` (Zig)** — the shared backend library. ALPM through
  `libalpm`, AUR, Flatpak, AppImage, Cargo, NPM, the unified inventory record and the
  release registry.
- **`Calix.Flatpak.Backend/` (Zig)** — optional `libflatpak` bridge, loaded at runtime
  so the base CLI does not depend on Flatpak.
- **`packaging/`** — the AUR symlink, the `calix-daemon` user service and the legacy
  sync units. The current package definition is `PKGBUILD-calix`.

The Python engine in `scripts/` is **frozen**. It is a transitional bridge for release
tracking and receives fixes only, no new features. New work belongs in Zig.

---

## Local development setup

### Prerequisites

On Arch Linux or an Arch-based distribution:

```bash
pkexec pacman -S --needed base-devel git rust cargo pacman clang pkgconf \
  libglvnd fontconfig freetype2 wayland libx11 libxcb libxkbcommon \
  libxkbcommon-x11 mesa vulkan-icd-loader polkit
```

Zig **0.16.0** is required (`Calix.Cli.Zig/build.zig.zon` declares
`minimum_zig_version = "0.16.0"`). Zig 0.17 changed the build API and this tree does not
build with it; Arch's `zig` package has moved past 0.16. The README explains how to
extract a 0.16.0 package into `vendor/zig-0.16` and put it first on `PATH`.

This project elevates through polkit (`pkexec`), not `sudo`, `su` or `doas`. Do not add
code paths that invoke those tools.

### Workflow

1. **Clone the repository:**

   ```bash
   git clone https://github.com/tension-atoi/calix.git
   cd calix
   ```

2. **Build the Zig backend** (produces `Calix.Cli.Zig/zig-out/bin/calix` and
   `Calix.Cli.Zig/zig-out/bin/calix-daemon`):

   ```bash
   make zig
   ```

3. **Start the daemon** in one terminal:

   ```bash
   ./Calix.Cli.Zig/zig-out/bin/calix-daemon
   ```

4. **Run the frontend** in another:

   ```bash
   make run
   ```

5. **Check compilation without linking:**

   ```bash
   make check
   ```

6. **Run tests:**

   ```bash
   make test
   (cd Calix.Cli.Zig && zig build test)
   (cd Calix.PackageManager && zig build test)
   ```

7. **Test the package build:**

   ```bash
   make package
   ```

   Do not use `makepkg -si`: its install step invokes `sudo` on a default
   `makepkg` configuration. Run `makepkg --noconfirm` and install the result
   with `pkexec pacman -U` instead.

---

## Code guidelines

- **Zero warnings.** Rust crates deny dead code, unused variables, unused imports and
  unused `must_use` values at the crate root. Do not silence warnings with `allow`
  attributes; remove the dead code or wire it up.
- **Keep I/O off the render thread.** Never run blocking process calls or network
  requests on the GPUI render thread. Use the Tokio worker (`ProcessRunner`) and GPUI's
  background executor.
- **Respect the protocol boundary.** The daemon's JSON-lines protocol and the inventory
  document are versioned. A breaking change needs a version bump and a compatibility
  test on both sides (Zig and Rust).
- **Keep the privilege boundary explicit.** The UI and daemon stay unprivileged. Root
  operations belong in the CLI path and go through polkit.
- **Idiomatic formatting.** Run `cargo fmt` for Rust; follow the surrounding Zig style.
- **Commit messages.** Use concise conventional messages (`feat:`, `fix:`, `docs:`,
  `refactor:`, `test:`, `chore:`).

---

## Submitting changes

1. Fork the repository and create a feature branch.
2. Make the change, keeping unrelated refactors out of the same commit.
3. Run the relevant checks (`make check`, `make test`, and the Zig tests for the
   component you touched).
4. Open a pull request against `main`. Describe the problem, the approach, and what you
   verified — including anything you could not verify.

If a change touches naming, packaging, the inventory schema, the daemon protocol or the
polkit policy, say so explicitly in the pull request. Those areas have transitional
constraints documented in [`docs/legacy-references.md`](docs/legacy-references.md).
