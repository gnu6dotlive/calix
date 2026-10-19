# ⚡ Calix GPUI

<div align="center">

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Zig](https://img.shields.io/badge/Zig-0.13%20%7C%200.14%20%7C%200.16-f7a41d.svg)](https://ziglang.org/)
[![GPUI](https://img.shields.io/badge/UI-GPUI%200.2-blueviolet.svg)](https://github.com/zed-industries/zed/tree/main/crates/gpui)
[![Arch Linux](https://img.shields.io/badge/Arch%20Linux-ALPM%20%2B%20AUR-1793d1.svg)](https://archlinux.org/)

**High-performance GPU-accelerated unified package manager for Arch Linux.**  
*Official repositories (ALPM), AUR, Flatpaks, and AppImages brought together in a fast, keyboard-first interface powered by GPUI.*

[Features](#-features) •
[Architecture](#-architecture) •
[Installation & Building](#-installation--building) •
[Keybindings & Usage](#-keybindings--usage) •
[Arch Linux Packaging (AUR)](#-arch-linux-packaging-aur) •
[Credits](#-credits)

</div>

---

## 🚀 Features

- ⚡ **Native Hardware GPU Acceleration**: Built on Zed's **GPUI 0.2** framework, rendering directly via Vulkan/Wayland/X11 with Blade Graphics at 120+ FPS.
- 📦 **Multi-Backend Package Management**:
  - **Official Arch Linux Repositories** (via `libalpm` engine)
  - **Arch User Repository (AUR)** (Git cloning, clean chroot builds)
  - **Flatpak Applications** (Flathub catalogs, runtimes, and metadata)
  - **AppImage Management** (directory indexing and lifecycle management)
- 🔍 **Interactive Live Search (`TextInput`)**: Non-blocking typing with 300 ms predictive debounce powered by GPUI's native background executor.
- 📋 **Rich Package Inspector**: Complete view of dependencies, optional dependencies, reverse dependencies ("Required by"), base packages, install reasons, licenses, and verified package sizes.
- 📟 **Embedded Terminal Log Drawer**: Collapsible lower drawer streaming live stdout/stderr transaction logs with Polkit privilege escalation.
- 🔄 **Automatic State Refresh**: Instant synchronization of update counters and active listings upon successful package installations, removals, or upgrades.
- 🎨 **Modern Catppuccin Mocha Theme**: Curated typography, high-contrast indicators, and distinctive source badges.

---

## 🏗 Architecture

Shelly GPUI enforces a strict separation of concerns between reactive GUI rendering and system transaction management:

```mermaid
graph TD
    subgraph Frontend ["Shelly.Ui.Gpui (Rust)"]
        GPUI["GPUI / Blade Renderer"] --> Workspace["WorkspaceView & Panes"]
        Workspace --> LiveSearch["Live Search / Debounce 300ms"]
        Workspace --> LogDrawer["Terminal Log Drawer"]
        LogDrawer --> Stream["Tokio Async Streaming Process"]
    end

    subgraph Backend ["Shelly.Cli.Zig (Zig)"]
        Stream -->|shelly --ui-mode -j| CLI["Shelly CLI Engine"]
        CLI --> ALPM["libalpm / Pacman Engine"]
        CLI --> AUR["AUR RPC / Git Builder"]
        CLI --> FP["Flatpak Engine"]
        CLI --> AI["AppImage Manager"]
        CLI --> Polkit["Polkit / pkexec Auth"]
    end
```

- **Frontend (`Shelly.Ui.Gpui`)**: Native Rust application written with GPUI 0.2, managing vector GPU rendering, layout calculations, and reactive UI state.
- **Backend (`Shelly.Cli.Zig`)**: Native Zig CLI interacting directly with `libalpm.so` and external package sources. It outputs structured JSON for queries and framed streams via `--ui-mode` for live transaction logging.

---

## 🛠 Installation & Building

### Prerequisites

On Arch Linux or Arch-based distributions:

```bash
sudo pacman -S --needed base-devel git rust cargo zig pacman libglvnd fontconfig freetype2 wayland mesa vulkan-icd-loader polkit
```

### 1. Build and Run with `make`

The repository provides a root `Makefile` that automatically compiles the Zig backend and launches the GPUI frontend:

```bash
git clone https://github.com/tension-atoi/calix.git
cd calix

# Launch directly in development mode
make run

# Or build an optimized release binary
make build
```

Available `make` targets:
- `make run`: Compiles the Zig backend (if needed) and starts `cargo run` with the local CLI.
- `make build`: Generates the optimized release binary (`calix-gpui`).
- `make check`: Runs fast static compilation checks without linking.
- `make install`: Installs the binaries and desktop integration files into `/usr/local`.
- `make package`: Builds and tests the Arch Linux package via `makepkg`.
- `make clean`: Cleans build artifacts for both Rust and Zig.

---

## ⌨ Keybindings & Usage

| Key / Action | Context | Description |
|---|---|---|
| **Click Search Bar** | Left Pane | Focuses the search input with visual cursor `▌` |
| **Type Query** | Search Bar | Triggers live search with automatic 300 ms debounce |
| **Enter** | Search Bar | Executes search immediately (bypasses debounce) |
| **Escape** | Search Bar | Clears query and releases keyboard focus |
| **Click Bottom Drawer** | Global | Expands or collapses the terminal log drawer |
| **Tab Buttons** | Top Navigation | Switches between ALPM, AUR, Flatpaks, AppImages, Updates, News, and Settings |

---

## 📦 Arch Linux Packaging (AUR)

The [`PKGBUILD-calix`](PKGBUILD-calix) is provided for clean Arch Linux packaging, and
[`packaging/aur/PKGBUILD`](packaging/aur/PKGBUILD) is a symlink to it so the AUR copy can never drift
from the in-tree one:

```bash
# Build the package locally
makepkg -si --noconfirm
```

The package installs:
- `/usr/bin/calix-gpui`: System wrapper script with automated environment discovery
- `/usr/lib/shelly/shelly`: Native Zig CLI engine
- `/usr/lib/calix/calix-gpui`: GPU-accelerated Rust frontend binary
- `/usr/share/applications/com.calix.calix-gpui.desktop`: XDG desktop application entry
- `/usr/share/icons/hicolor/scalable/apps/calix-gpui.svg`: Scalable Catppuccin vector icon
- `/usr/share/polkit-1/actions/com.calix.calix-gpui.policy`: Polkit authorization rules for seamless system operations

---

## 🤝 Credits & Acknowledgments

- [Seafoam-Labs/Shelly-ALPM](https://github.com/Seafoam-Labs/Shelly-ALPM) for the original Shelly package manager engine and ALPM / Zig architecture.
- [Zed Industries](https://github.com/zed-industries/zed) for the [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) framework.
- [Catppuccin](https://github.com/catppuccin/catppuccin) for the Mocha palette.

---

## 📄 License

This project is licensed under the **GPL-3.0 License**. See the [LICENSE](LICENSE) file for details.
