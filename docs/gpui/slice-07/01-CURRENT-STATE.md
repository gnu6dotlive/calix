# 01 — Current State: What Navigation Is Today

## 1. The surface as it is

The application is one window with three regions: a collapsible rail, a content area,
and a log drawer that carries the operation console.

| Region | Width / height | Source |
|---|---|---|
| Rail | `190px` expanded, `56px` collapsed | `UiMetrics::SIDEBAR_EXPANDED`, `UiMetrics::SIDEBAR_COLLAPSED` |
| Search input | `40px`, Browse only | `UiMetrics::SEARCH_INPUT_HEIGHT` |
| Query controls | `34px`, Browse only | `UiMetrics::QUERY_CONTROLS_HEIGHT` |
| Status rail | `24px`, Browse only | `components/query_workbench.rs:1031` |
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

The station is the list and the inspector, and it is shared. Its *top bar* is not:
`views/package_workstation.rs:657` matches on the destination, and only the Browse arm
renders the query workbench (`views/package_workstation.rs:673`). The Installed and
Updates arms render a thin header — the title, the count, the table/cards switch
(`views/package_workstation.rs:748`). So the three scopes are the same surface with
three different top bars, one of which is a full query surface and two of which are
bare.

**Two of them replace the whole workstation.** `views/workspace.rs:1939` renders
`NewsView` and `SettingsView` in place of the station. The list, the inspector and the
query workbench disappear; only the rail survives.

**The query workbench exists in one of the five.** Search, the source control, the
state control, the sort control and the status rail are rendered in the Browse scope
only. In Installed and Updates there is no search input and no filter control at all
— only the title, the count and the view switcher. In News and Settings the list
disappears entirely.

The consequence is a rail that mixes a scope switch with page navigation:

- Selecting **Updates** does not move the user to a place; it changes the rows of the
  list they are already looking at.
- Selecting **News** moves the user to a different kind of surface, and the frame that
  told them what the app was doing is gone.

**There is no way back.** The only `on_back` handler in the tree belongs to the
render lab (`views/render_lab.rs:29`). News and Settings have no return path, no
breadcrumb, and no context header. A user who follows a link into Settings has no
affordance that tells them where they are or how to return.

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

## 4. Two findings from the baseline capture

The pre-change capture (`evidence/runs/calix-slice-07-baseline-20261020/`) was taken
on a live Wayland session, driving the application through the control surface. It
produced two findings that the code reading alone had missed, and both are navigation
defects rather than visual ones.

### 4.1 The filters act where they cannot be seen

The filter state is applied to **all three scopes** —
`views/package_workstation.rs:610-628` filters and sorts the rows of whichever data
set the destination selected:

```rust
let packages: std::sync::Arc<[UnifiedPackage]> = {
    if source_scope.is_all()
        && state_filter == PackageStateFilter::All
        && sort_mode == SortMode::Relevance
    {
        raw_packages
    } else {
        let mut filtered: Vec<UnifiedPackage> = raw_packages
            .iter()
            .filter(|p| {
                source_scope.contains_str(&p.source_type)
                    && state_filter.matches(p.is_installed, p.has_update)
            })
            .cloned()
            .collect();
        sort_mode.sort_packages(&mut filtered);
        std::sync::Arc::from(filtered)
    }
};
```

The controls that change that state are rendered in the Browse scope only
(`views/package_workstation.rs:673`). So:

1. The user narrows the sources in Browse — say, everything but ALPM.
2. The user clicks **Installed**.
3. The installed list is now filtered the same way, the count in the header follows
   the filter, and **there is no control anywhere on that surface** that says so or
   undoes it.

The same applies to the state filter and to the sort order. A filter set in one scope
silently governs the other two, and the scopes where it governs are the scopes that
cannot show it. This is the sharpest form of the complaint that started this slice:
the interface does not say what it is showing.

It also makes the ecosystem bug of §3 reachable: the three inventory-only
ecosystems — Cargo, NPM and the tracked releases — are dropped from the Installed
list whenever ALPM is deselected, and the control that would restore them lives in
another scope.

### 4.2 The geometry is computed once, and never again

The measured invariants of the previous phases hold in the application's model, not
on the screen. The workstation computes its widths from the window bounds it knows
when it renders, and the workspace caches the workstation's element
(`views/workspace.rs:1936`, `self.workstation.clone()`), so a resize of the window
does not re-render the workstation: the clamped widths persist, and the inspector,
which is a flex child, absorbs the difference.

Measured in one session, with the window requested at $1440px$ and tiled by the
compositor:

| The compositor gave the window | The list pane | The inspector | The cap | The floor |
|---|---|---|---|---|
| $1056px$ | $724px$ | $137px$ | $371px$ | $320px$ |
| $2542px$ | $724px$ | $1622px$ | $520px$ | $320px$ |

The list pane is $724px$ in both cases: it does not depend on the width the window
actually has. The inspector is below its floor in the first case and above its cap in
the second, and the application believes it is respecting both. A tiling compositor
resizes the window whenever the focus changes, so this is the normal state of the
application, not an edge case.

The consequence for this slice is direct: the wireframes and the invariants of
`04-NAVIGATION-CONTRACT.md` §9 are drawn for a layout that the application does not
currently produce. They are a specification of what the layout must be, and one of
the requirements of Slice-08 is that the geometry is recomputed when the bounds
change.

### 4.3 One of the five rail entries is dead

The Updates scope does not load. The capture shows the whole of it:

```
┌──────────────────────────────────────────────────────────────────┐
│  ↻ Available System Updates                    [Upgrade All]     │
│                                                                  │
│      Failed to check for updates: Désérialisation de la liste    │
│      des mises à jour                     [ Retry ]              │
└──────────────────────────────────────────────────────────────────┘
```

The cause is a disagreement between the two halves of the product, on the shape of
one payload. The frontend asks the CLI for every backend's updates and reads the
answer as a flat list:

```rust
// Calix.Ui.Gpui/src/backend/client.rs:129
pub async fn list_updates(&self) -> Result<Vec<PackageUpdateItem>> {
    let raw =
        ProcessRunner::run_json_command(&self.binary_path, &["list-updates", "all", "-j"])
            .await?;
    ...
    serde_json::from_str::<Vec<PackageUpdateItem>>(trimmed)
        .context("Désérialisation de la liste des mises à jour")
}
```

The CLI answers the `all` variant with a **grouped object**, one array per backend
(`Calix.Cli.Zig/src/commands/list_updates.zig:362`):

```json
{"Packages": [], "Aur": [ … ], "AppImage": [ … ], "Flatpak": [ … ]}
```

The flat array is what the *per-backend* variants return (`list-updates standard -j`,
`list-updates aur -j`, `list-updates.zig:346-354`); the `all` variant is the one
that groups. The frontend asks for the grouped one and parses it as the flat one, so
the deserialisation always fails, and the Updates scope has never loaded in the
released application. The error is reported honestly, with a retry — but a rail entry
that always fails is a rail entry that is broken.

This is not a navigation defect, but it is the navigation's first casualty: a scope
that cannot load cannot be named, counted, filtered or dated, so four requirements of
the contract (N12, N13, N33, N37) have nothing to say. The contract states the
requirement (N33) and the implementation slice has to close it.

## 5. What the documentation has, and what it does not

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

## 6. The five questions a navigation has to answer

| Question | Today |
|---|---|
| Where am I? | A highlighted rail row, and nothing else |
| What can I do here? | The rail implies it; the surface does not confirm it |
| How do I get somewhere else? | The rail, and it is the only way |
| What is the current state? | A count on the list header, a filter summary on row 3, and nothing that names the scope |
| What just happened? | The log drawer, which is collapsed by default and is not part of the frame |

## 7. Evidence

| Artefact | What it shows |
|---|---|
| `evidence/runs/calix-slice-07-baseline-20261020/` | The pre-change capture: one frame per place and per scope, each paired with the control surface's status document, and the measurements of §4 |
| `docs/media/calix-browse.png` | The package surface with results: 18 rows from two sources, no context header, no scope indication |
| `docs/media/calix-installed.png` | The same surface showing 1 911 installed rows: the surface is identical, only the rows differ |
| `docs/gpui/ux-workbench/evidence_ux02_*.png` | The workbench as it was measured — the geometry, without the model |
| `docs/gpui/ux-workbench/09-RUNTIME-EVIDENCE.md` | The runtime gallery of the previous phases |

The two `docs/media` captures are the clearest statement of the problem: they are the
same application, and a user cannot tell from the picture which one they are looking
at. The baseline capture adds the third one: `installed.png` and `updates.png` are
the same surface, and the *top bars* are not even the same.

## 8. What must not regress

Slice-07 changes where the user is told things, not the measurements that were
already made. The following stay as they are, and `05-ACCEPTANCE-MATRIX.md` repeats
them so a later slice cannot quietly drop them:

- the inspector ratio cap, the $320px$ and $340px$ floors, and the stacked fallback
  below $746.67px$ of content width — **and, new in this slice, they must hold after
  a compositor resize, which today they do not (§4.2)**;
- the sticky query surface: the search input and the controls stay anchored at the top
  of the list pane while the rows scroll beneath;
- zero overlay: the inspector never floats over the list, and the news and settings
  surfaces never float over the frame;
- the operation console remains a docked region, never a modal.
