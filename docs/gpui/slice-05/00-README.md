# Shelly GPUI — Slice-05 Documentation Kit

## Target Slice

**SLICE-05 — Visual System, Desktop Adaptation, Settings IA & Product Finishing**

This slice builds upon the Slice-04 interaction and motion layer, executing a comprehensive product finishing pass across the entire desktop experience. It transforms Shelly GPUI from a functional workstation prototype into an integrated, production-grade Linux desktop package manager.

### Core Architectural Deliverables

1. **Unified Density & UiMetrics Invariants (`ui_metrics.rs`)**:
   - Centralized layout metric constants: card wrapper/content heights (Normal: 78/72px, Compact: 68/62px), table row heights (Normal: 36px, Compact: 32px), splitter width (5px), and sidebar expanded/collapsed bounds (190px / 56px).
   - Invariant-guaranteed dynamic splitter calculation: enforces `INSPECTOR_MIN_WIDTH = 320.0px` across reference viewports (1024x680, 1280x840, 1600x1000).
   - Cohesive `compact_view` density propagation across package cards, table rows, and navigation drawer.

2. **Compile-Time Embedded SVG Asset Source (`icons.rs`)**:
   - 22 custom vector SVG icons embedded into the binary via `include_bytes!`: Browse, Search, Installed, Updates, News, Settings, Overview, Dependencies, FilesBuild, Cards, Table, Copy, ExternalUrl, FilePath, Collapse, Expand, Trash, Shelly, Check, Close, Info, Warning.
   - Typed `AppIcon` enum with `AppIcon::ALL` coverage and runtime-independent `AppIcons: AssetSource` implementation.
   - Complete elimination of emoji glyphs and unicode lightning symbols in UI chrome, replacing them with theme-colored vector SVGs.

3. **Complete English Copy & Settings IA Overhaul (`views/settings.rs`)**:
   - Translation of all user-facing strings from French to English across news, console, log drawer, settings, and workspace notifications.
   - Restructured `SettingsView` into exactly 5 logical, honest sections:
     1. **Package Sources** (AUR, Flatpak, AppImage toggles)
     2. **Package Management** (Cascade deletion, configuration removal)
     3. **Appearance & Density** (Dark theme, Compact density)
     4. **Motion & Feedback** (Reduce motion toggle)
     5. **Logs & Operations** (Auto-open execution drawer)
   - Removal of ghost/unbacked options (`shelly_search_enabled`, `no_confirm`).
   - Introduction of clean "Reset Changes" action and visual dirty-state indicator.

4. **Sanitized Configuration Snapshot & Window Geometry Sampling (`config.rs`, `main.rs`)**:
   - Single sanitized configuration snapshot loaded once at application launch.
   - Hardened `GpuiUiConfig` deserialization with `#[serde(default)]` on all fields.
   - Geometry sanitization ensuring window dimensions are strictly $\ge 1024 \times 680$ px.
   - Discrete window bounds sampling upon explicit "Save Settings" via `window.window_bounds()`, eliminating continuous resize polling.
   - `last_selected_tab` persistence derived from `last_workspace_destination`, never stranding the user on the Settings screen across app restarts.

5. **Active Source Filtering & Real Package Management Flags (`backend/client.rs`, `views/workspace.rs`)**:
   - `aur_enabled`, `flatpak_enabled`, and `appimage_enabled` are actively wired: disabled sources are omitted from unified search queries and hidden from filter bars.
   - Source reconciliation on Save: if active filter source is disabled, automatically falls back to `SourceFilter::All`.
   - `package_management_cascade_delete` and `package_management_remove_configs` wire directly to `ShellyClient::remove_package` CLI flags (`--cascade`, `--remove-config`).

6. **Single Busy Execution Authority (`state/console.rs`)**:
   - `ConsoleModel::is_running()` serves as the sole authoritative signal for package mutation progress.
   - Redundant `is_mutating` booleans eliminated from workspace state.
   - Package workstation subscribes strictly to `OperationStarted` and `OperationFinished` lifecycle events, avoiding root reflows on log output.

7. **Desktop Integration & Strict Packaging (`assets/`, `PKGBUILD-gpui`)**:
   - Standards-compliant `.desktop` file with English default strings and bilingual French translations.
   - Strict `PKGBUILD-gpui` with `cargo fetch --locked`, `cargo build --release --locked`, and `cargo test --release --locked` enforced without error suppression or fallback chains.
   - Interactive news feed linking directly to Arch Linux official announcements via `cx.open_url`.

8. **Strict Compiler Enforcement & Zero-Deadcode Architecture**:
   - `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, `#![deny(unused_must_use)]` strictly enforced.
   - 70 unit tests passing with zero failures and zero linter warnings under `cargo clippy -- -D warnings`.

---

## Repository Baseline

Repository:

    gnu6dotlive/calix

Canonical branch:

    main

Pinned Frontend Dependency:

    gpui = "0.2.2"

Upstream CLI Authority:

    Seafoam-Labs/Shelly-ALPM (Shelly.Cli.Zig)

---

## Documentation Kit Map

- [`00-README.md`](00-README.md): High-level overview, deliverables summary, and repository baseline.
- [`01-CURRENT-STATE.md`](01-CURRENT-STATE.md): Component inventory, module structure, and state architecture.
- [`02-WORK-CONTRACT.md`](02-WORK-CONTRACT.md): Contractual obligations, invariant boundaries, and non-regression guarantees.
- [`03-VISUAL-SYSTEM.md`](03-VISUAL-SYSTEM.md): Typography, color palettes, 22 SVG icons, and density metrics.
- [`04-INTERACTION-AND-FOCUS.md`](04-INTERACTION-AND-FOCUS.md): Keyboard navigation, focus rings, interactive controls, and URL linking.
- [`05-CONFIG-AND-DESKTOP-GEOMETRY.md`](05-CONFIG-AND-DESKTOP-GEOMETRY.md): Settings persistence, geometry sanitization, and tab reconciliation.
- [`06-PACKAGING-AND-ASSETS.md`](06-PACKAGING-AND-ASSETS.md): Asset embedding, .desktop metadata, and PKGBUILD verification.
- [`07-ACCEPTANCE-MATRIX.md`](07-ACCEPTANCE-MATRIX.md): Detailed verification matrix mapping requirements to empirical proofs.
- [`08-ACCEPTANCE-REPORT.md`](08-ACCEPTANCE-REPORT.md): Final acceptance report with full test suite logs and quality certification.
