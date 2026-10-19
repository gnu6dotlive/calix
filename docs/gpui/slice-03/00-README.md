# Shelly GPUI — Slice-03 Documentation Kit

## Target Slice

**SLICE-03 — Package Surface & Semantic Inspector**

This slice follows the Slice-02 foundation (`AppSession`, `PackageStore`, `ConsoleModel`, `PackageKey`, workstation navigation, unified search, source-aware caching) and elevates the package workstation into a production-grade working surface.

It purposefully delivers:

1. **Dual virtualized package surface**:
   - High-density **Table mode** (`PackageViewMode::Table`) with a fixed 32px sticky header and 36px fixed-height rows (`NAME`, `VERSION`, `SOURCE`, `SIZE`, `STATUS`).
   - Rich **Cards mode** (`PackageViewMode::Cards`) with 78px container and 72px package card.
   - Seamless view switcher pill toggle (`▦ Cards` / `☰ Table`) in the search filter bar, sharing identical `PackageStore` results and `PackageKey` selection invariants.
2. **Decomposed 3-tab semantic inspector**:
   - Hero header (`InspectorHeader`) with package name, version, badges (source, status, repository), capability-driven action buttons (Install/Remove/Update), canonical install command copy with visual feedback, and tab switcher bar.
   - Discrete tab views:
     - `Overview`: Concise metadata grid (repository, licenses, installed/download sizes, upstream URL, packager, install reason).
     - `Dependencies`: Typed dependency groups (`runtime`, `make`, `opt`, `required_by`) with parsed interactive `DependencyRef` pills.
     - `Files & Build`: Capability-driven build/files view providing live AUR `PKGBUILD` scripts via `shelly search aur <pkg> -p`, AppImage paths and update URLs, ALPM build dates and honest capability notices, and Flatpak application IDs.
3. **Typed semantic domain model**:
   - `DependencyRef` with robust string parsing (`name`, `constraint`, `description`) replacing ad-hoc string slicing.
   - `SemanticTarget`: `Package`, `ExternalUrl` (`cx.open_url()`), `FilePath`, `CopyText` (`cx.write_to_clipboard()`).
   - `PackageCapabilities::derive(pkg, alpm_detail)` driving UI actions and inspection tabs.
   - `canonical_install_command(pkg)` generating CLI commands verified against Shelly upstream authority.
4. **Backend authority boundary & zero deadcode compliance**:
   - Strict adherence to Seafoam-Labs/Shelly-ALPM upstream authority without raw pacman bypasses.
   - Hard compiler enforcement (`#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]`).
   - English language normalization across all UI strings.

---

## Repository Baseline

Repository:

    tension-atoi/calix

Canonical branch:

    main

Frontend dependency:

    gpui = "0.2.2"

Functional upstream authority:

    Seafoam-Labs/Shelly-ALPM

---

## Documentation Kit Map

- [`00-README.md`](00-README.md): High-level slice scope, primitives, and repository baseline.
- [`01-CURRENT-STATE.md`](01-CURRENT-STATE.md): Architectural snapshot and component inventory.
- [`02-WORK-CONTRACT.md`](02-WORK-CONTRACT.md): Detailed work contract commitments, constraints, and scope boundaries.
- [`03-SEMANTIC-MODEL.md`](03-SEMANTIC-MODEL.md): Formal specification of semantic types, capabilities, and command generation.
- [`04-ACCEPTANCE-MATRIX.md`](04-ACCEPTANCE-MATRIX.md): Detailed verification matrix mapping specifications to evidence classes.
- [`05-ACCEPTANCE-REPORT.md`](05-ACCEPTANCE-REPORT.md): Final acceptance signoff with empirical test telemetry and native Wayland runtime evidence.
