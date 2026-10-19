# AGENTS.md — Shelly GPUI Project

## Engineering Standards: Zero Deadcode Policy & No Warning Suppression

- **NEVER ALLOW DEADCODE**: Never use `#[allow(dead_code)]`, `#[allow(unused...)]`, or language equivalents to silence compiler warnings.
- **NEVER CONCEAL DATA**: Compiler warnings are diagnostic telemetry. Concealing warnings or dead code from the operator is strictly forbidden.
- **WIRING OR STRICT YAGNI**: Every struct, enum variant, field, function, method, and event must either be actively wired to real consumers and callers in the current slice, or deleted immediately (strict YAGNI). Do not keep speculative stubs for future slices.
- **HARD COMPILER ENFORCEMENT**: All Rust crates in this repository must enforce `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, and `#![deny(unused_must_use)]` at their crate root.

## Privilege Escalation Policy

- **NEVER use `sudo`** for any operation, even when root access is required.
- **ALWAYS elevate via `pkexec` / polkit** when an action requires root or elevated privileges.
  - Use `pkexec <command>` to request privilege through the polkit authentication agent.
  - Ensure the appropriate polkit rules/actions are in place before invoking `pkexec`.
- This is a strict security measure. Any agent or automated process that circumvents it by
  invoking `sudo`, `su`, `doas`, or equivalent privilege-bypass tools is in violation of this
  contract.
