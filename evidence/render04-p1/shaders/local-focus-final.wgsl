// B2/final: tightened the core and reduced peak intensity.
// Runtime parameters: center=(0.5,0.5), inner=0.08, outer=0.55, intensity=0.72.
struct LocalFocusParams {
    bounds: vec4<f32>,
    focus: vec4<f32>,
    color: vec4<f32>,
}

var<uniform> params: LocalFocusParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return VertexOutput(vec4<f32>(positions[vertex_index], 0.0, 1.0));
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = (position.xy - params.bounds.xy) / params.bounds.zw;
    let aspect = params.bounds.z / params.bounds.w;
    let local = (uv - params.focus.xy) * vec2<f32>(aspect, 1.0);
    let distance = length(local);
    let focus = 1.0 - smoothstep(params.focus.z, params.focus.w, distance);
    let color = params.color.yzw + vec3<f32>(focus * params.color.x);
    return vec4<f32>(color, 1.0);
}
