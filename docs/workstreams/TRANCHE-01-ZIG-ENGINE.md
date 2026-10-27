# 🏺 Calix Workstream — Tranche 1: Native Zig Engine & Systemd Pipeline

> **Document Type:** Autonomous Agent Handoff Specification  
> **Repository:** `~/Projects/calix`  
> **Status:** APPROVED / IN PROGRESS  
> **Ratified Date:** 2026-10-27  
> **Target Release:** Calix v0.4.0  

---

## 1. Context & Operational Baseline

On 2026-10-27, the host workstation formally completed the transition to Calix:
1. **Shelly Retirement:** Upstream `/usr/bin/shelly` (CachyOS ALPM package) and legacy `shelly-gpui-git` were uninstalled via `pkexec pacman -Rns`. Calix is ratified as the single desktop and CLI package management surface on this system.
2. **Catalog Deduplication:** AppImage metadata was standardized to reverse-DNS XDG desktop entries (`ai.storyteller.<app>.desktop`) in `~/.local/share/applications/`, and all redundant bare duplicates (`<app>.desktop`) and orphaned entries were purged.
3. **Current State:** The system is temporarily synchronized using the Python bridge (`scripts/calix-sync.py`), which polls and writes state snapshots to `~/.local/state/calix/*.json` and `~/.config/calix/appimage-metadata-v2.db`.

This workstream specifies the full migration of the transitional Python sync pipeline into the native Zig backend (`Calix.Cli.Zig`).

---

## 2. Workstream Objective

**Primary Goal:** Replace `scripts/calix-sync.py` with native Zig implementations inside `Calix.Cli.Zig`, producing an atomic, self-healing, zero-Python package discovery and synchronization engine that runs synchronously or as a lightweight systemd user service.

### Scope of Migration
1. **Native AppImage Discovery:** Scan `~/Applications/` directly in Zig, extract metadata from `~/.local/share/applications/*.desktop` (supporting `X-AppImage-Version`, `X-Craft-Version`, and reverse-DNS entries), and persist into `appimage-metadata-v2.db`.
2. **Native Cargo & NPM Discovery:** Ingest `~/.cargo/.crates2.json` and parse global node modules from `/usr/lib/node_modules/`.
3. **Atomic State Publisher:** Write JSON state files directly to `~/.local/state/calix/*.json` via atomic rename (`write to .tmp` -> `rename()`), ensuring zero read tears for the GPUI frontend (`Calix.Ui.Gpui`).
4. **Polkit Integration:** Implement a native, non-interactive polkit invocation helper for package installation, upgrades, and purges using `pkexec` (never `sudo`).

---

## 3. Strict Invariants & Anti-Goals

> [!CAUTION]
> Any agent working on this repository MUST strictly follow these anti-goals. Failure to comply constitutes a violation of the workstation contract.

* **ANTI-GOAL 1 (No Shell String Execution):** Never construct shell command strings or invoke `/bin/sh -c` for package commands. All subprocess invocations must use typed, fixed argv arrays (`std.process.Child`).
* **ANTI-GOAL 2 (Zero Sudo):** Never invoke `sudo`, `su`, or `doas`. All privilege escalation must pass through `pkexec` configured with the `live.gnu6.calix` polkit action.
* **ANTI-GOAL 3 (Zero Schema Regression):** Never break or alter the JSON output schemas consumed by `Calix.Ui.Gpui` (`status.json`, `pacman.json`, `appimages.json`, `cargo.json`, `npm.json`). The frontend depends on exact key parity.
* **ANTI-GOAL 4 (No ALPM Reimplementation in Zig):** Do not attempt to write an ALPM database parser from scratch in Zig. Continue delegating ALPM operations through libalpm C ABI bindings or pacman subprocess queries.
* **ANTI-GOAL 5 (No Snap Support):** Snap packages are explicitly barred by workstation policy. Do not add Snap discovery or backend logic.

---

## 4. Architectural Implementation Blueprint

```
┌────────────────────────────────────────────────────────┐
│                   Calix.Ui.Gpui (Rust)                 │
│         Reads ~/.local/state/calix/*.json              │
└───────────────────────────▲────────────────────────────┘
                            │ (Atomic File Reads)
┌───────────────────────────┴────────────────────────────┐
│              Calix.Cli.Zig (Native Engine)             │
│                                                        │
│  ┌────────────────────┐      ┌──────────────────────┐  │
│  │ AppImage Scanner   │      │ Cargo / NPM Scanner  │  │
│  │ ~/Applications     │      │ ~/.cargo/.crates2    │  │
│  │ XDG Desktop parser │      │ /usr/lib/node_module │  │
│  └─────────┬──────────┘      └──────────┬───────────┘  │
│            │                            │              │
│  ┌─────────▼────────────────────────────▼───────────┐  │
│  │              State Publisher (Atomic)            │  │
│  │       ~/.local/state/calix/status.json.tmp       │  │
│  │                        ↓                         │  │
│  │       ~/.local/state/calix/status.json           │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

### Module Responsibilities in `Calix.Cli.Zig`

* `src/backends/appimage.zig`:
  * Direct directory iteration of `~/Applications/` using `std.fs.Dir.iterate()`.
  * Desktop entry harvesting from `~/.local/share/applications/` matching the AppImage path in `Exec=`.
  * Support for reverse-DNS naming (`ai.storyteller.<app>.desktop`) and `X-AppImage-Version` extraction.
* `src/backends/cargo.zig`:
  * Direct JSON deserialization of `~/.cargo/.crates2.json` into typed structs (`installs: map[String]CrateMeta`).
* `src/backends/npm.zig`:
  * Directory traversal of `/usr/lib/node_modules/` reading `package.json` names and versions.
* `src/sync/publisher.zig`:
  * Serializes unified snapshot into temporary files and performs `std.fs.rename` to replace state files atomically.

---

## 5. Milestone & Verification Sequence

### Phase 1: Zig AppImage & Cargo Harvester
* **Task:** Implement `appimage.zig` and `cargo.zig` with standalone CLI commands `calix-zig list-appimages` and `calix-zig list-cargo`.
* **Gate:** Output must match byte-for-byte or key-for-key the output of `scripts/calix-sync.py` on the live workstation.

### Phase 2: Atomic State Synchronization
* **Task:** Implement `calix-zig sync --runtime` replacing `python3 scripts/calix-sync.py --runtime`.
* **Gate:** `Calix.Ui.Gpui` must start, render the Installed table with all 15 AppImages and 20 Cargo crates, and show zero layout errors or missing version fields.

### Phase 3: Polkit Elevation & Package Action Wiring
* **Task:** Bind `update` and `remove` commands to the `pkexec` wrapper with pre-flight Snapper snapshot validation.
* **Gate:** A test update dry-run executes via polkit without environment bleed or unhandled errors.

### Phase 4: Deprecation of Python Bridge
* **Task:** Remove `scripts/calix-sync.py`, update systemd user units in `packaging/systemd/` to invoke the Zig binary, and run a complete qualification pass.

---

## 6. Handoff Checklist for Autonomous Agents

Before opening a PR or submitting a run record:
- [ ] Run `zig build test` — all unit tests pass green.
- [ ] Run `cargo test` in `Calix.Ui.Gpui` — all 265 UI layout and state tests pass green.
- [ ] Run `calix health` — confirms DB lock is free and states are synchronized.
- [ ] Verify no files in `/tmp` survive the test run (work belongs in `.ci-cache/`).
- [ ] Record qualification evidence in `evidence/runs/calix-tranche-01-<date>/run.json`.
