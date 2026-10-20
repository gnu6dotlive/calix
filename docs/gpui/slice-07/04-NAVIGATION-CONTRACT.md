# 04 — Navigation Contract

Every requirement is numbered so the acceptance matrix can point at it. "Must" is
binding on Slice-08; "may" is left to the implementation.

## 1. Places and scopes

**N1 — One place at a time.** The application has exactly one place active: the
package surface, News, or Settings. The render lab is a development surface, excluded
from the rail, and is not a place (it declares itself as such when open).

**N2 — One scope at a time.** The package surface shows exactly one scope: Search
results, Installed, or Updates. The scope is named in the context header, always.

**N3 — The rail and the scope switcher are bound.** The first three rail entries and
the scope switcher show the same value. A change in one changes the other in the same
frame; they may never disagree, and there is no state in which the rail says one thing
and the header another.

**N4 — The rail is never a filter.** The rail contains no source control, no state
control, and no query state. Selecting a rail entry changes the scope or the place and
nothing else.

**N5 — A scope change reloads the scope's data.** Switching scope must show the
scope's rows, not a filtered remnant of the previous scope's rows.

**N6 — A scope change preserves the filters.** The source and state filters survive a
scope change, subject to R1 and R2 in `02-IA.md`: the *vocabulary* of the controls
follows the scope, the *values* persist.

**N7 — A place change preserves the scope.** Leaving the package surface for News or
Settings and coming back lands on the same scope.

**N8 — The last package scope is remembered.** Across a place change and across a
restart, the application returns to the last package scope the user was in. The
scope is stored with the rest of the window state (`gpui-ui.json`).

**N9 — The scope switcher is only in the package surface.** News and Settings have
no scope switcher, because they have no scope.

## 2. The context header

**N10 — The header is present in every place**, including while an operation runs and
while a surface is in an error state.

**N11 — The header names the active scope or place**, and the name is never empty.

**N12 — The header carries the row count** of the current scope, `—` while loading
and on error, and it is hidden in Settings.

**N13 — The header carries the coverage**: the ecosystems the current scope can
contain, with the inventory-only ones marked as such in the Installed scope.

**N14 — The header carries the filter summary** when a filter is not at its default,
and offers a single action that clears the filters.

**N15 — The header is `44px` tall and does not scroll.** It is sticky in every place.

## 3. The back path

**N16 — No dead ends.** From News and from Settings, the rail returns to the package
surface, and the scope the user comes back to is the one they left (N7).

**N17 — Escape is the local back.** `Escape` clears the search input when it has focus
and is not empty; otherwise it leaves News or Settings for the package surface;
otherwise it clears the inspector selection. It never does nothing when something is
possible.

**N18 — A global search implies the Results scope; a query narrows the scope it is
typed in.** Two different things are called a search, and the contract keeps them
apart:

- The control surface's `search` verb performs a *global* search: it lands in the
  Results scope with the query applied, and the scope the user came from is
  remembered for `Escape` and for the back path (N7).
- The search input of the package surface narrows the *current* scope: in the Results
  scope the query is dispatched to the backend, and in the Installed and Updates
  scopes it filters the rows that are already loaded (R6). The input's placeholder
  states the universe it searches, so the difference is never a surprise.

**N19 — Inspect does not change the scope.** `inspect` sets the selection only. If
the package is not among the current scope's rows, the inspector opens it anyway and
the header marks it as outside the current scope.

## 4. Keyboard

**N20 — The rail is reachable and operable by keyboard**, in its visual order, with
the active entry marked and a roving focus.

**N21 — The global keys are:**

| Key | Action |
|---|---|
| `⌘1` … `⌘5` | Rail entry 1 to 5: Search, Installed, Updates, News, Settings |
| `⌥←` / `⌥→` | Previous / next scope, within the package surface |
| `⌘F` | Focus the search input |
| `⌘L` | Toggle the operation console |
| `⌘⌥C` | Collapse or expand the rail |
| `Escape` | The local back, per N17 |

**N22 — The keys do not fire while a text field has focus**, except the `Escape` of
N17 and the text-editing keys the field owns.

**N23 — The focus order per place is fixed:**

| Place | Focus order |
|---|---|
| Package surface | rail → scope switcher → search input → query controls → list → inspector → console handle |
| News | rail → header → feed → console handle |
| Settings | rail → header → section list → panel → console handle |

**N24 — A place change moves the focus to the new place's first element**, and the
scope switcher moves the focus to the list, so the arrow keys reach the rows.

## 5. The rail

**N25 — The rail keeps its five entries, in their current order, with their current
icons and labels** (Search, Installed, Updates, News, Settings).

**N26 — The active entry is marked** with the accent bar and the active surface
colour, and the entry for the current scope is the active one (N3).

**N27 — The Updates entry keeps its count badge** when there are pending updates.

**N28 — The collapsed rail keeps the icons and the active marker**, and the context
header carries the scope, so collapsing never costs the user their orientation.

**N29 — The rail does not scroll** and does not grow entries.

## 6. The work surfaces

**N30 — The query surface is sticky, and it is present in every scope.** The search
input (`40px`) and the query controls (`34px`) stay anchored at the top of the list
pane while the rows scroll beneath, in the Search results, Installed and Updates
scopes alike. The `24px` status rail of the current application is retired: the count
and the filter summary move into the context header (N12, N14).

**N31 — The controls that change a filter are present in every scope.** The source
control, the state control and the sort control are rendered in all three scopes,
because the filters act on all three (R5). A scope that applies a filter the user
cannot see, and cannot undo, is a scope the user cannot trust.

**N32 — The inspector is docked, never an overlay**, and its header and tabs are
pinned while only its body scrolls. The $3/7$ ratio cap, the $320px$ and $340px$
floors and the stacked fallback below $746.67px$ of content width are unchanged.

**N33 — News and Settings replace the station inside the frame.** The rail and the
header remain; the list and the inspector are not rendered.

**N34 — The operation console is docked and global.** It opens automatically when an
operation starts, from any place, and it never covers the content area.

**N35 — An error never clears the surface.** The rows the user was looking at stay,
the error is reported in place, and a retry action is offered.

**N36 — The Updates scope states the age of its answer** — when the update set was
last checked — in the context header and in the empty state.

## 7. The control surface

The control CLI is an automation surface, not a convenience. Its verbs are part of
the product contract (`docs/gpui/control-cli/01-COMMAND-SURFACE.md`).

**N37 — The existing verbs keep working, unchanged in meaning:**

| Verb | Behaviour after the change |
|---|---|
| `navigate browse` | Package surface, Search results scope |
| `navigate installed` | Package surface, Installed scope |
| `navigate updates` | Package surface, Updates scope |
| `navigate news` | News place |
| `navigate settings` | Settings place |
| `search <query>` | Search results scope with the query applied (N18) |
| `inspect <package>` | Sets the selection, scope unchanged (N19) |
| `view table\|cards` | View mode, unchanged |
| `inspector <tab>` | Inspector tab, unchanged |
| `logs show\|hide\|clear` | Operation console, unchanged |
| `status [--json]` | Extended, per N39 |
| `open`, `focus`, `quit` | Unchanged |

**N38 — A new `scope` verb** changes the scope without leaving the package surface:
`calix-gpui scope installed`. It accepts `results`, `installed` and `updates`, and
`browse` is accepted as an alias of `results`.

**N39 — `status --json` gains the navigation state.** The envelope is unchanged; the
`data` object gains the fields below, and `protocol_version` becomes `5` when they
appear, so the control surface reports the version it speaks.

```json
{
  "version": 5,
  "ok": true,
  "message": "Running",
  "data": {
    "protocol_version": 5,
    "gui_running": true,
    "pid": 1299363,
    "executable": "/usr/lib/calix/calix-gpui",
    "place": "packages",
    "scope": "installed",
    "scope_sources": ["alpm", "aur", "flatpak", "appimage", "cargo", "npm", "release"],
    "inventory_only_sources": ["cargo", "npm", "release"],
    "filters": { "sources": ["alpm"], "state": "all", "sort": "relevance", "view": "table" },
    "row_count": 1911,
    "checked_at": null,
    "destination": "installed",
    "query": "",
    "view_mode": "table",
    "inspector_tab": "overview",
    "selected_package": null,
    "operation_running": false,
    "render_lab_active": false
  }
}
```

`destination` is retained as a deprecated alias of `scope`, so existing automation
keeps working.

**N40 — The control surface never leaves the application in an inconsistent state.**
Every verb that changes the place or the scope returns only after the frame has been
re-rendered with the new state, so a `navigate` followed by a `status` always reports
the new state.

## 8. Persistence

**N41 — The persisted window state gains the last package scope** (`last_scope`), next
to the existing `window_width`, `window_height`, `view_mode`, `visual_style`,
`reduce_motion` and `log_drawer_*` fields. The file keeps its current name and format
compatibility: a state file without `last_scope` loads with the default scope.

**N42 — The filters are persisted** with the rest of the window state, so a restart
returns to the view the user left, filtered the same way.

## 9. Invariants that must not regress

These are the measurements the previous phases made. Slice-08 may not weaken them, and
`05-ACCEPTANCE-MATRIX.md` repeats them as requirements.

| # | Invariant |
|---|---|
| N43 | $w_{inspector} \le \min(520,\; 3/7\,w_c)$ |
| N44 | $w_{inspector} \ge 320px$ in horizontal mode |
| N45 | $w_{list} \ge 340px$ |
| N46 | Horizontal split at $w_c \ge 746.67px$, stacked below |
| N47 | The query surface is sticky (N30) |
| N48 | Zero overlay: the inspector, the menus and the news and settings surfaces never float over the list |
| N49 | The console is docked, never a modal (N34) |
| N50 | The splitter resize is a pure differential, invariant to the rail being collapsed or expanded |
| N51 | The render lab stays out of the rail and keeps its return path to Settings |
| N52 | The geometry is recomputed when the window bounds change (below) |

**N52 — The geometry follows the window the compositor gives the application.**
The workstation's widths are recomputed from the *mapped* window bounds every time
those bounds change, and not only when the workstation happens to re-render. The
workspace caches the workstation's element, so a resize of the window does not
re-render it, and the clamped widths survive the resize while the flex inspector
absorbs the difference.

The measured failure, in one session with the window requested at $1440px$ and tiled
by the compositor (`01-CURRENT-STATE.md` §4.2):

| The compositor gave the window | The list pane | The inspector | Cap | Floor |
|---|---|---|---|---|
| $1056px$ | $724px$ | $137px$ | $371px$ | $320px$ |
| $2542px$ | $724px$ | $1622px$ | $520px$ | $320px$ |

After the requirement is met, the same two windows yield an inspector inside
$[320,\; \min(520,\; 3/7\,w_c)]$ in both cases, and a list pane of at least $340px$.

The window's own bounds are the only source of truth. The size the application
*requested* is a starting hint, not a measurement, and the difference between the
two is what a tiling compositor decides on every focus change.
