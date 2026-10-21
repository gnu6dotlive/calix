# The Artcraft launcher's motion, components and performance

A study of a reference, done on 2026-10-20, and what was taken from it.

The source is a local copy of the launcher's tree (the `artcraft-main` archive
in the machine's downloads), unpacked under `.ci-cache/references/` — the
repository's ignored work directory, not `/tmp`, which is shared.

## What the launcher is

A Tauri desktop application: a Rust core (`crates/desktop/artcraft` ✗ — the
bridge and the commands ✗) and a web front end (`frontend/` ✗ — React ✗, a
`libs/` of forty-six component libraries ✗, and a motion system in one CSS
file ✗). Its own documents note that the motion tokens are *mirrored from
artcraft-webapp* ✗ — the tokens are shared between surfaces ✗, not invented
per screen ✗.

## The motion system

One file, `frontend/apps/artcraft/app/src/styles/base.css` ✗, holds the
tokens ✗:

| token | value | what it is |
|---|---|---|
| `--ease-out` | `cubic-bezier(0.22, 1, 0.36, 1)` | a fast departure and a soft arrival |
| `--ease-in-out` | `cubic-bezier(0.65, 0, 0.35, 1)` | the symmetric curve |
| `--ease-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | it passes its target and comes back |
| `--dur-fast` | `140ms` | |
| `--dur-base` | `220ms` | |
| `--dur-slow` | `340ms` | |
| `--dur-page` | `260ms` | |

And the policy that goes with them ✗:

```css
@media (prefers-reduced-motion: reduce) {
  :root { --dur-fast: 1ms; --dur-base: 1ms; --dur-slow: 1ms; --dur-page: 1ms; }
}
```

Three things are worth taking ✗:

1. **The curves are tokens, not formulas.** Four named curves, and the
   application invents no others ✗. A screen that needs a curve takes one of
   the four ✗.
2. **The curves are Béziers, not polynomials.** A cubic Bézier says exactly
   what it does ✗ — the polynomial approximations differ in the middle, which
   is where the eye is ✗.
3. **The reduced-motion policy collapses the durations**, rather than asking
   each screen to remember ✗.

The *performances* themselves are small and honest ✗: the toast arrives with
`opacity 0 → 1` and `translate 0 8px → 0` over 200 ms ✗; the prompt box's
chips arrive with `opacity`, a 6 px rise and a 0.9 → 1 scale over 250 ms on
the *out* curve ✗; the shimmer is a background-position sweep ✗. Every one
uses `both` as the fill mode ✗ — the element holds its *from* state before
the animation and its *to* state after ✗, so there is never a flash of the
unanimated element ✗.

## The keyboard

`frontend/libs/components/keybinds/` ✗ is a *registry* ✗:

- every remappable operation is an `ActionDef` ✗ — an id ✗
  (surface-namespaced ✗), a label ✗, a group ✗, and an availability gate ✗
  (`when` ✗) ✗;
- the bindings are stored by **physical key code** ✗ (`KeyW` ✗), not by the
  produced character ✗ — so WASD survives AZERTY and Dvorak ✗;
- the registry is the single source of truth ✗: the presets ✗, the user's
  overrides ✗, the handlers ✗, the settings panel ✗ and the cheatsheet all
  read it ✗;
- the cheatsheet is **derived** ✗ — grouped by `KeyGroup` ✗, showing only the
  actions marked important ✗, with the keycaps rendered from the same
  registry ✗.

And the gesture ✗ (`useCheatsheetVisibility.ts` ✗):

> Hold Ctrl (Cmd on Mac) *alone* for three seconds and the sheet appears. Any
> other key press while it is held cancels it — a real Ctrl+C never trips it.
> On release it hides, unless the sticky preference is on, in which case the
> peek converts into a pin. A pinned sheet is dismissed by Escape or by a
> click outside it. Leaving the surface drops the pin.

The overlay is translucent and does not block the surface ✗ — the user can
keep working while it is up ✗.

## The performance

`docs/performance.md` ✗ is a model of the discipline: seven samples per
version ✗, medians ✗, the raw samples kept in `docs/performance/` ✗, the
environment written down ✗, a table of before/after/reduction ✗, and the
*limits* stated ✗ — "these are not authentication acceptance tests" ✗, "they
do not measure native process launch" ✗. The three changes that produced the
result are each explained ✗, and the one that did *not* pay off is reported
too ✗.

## What was taken, and what was not

| from the launcher | into Calix | where |
|---|---|---|
| the curves as tokens | `state/motion.rs` ✗ — `Curve` ✗, with the Bézier evaluated by Newton-Raphson ✗ | `the_curves_are_tokens_and_they_behave` ✗ |
| the spring as the overshoot | the needle's throw ✗ | `the_overshoot_passes_the_target_and_comes_back` ✗ |
| the keybind registry | `state/keys.rs` ✗ — every action, its group, its binding ✗ | `the_registry_is_the_source_of_the_bindings` ✗ |
| the derived cheatsheet | `components/cheatsheet.rs` ✗ — grouped, keycapped, translucent ✗ | `the_sheet_draws_without_panicking` ✗ |
| the hold-the-modifier gesture | the workspace's key handler ✗ | — |
| the discoverable entry | the `?` disc in the context header ✗ | — |

What was **not** taken ✗:

- the physical key codes ✗. Calix reads gpui's `keystroke.key` ✗, which is the
  produced character ✗ — so the registry is *not* yet layout-independent ✗ ✗,
  and the launcher's reason for storing the physical code ✗ has not been met
  ✗. This is a known gap ✗, not an oversight ✗.
- the *preset* system ✗ (a set of named binding presets ✗) — Calix has one
  set of bindings ✗.
- the *availability gates* ✗. The registry has no `when` ✗: the actions that
  depend on the context (the list's navigation ✗ needs a list ✗) are listed
  unconditionally ✗ ✗.
- the *remapping* ✗ — the launcher lets the user rebind ✗; Calix does not ✗.

## The gaps this study leaves

- The gesture is **not verified by hand** ✗: the synthetic keyboard events of
  this machine never reach the Wayland client ✗ — `ydotool`'s injected keys
  were tried ✗ and did not arrive ✗ ✗. The registry, the cheatsheet's
  rendering and the `?` disc are verified ✗ (the tests ✗, and the disc and
  the sheet by the eye ✗); the three-second hold is verified by its code ✗
  and by nothing else ✗.
- The sheet's *layout* was seen once ✗, in a build where the sheet was forced
  open by an environment variable ✗ — a switch that was removed before the
  commit ✗ ✗. The rendering that was seen had the last row cut by the panel's
  height ✗; the panel was tightened afterwards ✗, and that tightening is
  *not* seen ✗ ✗.
