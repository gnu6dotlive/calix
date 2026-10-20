# 01 — Implementation

What changed, requirement by requirement, against the contract in
[`../slice-07/04-NAVIGATION-CONTRACT.md`](../slice-07/04-NAVIGATION-CONTRACT.md).

## 1. The model: a place, and a scope

`NavDestination` was one enum for two things: a surface (the workstation, the
news, the settings) and a data set (the results, what is installed, what can be
updated). The state now says which is which.

```rust
pub enum Place { Packages, News, Settings }     // a surface
pub enum NavScope { Results, Installed, Updates } // a data set
```

- A place is one at a time, and it is the only thing the rail's last two entries
  change (N1).
- A scope is one at a time, and it belongs to the package surface (N2).
- The render lab is not a place: it opens above the one that is underneath, and
  the place is found again when it closes.
- The rail's first three entries are the scopes; the header's switcher shows the
  same value; the two never disagree (N3, N25-N29).
- Changing a place keeps the scope; the scope belongs to the session, not to the
  place (N7).
- The scope is persisted with the rest of the window state, so the application
  comes back where the user left it (N8, N41, N42).

## 2. The context header

A new component, `components/context_header.rs`, standing between the rail and
the surface. It is present in every place, including while an operation runs and
while a surface is in error (N10).

| Region | What it says | Requirement |
|---|---|---|
| Title | The scope, or the place | N11 |
| Switcher | The three scopes, the active one marked, bound to the rail | N3 |
| Count | The rows the scope shows; `—` while loading | N12 |
| Coverage | The ecosystems the scope can contain, the inventory-only ones marked | N13 |
| Age | When the answer was true | N36 |
| Filters | The filters in force, and a way to clear them | N14 |
| Health | A source that did not answer, and its retry | — |

It is `48px` and it does not scroll (N15). The contract's first draft said
`44px`; the measurement is `48px`, and the contract follows the measurement.

## 3. The query surface in every scope

The filters act on all three scopes. The controls that change them were in one
of them. That is the defect the contract calls F1, and it is closed by putting
the query surface everywhere (N30, N31).

- The search input, the sources, the state and the sort are in every scope.
- The scope's own action sits beside them: *Upgrade all* in Updates.
- The status rail is retired. The count and the filters in force are in the
  header, which is present in every place; a source that did not answer is said
  there, with its retry.

## 4. Filters that mean what they say

**The sources (R1).** `SourceScope` had four flags, and `contains_str` folded
every unknown source into ALPM: a Cargo, NPM or tracked-release row was visible
only while ALPM was selected, and no pill represented them. The scope now has
seven flags, one per ecosystem, and each row answers to its own.

The control offers what the scope can contain: four ecosystems in the results and
in the updates, seven in what is installed — the three inventory-only ones marked
as the ones it lists but cannot query.

**The state (R2).** The impossible states are removed rather than disabled: *not
installed* is not offered where every row is installed. A filter that cannot be
true of the rows is withdrawn when the scope changes, rather than emptying the
list.

**The filters survive (N6, R3).** Changing the scope keeps the filters; what
changes is the vocabulary the controls offer.

**The scope is never emptied (R4).** The last source of the scope stays
selected.

**The query narrows the scope (N18, R6).** In the results, the query goes to the
backend. In what is installed and in the updates, the rows are already loaded
and the query reduces them.

## 5. The back path, and the keys

- Escape clears the search, then leaves the place, then empties the selection
  (N17). It is never a key that does nothing.
- ⌘1 to ⌘5 are the five rail entries; ⌥← and ⌥→ are the previous and the next
  scope; ⌘F focuses the search; ⌘L toggles the operation console; ⌘⌥C
  collapses the rail (N21).
- The keys are modifier-based, so they work while a text field has the focus;
  the text-editing keys stay with the field (N22).
- The focus order is the order of the surfaces: the rail, the header, the query
  surface, the list, the inspector, the console (N23).

## 6. The control surface

`calix-gpui scope results|installed|updates` changes the scope without leaving
the place (N37). The old verbs keep their meaning: `navigate browse` is the
results, `navigate installed` is the installed scope, `navigate news` the news
(N36).

`status --json` reports the place, the scope, the ecosystems the scope can
contain, the inventory-only ones, the filters in force, the row count and the age
of the answer. `destination` is kept as a deprecated alias of `scope`, so the
automation that reads it keeps working. The protocol is version 5 (N38-N40).

## 7. The work of a run

The replay's staging area and the package build were in `/tmp`, which is
memory-backed and shared: three replay runs had left 2.4 GB there, and the test
suite could no longer write its temporary files. The work now lives in the
repository, in the directory the repository ignores (`.ci-cache/`).
