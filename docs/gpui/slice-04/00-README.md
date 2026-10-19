# Shelly GPUI — Slice-04 Documentation Kit

## Target Slice

**SLICE-04 — Motion, Interaction Polish, Feedback & High-Frequency UI Isolation**

This slice follows the Slice-03 workstation foundation (`PackageViewMode::Table`, `PackageViewMode::Cards`, decomposed 3-tab inspector, `DependencyRef`, capability derivation) and delivers a high-polish, GPU-accelerated interaction layer.

It purposefully delivers:

1. **Pure Settings View & Draft State (`SettingsView`)**:
   - Complete removal of render-time disk mutations (`save_*` calls during rendering).
   - Pure, in-memory draft configuration state (`SettingsDraft`) with dirty tracking (`is_dirty()`).
   - Explicit "Save Settings" dispatch with persistent storage via `ConfigManager` and immediate toast feedback.
   - Clean save button is strictly inert (no cursor pointer, no hover style, no click listener).
2. **Backward-Compatible Motion Configuration**:
   - `reduce_motion: bool` integrated into `GpuiUiConfig` with `#[serde(default)]` and robust JSON compatibility.
   - Live synchronization across all open views immediately upon toggle.
3. **Formalized Motion System & Primitives (`state::motion`)**:
   - Standardized motion token durations: `FAST = 120ms`, `STANDARD = 160ms`, `EMPHASIS = 220ms`.
   - Continuous, interruptible, reversible transitions via `AnimatedScalar` with quintic (`ease_out_quint`) and piecewise quadratic (`ease_in_out`) easings.
   - Discrete element transitions: 120ms entrance opacity fade (`0.0 -> 1.0`) for destinations and inspector tabs (`with_animation`), with stationary hero header.
4. **High-Frequency UI Invalidation Isolation**:
   - Encapsulation of list pane, splitter, and inspector into a dedicated `PackageWorkstationView`.
   - Pure pointer-delta drag math: `(start_width + (current_x - start_x)).clamp(280.0, 700.0)`.
   - Zero-copy package list rendering with `Arc<[UnifiedPackage]>` slices in `PackageStore`.
   - Splitter dragging strictly invalidates only the child workstation, protecting root `WorkspaceView` and sidebar from layout reflows.
5. **Continuous Frame Ownership Isolation**:
   - `SidebarView` (`views/sidebar.rs`) encapsulates sidebar width transition (56px ↔ 190px) and drives its own `request_animation_frame()` loop exclusively during active motion.
   - `OperationConsoleView` (`views/operation_console.rs`) encapsulates console drawer disclosure (0px ↔ 220px) and drives its own local frame loop.
   - `WorkspaceView::render` performs **zero** continuous frame requests.
6. **Bounded Toast Center & Feedback Overlay (`ToastCenter` & `ToastOverlay`)**:
   - Monotonic IDs, bounded maximum of 3 concurrent visible notifications.
   - Deterministic eviction preferentially preserving error notifications when non-error items exist.
   - Race-free lifecycle (`transition_lifecycle`) strictly enforcing forward-only progression (`Entering -> Visible`, `Entering -> Exiting`, `Visible -> Exiting`).
   - 120ms entering fade-in, 3500ms visible steady state, and 120ms exiting fade-out via GPUI `with_animation`.
7. **Console State Single Authority & Log Isolation**:
   - `ConsoleModel.is_open` is the sole authority for log drawer disclosure; `OperationConsoleView` height strictly derives from `ConsoleEvent::Toggled`.
   - "Ouvrir automatiquement le tiroir de logs lors d'une action" (`log_drawer_open`) is honored on operation start and immediately synchronized.
   - Redundant log-stream subscriptions eliminated from `WorkspaceView`, protecting root from continuous invalidation during subprocess runs.
8. **Strict Compiler Enforcement & Zero Scaffolding**:
   - Hard `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, `#![deny(unused_must_use)]`.
   - Zero test hooks or screenshot environment variables in production code.
   - 0 warnings, pinned GPUI 0.2.2 compatibility, 52 passing unit tests.

---

## Repository Baseline

Repository:

    tension-atoi/calix

Canonical branch:

    main

Frontend dependency:

    gpui = "0.2.2"

Functional upstream authority:

    Seafoam-Labs/Shelly-ALPM

---

## Documentation Kit Map

- [`00-README.md`](00-README.md): High-level slice scope, primitives, and repository baseline.
- [`01-CURRENT-STATE.md`](01-CURRENT-STATE.md): Architectural snapshot and component inventory with continuous frame isolation.
- [`02-WORK-CONTRACT.md`](02-WORK-CONTRACT.md): Detailed work contract commitments, constraints, and scope boundaries.
- [`03-MOTION-ARCHITECTURE.md`](03-MOTION-ARCHITECTURE.md): Mathematical motion architecture, piecewise quadratic easing curves, and duration tokens.
- [`04-FEEDBACK-MODEL.md`](04-FEEDBACK-MODEL.md): Notification overlay system, GPUI animation lifecycle, search telemetry, and dirty-state feedback.
- [`05-PERFORMANCE-EVIDENCE.md`](05-PERFORMANCE-EVIDENCE.md): Pointer drag isolation, zero-copy Arc slices, isolated RAF loops, and zero scaffolding audit.
- [`06-ACCEPTANCE-MATRIX.md`](06-ACCEPTANCE-MATRIX.md): Comprehensive verification matrix mapping requirements to evidence classes and empirical proofs.
- [`07-ACCEPTANCE-REPORT.md`](07-ACCEPTANCE-REPORT.md): Final acceptance signoff with 52 automated test results and native Wayland framebuffer evidence.
