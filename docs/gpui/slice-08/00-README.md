# Calix GPUI — Slice-08 Documentation Kit

## Target Slice

**SLICE-08 — Information Architecture Implementation**

The contract published in [`../slice-07/`](../slice-07/) is implemented here. The
slice changes what the application says about where it is, and what it lets the
user do from every surface.

---

## What the slice delivers

| Requirement | What it is | Where |
|---|---|---|
| The place and the scope | One place at a time (the package surface, the news, the settings), one scope at a time (the results, what is installed, what can be updated) | `state/session.rs` |
| The context header | The scope's name, the scope switcher, the row count, the ecosystems the scope can contain, the age of the answer, the filters in force, a source that did not answer | `components/context_header.rs` |
| The rail, bound to the scope | Five entries, the first three being the scopes of the package surface; the active one and the switcher always agree | `components/sidebar.rs` |
| The query surface in every scope | The search input, the sources, the state and the sort, everywhere; the scope's own action beside them | `components/query_workbench.rs` |
| Filters that mean what they say | The sources are the scope's, the three inventory-only ecosystems are marked, a Cargo row no longer disappears with ALPM; the state filter only offers what can be true of the rows | `state/query.rs` |
| The back path | Escape clears the search, then leaves the place, then empties the selection; ⌘1 to ⌘5, ⌥← and ⌥→, ⌘F, ⌘L, ⌘⌥C | `views/workspace.rs` |
| The control surface | `calix-gpui scope installed`; `status --json` reports the place, the scope, the coverage, the filters and the row count, with `destination` kept as a deprecated alias; protocol version 5 | `control/protocol.rs`, `control/cli.rs` |
| The remembered scope | The window state keeps the last scope | `config.rs` |

## Reading order

`01-IMPLEMENTATION.md` says what changed and why, requirement by requirement.
`02-ACCEPTANCE-REPORT.md` says what is proved, what is not, and where the evidence
is.

## The one place the contract was adjusted

The contract said the context header would be `44px`. The implementation measures
`48px`: the place and the scope, the scope switcher, then the coverage and the
age. The contract follows the measurement (`04-NAVIGATION-CONTRACT.md` N15, and the
wireframes in `../slice-07/03-WIREFRAMES.md`).

## Naming note

This kit is Calix. The earlier kits keep their historical names, as the
repository's naming boundary requires.
