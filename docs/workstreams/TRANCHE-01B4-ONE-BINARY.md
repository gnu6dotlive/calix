# 🏺 Calix Workstream — Tranche 1b4: One Binary, One Writer

> **Document Type:** Decision record + autonomous-agent handoff
> **Repository:** `gnu6dotlive/calix` — main worktree `~/cwt`, working copy `~/Projects/calix`
> **Status:** RATIFIED — 2026-10-28, decisions D1–D5 in §4
> **Decision date:** 2026-10-28
> **Extends:** `docs/calix-architecture.md` — Décisions 1, 2, 6, 10, and tranche 1 items 9, 10, 12.
> **Supersedes:** nothing. This is the work tranche 1 already named and did not finish.

---

## 1. What the 2026-10-28 session established

Every row below is a fact produced by a command run against the machine, not a reading of the plan.

| # | Fact | How it was seen | Consequence |
| --- | --- | --- | --- |
| F1 | `~/.local/bin/calix` is a symlink into a git worktree: → `~/Projects/calix/scripts/calix` | `readlink -f "$(command -v calix)"` | The CLI on `PATH` is not the packaged one. `~/.local/bin` precedes `/usr/bin`. |
| F2 | That dispatcher prefers the dev build for `inventory` and for its generic fallback: `$CALIX_HOME/Calix.Cli.Zig/zig-out/bin/calix` before `/usr/lib/calix/calix` | `scripts/calix` lines 191, 207 | `calix inventory` typed in a terminal can execute a branch build. The hashes differ from the installed binary. |
| F3 | `~/.config/systemd/user/calix-sync.service` **and** `.timer` are symlinks into the same worktree: → `~/Projects/calix/packaging/systemd/…` | `ls -l ~/.config/systemd/user/ \| grep -- '->'` | An installed, enabled user unit resolves into a source tree. Move or delete the worktree and the periodic sync stops, silently. |
| F4 | `calix-sync.service` runs `%h/.local/bin/calix sync --quiet --runtime`, i.e. the frozen Python dispatcher, every 4 h | `cat ~/.config/systemd/user/calix-sync.service` | The machine's periodic state pipeline is the Python engine, reached through two symlinks. |
| F5 | Of the six documents that pipeline writes into `~/.local/state/calix/`, **five have no reader at all**: `pacman.json`, `appimages.json`, `cargo.json`, `npm.json`, `status.json` | `grep` across the repository **and** across `~/.config`, `~/.local/bin` | The timer's output is, apart from one file, dead. There is nothing to port. |
| F6 | The one live document is `releases.json`: written by `scripts/calix-sync.py` → `sync_releases()` → `write_release_state()`, read only by `Calix.PackageManager/src/release/tracked.zig` | `scripts/calix_core.py:428`, `calix-sync.py:307-330`, `tracked.zig:1-26` | `tracked.zig` calls itself transitional and says it dies with the release watcher. That watcher is tranche 1b4. |
| F7 | The daemon carries a **second** document for the same subject: `<state>/release-registry.json`, its notification memory, written by nothing else | `grep -rn release-registry.json` → one site, the reader | Two files, two owners, one concept. This is the defect the 2026-10-28 fix exposed but did not remove. |
| F8 | `/usr/bin/calix` → `/usr/lib/calix/calix`, both owned by the package | `pacman -Ql calix-gpui-git` | The packaged binary is correct and complete. Nothing about F1–F4 is a packaging fault. |

### The chain, drawn

```
calix-sync.timer ──symlink──▶ ~/Projects/calix/packaging/systemd/…
        │
        ▼
calix-sync.service ──symlink──▶ ~/Projects/calix/packaging/systemd/…
        │  ExecStart=%h/.local/bin/calix
        ▼
~/.local/bin/calix ──symlink──▶ ~/Projects/calix/scripts/calix   (frozen, "STATUT : GELÉ")
        │
        ▼
scripts/calix-sync.py ──▶  <state>/pacman.json      (no reader)
                            <state>/appimages.json  (no reader)
                            <state>/cargo.json      (no reader)
                            <state>/npm.json        (no reader)
                            <state>/status.json     (no reader)
                            <state>/releases.json   (one reader: tracked.zig)
```

Five of the six outputs are dead. The sixth is the only thing keeping the chain alive, and it is the exact capability tranche 1b4 is for.

**Four of the five links are symlinks into a git worktree.** After S1 the chain survives, in a different shape: systemd keeps the clock, the daemon keeps the pen, and neither is a symlink into a source tree.

---

## 2. Why these five paths, and why in this order

The five paths chosen on 2026-10-28 are not five projects: they are one arc, plus two guards and one tidy-up.

| Chosen path | Where it already lives in the plan | Slice |
| --- | --- | --- |
| The watcher owns the intention | Tranche 1b4; `tracked.zig`'s own header | **S1** |
| One binary | Tranche 1 item 10; Décision 10 | **S2** |
| `calix doctor`, the invariant as a command | New — the guard that makes F1–F4 detectable after the fact | **S3** |
| Doctrine + guard in `verify` | New — the guard that makes F1–F4 impossible to merge | **S0** |
| Hygiene: worktrees and artefacts | New — housekeeping, no defect | **S4** |

**S0 before S1** because a guard is cheap and the work it guards is not. **S1 before S2** because a CLI cannot own a capability that has no single owner yet; porting the release surface onto the Python bridge would build on the thing about to be deleted. **S3 after S2** because the doctor's definition of "correct" is only settled once S1 and S2 decide who writes what. **S4 last** because it is the only slice with no defect behind it.

---

## 3. The slices

Each slice is a ticket, and each ticket's definition of done is the same as the repository's: a red test first, `scripts/verify` green locally, the push going through `pre-push`, the machine updated by `scripts/install-machine`, and a record under `evidence/runs/` with raw logs in `evidence/logs/`.

---

### 1B4-S0 — The dated exception ledger

**Problem.** `~/.local/bin/calix` and the two `calix-sync` units are installed artefacts that resolve into a git worktree. A checker that fails on them today cannot land; a checker that ignores them teaches nothing.

**The mechanism.** A check that knows which violations exist, refuses new ones, and refuses to let the known ones outlive their date.

- `scripts/check-installed-paths` — new. Enumerates the artefacts that *should* be owned by the machine: every file of `pacman -Ql calix-gpui-git`, every entry of `~/.local/bin` named `calix*`, every enabled user unit named `calix*`. For each, resolve the real path and fail if it lands inside a git worktree.
- `packaging/known-installed-path-violations.txt` — the ledger. One line per known violation: the path, and `until=YYYY-MM-DD`.
- `scripts/verify` gains a stage `installed-paths` that runs the check.

**Three rules, all of them enforced:**

1. A violation **not** in the ledger is a failure. New shadowing cannot merge.
2. A ledger entry with **no** live violation is a failure. A fixed entry must be deleted, so the ledger cannot rot into decoration.
3. A ledger entry past its `until` date is a failure, even if the violation is still live. This is the rule that answers Décision 1's own lesson — *« Un pont temporaire sans échéance devient permanent »* — by making the deadline the thing that breaks, not the bridge.

**Initial ledger (to be confirmed at implementation time):**

```
~/.local/bin/calix                                  until=2026-11-15
~/.config/systemd/user/calix-sync.service           until=2026-11-15
~/.config/systemd/user/calix-sync.timer             until=2026-11-15
```

**Steps.** Write the check; add the stage; seed the ledger; prove all three rules bite by manufacturing each case in a scratch directory.

**Proofs.**

- `scripts/verify` runs the new stage, and the stage prints the ledger it is tolerating — toleration must be visible on every run, not buried in a file.
- Three tests, one per rule, each asserting a failure it caused on purpose.
- A clean line in the run record: `installed-paths ok (3 known, until 2026-11-15)`.

---

### 1B4-S1 — The daemon owns the release document

**Problem.** The only live release pipeline is a 4-hourly timer running a frozen Python script through two symlinks (F3–F6), writing one document whose transitional reader says it should not exist. A second document holds the same subject (F7).

**Scope.** One capability, one writer, one document.

- The daemon reads intent from `~/.config/calix/releases.json` (read-only for the daemon).
- The daemon probes the tracked packages, and writes observations and notification state **into one document**, `<state>/releases.json`, schema-versioned.
- `release-registry.json` is retired. Its content is absorbed: `last_seen`, `notified`, `consecutive_failures` join the observation under one schema version.
- `tracked.zig` loses its transitional reader and its transitional note; it reads the daemon's schema.
- The five unread documents are **not ported** — they are deleted from the writer (F5). Porting dead output is how a migration doubles in size for no user.
- `calix-sync.{service,timer}` stay, and change job. Per D1 systemd keeps the clock, so their command becomes a request to the daemon rather than a write, and they are renamed `calix-releases.{service,timer}` in the same transaction — a unit called `sync` that no longer syncs is a lie the next reader has to decode.

**The probe must not block the socket.** The daemon serves connections serially and holds mutable state; a network probe of several forges on that path would make `status` wait on a forge. Ratify D2 before writing code.

**Steps.**

1. Red test: with a scratch state directory and one package in intent, drive the daemon's `releases` action and assert `<state>/releases.json` now exists with schema 2 and one observation. Today the daemon writes `release-registry.json` and `releases.json` stays absent — the test fails for the right reason.
2. Schema: define version 2, write the migration for a version-1 document, and refuse a document from a future version with a named error rather than a silent empty read.
3. The probe runs on a worker thread and commits atomically (D2); requests read the last committed document.
4. Cut the Python writer: `sync_releases()` stops writing `<state>/releases.json`; the five dead writes are deleted outright.
5. Replace the two symlinks with real files, `calix-releases.{service,timer}`, whose `ExecStart` is `/usr/bin/calix releases --refresh` and whose timer keeps `Persistent=true`; disable and remove `calix-sync.*`. The three ledger entries go to zero.
6. `tracked.zig`: delete the transitional reader.

**Proofs.**

- The red test above, then green.
- `grep -rn "release-registry" --include='*.zig' .` returns nothing.
- `grep -rn "pacman.json\|appimages.json\|cargo.json\|npm.json" scripts/` returns nothing.
- On the host: `systemctl --user list-timers` lists `calix-releases.timer` and no `calix-sync.timer`; both unit paths are real files, not symlinks; `~/.local/state/calix/releases.json` still advances, and its `checked_at` moves after a timer firing.
- A daemon that is asked `status` **while** a probe is running answers within the protocol's normal time — measured, not asserted.

---

### 1B4-S2 — One binary

**Problem.** `calix` on `PATH` is a frozen dispatcher (F1) that can execute a branch build (F2), and the release surface (`registry`, `probe`, `check`, `update`, `verify`) lives only there.

**Scope.**

- The Zig CLI gains the release surface. Per D4, `probe` and `check` fold into `releases`; `registry` and `verify` stay distinct because they are distinct capabilities, not conveniences.
- `~/.local/bin/calix` is deleted. `scripts/calix` is deleted with its last caller.
- The `zig-out` preference disappears with the file that contains it.
- The two systemd symlinks were already removed in S1; the ledger's three entries are now empty and are deleted, which is rule 2 doing its job.
- `scripts/verify` gains the second rule of `check-installed-paths`: anything named `calix` that `PATH` resolves must be a file the package owns.

**Steps.** Port the surface; delete the dispatcher; delete the ledger entries; add the second rule; raise the CLI floor to the new count.

**Proofs.**

- `command -v calix` → `/usr/bin/calix`; `readlink -f` → `/usr/lib/calix/calix`.
- `~/.local/bin/calix` and `scripts/calix` no longer exist; `grep -rn "zig-out/bin/calix"` returns nothing.
- The ledger file is empty **and** the check still runs — an empty ledger must not be indistinguishable from a missing one.
- Every subcommand the dispatcher served has a named successor or a recorded decision that it was dropped.

---

### 1B4-S3 — `calix doctor`, the invariant as a command

**Problem.** Everything in §1 was found by hand, over a session, by reading symlinks and grepping consumers. That is not a repeatable diagnostic.

**Scope.** One command, four checks, three exit codes.

| Check | Question |
| --- | --- |
| Package | Is every installed file still the package's file? (`pacman -Qkk` altered count) |
| Running code | Is each service's `/proc/<pid>/exe` a live inode whose hash equals the installed binary? |
| State documents | Does the on-disk schema match the schema the daemon serves? Does intent carry a package with no observation, or an observation with no intent? |
| Resolution | Does `calix` on `PATH` resolve to a file the package owns? |

Exit codes per D5: `0` clean, `1` defects found, `2` not running (nothing broken, the daemon is simply down). The third code matters: a daemon that is stopped is not a broken machine, and a doctor that conflates them trains the operator to ignore it.

**Proofs.**

- Exit `0` on the host as it stands after S2.
- Exit `1` with a **named** finding when one package file is truncated in a scratch copy.
- Exit `2`, and no findings, when the daemon is stopped.
- A test per exit code.

---

### 1B4-S4 — Hygiene

**Problem.** Rollback packages are split across two ignored directories (`r4874`, `r5001` under `~/Projects/calix/.ci-cache/pkg`; `r5003`, `r5005` under `~/cwt/.ci-cache/pkg`), and two worktrees sit on different branches.

**Scope.** One `.ci-cache/pkg` holding every rollback artefact; every branch with an upstream; `git worktree list` showing only worktrees that are in use; `git status --porcelain` empty everywhere.

**Proofs.** The listing, before and after, in the run record. No defect is claimed or fixed here.

---

## 4. Decisions, ratified

Ratified 2026-10-28. All five took the recommendation. D1's recommendation changed during ratification, and the change is recorded rather than quietly applied — a decision record that conceals its own revision is worth nothing.

| # | Question | Recommendation | Why |
| --- | --- | --- | --- |
| D1 | Who schedules the probe: a loop inside the daemon, or a systemd timer? | **systemd keeps the clock; the daemon does the work.** The timer's command is a protocol request, never a write. | An interval loop does not catch up after suspend, and it hides *when* a probe happened from `systemctl`. This file first recommended the opposite — a daemon loop, the timer retired — which was wrong for that reason. The fault that recommendation feared, a timer that writes state, is avoided by making its command a request instead of a write. |
| D2 | Does the probe run on the connection path? | **No: worker thread, atomic commit, requests read the last committed document.** | The daemon serves connections serially. A probe on that path makes `status` wait on a forge, and a forge that hangs makes the daemon look dead. |
| D3 | One document for observation and notification, or two? | **One, schema 2.** | `release-registry.json` and `releases.json` describe the same packages and were confused for each other once already (F7). One document makes the confusion unrepresentable. |
| D4 | Keep `probe` and `check` as commands, or fold them into `releases`? | **Fold them; keep `registry`, `verify`, `update`.** | `probe` is what `releases` does for one package and `check` is what it reports. `verify` and `update` are different acts — one audits provenance, the other changes the machine. Confirm against actual usage before deleting anything. |
| D5 | `calix doctor` exit codes. | **`0` clean / `1` defects / `2` not running.** | It is the convention `browseros-health` already uses on this machine; an operator should not have to learn a second one. |

**Derived from D1 and the anti-goal *No second writer* — a consequence, not a new choice, recorded so nobody re-opens it by accident:**

- `calix releases` (read) works with the daemon down, by reading the committed document. Décision 10's guarantee is about the one-shot read, and it survives intact.
- `calix releases --refresh` requires the daemon. A refresh is a mutation, and mutations belong to the single writer. With the daemon down the command says so and names the way to start it. It does not become a second writer behind a lock file: a lock is an admission that the single-writer rule was not worth designing for.

---

## 5. Anti-goals

Binding, in the manner of `TRANCHE-01-ZIG-ENGINE.md` §3.

- **No `sudo`.** Elevation is `pkexec`, always, per the workstation contract.
- **No second writer.** No process other than the daemon writes a document under the state directory.
- **No second scheduler.** One thing decides when a probe happens.
- **No daemon privilege.** The daemon never escalates; elevation stays a narrow, separate helper.
- **No porting dead output.** A state document that no reader consumes is deleted, not migrated (F5). This is the rule that would have prevented F5.
- **No state document without a named consumer.** A writer ships with its reader, or it does not ship. Record the consumer in the commit that adds the writer.
- **No silent empty read.** A document from a future schema version is an error with a name, not an empty list.
- **No reader outside the daemon.** The frontend reads the daemon; that was Décision 2 and it has already been paid for once.

---

## 6. Definition of done, per slice

1. A test that failed before the change, and the output of it failing, kept.
2. `scripts/verify` green locally, on the commit that is pushed — not on a dirty tree.
3. The push goes through `pre-push`; no GitHub Actions run is used to find out whether something works.
4. The machine is updated by `scripts/install-machine`, never by copying files.
5. `evidence/runs/<slice>/run.json` with the raw logs in `evidence/logs/`.
6. For S0 and S2 specifically: the exception ledger is smaller than it was, and the check still runs.

---

## 7. Known doc drift

Recorded here rather than fixed silently, because a stale blueprint is how a future session repeats this one.

- `docs/workstreams/TRANCHE-01-ZIG-ENGINE.md` §4 draws the frontend reading `~/.local/state/calix/*.json`. Décision 2 of `docs/calix-architecture.md` and tranche 1 item 11 say the opposite, and 1b3 implemented the opposite: `calix_state.rs` is deleted and the UI reads the daemon. The blueprint is wrong; correct it in the S1 commit that retires `release-registry.json`.
- The same document states `Repository: ~/Projects/calix`. Since the 2026-10-27 packaging work, commits and pushes happen in the `main` worktree at `~/cwt`, and `~/Projects/calix` carries a branch. Name both.
- `docs/calix-architecture.md` item 9 says the scheduler "reste à faire" but does not say where it goes. D1 answers that; the answer belongs in the document, not only here.
