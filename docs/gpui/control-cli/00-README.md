# Calix GPUI — Native Frontend Control CLI & Agent Automation Surface

## Architecture Overview

Phase **CONTROL-01** provides a unified, production-grade CLI control interface and agent automation surface for the `calix-gpui` desktop frontend.

Prior to CONTROL-01, inspecting frontend state or automating UI operations required either interactive manual clicking or brittle GUI-level synthetic key injection (e.g. `ydotoold`). CONTROL-01 introduces a **native typed control protocol** and a **settings authority** directly embedded in the single `calix-gpui` executable.

```
                           +----------------------------------------+
                           |  calix-gpui [OPTIONS] [SUBCOMMAND]     |
                           +----------------------------------------+
                                              |
                              Pre-GPUI Argument Parsing (<100µs)
                                              |
                  +---------------------------+---------------------------+
                  |                                                       |
        Is instance running?                                     Is instance running?
             [ YES ]                                                   [ NO ]
                  |                                                       |
   Connect to Unix Socket:                                   Handle command offline:
   $XDG_RUNTIME_DIR/calix-gpui/control.sock                 - Settings: atomic disk config
   Send ControlRequest (v1 JSON)                             - Open / Navigate / Search / Inspect:
   Receive ControlResponse (v1 JSON)                           boot GPUI with StartupIntent
   Format Output (human or --json)                           - Focus / Quit / Runtime Inspector:
   Exit 0                                                      clean exit with diagnostic
```

## Key Architectural Principles

1. **Single-Binary Entrypoint**:
   - `calix-gpui` acts as both the full GPUI desktop GUI application and a sub-millisecond CLI control tool.
   - Arguments are evaluated before GPUI or Wayland subsystems initialize.
   - Non-interactive and query operations execute in under 2ms without spinning up a graphical context.

2. **Single-Instance Enforcement**:
   - Calling `calix-gpui` or `calix-gpui open` when an existing instance is already running brings that window into focus rather than spawning a second duplicate GUI process.
   - Stale socket files from terminated or crashed instances are automatically detected and safely removed.

3. **Zero Deadcode & Strict Safety**:
   - Developed under `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, and `#![deny(unused_must_use)]`.
   - Strictly banishes synthetic keypresses (`ydotoold`), UI scraping, and volatile `/tmp/` staging.

4. **Atomic Configuration Writes**:
   - All disk mutations to `~/.config/calix/settings.json` and `~/.config/calix/gpui-ui.json` use atomic write semantics: temporary sibling files in the same directory (`.{filename}.tmp.{pid}.{nanos}`), explicit `fsync` of file data and parent directory, followed by atomic rename.

5. **Stop Rule Adherence**:
   - As mandated by the operator, package mutating actions (`action install/remove/upgrade-all`) are strictly deferred until V1 is reviewed and ratified.
