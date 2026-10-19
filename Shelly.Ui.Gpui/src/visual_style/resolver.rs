use crate::render_lab::CapabilityClass;
use crate::visual_style::profile::{ColorScheme, VisualStyleId, VisualStyleRegistry};
use crate::visual_style::projection::SurfacePaintProjection;
use crate::visual_style::roles::{SurfaceRole, SurfaceTreatment};
use serde::{Deserialize, Serialize};

/// Semantic effect requested by a visual style profile.
///
/// A style never names a concrete shader or backend primitive. It requests
/// an intention; the resolver reports how (and whether) the stock backend
/// can satisfy it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffectKind {
    BackdropBlur,
    Microstructure,
    RimResponse,
    ContactDepth,
}

impl EffectKind {
    pub const ALL: &[EffectKind] = &[
        EffectKind::BackdropBlur,
        EffectKind::Microstructure,
        EffectKind::RimResponse,
        EffectKind::ContactDepth,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackdropBlur => "backdrop-blur",
            Self::Microstructure => "microstructure",
            Self::RimResponse => "rim-response",
            Self::ContactDepth => "contact-depth",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BackdropBlur => "Backdrop Blur",
            Self::Microstructure => "Microstructure",
            Self::RimResponse => "Rim Response",
            Self::ContactDepth => "Contact Depth",
        }
    }
}

impl std::fmt::Display for EffectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Honest per-effect capability report for the stock GPUI backend.
///
/// In STYLE-00B, effects reflect actual empirical confrontation from RENDER-01/02:
/// - ContactDepth: Proven NATIVE via stock BoxShadow (depth.contact-shadow)
/// - RimResponse: Proven NATIVE via stock perimeter borders (depth.rim-highlight-outline)
/// - Microstructure: Proven TEXTURE_PROOF via deterministic immutable memory textures (Board K)
/// - BackdropBlur: Remains UNKNOWN (per-surface backdrop blur unproven in stock GPUI 0.2.2)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectResolution {
    pub kind: EffectKind,
    pub requested: bool,
    pub resolved: CapabilityClass,
    pub note: &'static str,
}

impl EffectResolution {
    pub fn resolve(kind: EffectKind) -> Self {
        let (resolved, note) = match kind {
            EffectKind::BackdropBlur => (
                CapabilityClass::Unknown,
                "Stock window-level appearance exists, but per-surface backdrop blur is unproven in stock GPUI 0.2.2; active=false",
            ),
            EffectKind::Microstructure => (
                CapabilityClass::TextureProof,
                "Proven TextureProof in RENDER-01/02 across 16 Board K/J fixtures via deterministic immutable textures and RenderImage",
            ),
            EffectKind::RimResponse => (
                CapabilityClass::Native,
                "Proven Native in RENDER-01/02 via stock perimeter borders and directional rim highlights (depth.rim-highlight-outline)",
            ),
            EffectKind::ContactDepth => (
                CapabilityClass::Native,
                "Proven Native in RENDER-01/02 via stock BoxShadow and directional borders (depth.contact-shadow / h01)",
            ),
        };
        Self {
            kind,
            requested: true,
            resolved,
            note,
        }
    }

    pub fn is_active(&self) -> bool {
        self.requested && self.resolved != CapabilityClass::Unknown
    }
}

/// Resolved visual projection for one (style, role, scheme) triple.
///
/// Projections carry semantic classification and deterministic paint magnitudes
/// consumed by `apply_surface_projection`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StyleProjection {
    pub style: VisualStyleId,
    pub role: SurfaceRole,
    pub treatment: SurfaceTreatment,
    pub color_scheme: ColorScheme,
    pub opaque: bool,
    pub content_scrim: bool,
    pub effects: Vec<EffectResolution>,
    pub paint: SurfacePaintProjection,
}

/// Machine-readable appearance authority report.
///
/// `source` names the authority actually read: the committed Shelly-local
/// on-disk configuration (`gpui-ui.json`), never an unsaved GUI draft and
/// never a global desktop configuration authority (which remains OPEN).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppearanceStatus {
    pub schema: &'static str,
    pub visual_style: VisualStyleId,
    pub color_scheme: ColorScheme,
    pub active_profile_revision: u32,
    pub source: &'static str,
    pub effects: Vec<EffectResolution>,
}

pub const APPEARANCE_STATUS_SCHEMA: &str = "calix.appearance-status/1";

/// Deterministically resolves a style projection for one surface role.
///
/// Given identical (style, role, scheme), the output is identical.
/// Resolution performs no I/O.
pub fn resolve_style(
    style: VisualStyleId,
    role: SurfaceRole,
    color_scheme: ColorScheme,
) -> StyleProjection {
    let profile = VisualStyleRegistry::get(style);
    let effects: Vec<EffectResolution> = profile
        .requests
        .iter()
        .copied()
        .map(EffectResolution::resolve)
        .collect();
    let opaque = style == VisualStyleId::Standard;
    let content_scrim = style == VisualStyleId::Transparency && role.requires_content_protection();
    let paint = SurfacePaintProjection::compute(style, role, color_scheme);
    StyleProjection {
        style,
        role,
        treatment: role.treatment(),
        color_scheme,
        opaque,
        content_scrim,
        effects,
        paint,
    }
}

/// Builds the appearance status report from committed configuration values.
pub fn status_for_config(visual_style: VisualStyleId, dark_theme: bool) -> AppearanceStatus {
    let profile = VisualStyleRegistry::get(visual_style);
    AppearanceStatus {
        schema: APPEARANCE_STATUS_SCHEMA,
        visual_style,
        color_scheme: ColorScheme::from_dark_theme(dark_theme),
        active_profile_revision: profile.revision,
        source: "committed-calix-local-config",
        effects: profile
            .requests
            .iter()
            .copied()
            .map(EffectResolution::resolve)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_effect_kind_kebab_serde_and_labels() {
        assert_eq!(EffectKind::ALL.len(), 4);
        for kind in EffectKind::ALL {
            let json = serde_json::to_string(kind).expect("Serialize effect kind");
            assert_eq!(json, format!("\"{}\"", kind.as_str()));
            let parsed: EffectKind = serde_json::from_str(&json).expect("Deserialize effect kind");
            assert_eq!(*kind, parsed);
            assert_eq!(kind.to_string(), kind.as_str());
        }
        assert_eq!(EffectKind::BackdropBlur.label(), "Backdrop Blur");
        assert_eq!(EffectKind::ContactDepth.label(), "Contact Depth");
    }

    #[test]
    fn test_resolution_evidence_resolution_in_style_00b() {
        for kind in EffectKind::ALL {
            let res = EffectResolution::resolve(*kind);
            assert!(res.requested);
            assert!(!res.note.is_empty());
            match kind {
                EffectKind::ContactDepth => {
                    assert_eq!(res.resolved, CapabilityClass::Native);
                    assert!(res.is_active());
                }
                EffectKind::RimResponse => {
                    assert_eq!(res.resolved, CapabilityClass::Native);
                    assert!(res.is_active());
                }
                EffectKind::Microstructure => {
                    assert_eq!(res.resolved, CapabilityClass::TextureProof);
                    assert!(res.is_active());
                }
                EffectKind::BackdropBlur => {
                    assert_eq!(res.resolved, CapabilityClass::Unknown);
                    assert!(!res.is_active(), "BackdropBlur remains inactive (Unknown)");
                }
            }
            assert_ne!(
                res.resolved,
                CapabilityClass::ShaderRequired,
                "ShaderRequired is forbidden before RENDER-03"
            );
        }
    }

    #[test]
    fn test_resolve_deterministic_across_all_roles_styles_schemes() {
        for style in VisualStyleId::ALL {
            for role in SurfaceRole::ALL {
                for scheme in ColorScheme::ALL {
                    let first = resolve_style(*style, *role, *scheme);
                    let second = resolve_style(*style, *role, *scheme);
                    assert_eq!(first, second, "Resolution must be deterministic");
                    assert_eq!(first.style, *style);
                    assert_eq!(first.role, *role);
                    assert_eq!(first.color_scheme, *scheme);
                    assert_eq!(first.opaque, *style == VisualStyleId::Standard);
                    assert_eq!(first.treatment, role.treatment());
                    assert_eq!(
                        first.content_scrim,
                        *style == VisualStyleId::Transparency && role.requires_content_protection()
                    );
                    let expected_requests = VisualStyleRegistry::get(*style).requests.len();
                    assert_eq!(first.effects.len(), expected_requests);

                    // Check paint projection properties
                    if *style == VisualStyleId::Standard {
                        assert_eq!(first.paint.surface_opacity, 1.0);
                        assert!(!first.paint.rim_active);
                        assert_eq!(
                            first.paint.shadow_active,
                            first.treatment == SurfaceTreatment::ElevatedSurface
                        );
                    } else {
                        assert!(first.paint.surface_opacity < 1.0);
                        assert_eq!(
                            first.paint.content_scrim,
                            role.requires_content_protection()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_resolve_light_dark_parity_same_effects() {
        for style in VisualStyleId::ALL {
            for role in SurfaceRole::ALL {
                let light = resolve_style(*style, *role, ColorScheme::Light);
                let dark = resolve_style(*style, *role, ColorScheme::Dark);
                assert_eq!(light.effects, dark.effects);
                assert_eq!(light.opaque, dark.opaque);
                assert_eq!(light.content_scrim, dark.content_scrim);
            }
        }
    }

    #[test]
    fn test_appearance_status_schema_and_serialization() {
        let status = status_for_config(VisualStyleId::Transparency, true);
        assert_eq!(status.schema, APPEARANCE_STATUS_SCHEMA);
        assert_eq!(status.visual_style, VisualStyleId::Transparency);
        assert_eq!(status.color_scheme, ColorScheme::Dark);
        assert_eq!(status.active_profile_revision, 1);
        assert_eq!(status.source, "committed-calix-local-config");
        assert_eq!(status.effects.len(), 4);
        let json = serde_json::to_string(&status).expect("Serialize status");
        assert!(json.contains("\"transparency\""));
        assert!(json.contains(APPEARANCE_STATUS_SCHEMA));
        assert!(json.contains("\"committed-calix-local-config\""));

        let standard = status_for_config(VisualStyleId::Standard, false);
        assert_eq!(standard.color_scheme, ColorScheme::Light);
        assert_eq!(standard.effects.len(), 1);
        assert_eq!(standard.effects[0].kind, EffectKind::ContactDepth);
        assert_eq!(
            standard.effects[0].resolved,
            CapabilityClass::Native,
            "Standard requests ContactDepth and RENDER-01 proves it Native"
        );
    }
}
