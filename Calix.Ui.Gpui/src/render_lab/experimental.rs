use blade_graphics::{
    ColorTargetState, Context, ContextDesc, Extent, FinishOp, InitOp, Memory, RenderPipelineDesc,
    RenderTarget, RenderTargetSet, ShaderDesc, TextureColor, TextureDesc, TextureDimension,
    TextureFormat, TextureSubresources, TextureUsage, ViewDimension,
};
use gpui::RenderImage;
use image::{Frame, RgbaImage};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

const PROBE_WIDTH: u32 = 8;
const PROBE_HEIGHT: u32 = 8;
const BYTES_PER_PIXEL: usize = 4;
const BYTES_PER_ROW: u32 = PROBE_WIDTH * BYTES_PER_PIXEL as u32;

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
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(position.x / 8.0, position.y / 8.0, 0.25, 1.0);
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderProbeSample {
    pub x: u32,
    pub y: u32,
    pub rgba: [u8; 4],
}

#[derive(Debug)]
pub struct ShaderProbeOutput {
    pub image: Arc<RenderImage>,
    pub samples: [ShaderProbeSample; 3],
    pub bytes_copied: usize,
    pub encode_submit_micros: u128,
    pub completion_readback_micros: u128,
    pub total_micros: u128,
}

static PROBE: OnceLock<Result<ShaderProbeOutput, String>> = OnceLock::new();

pub fn shader_present_probe() -> &'static Result<ShaderProbeOutput, String> {
    PROBE.get_or_init(render_shader_once)
}

fn render_shader_once() -> Result<ShaderProbeOutput, String> {
    let started = Instant::now();
    let context = unsafe {
        Context::init(ContextDesc {
            presentation: false,
            validation: true,
            ..Default::default()
        })
    }
    .map_err(|error| format!("Blade context initialization failed: {error:?}"))?;

    let shader = context
        .try_create_shader(ShaderDesc { source: SHADER })
        .map_err(|error| format!("Blade shader validation failed: {error}"))?;
    let color_targets = [ColorTargetState::from(TextureFormat::Rgba8Unorm)];
    let mut pipeline = context.create_render_pipeline(RenderPipelineDesc {
        name: "render04-p0-i03-uv",
        data_layouts: &[],
        vertex: shader.at("vs_main"),
        vertex_fetches: &[],
        primitive: Default::default(),
        depth_stencil: None,
        fragment: Some(shader.at("fs_main")),
        color_targets: &color_targets,
        multisample_state: Default::default(),
    });

    let extent = Extent {
        width: PROBE_WIDTH,
        height: PROBE_HEIGHT,
        depth: 1,
    };
    let texture = context.create_texture(TextureDesc {
        name: "render04-p0-i03-offscreen",
        format: TextureFormat::Rgba8Unorm,
        size: extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        usage: TextureUsage::TARGET | TextureUsage::COPY,
    });
    let subresources = TextureSubresources {
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    };
    let view = context.create_texture_view(
        texture,
        blade_graphics::TextureViewDesc {
            name: "render04-p0-i03-offscreen-view",
            format: TextureFormat::Rgba8Unorm,
            dimension: ViewDimension::D2,
            subresources: &subresources,
        },
    );
    let readback = context.create_buffer(blade_graphics::BufferDesc {
        name: "render04-p0-i03-readback",
        size: (PROBE_WIDTH * PROBE_HEIGHT * BYTES_PER_PIXEL as u32) as u64,
        memory: Memory::Shared,
    });

    let mut encoder = context.create_command_encoder(blade_graphics::CommandEncoderDesc {
        name: "render04-p0-i03-one-shot",
        buffer_count: 1,
    });
    encoder.start();
    encoder.init_texture(texture);
    {
        let targets = [RenderTarget {
            view,
            init_op: InitOp::Clear(TextureColor::TransparentBlack),
            finish_op: FinishOp::Store,
        }];
        let mut render = encoder.render(
            "render04-p0-i03-draw",
            RenderTargetSet {
                colors: &targets,
                depth_stencil: None,
            },
        );
        let mut bound = render.with(&pipeline);
        bound.draw(0, 3, 0, 1);
    }
    {
        let mut transfer = encoder.transfer("render04-p0-i03-readback");
        transfer.copy_texture_to_buffer(texture.into(), readback.into(), BYTES_PER_ROW, extent);
    }
    let sync = context.submit(&mut encoder);
    let encode_submit_micros = started.elapsed().as_micros();
    if !context.wait_for(&sync, !0) {
        return Err("Blade GPU completion wait failed".into());
    }

    let bytes = unsafe {
        std::slice::from_raw_parts(
            readback.data(),
            (PROBE_WIDTH * PROBE_HEIGHT * BYTES_PER_PIXEL as u32) as usize,
        )
    }
    .to_vec();
    let completion_readback_micros = started.elapsed().as_micros();
    let samples = [
        sample_at(&bytes, 0, 0),
        sample_at(&bytes, PROBE_WIDTH / 2, PROBE_HEIGHT / 2),
        sample_at(&bytes, PROBE_WIDTH - 1, PROBE_HEIGHT - 1),
    ];
    validate_uv_samples(&samples)?;
    let image = RgbaImage::from_raw(PROBE_WIDTH, PROBE_HEIGHT, bytes)
        .ok_or_else(|| "Blade readback did not produce an RGBA image".to_string())?;

    context.destroy_command_encoder(&mut encoder);
    context.destroy_buffer(readback);
    context.destroy_texture_view(view);
    context.destroy_texture(texture);
    context.destroy_render_pipeline(&mut pipeline);

    Ok(ShaderProbeOutput {
        image: Arc::new(RenderImage::new(vec![Frame::new(image)])),
        samples,
        bytes_copied: (PROBE_WIDTH * PROBE_HEIGHT * BYTES_PER_PIXEL as u32) as usize,
        encode_submit_micros,
        completion_readback_micros,
        total_micros: started.elapsed().as_micros(),
    })
}

fn sample_at(bytes: &[u8], x: u32, y: u32) -> ShaderProbeSample {
    let offset = ((y * PROBE_WIDTH + x) * BYTES_PER_PIXEL as u32) as usize;
    ShaderProbeSample {
        x,
        y,
        rgba: [
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ],
    }
}

fn validate_uv_samples(samples: &[ShaderProbeSample; 3]) -> Result<(), String> {
    for sample in samples {
        let expected_red = ((sample.x as f32 + 0.5) / PROBE_WIDTH as f32 * 255.0) as i32;
        let expected_green = ((sample.y as f32 + 0.5) / PROBE_HEIGHT as f32 * 255.0) as i32;
        let red_delta = (sample.rgba[0] as i32 - expected_red).abs();
        let green_delta = (sample.rgba[1] as i32 - expected_green).abs();
        if red_delta > 24 || green_delta > 24 || sample.rgba[2] < 48 || sample.rgba[3] < 240 {
            return Err(format!(
                "UV provenance mismatch at ({}, {}): {:?}, expected approximately ({expected_red}, {expected_green}, 64, 255)",
                sample.x, sample.y, sample.rgba
            ));
        }
    }
    Ok(())
}
