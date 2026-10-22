# Calix v0.3.1

Calix v0.3.1 is the release that **runs on the machine**. The front-end is
ported to the toolkit's current generation, the package states the build flags
it needs instead of inheriting them, and the gate now proves the package's
metadata rather than assuming it. The spatial numbers did not move: where the
documents disagreed with the code, the documents were corrected.

## The port

The front-end now builds on `gpui-pre 0.3.9` — the toolkit's current
generation — with `gpui-pre-platform`, `gpui-pre-linux` and `blade-graphics
0.6`. Twenty-four compilation errors, down to zero, and **no layout
measurement moved**: every number in the bench (the catalogue, the plate, the
row, the compositions) is the number it was before the port.

## The package states its own build flags

`PKGBUILD-calix` inherited three flags from the machine's `makepkg`
configuration, and each one was breaking something. Each was proved by A/B,
not guessed:

- **`-flto=auto`** turned the shim handed to the linker into LTO bytecode:
  `undefined symbol: wl_log_trampoline_to_rust_client`.
- **`-C opt-level=3`** silently disables `debug_assertions`, and the bench
  records bounds only when they are on: 18 of 265 UI tests failed under a
  release profile. With `-C debug-assertions=on`, 265 of 265 pass.
- **`-C target-cpu=native`** rebuilt the same tree with 233 908 AVX2
  instructions where the portable build has 2 238.

The package now declares its own `CFLAGS`/`CXXFLAGS`/`LDFLAGS`/`RUSTFLAGS`,
and the gate refuses all three hazards, from the environment *and* from a
regressed `PKGBUILD` — checked where the flags enter, not by disassembling a
binary afterwards.

## The gate proves the package's metadata

`scripts/verify` used to build the crate and nothing else, while the one path
that ran `makepkg` was the release itself. A packaging defect could therefore
reach a tag without the gate seeing it. The gate now runs `package()` in a
scratch directory and checks the dependency closure — no `makepkg`, no root,
no privilege — and the lockfile's currency is a check before any push that
touches `Cargo.toml`, `Cargo.lock` or the dependency script.

## The plate yields, and never leaves the flow

The plate never comes over the catalogue. When the band is too short it
**yields first**, band by band — body, then actions, then tabs, then the
identity — and it always stays in the flow, so the selection is always
readable. (v0.3.0's notes described the alternative that was declined, an
overlay; the audit had carried it too. Both are corrected.)

## The numbers the documents had missed

The code has implemented a catalogue minimum of **560 px** — the operator's
target — and a window floor of **620 × 520**, since the row's ladder was
built. The contract and the audit still carried the pre-ladder **660 px** and
**720 × 480**, so the two documents that exist to fix the numbers disagreed
with the code. Measured on the real row, in the bench:

| the content | the name's cell | the name's text | version | source | measurement |
|---|---|---|---|---|---|
| 495 px | 165 px | 143 px | 80.5 | 6 | 54.5 |
| **560 px** | **165 px** | **143 px** | 90 | 45 | 71 |
| 620 px | 165 px | 143 px | 99 | 81 | 86 |
| 660 px | 165 px | 143 px | 105 | 105 | 96 |

The name's text is 143 px — the p90 of 1884 installed packages, whole — at
every width from 400 px up: the ladder gives the name its floor, and the
columns that say *where* give way first. The historical numbers are kept, and
so is the case that fails. The contract is now version 1.3 and the audit
version 1.5.

## On the machine

The ported application is installed as the system package
(`calix-gpui-git r4874.gaaa19992-1`, `/usr/lib/calix/calix-gpui`) on native
Wayland, with the daemon and the notifier active — and the operator has seen
it running. That was the last limitation the audit carried about this
release.

## What is not proved

- **The measurement bar at the catalogue's minimum.** Its track is 96 px wide
  inside a cell the ladder narrows, right-justified and clipped: below 660 px
  its *length* no longer encodes the size. At 560 the bar costs the source
  (45 px of 105) and the version (90 of 105) — without it, both are whole and
  the name starts growing at 600 instead of 720. The trade is a product
  decision, and it is open.
- **A 150 % interface scale** is reasoned, not measured: the geometry is
  logical and the scale factor does not leak into it — which *is* measured —
  but the test platform's scale factor is fixed at 2.0.
- **The plate's sheet**, and any gesture that needs a pointer: the bench has
  no pointer. Composition is measured; feel is not.

## What v0.3.0 brought

Calix v0.3.0 was the spatial release: the written contract, the audit that
confronts it with the repository, the workspace owning the vertical budget,
the narrow window authored, the movement measured on the bounds the window
actually draws.

---

# Calix v0.3.0

Calix v0.3.0 is the **spatial** release. The application now has a written
contract for where things are, what gives way when the space runs short, and
what never disappears — and, for the first time, the claims are *measured* on
the real views rather than asserted.

The contract is published in
[`docs/gpui/CONTRACT-01-SPACE.md`](docs/gpui/CONTRACT-01-SPACE.md), and the
audit that confronts it with the repository — with the numbers, and with the
cases that fail — in
[`docs/gpui/CONTRACT-01-SPACE-AUDIT.md`](docs/gpui/CONTRACT-01-SPACE-AUDIT.md).

## The priority, stated and enforced

> The layout protects the essential functions before it preserves the secondary
> ones.

Three layers, in that order of priority: the **spine** (the destinations, the
search, the primary actions), the **body** (the catalogue), and the
**context surfaces** (the plate, the log, the metadata).

Before this release, that priority was *inverted*, and the inversion was
measurable: with the log drawer open, the catalogue fell to **6.5 pixels**
while its own caption kept 260. The workspace now owns the vertical budget —
it computes the space once, from the real allocation and the height of the
chrome, and attributes an explicit height to the station and to the drawer.
No surface computes its usable height from the whole window when it occupies
a part of it. In the same 372-pixel band, the catalogue now has **170.5**.

## The window's floor, derived instead of chosen

The minimum was 1024 × 680 — a number nobody had justified. It is now
**720 × 480**, and it adds up:

```
495  the row's five fixed columns and its padding
+ 22 the identity glyph, its gap
+ 143 the name, for 90 % of the real packages of a real machine
+ 56  the navigation spine, folded
= 716 → 720
```

The catalogue's own minimum is **660 pixels**, and it is measured on the real
row and on the real package names — the p90 of 1884 installed packages is 143
pixels of text. The 560 that was tried first does not hold: at 560 the name
showed six characters. The failing case is kept in the test, not hidden.

*Corrected in v0.3.1: this paragraph measured the row without its ladder. With
the ladder the name is whole at 560, so the catalogue's minimum is 560 and the
window's floor, re-derived from it, is 620 × 520 — see “The numbers the
documents had missed” above.*

## The narrow window, authored

The toolbar's three forms are now three *different* layouts, each measured:
the wide one says what the commands do (*Sort: Relevance*), the middle one
says what they are (*Sort: Rel*), and the narrow one their word alone (*Rel*).
The spine folds itself when the catalogue would lose its room, and takes back
the user's shape when the window gives it back. There is no maximum: the
compositor and the work area decide.

## The row's ladder

The row's five fixed columns cost 495 pixels, and the name is what is left.
The row now gives way in a stated order, expressed in the flex factors so
that the numbers *are* the order: the source gives way four times as fast as
the version, the instrument's reading twice, and the size and the state never
shrink. The name never shrinks, and has a floor of 165 pixels of cell. At 660
and above the row is exactly the row it was; below, it degrades instead of
breaking, and nothing is dropped.

## The plate, when the band is too short

Six rows are a catalogue; two are a strip. The catalogue's vertical floor is
written down, and when the band cannot hold the catalogue *and* the plate's
chrome, the plate leaves the flow and comes **over** the catalogue, summoned
by the same edge that used to resize it. The catalogue takes the band whole.

*Corrected in v0.3.1: this described the alternative that was declined. The
decision is that the plate yields first, band by band, and never comes over
the catalogue.*

## The movement, and how it is measured

The application moves by one law: a **travel** takes the aller-retour curve,
an **entry** takes the out curve, and a mark that travels is one *porteur*
shared by three surfaces — the list's bar, the rail's mark, and the
commutateur's mark.

The entries ignite on the **distance** they have covered, not on the time
they have spent. The out curve is front-loaded — at 35 % of the time it has
already covered 86 % of the path — so an ignition measured in time swallowed
the gesture before anyone could see it. That was why the plate's tab switch
looked like a fade and was not one.

Every entry is now measured on the bounds the window actually drew: the
bank's arrival, the menu's drop, the toast's rise, the plate's cut, and the
commutateur's mark — each one broken on purpose to prove that the
measurement can fail.

## What is not proved

- **A 150 % interface scale** is reasoned, not measured: the layout is in
  logical pixels and the scale factor does not leak into the geometry — which
  *is* measured — but the test platform's scale factor is fixed at 2.0 and
  cannot be set to 1.5.
- **The plate's sheet** is measured in its composition, not in its feel: the
  bench has no pointer.
- **The window's floor is a request.** A compositor that grants 600 × 480
  leaves 544 pixels — under the qualified minimum. The row degrades there
  now; the catalogue's minimum is still 660.

## What v0.2.5 brought, and what v0.2.4 and v0.2.3 fixed

# Calix v0.2.5

Calix v0.2.5 is the navigation release. The application says where it is: a
context header names the scope, counts what it shows, and states which
ecosystems that scope can contain. The filters are in every scope, and they
mean what they say. The search box narrows what you are looking at.

The requalification was contracted before it was written — the contract is
published in [`docs/gpui/slice-07/`](docs/gpui/slice-07/), the implementation
record in [`docs/gpui/slice-08/`](docs/gpui/slice-08/).

## The context header

A permanent header, 48 pixels, between the rail and the surface. It answers the
question the rail never answered: where am I?

- the **scope** — the results, what is installed, what can be updated — and the
  switcher that changes it, bound to the rail: the two always show the same
  value;
- the **count** of the rows the scope shows;
- the **coverage**: the ecosystems the scope can contain, with the three it
  lists but cannot query marked as such — `alpm · aur · flatpak · appimage +
  cargo · npm · release (inventory only)`;
- the **filters in force**, and a way to clear them;
- a source that **did not answer**, with its retry.

## The filters are in every scope

The filters were applied to all three scopes while the controls that changed
them lived in one of them: a source set in Search silently governed Installed
and Updates, in surfaces that could neither show it nor undo it. The search
box, the sources, the state and the sort now stand in every scope, and the
scope's own action beside them — *Upgrade all* in Updates.

The sources are the scope's, not the four the filter knew about. A scope that
lists Cargo, NPM and the tracked releases offers them, marked as the ones it
lists but cannot query — and a Cargo row no longer disappears when ALPM is
switched off, which is what folding every unknown source into ALPM meant.

The state filter offers only what can be true of the rows: *not installed* is
not offered where every row is installed, and a filter that cannot be true of
the rows is withdrawn rather than emptying the list.

## The search narrows the scope

In the results, the query goes to the backend, as it always did. In what is
installed and in what can be updated, the rows are already loaded and the query
reduces them. Searching among the 1914 installed packages is now possible at
all — it was not, before.

## The back path, and the keys

Escape is the local back: it clears the search, then leaves the place, then
empties the selection. It is never a key that does nothing.

| Key | Action |
|---|---|
| `⌘1` … `⌘5` | The five entries of the rail |
| `⌥←` / `⌥→` | The previous and the next scope |
| `⌘F` | The search box |
| `⌘L` | The operation console |
| `⌘⌥C` | The rail |

## The control surface

```bash
calix-gpui scope installed
calix-gpui status --json
```

`status` now reports the place, the scope, the ecosystems the scope can
contain, the inventory-only ones, the filters in force and the row count.
`destination` is kept as a deprecated alias of `scope`, so the automation that
reads it keeps working. The protocol is version 5.

## What v0.2.4 brought, and what v0.2.3 fixed

## The mark, and the colour the user chooses (v0.2.4)

The rail header now shows the shell in a chalice, painted by the accent of the
interface. Two colourways, one geometry, and the choice is the user's:

- **Signal Orange** — `#FF6A00`, hover `#FF8E40` (the default);
- **Royal Blue** — `#4169E1`, hover `#87A0F4`.

The setting is in the appearance section of the settings, and on the control
surface:

```bash
calix-gpui settings set brand-color royal
calix-gpui settings get brand-color
```

The choice changes the mark, the accent (buttons, focus ring, active row) and the
focus ring itself. It does not change the source badges — ALPM, AUR, Flatpak,
AppImage, Cargo, NPM and Release each keep the colour that names their ecosystem.

The launcher, the taskbar and the window switcher read a system file, so they
cannot follow a per-user setting: the package ships the Signal Orange colour
master under the application id, with the raster sizes and the single-colour
variant the menus use.

The colours are design system tokens, taken as they are — no re-light, no
re-tint. The rules the application follows are written in
[`docs/brand.md`](docs/brand.md).

## What v0.2.3 fixed, and what v0.2.2 fixed before it

### The Updates scope never worked

One of the five entries in the rail — **Updates** — answered
*Failed to check for updates: Désérialisation de la liste des mises à jour*
in every released version, with a Retry button that could never succeed.

The cause was a disagreement between the two halves of the product, on the
shape of one payload. The frontend asked the CLI for every backend's updates
and read the answer as a flat list:

```rust
// the frontend, before
serde_json::from_str::<Vec<PackageUpdateItem>>(trimmed)
```

The CLI's `all` variant answers with an **object**, one array per backend:

```json
{"Packages": [], "Aur": [], "AppImage": [], "Flatpak": []}
```

The flat array is what the per-backend variants return
(`calix list-updates standard -j`, `list-updates aur -j`). The frontend asked
for the grouped one and parsed it as the flat one, so the deserialisation
always failed.

The frontend now reads the grouped answer and flattens it. The scope lists
the pending updates, the rail entry carries the count badge, and the two
sides of the agreement are pinned by tests: the CLI's own test already fixed
the shape of its answer, the frontend now parses that exact string, and a
third test parses the answer of the real CLI binary when it is present.

## The layout follows the mapped window

The workstation computed its widths from the size the application *requested*
at launch, not from the size the compositor grants it. A tiling compositor
resizes the window whenever the focus changes, and the request stays the one
from the launch. The result was an inspector of **137px** — below its 320px
floor — in a window of 1056px, and of **1622px** — above its 520px cap — in a
window of 2542px, in the same session.

The workstation now measures the mapped window (`viewport_size`), which is
what the compositor really granted, and the inspector stays inside
`[320, min(520, 3/7·w_c)]` at every width.

## What is in this release

- `calix-gpui`, the GPUI desktop application.
- `calix`, the Zig package-management CLI.
- `calix-daemon`, the per-user inventory service, packaged with a systemd user unit.
- `calix-notify`, the session-bus notification bridge, packaged with its own user unit.
- `libcalix-flatpak-backend.so.1`, the optional Flatpak backend, packaged and shipped.
- The desktop entry, icon and polkit policy under the `live.gnu6.calix` namespace.
- Packaging through `PKGBUILD-calix`; the package is `calix-gpui-git`.
- `pkexec` as the only elevation path. `sudo`, `doas` and `su` are rejected.
- A single-profile CI gate: the Rust tests and release binary share one release build.

## The front-end requalification, and where it stands

The navigation of the application was requalified before it was redrawn. The
contract is published in `docs/gpui/slice-07/`: the current state with code
pointers, the information architecture, the wireframes at three widths,
fifty-three numbered requirements and the acceptance matrix. The
implementation record is in `docs/gpui/slice-08/`.

This release implements it. Two things are not yet proved: the clicks, because
the synthetic pointer events never reached the Wayland client on the machine
where the evidence was taken — the wheel did not scroll and the clicks did not
land — and the focus that a place change should move to the first element of
the new place, because the surfaces do not hold a focus handle for it.

## Desktop notifications

`calix` already emits `live.gnu6.calix.Refresh` on the session bus after an
operation that changed installed state. `calix-notify` is a small bridge that
subscribes to that signal and turns it into one `org.freedesktop.Notifications`
notification. It links libdbus directly: there is no `dbus-monitor` output to
parse and no `notify-send` process in the path.

The bridge is installed but not enabled, like the daemon. Enable it per account:

```bash
systemctl --user enable --now calix-notify.service
systemctl --user status calix-notify.service
```

Two things it deliberately does not do:

- **It never autolaunches a session bus.** It takes `DBUS_SESSION_BUS_ADDRESS`,
  or `XDG_RUNTIME_DIR` when that is absolute, and exits otherwise. A machine with no
  desktop session must not grow a bus because a package was installed.
- **It never fails a package operation.** A missing notification daemon means the
