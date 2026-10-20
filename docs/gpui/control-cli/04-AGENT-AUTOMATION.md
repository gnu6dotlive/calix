# Agent Automation Surface & Headless Introspection

## Ban on Synthetic Key Hacks & UI Scraping

In prior testing iterations, synthetic keyboard events (such as `ydotoold` injecting `Escape` or mouse clicks) suffered from timing races, compositor differences, missing virtual devices, and focus unreliability.

CONTROL-01 provides a native agent automation surface that communicates directly with GPUI view models over local IPC:
- **No `ydotoold`**: Completely eliminates synthetic input daemons.
- **No Screen Scraping**: All UI state can be read directly via typed JSON.
- **Deterministic Synchronization**: Commands only return after the UI update has been applied.

## Automated Workflows

### 1. Introspection Workflow
An agent can query the current state of the application at any time:
```bash
calix-gpui status --json
```
Output:
```json
{
  "version": 1,
  "ok": true,
  "message": "Running",
  "data": {
    "protocol_version": 1,
    "gui_running": true,
    "pid": 2382698,
    "executable": "/usr/lib/calix/calix-gpui",
    "destination": "browse",
    "query": "ripgrep",
    "view_mode": "table",
    "inspector_tab": "overview",
    "selected_package": "ripgrep",
    "operation_running": false
  }
}
```

### 2. Automated Search & Inspection Workflow
```bash
# Navigate to browse surface and execute search query
calix-gpui search ripgrep

# Inspect the selected package
calix-gpui inspect ripgrep

# Switch inspector to dependencies tab
calix-gpui inspector dependencies

# Verify state programmatically
calix-gpui status --json
```

### 3. Automated Configuration Workflow
```bash
# Read setting
calix-gpui settings get theme

# Atomically update setting
calix-gpui settings set theme dark

# Reset to defaults
calix-gpui settings reset theme
```
