# CI: what runs, and why the container is pinned

## The rule

A push is a claim, not an experiment. Before anything goes to GitHub:

1. `scripts/verify` — the gate the pre-push hook installs.
2. `scripts/ci-replay` — the same gate in `archlinux:latest`, from a clean
   export of only the files git would carry.

`scripts/install-hooks` points this clone at `.githooks/`, so (1) happens
without being remembered. (2) is for changes that touch the toolchain, the
workflow, or anything the host might be quietly providing.

## Why the gate compiles Rust once

`scripts/verify` runs the Rust suite with `cargo test --release --locked` and
then `cargo build --release --locked` in the same target directory. The old
gate ran a debug `cargo check` and `cargo test` first, then compiled the
release profile separately. On a cold runner that compiled GPUI's dependency
graph twice; the release build alone took 14m24s in the last green remote run,
and the debug work added several more minutes.

The workflow caches `~/.cargo`, `Calix.Ui.Gpui/target`, and `~/.cache/zig`,
keyed by `Cargo.lock` and the Zig manifests. The first run after a dependency
change still pays for a cold compile; later runs restore the artifacts.
`Verify` has a 35-minute step timeout and the job a 45-minute timeout, so an
unexpectedly slow run fails instead of consuming hours.

The gate also builds the optional Flatpak backend and checks that the library
is there. The Arch package ships that library and the release artifact carries
it, so a backend that no longer builds would ship a package whose Flatpak
support silently never works. The container installs the Flatpak development
files for the same reason.

## Why the job is pinned to a date

`archlinux:latest` is a moving target, and the job used to install `zig` from
pacman. On 2026-10-12 Arch moved to Zig 0.17, which cannot parse this tree's
`build.zig`:

    error: no field named 'args' in struct 'Build'

Five pushes failed on that alone before it was noticed.

Pinning a Zig binary is not enough. The distro build links the system LLD
(`liblldELF.so.21.1`, `libLLVM.so.21.1`), and that linker is what relocates
the `.sframe` sections a current glibc emits in `crt1.o`. The upstream 0.16.0
tarball ships its own linker and fails, on the same machine that builds the
suite with the distro binary:

    error: fatal linker error: unhandled relocation type R_X86_64_PC64
        note: in .../crt1.o:.sframe

So the reproducible unit is the userland, not the compiler.

`scripts/ci-setup` points pacman at the `2026-10-01` snapshot — the last one
carrying `zig 0.16.0-1` with `clang21 / lld21 / llvm21-libs` — and reconciles
with `-Syyuu`. `-Syu` against a newer image would keep the newer packages and
leave a half-pinned system, which is the state that produced the linker
failure.

## Bumping the snapshot

1. Change `SNAPSHOT` in `scripts/ci-setup`.
2. Run `scripts/ci-replay` and wait for green.
3. Commit the date change on its own, with the `ci-setup` output quoted in the
   message.

The date is the only place the toolchain version lives. The workflow contains
no build commands of its own — bootstrap git, checkout, `ci-setup`, `verify` —
so a local pass and a CI pass cannot mean different things.

## When CI runs

Tags (`v*`), pull requests, and `workflow_dispatch`. Not every push: the
per-commit gate is the local hook, and the replay covers the container. A push
that only re-runs what the hook just ran spends minutes to learn nothing.

`ci.yml` is the only workflow. It owns the tag build and the GitHub release;
the old `release.yml` and `build-and-publish.yml` were deleted because they
referenced removed components and a `master` branch that no longer exists.

## Diagnosing a red run

- `gh repo view --json nameWithOwner` first. With an `upstream` remote
  configured, `gh` will answer for the upstream repository unless
  `gh repo set-default` points it here.
- `gh run view <id> --log-failed` prints only the failing step's log.
- If it is a toolchain error, reproduce with `scripts/ci-replay` before
  changing anything. The failure is almost always the environment, and the
  environment is now one date.
