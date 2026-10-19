# Shelly GPUI — Slice-06 Work Contract

## Objective

Deliver **SLICE-06: Search Correctness, Query Workbench & Privileged Operations** in `tension-atoi/calix` according to the Operator ratification plan.

---

## Deliverables & Acceptance Requirements

### 1. Privilege Elevation under UI Mode (P0-A)
- When `--ui-mode` is passed to `Shelly.Cli.Zig` for non-root ALPM operations (`install standard`, `remove standard`, `upgrade all`), the CLI automatically invokes `pkexec` elevation when UID is not root.
- User-scoped targets (`flatpak install --user`, `appimage`) do not invoke elevation.
- Elevation errors are surfaced as structured `AlpmError` frames to GPUI.

### 2. Structured Protocol & Semantic Timeline (P0-B, P0-C)
- Create `Shelly.Ui.Gpui/src/backend/protocol.rs` implementing resilient framing and decoding of `[JSON]<base64>[/JSON]`.
- Strongly typed `UiFrame`: `AlpmInfo`, `AlpmError`, `OperationProgress`, `TransactionEvent`.
- `ConsoleModel` maintains an operation timeline of phase statuses (`Pending`, `InProgress`, `Success`, `Failed`) for ALPM, AUR, Flatpak, AppImage.
- Clean technical detail log view and clipboard copy preserving complete log streams.

### 3. Canonical Interactive Search Input (P1, P2, P3)
- Replace synthetic input with `Entity<SearchInputView>` implementing `EntityInputHandler` and `ElementInputHandler`.
- Support mouse-click caret placement, mouse drag text selection, I-beam cursor.
- Support Backspace, Delete, Left/Right/Home/End, Shift selection extension, Ctrl+A, Escape clear.
- Provide `[×]` clear button when non-empty, retaining focus upon clearing.
- Provide subtle search activity pulsing indicator respecting `reduce_motion`.

### 4. 2-Row Query Workbench (P4, P5, P11, P15, P16, P17)
- Row 1: Search input ($40\text{px}$).
- Row 2: Query controls ($34\text{px}$): Source pills/dropdown, State filter, Sort mode selector, View mode switcher.
- Responsive breakpoints for list pane width:
  - Wide ($\ge 540\text{px}$): Full source pills, state button, sort button, view switcher, filter summary pill.
  - Medium ($380\text{px} \le w < 540\text{px}$): Compact pill labels (`All`, `ALPM`, `AUR`, `FP`, `AI`).
  - Narrow ($< 380\text{px}$): Compact cycling source button, zero text clipping.
- One-click "Reset Filters" action.

### 5. Client-Side Sorting & State Filters (P6, P7, P8, P9)
- `SortMode`: `Relevance`, `NameAsc`, `NameDesc`, `Source`, `InstalledFirst`, `UpdatesFirst` with deterministic secondary/tertiary tie-breakers.
- `PackageStateFilter`: `All`, `Installed`, `NotInstalled`, `UpdatesAvailable`.
- 100% client-side execution with zero backend re-querying.

### 6. Dual Usable Geometry Invariant (P10)
- `LIST_MIN_USABLE = 340.0px`, `INSPECTOR_MIN_USABLE = 320.0px`.
- Splitter clamping ensures both panes maintain usable minimum widths at all viewport sizes.

### 7. Search Generation Poisoning Fix & Concurrency (P12, P13, P14)
- Remove `usize::MAX`; use monotonic `next_search_generation()` on clear.
- Discard any asynchronous response with generation $< \text{current\_generation}$.
- Multi-source search executes concurrently via `tokio::join!`.
- Strict deserialization: malformed JSON returns typed error context instead of silent empty vectors.

---

## Strict Engineering Standards

- **Zero Warning Suppressions**: No `#[allow(...)]` anywhere in Rust codebase.
- **Zero Deadcode**: Every struct, enum variant, field, function, and method must be actively wired or tested.
- **Hard Compiler Enforcement**: `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, `#![deny(unused_must_use)]`.
- **Packaging Gate**: Build native package from exact remote SHA, install via `pacman -U`, verify running PID under Wayland (`/proc/<pid>/exe -> /usr/lib/shelly/shelly-gpui-bin`).
