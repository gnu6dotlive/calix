# Slice 09 — the requalification of the sources

## 0. What this slice is

The application has two axes, and until now one of them was a filter hidden in
a menu. The seven sources were a list of checkboxes behind a button, in the
same row as the state filter, the sort and the view switcher — four controls
of the same kind, and the user could not tell that one of them chose *where
software comes from* while another chose *what state to show*.

The second design round settled the question of the arrangement. Four blind
authors, four metaphors, one application, and the round's clearest result was
that **three of the four moved the sources out of the left column and into a
band across the top**, and that every one of them prints what each source
*is* next to its count. The Instrument is the arrangement this slice takes:
a milled faceplate read from the top down in full-width bands.

This slice requalifies the sources as an axis, and moves the detail and the
log out of the positions the first round inherited.

## 1. The arrangement

```
┌────────────────────────────────────────────────────────────────────┐
│ identity                          READ · AGE      (?)              │  the header
├────────────────────────────────────────────────────────────────────┤
│ SOURCE ▸ ALPM  AUR  FLATPAK  APPIMAGE  CARGO  NPM  TRACKED         │  the bank
│ seven apertures: the kind, the count, the state                     │
├────────────────────────────────────────────────────────────────────┤
│ SCOPE · PERMITS · FORBIDS    actions    SHOWN / FILTER    [dial]   │  the strip
├────────────────────────────────────────────────────────────────────┤
│ PACKAGE   INSTALLED   AVAILABLE   PROVENANCE   Δ MiB  SIZE  STATE   │  the readout
│ ▏ the list, ruled, the delta measured as bars on a calibrated scale │
├────────────────────────────────────────────────────────────────────┤
│ the specification: the selection, as engraved cells on a plate      │  the nameplate
├────────────────────────────────────────────────────────────────────┤
│ the record: what the backend actually ran                           │  the tape
└────────────────────────────────────────────────────────────────────┘
```

Five bands, read from the top down: what is being read, what it is read
*from*, what it means, and what the machine actually did. No left column, no
right card, no bottom bar.

## 2. The rules

**S1 — The sources are a bank, not a filter.**
The seven sources are a horizontal bank of seven apertures under the header.
Each aperture states the kind of thing the source holds and how many rows it
carries in the current scope. The bank is always present: the user never
hunts for it in a menu. Toggling an aperture keeps the bank in place — a
bank is a control, not a place. Until the bank owns the keyboard alone, the
filters menu keeps the source entries as the narrow fallback; the two are
bound, and they are the same selection.

**S2 — The two axes never share a control, and never look alike.**
The sources are many equal positions: a row of apertures, browsable, each
carrying a fact about itself. The scope is one object with three positions: a
dial, standing on its own in the strip, above the readout it governs. Turning
the dial never changes the source. Choosing a source never changes the
scope. A user must be able to tell them apart with the mouse still.

**S3 — The bank states the non-interchangeability before the user asks.**
Each aperture carries the kind (`binaries`, `recipes`, `sandboxes`, `crates`,
`node packages`, `a single binary`, `a followed binary`) and the count. A
recipe cannot be updated like a binary; a single binary has no dependencies;
an ecosystem the engine does not manage has no command. The reason for an
absent action lives in the source's own entry.

**S4 — The detail is a nameplate, not a card in a column.**
The specification of the selection is a full-width band of engraved cells
under the readout: the version, the provenance, the dependencies, the files,
the product's own command. It is the list's caption. It does not sit in a
column, and it does not float over the rows.

**S5 — The log is a tape.**
What the backend actually ran is a band of perforated lines at the bottom: the
product's own command on one line, what the backend ran on the next. It is
the freest of the elements and it is the one that is allowed to be small.

**S6 — The absence of an action expresses a limit, and the reason is stated.**
When the selection cannot be installed, removed or updated, the reason is
written where the action would have been.

**S7 — The card says where else the name is found.**
A name is not a package: `ripgrep` is a binary in the official repositories, a
recipe in the AUR, a crate on crates.io. The nameplate carries the other
sources where the name is found, with their versions.

**S8 — The keyboard leaves the surface.**
The key map is a help overlay. No key glyph appears on any band.

**S9 — The counts are the scope's, not the query's.**
An aperture counts the rows the current scope carries in that source. The
count follows the scope, the filter and the query; it is never the number of
results of the last search.

**S10 — One region may be empty, on purpose.**
The readout in the quiet state is a calibrated recess with nothing in it. It
reads as glass, not as a missing panel: the grid, the rules and the scale stay
visible.

## 3. What this slice does not do

- It does not change what the backend does, or what the product says about
  the sources. The commands, the scopes and the filters are the Slice-07
  contract.
- It does not remove the source entries from the filters menu yet: they stay
  as the narrow fallback, bound to the same selection as the bank, until the
  bank carries the keyboard on its own.
- It does not redraw the list's rows. The Δ column is measured in the readout
  when the readout is requalified; until then, the list is the list.
- It does not build the rotary. The scope stays a segmented control in the
  strip until the dial's behaviour is specified and proven.
