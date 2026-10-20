# 01 — Current State: What Navigation Is Today

## 1. The surface as it is

The application is one window with three regions: a collapsible rail, a content area,
and a log drawer that carries the operation console.

| Region | Width / height | Source |
|---|---|---|
| Rail | `190px` expanded, `56px` collapsed | `UiMetrics::SIDEBAR_EXPANDED`, `UiMetrics::SIDEBAR_COLLAPSED` |
| Search input | `40px` | `UiMetrics::SEARCH_INPUT_HEIGHT` |
| Query controls | `34px` | `UiMetrics::QUERY_CONTROLS_HEIGHT` |
| Status rail | `24px` | `components/query_workbench.rs:1031` |
| Splitter | `5px` | `UiMetrics::SPLITTER_WIDTH` |
| List pane | $\ge 340px$ | `UiMetrics::LIST_MIN_USABLE` |
| Inspector | $320 \le I \le \min(520,\; 3/7\,w_c)$ | `UiMetrics::INSPECTOR_*` |
| Log drawer | `220px` open, `24px` handle | `gpui-ui.json` `log_drawer_height` |

The rail carries five entries — Browse, Installed, Updates, News, Settings — and a
sixth destination, `RenderLab`, that the rail explicitly refuses to render
(`components/sidebar.rs:68`, `unreachable!("RenderLab is excluded from sidebar")`).

## 2. What the five entries actually are

They are not five places. They are two kinds of thing, and the interface does not say
which is which.

**Three of them are the same surface.** `views/workspace.rs:1935` routes Browse,
Installed and Updates to the same `workstation` element. The destination only decides
which data set the list shows:

```rust
let main_content = match destination {
    NavDestination::Browse | NavDestination::Installed | NavDestination::Updates => {
        div().size_full().child(self.workstation.clone())
    }
```

**Two of them replace the whole workstation.** `views/workspace.rs:1939` renders
`NewsView` and `SettingsView` in place of the station. The list, the inspector and the
query workbench disappear; only the rail survives.

The consequence is a rail that mixes a scope switch with page navigation:

- Selecting **Updates** does not move the user to a place; it changes the rows of the
  list they are already looking at.
- Selecting **News** moves the user to a different kind of surface, and the frame that
  told them what the app was doing is gone.

**There is no way back.** The only `on_back` handler in the tree belongs to the
render lab (`views/render_lab.rs:29`). News and Settings have no return path, no
breadcrumb, and no context header. A user who follows a link into Settings has no
affordance that tells them where they are or how to return.

**The query workbench exists in three of the five.** Search, filters, sort and the
table/cards switch are part of the package surface. In News and Settings they are
absent — correctly, since neither is a package list — but nothing in the frame says
so.

## 3. The scope of the data, and the scope of the filter

The unified inventory covers seven ecosystems. The list can show all of them. The
source filter knows about four.

`state/query.rs:76` refuses to make Cargo, NPM and Release toggleable, with a
correct reason — they are not searchable sources. But `contains_str`
(`state/query.rs:84`) then folds them into ALPM:

```rust
pub fn contains_str(&self, source_type: &str) -> bool {
    match source_type {
        "Standard" | "Official" | "ALPM" => self.alpm,
        "AUR" => self.aur,
        "Flatpak" => self.flatpak,
        "AppImage" => self.appimage,
        _ => self.alpm,          // ← Cargo, NPM and Release are ALPM or nothing
    }
}
```

So a Cargo, NPM or tracked-release row is visible only while ALPM is selected, and no
pill in the filter represents them. The filter's vocabulary and the list's vocabulary
do not match.

The mirror image holds on the other side: the inventory reports seven ecosystems, the
UI can mutate three (ALPM, AUR, Flatpak). The state filter offers
`Installed / Not installed / Updates available` across a list whose members are not
uniformly mutable.

## 4. What the documentation has, and what it does not

The repository has an unusually strong measurement record and no information model:

| Documented | Not documented |
|---|---|
| Inspector ratio and width floors (`ux-workbench/02`) | Which destination a user is in, and how they know |
| Query workbench rows and breakpoints (`ux-workbench/03`) | The relation between a rail entry, a scope and a filter |
| Keyboard menus, motion contract (`slice-04/03`, `slice-04/04`) | The empty, loading and error states per surface |
| Determinism contract and fixture catalogue (`render-lab/`) | The back path, the deep links, the deep-link grammar |

Every slice to date specified *how a surface behaves*. None specified *what the
surfaces are*. That is the gap this slice closes, and it is why the wireframe plan
does not exist to build on.

## 5. The five questions a navigation has to answer

| Question | Today |
|---|---|
| Where am I? | A highlighted rail row, and nothing else |
| What can I do here? | The rail implies it; the surface does not confirm it |
| How do I get somewhere else? | The rail, and it is the only way |
| What is the current state? | A count on the list header, a filter summary on row 3, and nothing that names the scope |
| What just happened? | The log drawer, which is collapsed by default and is not part of the frame |

## 6. Evidence

| Artefact | What it shows |
|---|---|
| `docs/media/calix-browse.png` | The package surface with results: 18 rows from two sources, no context header, no scope indication |
| `docs/media/calix-installed.png` | The same surface showing 1 911 installed rows: the surface is identical, only the rows differ |
| `docs/gpui/ux-workbench/evidence_ux02_*.png` | The workbench as it was measured — the geometry, without the model |
| `docs/gpui/ux-workbench/09-RUNTIME-EVIDENCE.md` | The runtime gallery of the previous phases |

The two `docs/media` captures are the clearest statement of the problem: they are the
same application, and a user cannot tell from the picture which one they are looking
at.

## 7. What must not regress

Slice-07 changes where the user is told things, not the measurements that were
already made. The following stay as they are, and `05-ACCEPTANCE-MATRIX.md` repeats
them so a later slice cannot quietly drop them:

- the inspector ratio cap, the $320px$ and $340px$ floors, and the stacked fallback
  below $746.67px$ of content width;
- the sticky query surface: the search input and the controls stay anchored at the top
  of the list pane while the rows scroll beneath;
- zero overlay: the inspector never floats over the list, and the news and settings
  surfaces never float over the frame;
- the operation console remains a docked region, never a modal.
