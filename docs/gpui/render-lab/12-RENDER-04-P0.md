# RENDER-04-P0 Experimental Infrastructure Probe

Status: active experimental contract.

## Chosen Fixture

`optical.local-focus` (`i06-central-luminance-boost/r1`). It is small enough to probe in isolation and exposes a useful comparison between a stock layered composition and a custom radial luminance effect.

## Probe A: Stock GPUI

- Use the existing Render Lab recipe/compiler path.
- Treat the current stock composition as the baseline implementation.
- Record fidelity, recipe structure, and any runtime/capture cost only as secondary context.

## Probe B: Minimal Shader

- Attempt one custom fragment effect that computes a radial luminance boost from normalized fragment coordinates.
- Keep the attempt lab-only and local to Render Lab experiment code.
- Do not introduce a general shader ABI, shader manager, persistence, configuration, public API, product dependency, GPUI fork, or new renderer.
- The first attempt may fail to compile or run. Such failure is evidence and must be recorded before architectural expansion.

## Isolation Boundary

- Expected files are limited to this contract, an experiment-local probe module or fixture wiring, and an evidence record.
- No product views, product state, persistence authority, package contracts, or public ABI may depend on the probe.
- The probe must not become the canonical recipe implementation.

## Evidence Protocol

Record for each attempt:

```text
PROBE ID
HYPOTHESIS
IMPLEMENTATION ATTEMPT
COMPILE RESULT
RUNTIME RESULT
OBSERVED CONSTRAINT
ARCHITECTURAL IMPLICATION
NEXT ITERATION
```

The first compile error, if any, is retained with the smallest complete diagnostic that explains the constraint. A successful compile must be followed by a real runtime attempt before claiming the pipeline works.

## Stop Conditions

Stop and record the result before adding abstractions when:

- GPUI does not expose the required renderer/resource hook.
- The proposed resource lifetime or ownership cannot be expressed at the attempted scope.
- Pipeline creation, surface integration, format compatibility, or frame scheduling fails.
- Continuing would require product wiring, persistence authority, a public ABI, a GPUI fork, or a second abstraction layer.

## Intended Learning

Probe A teaches the stock composition cost and behavior. Probe B teaches whether Shelly-GPUI can host one real custom GPU effect, where its resources must live, how parameters enter the frame lifecycle, and which constraints are compile-time versus runtime. Neither probe is production authority.
