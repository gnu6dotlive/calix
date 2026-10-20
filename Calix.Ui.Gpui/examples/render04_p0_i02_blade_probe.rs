//! RENDER-04-P0-I02: isolated Blade shader/pipeline probe.

use blade_graphics::{ColorTargetState, Context, ContextDesc, RenderPipelineDesc, ShaderDesc};

const SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.12, 0.34, 0.56, 1.0);
}
"#;

fn main() -> anyhow::Result<()> {
    let context = unsafe {
        Context::init(ContextDesc {
            presentation: false,
            validation: true,
            ..Default::default()
        })
        .map_err(|error| anyhow::anyhow!("{error:?}"))?
    };

    let shader = context
        .try_create_shader(ShaderDesc { source: SHADER })
        .map_err(|error| anyhow::anyhow!(error))?;
    let color_targets = [ColorTargetState::from(
        blade_graphics::TextureFormat::Rgba8Unorm,
    )];
    let pipeline = context.create_render_pipeline(RenderPipelineDesc {
        name: "render04-p0-i02-constant-color",
        data_layouts: &[],
        vertex: shader.at("vs_main"),
        vertex_fetches: &[],
        primitive: Default::default(),
        depth_stencil: None,
        fragment: Some(shader.at("fs_main")),
        color_targets: &color_targets,
        multisample_state: Default::default(),
    });

    drop(pipeline);
    println!("PIPELINE_CREATED");
    Ok(())
}
