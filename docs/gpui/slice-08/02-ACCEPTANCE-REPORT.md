# 02 — Acceptance Report

What the implementation of the Slice-07 contract is proved by, and what it is
not.

## The state of the contract

| Requirement | State | Proved by |
|---|---|---|
| N1, N2 — one place, one scope | implemented | The state model (`state/session.rs`), the unit tests on the session |
| N3, N25-N29 — the rail bound to the scope | implemented | The rail's entries are the scopes; the active entry and the switcher read the same value |
| N5, N6 — a scope change reloads and keeps the filters | implemented | The scope's own loader; the filters live in the session |
| N7, N8 — the scope survives a place change and a restart | implemented | `the_scope_survives_a_place_change`; the window state keeps `last_scope` |
| N10-N15 — the context header | implemented | The captures in `evidence/runs/calix-slice-08-20261020/`; the header's own tests |
| N16, N17 — the back path | implemented | The key handler; the escape precedence |
| N20-N24 — the keyboard and the focus order | implemented | The key handler; the focus order follows the surfaces |
| N30, N31 — the query surface in every scope | implemented | The captures of the three scopes |
| N32 — the news and the settings dock in the frame | implemented | The header is present in every place |
| N33 — the console is docked and global | unchanged | — |
| N34, N35, N36 — errors, the age, the health | unchanged, the health moved to the header | The captures |
| N37-N40 — the control surface | implemented | `calix-gpui scope installed`; `status --json` carries the new fields; the protocol is version 5 |
| N41, N42 — the persisted scope | implemented | The window state carries `last_scope` |
| N43-N51 — the geometry invariants | unchanged | The v0.2.3 fix; the invariants still hold at every width |
| N52 — the render lab stays out of the rail | implemented | The render lab is a surface above the place, not a place |
| N53 — the geometry follows the mapped window | closed in v0.2.3 | `evidence/runs/calix-v0.2.3-hotfix-20261020/` |
| R1 — the sources are the scope's | implemented | `the_recorded_updates_payload_parses_and_flattens` and the source scope's tests |
| R2 — the state follows the scope | implemented | `PackageStateFilter::options_for` |
| R3 — the filters survive | implemented | The filters live in the session |
| R4 — the scope is never emptied | implemented | The session's `toggle_source` |
| R6 — the query narrows the scope | implemented | `the_query_narrows_the_rows_of_the_scope` |

## What is not yet proved

- **The clicks.** The synthetic pointer events never reached the Wayland client
  on this desktop: the wheel does not scroll and the clicks do not land. Every
  control in this slice is therefore proved by its state, by its unit test and by
  the control surface, and not by a click. A session with a real pointer closes
  the rows that say *implemented* in the table above.
- **The age of the answer.** The header carries the age of the updates, but the
  daemon has no scheduler yet, so there is no age to show. The field is there,
  and it is empty until the watcher exists.
- **The focus order after a place change.** The order itself follows the
  surfaces; moving the focus to the first element of the new place is not
  implemented, because the surfaces do not hold focus handles for their first
  element.

## The evidence

| Artefact | What it shows |
|---|---|
| `evidence/runs/calix-slice-08-20261020/` | The three scopes with the context header and the query surface, the settings surface, the control surface's answers |
| `evidence/runs/calix-v0.2.3-hotfix-20261020/` | The geometry fix, measured before and after |
| `evidence/runs/calix-brand-20261020/` | The mark, in both colourways, and the settings row |

## The gate

The commit passed `scripts/verify` and `scripts/ci-replay` locally: 1388 package
manager tests, 392 CLI tests, 211 frontend tests, and the release builds.
