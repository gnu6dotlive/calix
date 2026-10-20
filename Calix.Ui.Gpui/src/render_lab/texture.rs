use gpui::RenderImage;
use image::{Frame, Rgba, RgbaImage};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// SplitMix64: minimal deterministic PRNG for lab texture synthesis.
///
/// Seeded exclusively by lab-assigned fixture seeds. No wall clock, no
/// thread randomness, no global state: identical seed and paint order
/// always produce identical bytes.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u64 << 24) as f32)
    }
}

use serde::{Deserialize, Serialize};

/// Procedural texture family painted into an immutable memory buffer and
/// served to stock GPUI through `img(RenderImage)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextureKind {
    UniformNoise,
    StratifiedJitter,
    FineGrit,
    BrushedMicro,
    AnisotropicGrain,
    Cellular,
    Stipple,
    ValueNoiseLattice,
    HalftoneMesh,
    WovenMatrix,
    CraterRelief,
    EtchedFiber,
    FineDither,
    CoarseGrain,
    BrushedHorizontal,
    BrushedCell,
    WarmPaper,
}

impl TextureKind {
    pub const ALL: &[TextureKind] = &[
        TextureKind::UniformNoise,
        TextureKind::StratifiedJitter,
        TextureKind::FineGrit,
        TextureKind::BrushedMicro,
        TextureKind::AnisotropicGrain,
        TextureKind::Cellular,
        TextureKind::Stipple,
        TextureKind::ValueNoiseLattice,
        TextureKind::HalftoneMesh,
        TextureKind::WovenMatrix,
        TextureKind::CraterRelief,
        TextureKind::EtchedFiber,
        TextureKind::FineDither,
        TextureKind::CoarseGrain,
        TextureKind::BrushedHorizontal,
        TextureKind::BrushedCell,
        TextureKind::WarmPaper,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::UniformNoise => "uniform-noise",
            Self::StratifiedJitter => "stratified-jitter",
            Self::FineGrit => "fine-grit",
            Self::BrushedMicro => "brushed-micro",
            Self::AnisotropicGrain => "anisotropic-grain",
            Self::Cellular => "cellular",
            Self::Stipple => "stipple",
            Self::ValueNoiseLattice => "value-noise-lattice",
            Self::HalftoneMesh => "halftone-mesh",
            Self::WovenMatrix => "woven-matrix",
            Self::CraterRelief => "crater-relief",
            Self::EtchedFiber => "etched-fiber",
            Self::FineDither => "fine-dither",
            Self::CoarseGrain => "coarse-grain",
            Self::BrushedHorizontal => "brushed-horizontal",
            Self::BrushedCell => "brushed-cell",
            Self::WarmPaper => "warm-paper",
        }
    }
}

fn clamp_channel(base: u8, delta: i16) -> u8 {
    (base as i16 + delta).clamp(0, 255) as u8
}

/// Paints uniform grain: `base ± amplitude` per pixel, alpha opaque.
pub fn paint_uniform_noise(seed: u64, w: u32, h: u32, base: [u8; 3], amplitude: u8) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let span = (amplitude as f32) * 2.0 + 1.0;
    let mut buf = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let delta = |rng: &mut SplitMix64| -> i16 {
                ((rng.next_f32() * span) - (amplitude as f32) - 0.5).round() as i16
            };
            buf.put_pixel(
                x,
                y,
                Rgba([
                    clamp_channel(base[0], delta(&mut rng)),
                    clamp_channel(base[1], delta(&mut rng)),
                    clamp_channel(base[2], delta(&mut rng)),
                    255,
                ]),
            );
        }
    }
    buf
}

/// Paints horizontal brushed streaks: one random offset per row plus a
/// small per-pixel jitter, alpha opaque. Pure function of inputs.
pub fn paint_brushed_horizontal(
    seed: u64,
    w: u32,
    h: u32,
    base: [u8; 3],
    streak: u8,
    jitter: u8,
) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let streak_span = (streak as f32) * 2.0 + 1.0;
    let jitter_span = (jitter as f32) * 2.0 + 1.0;
    let mut buf = RgbaImage::new(w, h);
    for y in 0..h {
        let row_delta = ((rng.next_f32() * streak_span) - (streak as f32) - 0.5).round() as i16;
        for x in 0..w {
            let j = ((rng.next_f32() * jitter_span) - (jitter as f32) - 0.5).round() as i16;
            let total = row_delta + j;
            buf.put_pixel(
                x,
                y,
                Rgba([
                    clamp_channel(base[0], total),
                    clamp_channel(base[1], total),
                    clamp_channel(base[2], total),
                    255,
                ]),
            );
        }
    }
    buf
}

/// Paints a brushed-metal cell: streaked flat base with a shaded sphere
/// composited above center. Shading follows the measured vertical profile
/// (dark rim, highlight band, metallic falloff). Pure function of inputs.
pub fn paint_brushed_cell(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut buf = paint_brushed_horizontal(seed, w, h, [192, 192, 194], 16, 5);
    let cx = w as f32 / 2.0;
    let cy = h as f32 * 0.32;
    let radius = (w.min(h) as f32) * 0.26;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > radius {
                continue;
            }
            let t = ((dy / radius) + 1.0) / 2.0;
            let edge = (dist / radius).clamp(0.0, 1.0);
            let rim = if edge > 0.92 { 0.72 } else { 1.0 };
            let band_center = 0.30;
            let band_width = 0.13;
            let band = (-((t - band_center) / band_width).powi(2)).exp();
            let falloff = 1.0 - 0.55 * t;
            let shade = (falloff * rim + band * 0.55).clamp(0.0, 1.35);
            let base_pixel = buf.get_pixel(x, y);
            let lit = |channel: u8| -> u8 { ((channel as f32) * shade).clamp(0.0, 255.0) as u8 };
            buf.put_pixel(
                x,
                y,
                Rgba([
                    lit(base_pixel[0]),
                    lit(base_pixel[1]),
                    lit(base_pixel[2]),
                    255,
                ]),
            );
        }
    }
    buf
}

/// K02: Stratified jitter dots on a grid
fn paint_stratified_jitter(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = RgbaImage::from_pixel(w, h, Rgba([230, 232, 235, 255]));
    let cell_size = 12u32;
    let cols = w / cell_size;
    let rows = h / cell_size;
    for cy in 0..rows {
        for cx in 0..cols {
            let jx = cx * cell_size + 2 + (rng.next_f32() * (cell_size - 4) as f32) as u32;
            let jy = cy * cell_size + 2 + (rng.next_f32() * (cell_size - 4) as f32) as u32;
            let dot_color = if rng.next_f32() > 0.4 {
                Rgba([70, 75, 82, 255])
            } else {
                Rgba([140, 145, 155, 255])
            };
            for dy in 0..2 {
                for dx in 0..2 {
                    if jx + dx < w && jy + dy < h {
                        buf.put_pixel(jx + dx, jy + dy, dot_color);
                    }
                }
            }
        }
    }
    buf
}

/// K03: Fine grit: multi-scale stochastic speckle with dark particle points
fn paint_fine_grit(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = paint_uniform_noise(seed, w, h, [218, 220, 224], 12);
    for y in 0..h {
        for x in 0..w {
            let p = rng.next_f32();
            if p > 0.982 {
                let dark = (40.0 + rng.next_f32() * 60.0) as u8;
                buf.put_pixel(x, y, Rgba([dark, dark, dark + 5, 255]));
            } else if p < 0.015 {
                let bright = (245.0 + rng.next_f32() * 10.0) as u8;
                buf.put_pixel(x, y, Rgba([bright, bright, bright, 255]));
            }
        }
    }
    buf
}

/// K04: Brushed micro: high-frequency horizontal scratch field
fn paint_brushed_micro(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = paint_uniform_noise(seed, w, h, [185, 188, 192], 6);
    let scratch_count = 600;
    for _ in 0..scratch_count {
        let y = (rng.next_f32() * h as f32) as u32;
        let x_start = (rng.next_f32() * w as f32) as u32;
        let len = (15.0 + rng.next_f32() * 90.0) as u32;
        let depth = ((rng.next_f32() - 0.5) * 45.0) as i16;
        for dx in 0..len {
            let x = (x_start + dx).min(w - 1);
            let px = buf.get_pixel(x, y);
            buf.put_pixel(
                x,
                y,
                Rgba([
                    clamp_channel(px[0], depth),
                    clamp_channel(px[1], depth),
                    clamp_channel(px[2], depth),
                    255,
                ]),
            );
        }
    }
    buf
}

/// K05: Anisotropic directional grain
fn paint_anisotropic_grain(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = RgbaImage::new(w, h);
    let mut col_waves = Vec::with_capacity(h as usize);
    for _ in 0..h {
        col_waves.push(((rng.next_f32() - 0.5) * 50.0) as i16);
    }
    for y in 0..h {
        let wave = col_waves[y as usize];
        for x in 0..w {
            let jitter = ((rng.next_f32() - 0.5) * 14.0) as i16;
            let total = wave + jitter;
            buf.put_pixel(
                x,
                y,
                Rgba([
                    clamp_channel(175, total),
                    clamp_channel(178, total),
                    clamp_channel(184, total),
                    255,
                ]),
            );
        }
    }
    buf
}

/// K06: Cellular Voronoi pattern
fn paint_cellular(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let num_points = 70;
    let mut points = Vec::with_capacity(num_points);
    for _ in 0..num_points {
        points.push((rng.next_f32() * w as f32, rng.next_f32() * h as f32));
    }
    let mut buf = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let px = x as f32;
            let py = y as f32;
            let mut d1 = f32::MAX;
            let mut d2 = f32::MAX;
            for &(ox, oy) in &points {
                let d = ((px - ox).powi(2) + (py - oy).powi(2)).sqrt();
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
            let border = d2 - d1;
            let val = (border * 18.0).clamp(40.0, 240.0) as u8;
            buf.put_pixel(x, y, Rgba([val, val + 2, val + 5, 255]));
        }
    }
    buf
}

/// K07: Stochastic stipple dots
fn paint_stipple(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = RgbaImage::from_pixel(w, h, Rgba([236, 238, 240, 255]));
    let dot_count = 3500;
    for _ in 0..dot_count {
        let x = (rng.next_f32() * w as f32) as u32;
        let y = (rng.next_f32() * h as f32) as u32;
        if x < w && y < h {
            let shade = (30.0 + rng.next_f32() * 80.0) as u8;
            buf.put_pixel(x, y, Rgba([shade, shade, shade + 10, 255]));
        }
    }
    buf
}

/// K08: Value noise lattice
fn paint_value_noise_lattice(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let grid_w = 16;
    let grid_h = 16;
    let mut grid = vec![0.0f32; grid_w * grid_h];
    for g in grid.iter_mut() {
        *g = rng.next_f32();
    }
    let mut buf = RgbaImage::new(w, h);
    for y in 0..h {
        let gy = (y as f32 / h as f32) * (grid_h - 1) as f32;
        let iy = gy.floor() as usize;
        let fy = gy - iy as f32;
        let sy = fy * fy * (3.0 - 2.0 * fy);
        for x in 0..w {
            let gx = (x as f32 / w as f32) * (grid_w - 1) as f32;
            let ix = gx.floor() as usize;
            let fx = gx - ix as f32;
            let sx = fx * fx * (3.0 - 2.0 * fx);

            let v00 = grid[iy * grid_w + ix];
            let v10 = grid[iy * grid_w + (ix + 1).min(grid_w - 1)];
            let v01 = grid[(iy + 1).min(grid_h - 1) * grid_w + ix];
            let v11 = grid[(iy + 1).min(grid_h - 1) * grid_w + (ix + 1).min(grid_w - 1)];

            let val =
                (v00 * (1.0 - sx) + v10 * sx) * (1.0 - sy) + (v01 * (1.0 - sx) + v11 * sx) * sy;
            let byte = (val * 160.0 + 60.0) as u8;
            buf.put_pixel(x, y, Rgba([byte, byte + 4, byte + 8, 255]));
        }
    }
    buf
}

/// K09: Halftone mesh
fn paint_halftone_mesh(_seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut buf = RgbaImage::from_pixel(w, h, Rgba([240, 242, 245, 255]));
    let step = 10.0f32;
    let cols = (w as f32 / step).ceil() as usize;
    let rows = (h as f32 / step).ceil() as usize;
    for r in 0..rows {
        for c in 0..cols {
            let cx = c as f32 * step + step / 2.0;
            let cy = r as f32 * step + step / 2.0;
            let dist_from_center =
                ((cx - w as f32 / 2.0).powi(2) + (cy - h as f32 / 2.0).powi(2)).sqrt();
            let radius = 1.0 + (dist_from_center / (w as f32 * 0.4)).clamp(0.0, 1.0) * 3.2;
            for y in (cy - radius - 1.0) as i32..=(cy + radius + 1.0) as i32 {
                for x in (cx - radius - 1.0) as i32..=(cx + radius + 1.0) as i32 {
                    if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                        let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                        if d <= radius {
                            let a = ((radius - d).clamp(0.0, 1.0) * 200.0) as u8;
                            let px = buf.get_pixel_mut(x as u32, y as u32);
                            px[0] = clamp_channel(px[0], -(a as i16));
                            px[1] = clamp_channel(px[1], -(a as i16));
                            px[2] = clamp_channel(px[2], -(a as i16));
                        }
                    }
                }
            }
        }
    }
    buf
}

/// K10: Woven matrix: interlaced thread structure
fn paint_woven_matrix(_seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut buf = RgbaImage::new(w, h);
    let band = 8u32;
    for y in 0..h {
        let ty = (y / band) % 2;
        let py = (y % band) as f32 / band as f32;
        let y_curve = (py * std::f32::consts::PI).sin();
        for x in 0..w {
            let tx = (x / band) % 2;
            let px = (x % band) as f32 / band as f32;
            let x_curve = (px * std::f32::consts::PI).sin();
            let top_is_x = (tx ^ ty) == 0;
            let shade = if top_is_x {
                160.0 + 75.0 * x_curve
            } else {
                130.0 + 75.0 * y_curve
            } as u8;
            buf.put_pixel(x, y, Rgba([shade, shade + 2, shade + 6, 255]));
        }
    }
    buf
}

/// K11: Crater relief: circular pore depressions with directional lighting
fn paint_crater_relief(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = paint_uniform_noise(seed, w, h, [195, 195, 198], 6);
    let crater_count = 140;
    for _ in 0..crater_count {
        let cx = rng.next_f32() * w as f32;
        let cy = rng.next_f32() * h as f32;
        let r = 2.0 + rng.next_f32() * 6.0;
        let depth = 0.4 + rng.next_f32() * 0.5;
        for y in (cy - r - 2.0) as i32..=(cy + r + 2.0) as i32 {
            for x in (cx - r - 2.0) as i32..=(cx + r + 2.0) as i32 {
                if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= r {
                        let nx = dx / r;
                        let ny = dy / r;
                        let sun = -(nx * -0.6 + ny * -0.8);
                        let shade = (sun * depth * 80.0) as i16;
                        let px = buf.get_pixel_mut(x as u32, y as u32);
                        px[0] = clamp_channel(px[0], shade);
                        px[1] = clamp_channel(px[1], shade);
                        px[2] = clamp_channel(px[2], shade);
                    }
                }
            }
        }
    }
    buf
}

/// K12: Etched fiber: curved organic fiber strokes
fn paint_etched_fiber(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = paint_uniform_noise(seed, w, h, [238, 235, 230], 5);
    let fiber_count = 200;
    for _ in 0..fiber_count {
        let mut x = rng.next_f32() * w as f32;
        let mut y = rng.next_f32() * h as f32;
        let len = (10.0 + rng.next_f32() * 30.0) as usize;
        let mut angle = rng.next_f32() * std::f32::consts::TAU;
        let dark = (rng.next_f32() * 50.0 + 30.0) as i16;
        for _ in 0..len {
            if x >= 0.0 && x < w as f32 && y >= 0.0 && y < h as f32 {
                let px = buf.get_pixel_mut(x as u32, y as u32);
                px[0] = clamp_channel(px[0], -dark);
                px[1] = clamp_channel(px[1], -dark);
                px[2] = clamp_channel(px[2], -dark);
            }
            angle += (rng.next_f32() - 0.5) * 0.4;
            x += angle.cos();
            y += angle.sin();
        }
    }
    buf
}

/// K13: Fine dither pattern
fn paint_fine_dither(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = RgbaImage::new(w, h);
    const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    for y in 0..h {
        for x in 0..w {
            let threshold = BAYER[(y % 4) as usize][(x % 4) as usize] as f32 / 16.0;
            let noise = (rng.next_f32() - 0.5) * 0.15;
            let val = if threshold + noise > 0.48 { 220 } else { 165 };
            buf.put_pixel(x, y, Rgba([val, val + 2, val + 5, 255]));
        }
    }
    buf
}

/// K14: Coarse grain
fn paint_coarse_grain(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let block = 4u32;
    let cols = w.div_ceil(block);
    let rows = h.div_ceil(block);
    let mut blocks = vec![0i16; (cols * rows) as usize];
    for b in blocks.iter_mut() {
        *b = ((rng.next_f32() - 0.5) * 44.0) as i16;
    }
    let mut buf = RgbaImage::new(w, h);
    for y in 0..h {
        let by = y / block;
        for x in 0..w {
            let bx = x / block;
            let delta = blocks[(by * cols + bx) as usize];
            buf.put_pixel(
                x,
                y,
                Rgba([
                    clamp_channel(190, delta),
                    clamp_channel(192, delta),
                    clamp_channel(195, delta),
                    255,
                ]),
            );
        }
    }
    buf
}

/// J07: Warm paper fibrous substrate
fn paint_warm_paper(seed: u64, w: u32, h: u32) -> RgbaImage {
    let mut rng = SplitMix64::new(seed);
    let mut buf = paint_uniform_noise(seed, w, h, [246, 240, 228], 8);
    let fibers = 120;
    for _ in 0..fibers {
        let mut x = rng.next_f32() * w as f32;
        let mut y = rng.next_f32() * h as f32;
        let len = (12.0 + rng.next_f32() * 28.0) as usize;
        let mut angle = rng.next_f32() * std::f32::consts::TAU;
        let shade = if rng.next_f32() > 0.5 {
            Rgba([220, 212, 198, 255])
        } else {
            Rgba([255, 252, 245, 255])
        };
        for _ in 0..len {
            if x >= 0.0 && x < w as f32 && y >= 0.0 && y < h as f32 {
                buf.put_pixel(x as u32, y as u32, shade);
            }
            angle += (rng.next_f32() - 0.5) * 0.3;
            x += angle.cos();
            y += angle.sin();
        }
    }
    buf
}

fn paint(kind: TextureKind, seed: u64, w: u32, h: u32) -> RgbaImage {
    match kind {
        TextureKind::UniformNoise => paint_uniform_noise(seed, w, h, [243, 243, 243], 7),
        TextureKind::StratifiedJitter => paint_stratified_jitter(seed, w, h),
        TextureKind::FineGrit => paint_fine_grit(seed, w, h),
        TextureKind::BrushedMicro => paint_brushed_micro(seed, w, h),
        TextureKind::AnisotropicGrain => paint_anisotropic_grain(seed, w, h),
        TextureKind::Cellular => paint_cellular(seed, w, h),
        TextureKind::Stipple => paint_stipple(seed, w, h),
        TextureKind::ValueNoiseLattice => paint_value_noise_lattice(seed, w, h),
        TextureKind::HalftoneMesh => paint_halftone_mesh(seed, w, h),
        TextureKind::WovenMatrix => paint_woven_matrix(seed, w, h),
        TextureKind::CraterRelief => paint_crater_relief(seed, w, h),
        TextureKind::EtchedFiber => paint_etched_fiber(seed, w, h),
        TextureKind::FineDither => paint_fine_dither(seed, w, h),
        TextureKind::CoarseGrain => paint_coarse_grain(seed, w, h),
        TextureKind::BrushedHorizontal => {
            paint_brushed_horizontal(seed, w, h, [176, 178, 182], 16, 5)
        }
        TextureKind::BrushedCell => paint_brushed_cell(seed, w, h),
        TextureKind::WarmPaper => paint_warm_paper(seed, w, h),
    }
}

type TextureKey = (TextureKind, u64, u32, u32);

static TEXTURE_CACHE: OnceLock<Mutex<HashMap<TextureKey, std::sync::Arc<RenderImage>>>> =
    OnceLock::new();

fn cache() -> &'static Mutex<HashMap<TextureKey, std::sync::Arc<RenderImage>>> {
    TEXTURE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Returns the stock-GPUI render image for a procedural texture, generating
/// and caching it on first request. Same inputs always yield the same `Arc`
/// (no per-frame texture churn); pixel bytes are a pure function of inputs.
pub fn texture_for(kind: TextureKind, seed: u64, w: u32, h: u32) -> std::sync::Arc<RenderImage> {
    let key = (kind, seed, w, h);
    let guard = cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(existing) = guard.get(&key) {
        return existing.clone();
    }
    drop(guard);
    let buffer = paint(kind, seed, w, h);
    let frame = Frame::new(buffer);
    let rendered = std::sync::Arc::new(RenderImage::new(vec![frame]));
    let mut guard = cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.insert(key, rendered.clone());
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_splitmix_deterministic_stream() {
        let mut a = SplitMix64::new(5001);
        let mut b = SplitMix64::new(5001);
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut c = SplitMix64::new(5002);
        assert_ne!(a.next_u64(), c.next_u64());
        let mut d = SplitMix64::new(7);
        for _ in 0..16 {
            let v = d.next_f32();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn test_texture_kinds_named() {
        assert_eq!(TextureKind::ALL.len(), 17);
        assert_eq!(TextureKind::UniformNoise.as_str(), "uniform-noise");
        assert_eq!(
            TextureKind::BrushedHorizontal.as_str(),
            "brushed-horizontal"
        );
        assert_eq!(TextureKind::BrushedCell.as_str(), "brushed-cell");
        assert_eq!(TextureKind::WarmPaper.as_str(), "warm-paper");
    }

    #[test]
    fn test_paint_bytes_deterministic_and_seeded() {
        let first = paint_uniform_noise(5001, 64, 64, [168, 168, 172], 14);
        let second = paint_uniform_noise(5001, 64, 64, [168, 168, 172], 14);
        assert_eq!(first.as_raw(), second.as_raw());
        let other_seed = paint_uniform_noise(5002, 64, 64, [168, 168, 172], 14);
        assert_ne!(first.as_raw(), other_seed.as_raw());

        let brushed_a = paint_brushed_horizontal(4005, 64, 64, [176, 178, 182], 16, 5);
        let brushed_b = paint_brushed_horizontal(4005, 64, 64, [176, 178, 182], 16, 5);
        assert_eq!(brushed_a.as_raw(), brushed_b.as_raw());

        let cell_a = paint_brushed_cell(4005, 64, 64);
        let cell_b = paint_brushed_cell(4005, 64, 64);
        assert_eq!(cell_a.as_raw(), cell_b.as_raw());
        let flat = paint_brushed_horizontal(4005, 64, 64, [192, 192, 194], 16, 5);
        assert_ne!(cell_a.as_raw(), flat.as_raw());
    }

    #[test]
    fn test_paint_has_variance_and_opaque_alpha() {
        let buf = paint_uniform_noise(5001, 48, 48, [168, 168, 172], 14);
        let mut distinct = std::collections::HashSet::new();
        for pixel in buf.pixels() {
            assert_eq!(pixel[3], 255);
            distinct.insert([pixel[0], pixel[1], pixel[2]]);
        }
        assert!(
            distinct.len() > 64,
            "grain must vary, got {}",
            distinct.len()
        );

        let brushed = paint_brushed_horizontal(4005, 48, 48, [176, 178, 182], 16, 5);
        let row0: Vec<u8> = (0..48).map(|x| brushed.get_pixel(x, 0)[0]).collect();
        let row0_spread = row0.iter().max().unwrap() - row0.iter().min().unwrap();
        assert!(row0_spread > 0, "brushed rows must carry jitter");
    }

    #[test]
    fn test_texture_cache_returns_stable_arc() {
        let a = texture_for(TextureKind::UniformNoise, 9001, 32, 32);
        let b = texture_for(TextureKind::UniformNoise, 9001, 32, 32);
        assert!(std::sync::Arc::ptr_eq(&a, &b));
        let c = texture_for(TextureKind::UniformNoise, 9002, 32, 32);
        assert!(!std::sync::Arc::ptr_eq(&a, &c));
    }
}
