use crate::theme::Theme;
use crate::visual_style::profile::ColorScheme;
use crate::visual_style::resolver::resolve_style;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::*;

/// Centralized stock-GPUI paint seam for all styled Calix surfaces.
///
/// Applies the deterministic [`SurfacePaintProjection`] for `role` under `style`:
/// - Under Standard: applies `base_bg` with 100% opacity, standard borders, standard shadows.
/// - Under Transparency: applies calibrated surface translucency, content protection scrim floor,
///   specular rim highlight, and elevation contact depth.
pub fn apply_surface_projection<E: Styled>(
    mut div: E,
    role: SurfaceRole,
    style: VisualStyleId,
    base_bg: Rgba,
    theme: &Theme,
) -> E {
    let scheme = ColorScheme::from_dark_theme(theme.is_dark());
    let projection = resolve_style(style, role, scheme);
    let paint = &projection.paint;

    // 1. Surface fill with projected opacity & scrim floor
    let final_bg = if style == VisualStyleId::Standard {
        base_bg
    } else {
        let alpha = (base_bg.a * paint.surface_opacity).clamp(0.0, 1.0);
        let mut r = base_bg.r;
        let mut g = base_bg.g;
        let mut b = base_bg.b;

        // When content scrim is active, blend in a subtle scrim floor for contrast reinforcement
        if paint.content_scrim && paint.scrim_floor_opacity > 0.0 {
            let scrim_target = if scheme == ColorScheme::Dark {
                0.05
            } else {
                0.95
            };
            let t = paint.scrim_floor_opacity * 0.15;
            r = r * (1.0 - t) + scrim_target * t;
            g = g * (1.0 - t) + scrim_target * t;
            b = b * (1.0 - t) + scrim_target * t;
        }

        rgba(
            ((r.clamp(0.0, 1.0) * 255.0) as u32) << 24
                | ((g.clamp(0.0, 1.0) * 255.0) as u32) << 16
                | ((b.clamp(0.0, 1.0) * 255.0) as u32) << 8
                | ((alpha * 255.0) as u32),
        )
    };
    div = div.bg(final_bg);

    // 2. Contact depth & elevation shadow
    if paint.shadow_active {
        let shadow_alpha = (paint.shadow_opacity * 255.0) as u32;
        div = div.shadow(vec![BoxShadow {
            offset: point(px(0.0), px(paint.shadow_offset_y_px)),
            blur_radius: px(paint.shadow_blur_px),
            spread_radius: px(0.0),
            color: rgba(shadow_alpha).into(),
        }]);
    }

    // 3. Specular rim highlight border
    if paint.rim_active {
        let rim_alpha = (paint.rim_opacity * 255.0) as u32;
        let rim_rgb = match scheme {
            ColorScheme::Dark => 0xffffff00,
            ColorScheme::Light => 0x00000000,
        };
        div = div.border_color(rgba(rim_rgb | rim_alpha));
    }

    div
}
