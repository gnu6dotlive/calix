# Contributing to Shelly GPUI

Thank you for your interest in contributing to **Shelly GPUI**!  
We welcome bug reports, feature suggestions, documentation improvements, and code contributions.

---

## 🏛 Project Architecture

Shelly GPUI is built as a modular system separating high-performance GUI rendering from system transaction logic:

- **`Shelly.Ui.Gpui/` (Rust)**:
  - Frontend built on [GPUI 0.2](https://github.com/zed-industries/zed/tree/main/crates/gpui) (the GPU framework powering the Zed editor).
  - Handles vector GPU rendering via Vulkan/Wayland/X11, layout, keyboard events, and asynchronous UI state.
  - Implements the interactive live search with non-blocking debounce, collapsible log drawer, and rich package inspector.

- **`Shelly.Cli.Zig/` (Zig)**:
  - High-performance CLI backend interacting with `libalpm.so` (Arch Linux Package Management), AUR RPC, Flatpaks, and AppImages.
  - Generates structured JSON (`-j`) for queries and framed streams (`--ui-mode`) for live operation logging.

- **`packaging/`**:
  - Contains Arch Linux `PKGBUILD` scripts for local building and AUR deployment (`shelly-gpui-git`).

---

## 🛠 Local Development Setup

### Prerequisites

On Arch Linux or an Arch-based distribution:

```bash
sudo pacman -S --needed base-devel git rust cargo zig pacman libglvnd fontconfig freetype2 wayland mesa vulkan-icd-loader polkit
```

### Development Workflow

1. **Clone the repository:**
   ```bash
   git clone https://github.com/tension-atoi/calix.git
   cd shelly-gpui
   ```

2. **Run in development mode:**
   ```bash
   make run
   ```
   This compiles the Zig CLI backend automatically (if not already compiled) and executes `cargo run` pointing to the local backend.

3. **Check compilation without linking (fast check):**
   ```bash
   make check
   ```

4. **Build an optimized release binary:**
   ```bash
   make build
   ```

5. **Test the package build:**
   ```bash
   make package
   ```

---

## 📐 Code Guidelines & Quality Standards

- **Zero Compiler Warnings**: All Rust code must compile with **0 warnings** (`cargo check`). No `#[allow(dead_code)]` or quick hacks are accepted without an explicit, documented reason.
- **Idiomatic Rust**: Follow standard Rust formatting (`cargo fmt`) and clippy recommendations (`cargo clippy`).
- **Non-blocking UI**: Never run synchronous blocking I/O, heavy process calls, or network requests on the main GPUI render thread. Use `ProcessRunner::run_json_command` (Tokio background workers) and GPUI's `cx.spawn(...)` with `cx.background_executor().timer(...)`.
- **Consistent Visual Design**: All UI elements should follow the Catppuccin Mocha palette defined in `Shelly.Ui.Gpui/src/theme.rs`.
- **Commit Messages**: Write concise, conventional commit messages (e.g. `feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`).

---

## 📬 Submitting Changes

1. Fork the repository and create your feature branch:
   ```bash
   git checkout -b feature/my-new-feature
   ```
2. Commit your changes and ensure `make check` passes cleanly.
3. Push to your branch and open a Pull Request against `main`.
4. Provide a clear description of the problem solved or feature implemented.
