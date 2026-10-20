use crate::visual_style::profile::{VisualStyleId, VisualStyleProfile};
use crate::visual_style::resolver::EffectKind;

/// Transparency profile declaration.
///
/// Transparency is a coherent material system (controlled translucency,
/// backdrop treatment, edge and depth response, deterministic
/// microstructure), not a theme with an alpha slider. STYLE-00A declares
/// its semantic effect requests; projection magnitudes and paint
/// integration arrive with STYLE-00B after Render Lab evidence.
pub static TRANSPARENCY_PROFILE: VisualStyleProfile = VisualStyleProfile {
    id: VisualStyleId::Transparency,
    name: "Transparency",
    revision: 1,
    requests: &[
        EffectKind::BackdropBlur,
        EffectKind::Microstructure,
        EffectKind::RimResponse,
        EffectKind::ContactDepth,
    ],
};
