# CONTROL-01 Acceptance & Quality Verification

## Gate Verification Summary

| Gate | Requirement | Status |
|:---|:---|:---|
| **Compilation** | Clean release build, zero warnings | **PASS** |
| **Strict Clippy** | `cargo clippy --release --locked -- -D warnings` | **PASS (0 warnings)** |
| **Formatting** | `cargo fmt --check` | **PASS (clean)** |
| **Unit Test Suite** | 126 tests passing | **PASS (126/126)** |
| **Zero Deadcode** | `#![deny(dead_code)]`, strict YAGNI | **PASS** |
| **Zero `/tmp/` Ban** | No volatile `/tmp/` staging | **PASS** |
| **No ydotoold** | No synthetic keystroke injection | **PASS** |
| **Stop Rule** | No package mutating commands | **PASS** |

## Verified Invariants

1. **Protocol Stability**: Protocol version 1 guarantees schema compatibility between client and server.
2. **Exit Code Determinism**: Commands exit 0 on `response.ok == true` and 1 on failure across both human and `--json` invocations.
3. **Search Determinism**: `search <query>` synchronously writes `session.search_query` before ACK, guaranteeing read-your-writes consistency for subsequent `status` queries.
4. **Live Settings Single Authority**: Updates reject dirty drafts in GUI, persist atomically to disk, reload committed values, and propagate to all runtime effects including `cascade-delete` and `remove-configs`.
5. **Atomic Durability**: Configuration writes perform sibling temp write, fsync, atomic rename, parent directory fsync, and cleanup on failure. `reset_all` provides per-file atomic crash-safety.
6. **Kernel-Level Single Instance**: `flock(LOCK_EX | LOCK_NB)` on `instance.lock` eliminates startup TOCTOU races. Healthy sockets are never unlinked.
7. **Inspect Provenance Truth**: Package inspect resolves exact matches across store, updates, installed, and authoritative database with optional `source:name` prefix. Returns explicit errors for ambiguity and missing packages instead of fabricating ALPM keys.
8. **Cold-Start Command Authority Closure**: Cold-start invocations (`calix-gpui navigate/search/inspect`) bootstrap background GUI daemon (`--internal-gui`), wait boundedly on the control socket, and route uniformly through the socket command handler. Invalid cold-start invocations exit 1 with structured JSON errors and do not block the caller terminal.
