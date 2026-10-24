# Calix brand

The Calix mark is a chalice whose cup is a scallop shell — an homage to the shell
that gave Shelly its name. One geometry, two colourways, and the user chooses.

## The two colourways

| Colourway | Setting value | Accent | Hover | Shell |
|---|---|---|---|---|
| Signal Orange | `orange` (default) | `#FF6A00` | `#FF8E40` | Warm light, with `#FFE5D2` as the rim light and `#C95400` as the shadow |
| Royal Blue | `royal` | `#4169E1` | `#87A0F4` | Cool light |

The colours are design system tokens, taken as they are: no re-light, no re-tint, no
shade outside the tokens. The shell's stem is brushed aluminium in both colourways
(`#A1A6AD` → `#667085`) and the tile background is anthracite (`#111418`).

## What the user's choice changes

The setting is `brand-color`, in the appearance section of the settings, and on the
control surface:

```bash
calix-gpui settings set brand-color royal
calix-gpui settings get brand-color
```

It changes:

- **the mark** in the rail header, which is painted by the accent;
- **the accent** of the interface: the buttons, the focus ring, the active row's bar,
  the selected package's marker;
- **the focus ring** (`border_focus`), which follows the accent rather than
  contradicting it.

It does not change: the source badges (ALPM, AUR, Flatpak, AppImage, Cargo, NPM,
Release) — those encode an ecosystem, not the brand — nor the semantic colours
(success, warning, danger), nor the surfaces.

The choice is a draft like every other setting: the accent changes under the
user's finger, and the save commits it.

## The system icon

The launcher icon, the taskbar entry and the window switcher read a *system* file,
so they cannot follow a per-user setting. The package ships the **Signal Orange**
colour master, named after the application id:

```
/usr/share/icons/hicolor/scalable/apps/live.gnu6.calix.svg
/usr/share/icons/hicolor/scalable/apps/live.gnu6.calix-symbolic.svg
/usr/share/icons/hicolor/{16,22,24,32,48,64,128,256,512}x*/apps/live.gnu6.calix.png
```

The in-application mark is the single-path mono master, painted by the accent, so it
follows the user's choice without a second asset.

## Rules

From the logo pack, and they are the rules the application follows:

- The monochrome geometry is a **single closed path** (cup and stem unioned).
- Over the brand colours, use the mono version (anthracite or shell white) or an
  opaque tile. The colour master belongs on a neutral substrate.
- Minimum size: `16px` for a favicon, `24px` for the interface. The rail header
  draws the mark at `24px`.
- Clear space: at least 10% of the mark's height on every side.
- Scale uniformly only. Never skew, never re-typeset.

## The interface: the workshop

The window is an instrument made of the mark's own materials: the brushed
aluminium of the stem and the anthracite of the tile. Calix opens in the
**day workshop** (light); the **night workshop** is a choice in the settings.

| Token | Day | Night | What it is |
|---|---|---|---|
| `bg_sidebar` | `#E7EAEE` | `#111418` | The chassis: aluminium by day, the brand's anthracite tile by night |
| `bg_app` | `#F7F8FA` | `#171B21` | The planes where one reads: the catalogue, the inspector, the settings' sections |
| `bg_surface` | `#FFFFFF` | `#1E232B` | Controls and fields |
| `border` | `#D5DAE0` | `#2B313A` | A plane's edge, a hairline between two facts |
| `text_primary` | `#111418` | `#ECEEF1` | The ink: the brand's anthracite by day |
| `accent_text` | derived | derived | The accent written as text, darkened toward the ink until it reads at 4.5:1 |

Rules the interface keeps:

- **The accent signals, it does not decorate.** It is the cursor (the
  selection's bar), the focus, the primary action, and the lamps of what is
  on. A figure (a size bar) is graphite.
- **Two kinds of surface.** Planes rest on the chassis with an edge and a fine
  shadow, separated by a groove of aluminium that is also their resize handle.
  The dial is the one object of the chassis with a shadow of its own.
- **Lamps for what is on.** A source in the scope, a toggle that is on: a dot
  lit in the accent; off, a ring.
- **Three voices** (`src/fonts.rs`, embedded, OFL): Instrument Sans for the
  interface, its SemiCondensed cut for labels, JetBrains Mono only for what is
  machine text (the log, a PKGBUILD, a command). Figures in columns are
  tabular.
- **Sentence case.** A label is set apart by its narrow cut and its colour,
  never by capitals; meta is joined by its own words ("Arch / extra"), not by
  middle dots.

## Provenance

The pack is a deterministic build: `src/build_calix_logo.py` regenerates the whole
tree from `src/paths.json`, and `src/verify_pack.py` checks it. The geometry is
traced from the selected concept image, mirrored about the vertical axis and
curve-fitted — about 125 cubic segments, 0.78% pixel difference against the source,
that is, edge antialiasing.

The pack's own `SHA256SUMS` lists every file it ships.
