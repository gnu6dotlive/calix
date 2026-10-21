# CONTRACT 01 — The spatial contract of Calix

**Version 1.1 — 2026-10-21 — status: ratified (decisions 1–4), in implementation**

Calix is a product. It is not a GPUI experiment that happens to have a
package manager in it. The rules that decide where things are, what gives way
when the space runs short, and what never disappears, belong to **Calix** and
not to the toolkit's incidental behaviour. This document states them.

Two things are deliberate about its shape:

- It states **constraints and priorities**, not a table of universal pixel
  widths. The numbers are qualified in
  [`CONTRACT-01-SPACE-AUDIT.md`](CONTRACT-01-SPACE-AUDIT.md) against the
  windows and the environments Calix is actually supported in.
- Every clause is **testable or it is not here**. A clause nobody can check
  is a wish; a wish in a contract is a lie with a number on it.

**Version 1.1** folds in the operator's four decisions: the window's floor,
the catalogue's minimum, the single owner of the vertical budget, and the
journal drawer's ceiling. The numbers they name are *qualification targets*:
they are measured in the audit, and where the measurement disagrees with a
target, the audit says so and the evidence is kept.

The companion audit confronts this contract with the repository as it stands.
Where the repository disagrees with this document, the audit says so, with the
measurement, and the clause is marked *open* until the gap is closed.

---

## 1. The philosophy: the essential is protected before the secondary

> The layout protects the essential functions before it preserves the
> secondary ones.

The composition of Calix is read as three layers, in this order of priority:

| Priority | Layer | What it holds | What may never happen to it |
|---|---|---|---|
| 1 | **The spine** | the destinations, the search, the primary actions | it may narrow, but it never disappears because the window did |
| 2 | **The body** | the catalogue, the results, their primary actions | it never falls below the width at which it is still a catalogue |
| 3 | **The context surfaces** | the plate, the inspector, the log, the metadata | they give way first, and they give way in a stated order |

A context surface that opens never condemns the body to an unusable column. A
window in portrait never produces a crushed version of a landscape layout. A
zoom of 150 % never simply enlarges a layout that was never designed for it.

## 2. The invariants

| Domain | Invariant |
|---|---|
| Window | the window's dimensions are coherent with what the application can actually compose, at the scale the system asks for |
| Spine | the spine keeps a stable presence; it narrows, it does not vanish |
| Body | the catalogue keeps a usable minimum width and stays the dominant surface |
| Context surfaces | their opening never reduces the body below its floor; on a narrow window they may overlay or close, by a stated policy |
| Portrait and landscape | the composition adapts to the space it is given, not to a declared orientation |
| Zoom | at every supported scale the content stays accessible: no overlap, no loss of an essential action |
| Transitions | a change of width, the opening of a surface, and a re-organisation preserve the visual continuity and the navigation context |
| Extensibility | a new surface uses the same primitives and the same rules; it does not bring its own exception |

## 3. The primitives

These are the shared mechanisms the composition is made of. They are the
*materialisation* of the decisions above, not a second engine of rules: each
one owns one question and answers it in one place.

| Primitive | Owns | Must not |
|---|---|---|
| **Window policy** | the window's floors and ceilings, and what happens when the system refuses them | invent a floor that the composition cannot honour |
| **Adaptive layout** | the division of the space, and the choice of composition | measure a space that is not the one the surface occupies |
| **The spine** | the identity of and the access to the destinations | change its width for a reason that is not the width of the window |
| **The body** | the central surface: it keeps the context and the usable space | let a context surface take what it needs |
| **The context surfaces** | the plate, the inspector, the log, the other secondaries | compete with the body for the space the body needs |
| **Presentation policy** | the transitions, the reduced motion, the zoom, the accessible re-organisation | decide anything about the space |

## 4. The policies

### 4.1 The window's floors

The window's minimum is **derived from the composition's floors**, not chosen
and then hoped for:

```
min_window_width  ≥  spine_floor + body_floor
min_window_height ≥  header + body_floor + context_floor
```

A window may be *technically* resizable to a size that composes into
something unusable. The two levels are therefore **tested separately**: the
window's floor and each component's floor.

### 4.2 The horizontal budget

One place divides the width. The spine takes its width; the body takes the
rest. The body never falls below its floor: below it, the layout changes
shape (§4.4) rather than squeezing the catalogue.

### 4.3 The vertical budget

One place divides the height. The header takes its band, the context surfaces
take their bands from the bottom, and the body takes the rest — and the
body's rest is **what is actually left**, not what the window was before the
other bands took their share. A component that measures the window when it
occupies a band is measuring a space it does not have.

### 4.4 The re-organisation ladder

When the space runs short, the composition re-organises by a **deterministic
ladder**, in this order:

1. the decorative spaces give way (gaps, padding, the ornamental bands);
2. the context surfaces shrink towards their floors;
3. a context surface becomes an overlay, or closes, by a stated policy;
4. the spine narrows to its floor;
5. the body keeps its floor — always.

The spine and the body's floor are **not** on the ladder: they are what the
ladder protects.

### 4.5 Portrait, landscape, zoom

The composition is expressed in **logical pixels** and never reads a declared
orientation. The system's scale is applied by the toolkit, and the window's
floors are floors in logical pixels: at 150 %, the physical window grows so
that the logical composition keeps its room. What the zoom must not do is
enlarge a layout that has no answer for the space it is given.

### 4.6 The presentation policy

The movement is part of the arrangement, not a decoration on it: one law of
the curves, one law of the marks that travel, and a reduced-motion path that
is honoured everywhere. This is stated in the
[acceptance matrix](slice-09/05-ACCEPTANCE-MATRIX.md) and is not repeated
here.

## 5. What this contract does not say

- It does not fix universal pixel widths. The numbers are qualified in the
  audit.
- It does not prescribe a visual style. That is `docs/brand.md` and the
  style documents.
- It does not name the toolkit's behaviours as Calix's. Where a rule comes
  from GPUI today, it is written here in Calix's own words, so that a change
  of toolkit does not change the product.

## 6. How a change is judged

A change to the composition is admissible when it can answer:

1. **Which layer does it serve?** The spine, the body, or a context surface.
2. **What does it make worse when the space runs short?** Nothing may be made
   worse for the body without the ladder of §4.4 saying so.
3. **What is the floor it assumes, and is that floor tested?**
4. **Does it re-use the primitives of §3, or does it introduce its own?**
5. **Is the claim measurable?** If it cannot be measured, it is not a claim.
