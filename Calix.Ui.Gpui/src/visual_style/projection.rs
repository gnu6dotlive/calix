use crate::visual_style::profile::{ColorScheme, VisualStyleId};
use crate::visual_style::roles::{SurfaceRole, SurfaceTreatment};
use serde::{Deserialize, Serialize};

/// Deterministic paint magnitudes for a resolved surface role.
///
/// Under [`VisualStyleId::Standard`], opacity is 1.0 (opaque),
/// rim is inactive, and contact depth is only active for elevated surfaces.
/// Under [`VisualStyleId::Transparency`], treatments project calibrated
/// surface opacity, content-protection scrim floor, specular rim highlight,
/// and elevation contact depth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfacePaintProjection {
    pub surface_opacity: f32,
    pub content_scrim: bool,
    pub scrim_floor_opacity: f32,
    pub rim_active: bool,
    pub rim_opacity: f32,
    pub shadow_active: bool,
    pub shadow_blur_px: f32,
    pub shadow_offset_y_px: f32,
    pub shadow_opacity: f32,
    pub microstructure_active: bool,
}

impl SurfacePaintProjection {
    pub fn compute(style: VisualStyleId, role: SurfaceRole, scheme: ColorScheme) -> Self {
        let treatment = role.treatment();
        match style {
            VisualStyleId::Standard => Self {
                surface_opacity: 1.0,
                content_scrim: false,
                scrim_floor_opacity: 0.0,
                rim_active: false,
                rim_opacity: 0.0,
                shadow_active: treatment == SurfaceTreatment::ElevatedSurface,
                shadow_blur_px: if treatment == SurfaceTreatment::ElevatedSurface {
                    16.0
                } else {
                    0.0
                },
                shadow_offset_y_px: if treatment == SurfaceTreatment::ElevatedSurface {
                    8.0
                } else {
                    0.0
                },
                shadow_opacity: if treatment == SurfaceTreatment::ElevatedSurface {
                    0.40
                } else {
                    0.0
                },
                microstructure_active: false,
            },
            VisualStyleId::Transparency => {
                let is_dark = scheme == ColorScheme::Dark;
                let protected = role.requires_content_protection();
                match treatment {
                    SurfaceTreatment::AmbientChrome => Self {
                        surface_opacity: if is_dark { 0.80 } else { 0.84 },
                        content_scrim: protected,
                        scrim_floor_opacity: if protected {
                            if is_dark {
                                0.15
                            } else {
                                0.20
                            }
                        } else {
                            0.0
                        },
                        rim_active: true,
                        rim_opacity: if is_dark { 0.12 } else { 0.35 },
                        shadow_active: true,
                        shadow_blur_px: 8.0,
                        shadow_offset_y_px: 0.0,
                        shadow_opacity: if is_dark { 0.25 } else { 0.15 },
                        microstructure_active: false,
                    },
                    SurfaceTreatment::InteractiveChrome => Self {
                        surface_opacity: if is_dark { 0.88 } else { 0.90 },
                        content_scrim: protected,
                        scrim_floor_opacity: if protected {
                            if is_dark {
                                0.25
                            } else {
                                0.30
                            }
                        } else {
                            0.0
                        },
                        rim_active: true,
                        rim_opacity: if is_dark { 0.15 } else { 0.40 },
                        shadow_active: false,
                        shadow_blur_px: 0.0,
                        shadow_offset_y_px: 0.0,
                        shadow_opacity: 0.0,
                        microstructure_active: false,
                    },
                    SurfaceTreatment::ContentPlane => Self {
                        surface_opacity: if is_dark { 0.94 } else { 0.96 },
                        content_scrim: protected,
                        scrim_floor_opacity: if protected {
                            if is_dark {
                                0.40
                            } else {
                                0.45
                            }
                        } else {
                            0.0
                        },
                        rim_active: false,
                        rim_opacity: 0.0,
                        shadow_active: false,
                        shadow_blur_px: 0.0,
                        shadow_offset_y_px: 0.0,
                        shadow_opacity: 0.0,
                        microstructure_active: false,
                    },
                    SurfaceTreatment::ElevatedSurface => Self {
                        surface_opacity: if is_dark { 0.92 } else { 0.94 },
                        content_scrim: protected,
                        scrim_floor_opacity: if protected {
                            if is_dark {
                                0.30
                            } else {
                                0.35
                            }
                        } else {
                            0.0
                        },
                        rim_active: true,
                        rim_opacity: if is_dark { 0.20 } else { 0.50 },
                        shadow_active: true,
                        shadow_blur_px: 20.0,
                        shadow_offset_y_px: 10.0,
                        shadow_opacity: if is_dark { 0.50 } else { 0.25 },
                        microstructure_active: false,
                    },
                }
            }
        }
    }
}
