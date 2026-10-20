# 03 — Wireframes

These are the wireframes the implementation is measured against. They are drawn on the
metrics the code already uses (`ui_metrics.rs`), so no measurement has to be
reinvented: the only new number is the `44px` context header.

## 1. Reading the wireframes

- `▓` is the rail, `─` the content area, `▒` the scrollable region, `▨` the docked
  region, `·` empty space.
- Widths are given for the *content* area, $w_c = w_{window} - w_{rail}$, with
  $w_{rail} = 190px$ expanded and $56px$ collapsed.
- Three widths are drawn: **wide** ($w_c \ge 1090$), **medium**
  ($746.67 \le w_c < 1090$) and **narrow** ($w_c < 746.67$, the stacked fallback).
- $w_{window}$ is the width the **compositor** gives the window, not the width the
  application asked for. The two differ whenever a tiling compositor re-tiles the
  window, and the geometry below is recomputed from the former (N53). Today it is
  not, which is why the baseline capture in `01-CURRENT-STATE.md` §4.2 shows an
  inspector of $137px$ and of $1622px$ in the same session.

## 2. W0 — The shell

The frame is the same in every place. Only the content area changes.

```
┌──────────────┬────────────────────────────────────────────────────────────────┐
│              │  CONTEXT HEADER                                   44px          │
│              ├────────────────────────────────────────────────────────────────┤
│  RAIL        │                                                                │
│  190px       │                  CONTENT AREA                                   │
│  (56px       │                  (one place)                                    │
│   collapsed) │                                                                │
│              │                                                                │
│              │                                                                │
│  ▒▒▒▒▒▒▒▒▒  │                                                                │
├──────────────┴────────────────────────────────────────────────────────────────┤
│  OPERATION CONSOLE            24px handle / 220px open                        │
└───────────────────────────────────────────────────────────────────────────────┘
```

| Region | Height | Scrolls | Always present |
|---|---|---|---|
| Rail | full | no | yes |
| Context header | `44px` | no | yes |
| Content area | flex | its own | yes |
| Operation console | `24px` / `220px` | yes, when open | yes |

The console is the only region that changes height when it is not open, and it never
covers the content area.

## 3. W1 — Package surface, Installed scope (wide)

```
┌──────────────┬────────────────────────────────────────────────────────────────┐
│ CALIX        │  Installed  [ Search | Installed | Updates ]   1 911 pkgs        │ 44
│              │  ALPM · AUR · Flatpak · AppImage · Cargo · NPM · Release          │
│  ⌕ Search    ├────────────────────────────────────────────────────────────────┤
│  ▣ Installed │  ⌕  search the 1 911 installed packages…                     40 │
│  ↻ Updates   ├────────────────────────────────────────────────────────────────┤
│  ▤ News      │  Sources ▾   All States ▾   Relevance ▾   ▦▤   ▨▥            │ 34
│  ⚙ Settings  │                                                                │
│              ├───────────────────────────────┬────────────────────────────────┤
│              │  NAME          VER   SOURCE  │  bash                     5.3-2 │
│              │  7zip          26.04  local   │  ALPM · local · Installed          │ 36
│              │  a52dec        0.8.0  local   │  ┌ Install ┐ ┌ Copy install cmd ┐    │
│              │  aalib         1.4rc5 local  │  │ Overview │ Dependencies │ Files │   │ 32
│              │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │  ────────────────────────────────    │
│              │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│              │  list pane  ≥340px           │  inspector  320≤I≤min(520,3/7·w_c)│
│  ‹ Collapse  │                               │                                │
└──────────────┴───────────────────────────────┴────────────────────────────────┘
```

| Region | Height | Scrolls | Notes |
|---|---|---|---|
| Context header | `44px` | no | title, scope switcher, count, coverage, filter summary |
| Search input | `40px` | no | sticky; its placeholder states the universe it searches |
| Query controls | `34px` | no | sticky; present in every scope |
| List pane | flex | **yes** | the only scrolling region of the list |
| Inspector | flex | body only | header and tabs pinned, body scrolls |

The scope switcher in the header and the three rail entries show the same value
(L3). The rail's *Installed* row is the active one. The `24px` status rail of the
current application is retired: the count and the filter summary move into the
header (IA-10).

## 4. W2 — Package surface, Search results (medium)

$w_c \in [746.67,\; 1090)$: the horizontal split still holds, and the query controls
collapse to their compact form (the breakpoint rule already in
`components/query_workbench.rs`).

```
┌────────┬──────────────────────────────────────────────────────────────┐
│        │  Search results  [ Search | Installed | Updates ]  18 pkgs  ▨▥ │ 44
│  ⌕     │  ALPM · AUR                                                  │
│  ▣     ├──────────────────────────────────────────────────────────────┤
│  ↻     │  ⌕  ripgrep                                          [×]    │ 40
│  ▤     ├──────────────────────────────────────────────────────────────┤
│  ⚙     │  Sources ▾   All States ▾   Relevance ▾                     │ 34
│        │                                                              │
│ 56px   ├─────────────────────────────┬────────────────────────────────┤
│        │  ripgrep   15.2.0  ALPM     │  ripgrep                   15.2 │
│        │  ripgrep-all 0.10.10 AUR    │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
└────────┴─────────────────────────────┴────────────────────────────────┘
```

Collapsed rail (`56px`): the icons only, with the active one marked. The context
header carries the scope and the count, which is why collapsing the rail never costs
the user their orientation.

## 5. W3 — Package surface, Updates scope

```
┌──────────────┬────────────────────────────────────────────────────────────────┐
│  CALIX       │  Updates  [ Search | Installed | Updates ]        3 pkgs      │ 44
│              │  ALPM · AUR · Flatpak · AppImage · checked 4 minutes ago         │
│  ⌕ Search    ├────────────────────────────────────────────────────────────────┤
│  ▣ Installed │  ⌕  search the 3 pending updates…                           40 │
│  ↻ Updates   ├────────────────────────────────────────────────────────────────┤
│  ▤ News      │  Sources ▾   All States ▾   Relevance ▾   ▦▤   ▨▥            │ 34
│  ⚙ Settings  │                                                                │
│              ├───────────────────────────────┬────────────────────────────────┤
│              │  linux  6.9.1 → 6.9.2  ALPM  │  linux                        │
│              │  rust   1.82 → 1.83    AUR   │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│              │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
└──────────────┴───────────────────────────────┴────────────────────────────────┘
```

The header carries the **age** of the update set. Today the Updates scope answers
"3 packages" and never says when that was true; a package manager that cannot say how
old its answer is cannot be trusted.

## 6. W4 — News (a place, docked in the frame)

```
┌──────────────┬────────────────────────────────────────────────────────────────┐
│  CALIX       │  Arch Linux news                                    3 unread   │ 44
│              ├────────────────────────────────────────────────────────────────┤
│  ⌕ Search    │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│  ▣ Installed │  2026-10-18  Package signing keys rotated         ● unread     │
│  ↻ Updates   │  2026-10-02  Manual intervention for …            ○ read       │
│  ▤ News      │  2026-09-28  Arch Linux install …                 ○ read       │
│  ⚙ Settings  │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│              │                                                                │
│  ‹ Collapse  │  the feed is the content area; the station is not shown       │
└──────────────┴────────────────────────────────────────────────────────────────┘
```

The rail and the header stay; the list and the inspector are gone. The feed is the
content area and scrolls on its own. There is no "back to packages" button in the
content: the rail *is* the way back, and it is one click away.

## 7. W5 — Settings (a place, docked in the frame)

```
┌──────────────┬────────────────────────────────────────────────────────────────┐
│  CALIX       │  Settings                                                       │ 44
│              ├────────────────────────────────────────────────────────────────┤
│  ⌕ Search    │  PACKAGE SOURCES                                                │
│  ▣ Installed │  ┌────────────────────────────────────────────────────────────┐ │
│  ↻ Updates   │  │ Arch User Repository (AUR)                    [ Enabled ]   │ │
│  ▤ News      │  │ Enable search and PKGBUILD inspection for AUR packages    │ │
│  ⚙ Settings  │  └────────────────────────────────────────────────────────────┘ │
│              │  ┌────────────────────────────────────────────────────────────┐ │
│              │  │ Flatpak Packages                               [ Enabled ]   │ │
│              │  │ Enable search and inspection for sandboxed applications   │ │
│              │  └────────────────────────────────────────────────────────────┘ │
│              │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│              │  PACKAGE MANAGEMENT                                            │
│              │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│  ‹ Collapse  │                                                                │
└──────────────┴────────────────────────────────────────────────────────────────┘
```

The settings surface is a single scrolling column of section cards, as it is today.
The sections are not places and not scopes: they are not rail entries, they do not
appear in the context header, and they are not addressable from the control surface.
The context header is the only addition, and it carries the place name and the state
of the configuration ("Up to date").

## 8. W6 — An operation in flight, from any place

```
┌──────────────┬────────────────────────────────────────────────────────────────┐
│  CALIX       │  Installed  [ Search | Installed | Updates ]        1 911 pkgs │ 44
│  ...         ├────────────────────────────────────────────────────────────────┤
│              │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
│              │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
├──────────────┴────────────────────────────────────────────────────────────────┤
│ ▶ Upgrading linux 6.9.1 → 6.9.2 …                                   220px    │
│  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  │
└───────────────────────────────────────────────────────────────────────────────┘
```

The console opens automatically when an operation starts, from **any** place, and the
place underneath is unchanged. The header is still there, so the user still knows
where they are while an operation runs. The console is docked: it never covers the
content area.

## 9. W7 — The empty states

One per scope. Each names the scope, says why it is empty, and offers the action
that fills it. The header stays in place in all of them.

```
  Search results          Installed            Updates
  ┌──────────────┐        ┌──────────────┐     ┌──────────────┐
  │      ⌕       │        │      ▣       │     │      ↻       │
  │  No results  │        │  Nothing is  │     │  Everything  │
  │  for "ripgr" │        │  installed   │     │  is up to    │
  │              │        │  yet         │     │  date        │
  │  ⌕ search    │        │              │     │              │
  │  packages    │        │  (a machine  │     │  ✓ checked   │
  │              │        │   with 0     │     │    4 min ago  │
  │  Try another │        │   packages)  │     │              │
  │  query       │        │              │     │  ✓ checked   │
  └──────────────┘        └──────────────┘     │    1 hour ago│
                                             └──────────────┘
```

The Updates empty state carries the age of the check, because "no updates" is only
meaningful next to "checked when".

## 10. W8 — The context header, in detail

```
┌────────────────────────────────────────────────────────────────────────────────┐
│  Installed ▾   [ Search | Installed | Updates ]            1 911 pkgs           │ 44
│  ALPM · AUR · Flatpak · AppImage · Cargo · NPM · Release  ·  Flatpak only     │
└────────────────────────────────────────────────────────────────────────────────┘
   ↑ title        ↑ scope switcher (bound to the rail)      ↑ count    ↑ coverage
```

| Element | Rule |
|---|---|
| Title | The active scope or place. Never empty. |
| Scope switcher | Only in the package surface. Bound to the rail (L3). The active target is marked. |
| Count | The number of rows in the current scope. `—` while loading, on error. Hidden in Settings. |
| Coverage | The ecosystems the scope contains. For the Installed scope, the inventory-only ones are marked, per R1. |
| Age | When the scope's answer was true. Only in the Updates scope. |
| Filter summary | Only when a filter is not at its default. Shows the filters and a clear action. |

## 11. W9 — The rail

```
 190px expanded              56px collapsed
┌────────────────┐          ┌────┐
│ CALIX          │          │ ⌕  │   Browse and search
│                │          │ ▣  │   Installed
│  ⌕ Search      │          │ ↻  │   Updates
│  ▣ Installed ● │          │ ▤  │   News
│  ↻ Updates     │          │ ⚙  │   Settings
│  ▤ News        │          │    │
│  ⚙ Settings    │          │    │
│                │          │    │
│  ‹ Collapse    │          │ ›  │
└────────────────┘          └────┘
```

- The active entry is marked with the accent bar and the surface colour, as today.
- The collapsed rail keeps the icons and the active marker; the scope is then carried
  entirely by the context header, which is why the header is not optional.
- `Updates` keeps its count badge when there are pending updates.
- The rail is never a filter. It contains no source pills and no state control.

## 12. State matrix

| Surface | Empty | Loading | Populated | Error | Operation running |
|---|---|---|---|---|---|
| Search results | W7 left | header count `—`, rows dimmed | rows + inspector | header count `—`, inline retry, previous rows kept | console opens, surface unchanged |
| Installed | W7 middle | header count `—` | rows + inspector | header count `—`, inline retry | console opens, surface unchanged |
| Updates | W7 right | header count `—`, "checking…" | rows + inspector | header count `—`, "last checked …" | console opens, surface unchanged |
| News | "no news yet" | skeleton rows | feed | inline retry, feed kept | console opens, feed unchanged |
| Settings | never empty | panel skeleton | panel | inline message, draft kept | console opens, panel unchanged |

Two rules hold in every cell: the header never disappears, and an error never clears
the surface the user was looking at.

The query surface — the search input and the controls row — is present in every scope
of the package surface, in every cell of the table. An error in the Installed scope
leaves the input and the controls in place, so the user can narrow the set that
failed to load.
