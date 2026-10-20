use crate::visual_style::resolver::EffectKind;
use serde::{Deserialize, Serialize};

/// First-class visual style authority identifier.
///
/// A visual style is a coherent projection policy over chrome materials,
/// surface opacity, depth response and optical effects. It is orthogonal to
/// [`ColorScheme`]: `Dark + Transparency` and `Light + Transparency` are the
/// same style, not separate profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum VisualStyleId {
    #[default]
    Standard,
    Transparency,
}

impl VisualStyleId {
    pub const ALL: &[VisualStyleId] = &[VisualStyleId::Standard, VisualStyleId::Transparency];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Transparency => "transparency",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Transparency => "Transparency",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "standard" => Ok(Self::Standard),
            "transparency" => Ok(Self::Transparency),
            other => Err(format!(
                "Invalid visual style '{other}', expected 'standard' or 'transparency'"
            )),
        }
    }
}

impl std::fmt::Display for VisualStyleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Color scheme authority, orthogonal to visual style.
///
/// `System` detection is explicitly deferred: the application currently
/// persists a concrete dark/light boolean, and STYLE-00A introduces no
/// platform theme probing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    pub const ALL: &[ColorScheme] = &[ColorScheme::Light, ColorScheme::Dark];

    pub fn from_dark_theme(dark_theme: bool) -> Self {
        if dark_theme {
            Self::Dark
        } else {
            Self::Light
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Static declaration of a selectable visual style profile.
///
/// STYLE-00A profiles declare semantic effect requests only. Projection
/// magnitudes and paint integration arrive with STYLE-00B, once the
/// Render Lab has produced stock capability evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisualStyleProfile {
    pub id: VisualStyleId,
    pub name: &'static str,
    pub revision: u32,
    pub requests: &'static [EffectKind],
}

/// Static registry of canonical visual style profiles.
///
/// Only `Standard` and `Transparency` exist canonically after STYLE-00A.
/// Future competitors are possible without rewriting consumers, but no
/// speculative profile may be introduced here.
pub struct VisualStyleRegistry;

static ALL_PROFILES: [&VisualStyleProfile; 2] = [
    &crate::visual_style::standard::STANDARD_PROFILE,
    &crate::visual_style::transparency::TRANSPARENCY_PROFILE,
];

impl VisualStyleRegistry {
    pub const PROFILE_COUNT: usize = 2;

    pub fn get(id: VisualStyleId) -> &'static VisualStyleProfile {
        match id {
            VisualStyleId::Standard => &crate::visual_style::standard::STANDARD_PROFILE,
            VisualStyleId::Transparency => &crate::visual_style::transparency::TRANSPARENCY_PROFILE,
        }
    }

    pub fn all() -> &'static [&'static VisualStyleProfile] {
        &ALL_PROFILES
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_visual_style_id_serde_round_trip() {
        for id in VisualStyleId::ALL {
            let json = serde_json::to_string(id).expect("Serialize style id");
            assert_eq!(json, format!("\"{}\"", id.as_str()));
            let parsed: VisualStyleId = serde_json::from_str(&json).expect("Deserialize style id");
            assert_eq!(*id, parsed);
            assert_eq!(id.to_string(), id.as_str());
        }
        assert!(serde_json::from_str::<VisualStyleId>("\"neon\"").is_err());
        assert!(serde_json::from_str::<VisualStyleId>("\"Dark Transparency\"").is_err());
    }

    #[test]
    fn test_visual_style_id_parse_and_default() {
        assert_eq!(VisualStyleId::default(), VisualStyleId::Standard);
        assert_eq!(
            VisualStyleId::parse("standard").expect("parse standard"),
            VisualStyleId::Standard
        );
        assert_eq!(
            VisualStyleId::parse("Transparency").expect("parse transparency"),
            VisualStyleId::Transparency
        );
        assert!(VisualStyleId::parse("acrylic").is_err());
        assert!(VisualStyleId::parse("").is_err());
        assert_eq!(VisualStyleId::Standard.label(), "Standard");
        assert_eq!(VisualStyleId::Transparency.label(), "Transparency");
    }

    #[test]
    fn test_color_scheme_mapping_and_serde() {
        assert_eq!(ColorScheme::from_dark_theme(true), ColorScheme::Dark);
        assert_eq!(ColorScheme::from_dark_theme(false), ColorScheme::Light);
        for scheme in ColorScheme::ALL {
            let json = serde_json::to_string(scheme).expect("Serialize scheme");
            assert_eq!(json, format!("\"{}\"", scheme.as_str()));
            let parsed: ColorScheme = serde_json::from_str(&json).expect("Deserialize scheme");
            assert_eq!(*scheme, parsed);
            assert_eq!(scheme.to_string(), scheme.as_str());
        }
        assert_eq!(ColorScheme::Dark.label(), "Dark");
        assert_eq!(ColorScheme::Light.label(), "Light");
    }

    #[test]
    fn test_registry_contains_exactly_standard_and_transparency() {
        assert_eq!(VisualStyleRegistry::PROFILE_COUNT, 2);
        let all = VisualStyleRegistry::all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, VisualStyleId::Standard);
        assert_eq!(all[1].id, VisualStyleId::Transparency);
        assert_eq!(
            VisualStyleRegistry::get(VisualStyleId::Standard).name,
            "Standard"
        );
        assert_eq!(
            VisualStyleRegistry::get(VisualStyleId::Transparency).name,
            "Transparency"
        );
        for profile in all {
            assert_eq!(profile.revision, 1);
            assert!(!profile.requests.is_empty());
        }
    }
}
