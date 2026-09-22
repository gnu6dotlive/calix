# RENDER-04-P0-I01: Window-Scope Shader Module

Status: `COMPILE_BLOCKED`

## Probe Record

```text
PROBE ID: RENDER-04-P0-I01
HYPOTHESIS: A minimal custom shader module can be created through the public
            GPUI Window surface from an isolated Render Lab probe.
IMPLEMENTATION ATTEMPT: A temporary Cargo example called
                        render04_p0_shader_probe passed &mut gpui::Window to
                        window.create_shader_module(...).
COMPILE RESULT: COMPILE_BLOCKED
RUNTIME RESULT: NOT REACHED
OBSERVED CONSTRAINT: gpui::Window has no public create_shader_module method.
ARCHITECTURAL IMPLICATION: The public Window element scope does not expose
                            the attempted shader creation seam. A renderer-
                            scoped or lower-level integration probe is needed
                            before any element-level shader architecture can
                            be justified.
NEXT ITERATION: Do not add an abstraction in response to this error. Inspect
                the concrete GPUI/Blade renderer boundary and choose one
                narrowly scoped renderer-level attempt, or record that the
                public GPUI surface is insufficient for this route.
```

## Exact Evidence

Command:

```text
cargo check --example render04_p0_shader_probe --locked
```

Compiler diagnostic:

```text
error[E0599]: no method named `create_shader_module` found for mutable reference `&mut gpui::Window` in the current scope
  --> examples/render04_p0_shader_probe.rs:10:20
   |
10 |     let _ = window.create_shader_module("render04-p0", "@fragment fn main() {}");
   |                    ^^^^^^^^^^^^^^^^^^^^ method not found in `&mut gpui::Window`
```

The temporary example was deleted after recording the result. No product code,
Render Lab authority, public ABI, or persistence surface was changed.
