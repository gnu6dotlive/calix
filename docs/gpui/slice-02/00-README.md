# Shelly GPUI — Next Version Documentation Kit

## Target slice

**SLICE-02 — Session Architecture, Workstation Navigation & Unified Search**

This slice follows the first performance/console hardening work and establishes
the application-state architecture required before the larger visual redesign.

It deliberately does NOT implement:

- the package DataTable;
- multi-tab package inspector;
- large motion system;
- toast framework;
- gpui-kit migration;
- persistent daemon;
- persistent database/cache;
- agent-aware UI;
- final localization pass.

The objective is narrower:

1. close the remaining Phase-01 correctness/evidence gaps;
2. remove package/session ownership from the monolithic `WorkspaceView`;
3. establish GPUI-native reactive state boundaries;
4. replace the current horizontal package-source tabs with a workstation sidebar;
5. introduce one coherent, source-aware search model;
6. preserve the existing virtualized package surface and all package operations.

This is the architectural foundation for the future Shelly GPUI workstation.

---

## Repository baseline

Repository:

    tension-atoi/calix

Canonical branch:

    main

Baseline HEAD when this kit was prepared:

    e43b03ffa4aaab5d99a151d6a2491370e1f65161

Frontend dependency:

    gpui = "0.2.2"

Upstream authority:

    Seafoam-Labs/Shelly-ALPM

Shelly GPUI remains a downstream GPUI frontend/product layer around Shelly.
Do not silently turn this slice into an independent package-manager fork.

---

## Current GPUI guidance

Current GPUI documentation confirms the intended primitives:

- `Entity<T>` for retained application state;
- `Context<T>` for entity-local updates;
- `cx.notify()` for observable state invalidation;
- `cx.observe()` for state observation;
- `cx.subscribe()` / `cx.emit()` for typed cross-entity events;
- `uniform_list()` for viewport-lazy rendering;
- `UniformListScrollHandle` for virtual-list position control;
- frame/animation APIs such as `request_animation_frame`,
  `on_next_frame`, and `with_animation`.

Slice-02 should use the entity/event model.

It should NOT introduce animation architecture yet merely because the framework
supports it.
