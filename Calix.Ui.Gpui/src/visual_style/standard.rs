use crate::visual_style::profile::{VisualStyleId, VisualStyleProfile};
use crate::visual_style::resolver::EffectKind;

/// Compatibility profile: the current non-transparent visual system.
///
/// Standard requests only [`EffectKind::ContactDepth`]. STYLE-00A resolves
/// nothing: the first observation belongs to the RENDER-01 ledger. Standard
/// is the default, the fallback, and the regression baseline, and STYLE-00A
/// performs no Standard redesign.
pub static STANDARD_PROFILE: VisualStyleProfile = VisualStyleProfile {
    id: VisualStyleId::Standard,
    name: "Standard",
    revision: 1,
    requests: &[EffectKind::ContactDepth],
};
