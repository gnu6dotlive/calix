# Control CLI — Command Surface Specification

The CLI surface supports global options and subcommands for window management, navigation, inspection, console logging, and configuration management.

## Global Options

- `-j`, `--json`: Emits machine-readable JSON for scripting, automated verification, and agent pipelines.
- `-h`, `--help`: Prints comprehensive help text detailing available commands and exits 0.
- `-V`, `--version`: Prints executable version and protocol version (`calix-gpui 0.2.0 (control protocol v4)`).

## Exit Code Semantics

The CLI guarantees strict exit code determinism across all invocations:
- **`0`**: The operation succeeded (`response.ok == true`).
- **`1`**: The operation failed (`response.ok == false`), the destination or inspector tab was invalid, the package was not found or ambiguous, or a settings validation or conflict error occurred. This exit code convention applies identically whether output is formatted for human terminals or `--json`.

## Runtime Commands

### `calix-gpui open`
- **Behavior**:
  - If the GUI is currently running: brings the existing window to focus via IPC and Wayland/Hyprland focus dispatcher.
  - If the GUI is not running: acquires the exclusive lifetime instance lock (`instance.lock`) and launches the desktop GUI. Secondary concurrent launches detect lock contention, forward intent over the control socket, and exit cleanly with code 0.
- **Output**:
  - Human: `Focused existing Calix window`
  - JSON: `{"version": 1, "ok": true, "message": "Calix window focused"}`

### `calix-gpui focus`
- **Behavior**:
  - If the GUI is running: focuses the window and exits 0.
  - If not running: reports error and exits with code 1.

### `calix-gpui status [--json]`
- **Behavior**:
  - Reports current operational status of the frontend.
- **Fields Reported**:
  - `protocol_version`: `1`
  - `gui_running`: boolean (`true` or `false`)
  - `pid`: process identifier (`0` if offline)
  - `executable`: binary path
  - `destination`: active navigation view (`updates`, `installed`, `browse`, `news`, `settings`)
  - `query`: active search query string
  - `view_mode`: `table` or `cards`
  - `inspector_tab`: `overview`, `dependencies`, or `files`
  - `selected_package`: package name currently displayed in inspector (or `null`)
  - `operation_running`: boolean indicating active background task or search in flight

### `calix-gpui quit`
- **Behavior**:
  - Gracefully terminates running GUI instance, cleans up Unix domain socket, and exits 0.
  - If already offline, reports error and exits 1.

### `calix-gpui navigate <destination>`
- **Arguments**: `updates`, `installed`, `browse`, `news`, `settings`.
- **Behavior**:
  - If destination is valid: navigates running view, updates state, and exits 0.
  - If destination is invalid: returns an explicit error and exits 1.
  - If offline: bootstraps the desktop GUI in the background, waits boundedly for socket readiness, sends the navigate command over the control socket, and exits deterministically (0 on valid destination, 1 on invalid).

### `calix-gpui search <query>`
- **Behavior**:
  - Synchronously updates `session.search_query = query` and search input text before returning the ACK response, ensuring immediate read-your-writes consistency for subsequent status queries.
  - Sets destination to Browse and dispatches debounced background backend search execution.
  - If offline: bootstraps the desktop GUI in the background, waits boundedly for socket readiness, executes the search over the control socket, and returns the response.

### `calix-gpui view <table|cards>`
- **Behavior**:
  - Validates argument. Rejects invalid modes with exit code 1.
  - Switches workstation presentation mode between tabular rows and card tiles.
  - Persists preference in `gpui-ui.json` and updates live runtime state atomically.
  - Works both online (live UI update + save) and offline (atomic save).

### `calix-gpui inspect <package>`
- **Behavior**:
  - Resolves package provenance accurately without fabricating fallback keys:
    - Supports optional explicit source prefix: `alpm:ripgrep`, `aur:yay`, `flatpak:org.mozilla.firefox`, `appimage:obsidian`.
    - Searches loaded active results, installed packages, updates, and search cache for exact name matches.
    - If not yet loaded and source is standard ALPM, queries authoritative ALPM backend synchronously.
  - **Ambiguity**: If multiple packages with different distribution sources match an un-prefixed name, returns an error detailing candidates and exits 1.
  - **Not Found**: If no package matches, returns an error and exits 1.
  - If exactly one package matches: selects package key in `AppSession`, triggers inspector load, and exits 0.
  - If offline: bootstraps the desktop GUI in the background, waits boundedly for socket readiness, dispatches the inspect command over the control socket, and returns the response with exit code 0 or 1.

### `calix-gpui inspector <tab>`
- **Arguments**: `overview`, `dependencies` (or `deps`), `files` (or `files-build`).
- **Behavior**:
  - If tab name is valid: switches active inspector tab and exits 0.
  - If tab name is invalid: returns error and exits 1.

### `calix-gpui logs <show|hide|clear>`
- **Behavior**:
  - `show`: opens bottom operation drawer and persists state.
  - `hide`: closes bottom operation drawer and persists state.
  - `clear`: clears in-memory log buffer and resets progress bar.
  - Invalid operations return error and exit 1.
