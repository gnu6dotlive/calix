# Local Agent Handoff

You are taking over Shelly GPUI at:

    gnu6dotlive/calix
    main
    e43b03ffa4aaab5d99a151d6a2491370e1f65161

Your authorized task is:

    SLICE-02 — Session Architecture, Workstation Navigation & Unified Search

Read before editing:

    00-README.md
    01-CURRENT-STATE.md
    02-SLICE-02-WORK-CONTRACT.md
    03-ARCHITECTURE-NOTES.md
    04-ACCEPTANCE-MATRIX.md

Important constraints:

- Preserve Shelly upstream semantics.
- Remain on current GPUI unless evidence requires otherwise.
- Do not add gpui-kit.
- Do not add a daemon or database.
- Do not implement DataTable, inspector tabs, or motion architecture.
- Do not expand this into a broad redesign.
- New presentation state should use GPUI's entity/observation model.
- Preserve `uniform_list`.
- Verify real auto-scroll before claiming it.
- Establish stable package identity before unified result merging.
- Empty Browse must not issue fake/default searches.
- Stale async results must never overwrite newer state.
- Capture evidence instead of describing expected performance as measured fact.

Execution order:

1. Establish clean baseline and run existing application.
2. Close console auto-scroll/evidence gaps.
3. Introduce PackageKey.
4. Introduce AppSession / PackageStore entities.
5. Migrate package/detail/search ownership.
6. Add workstation sidebar.
7. Add unified source-aware search.
8. Implement session cache + mutation invalidation.
9. Run acceptance matrix.
10. Document deviations and residual risks.
11. Stop.

If architecture discovered in the repository contradicts this contract, do not
silently improvise.

Document the contradiction and return to Operator for ratification.
