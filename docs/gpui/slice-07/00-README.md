# Calix GPUI — Slice-07 Documentation Kit

## Target Slice

**SLICE-07 — Information Architecture & Navigation**

This slice establishes the information architecture of the Calix desktop application
before any of it is re-drawn. The application was assembled through live iteration —
slices 01 to 06 and the UX workbench phases each added a surface, a measurement or a
state — and the result is an interface that works but does not say where it is.

The slice delivers the plan the next implementation slice will be held to. **No
application code changes in this slice.** The deliverable is the contract.

---

## The decision this slice records

The IA model was chosen over two alternatives, on the explicit grounds that the
correction must not require a refonte of the data flow or of the control surface:

| Model | Rail | Work surface | Data flow | Verdict |
|---|---|---|---|---|
| **A — stable rail, modes of one surface** | Keeps its five entries; the first three become *scopes* of the package surface | One surface, three scopes, one context header | Unchanged | **Chosen** |
| B — rail of tasks | Three task entries | Library and Activity surfaces | Refonte: inventory, updates, news | Rejected for now |
| C — single workbench | No rail | One surface, settings and news as overlays | Large, breaks the native-workstation principle | Rejected |

Model A is specified in [`02-IA.md`](02-IA.md). The one place where the chosen model's
wording admitted two readings — whether the rail keeps three scope entries or collapses
to a single *Packages* entry — is resolved in §3.4 of that document, with the
alternative recorded so the review can flip it with one edit.

---

## What the baseline capture changed

The contract was first written from the code alone. It was then checked against a live
session: the application was driven through the control surface and every place and
scope was captured, before any change
(`evidence/runs/calix-slice-07-baseline-20261020/`). Two premises of the first draft
were wrong, and the capture is kept as the evidence of both:

- **The query surface is not in three of the five entries, it is in one.** The query
  workbench is rendered for Browse only; the Installed and Updates scopes have a bare
  header. The first draft said otherwise, and the error would have carried into the
  wireframes.
- **The filters act everywhere, and are visible nowhere.** The filter state is applied
  to all three scopes, but the controls that change it exist in one of them. A filter
  set in Browse silently governs Installed and Updates, in surfaces that cannot show
  it.

Two more findings are not about the navigation, but about what the navigation stands
on. Both were fixed in the **v0.2.3** release, before the requalification started
(`evidence/runs/calix-v0.2.3-hotfix-20261020/`):

- **The layout ignored the window it was given.** The workstation's geometry was
  computed from the size the application requested, so the inspector has been measured
  at $137px$ — below its floor — and at $1622px$ — above its cap — in the same
  session. The contract requires the geometry to follow the mapped window (N53).
- **One of the five rail entries was dead.** The **Updates** scope had never loaded,
  in any released version: the frontend read the CLI's grouped updates payload as if
  it were a flat list, so the scope always answered *Failed to check for updates*. The
  contract states the requirement (N33).

---

## Documents in this kit

| Document | What it fixes |
|---|---|
| [`00-README.md`](00-README.md) | This kit: the target slice, the decision, the reading order |
| [`01-CURRENT-STATE.md`](01-CURRENT-STATE.md) | What navigation is today, with code pointers, the findings of the baseline capture, and the five questions it fails to answer |
| [`02-IA.md`](02-IA.md) | The information architecture: places, scopes, queries, filters, naming, the coverage rules |
| [`03-WIREFRAMES.md`](03-WIREFRAMES.md) | The wireframes: the shell, one per surface, at three widths, with the states |
| [`04-NAVIGATION-CONTRACT.md`](04-NAVIGATION-CONTRACT.md) | The behavioural contract: switching, back path, keyboard, control CLI, focus, states, layout |
| [`05-ACCEPTANCE-MATRIX.md`](05-ACCEPTANCE-MATRIX.md) | Every requirement of the slice, with how it will be verified |

## Reading order

`01` establishes the problem and is the shortest read. `02` is the decision document
and the one that needs arbitration. `03` and `04` are the specifications the
implementation is measured against. `05` is the checklist.

## The slice after this one

**SLICE-08 — IA Implementation** implements the contract: the context header, the
scope switcher, the rail bindings, the back path, the scope-aware source filter.
**SLICE-09 — Navigation Evidence** produces the runtime captures that close
`05-ACCEPTANCE-MATRIX.md`.

## Naming note

This kit is Calix. The earlier slice kits (`slice-01` … `slice-06`, `ux-workbench`,
`render-lab`) keep their historical names, as the repository's naming boundary
requires: the old name survives in the historical record only.
