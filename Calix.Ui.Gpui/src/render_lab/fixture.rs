use crate::render_lab::capability::CapabilityClass;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureIdError(pub String);

impl std::fmt::Display for FixtureIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid fixture id '{}'", self.0)
    }
}

impl std::error::Error for FixtureIdError {}

/// Stable identifier for a render fixture.
/// Must be lowercase alphanumeric, hyphens, and dots.
/// Non-empty, no leading or trailing dot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FixtureId(String);

impl FixtureId {
    pub fn new(value: impl Into<String>) -> Result<Self, FixtureIdError> {
        let s = value.into();
        if s.is_empty() || s.starts_with('.') || s.ends_with('.') {
            return Err(FixtureIdError(s));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
        {
            return Err(FixtureIdError(s));
        }
        Ok(Self(s))
    }

    pub fn from_static(s: &'static str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FixtureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Semantic fixture group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureGroup {
    Fields,
    Depth,
    Optical,
    Materials,
    Microstructure,
}

impl FixtureGroup {
    pub const ALL: &'static [FixtureGroup] = &[
        FixtureGroup::Fields,
        FixtureGroup::Depth,
        FixtureGroup::Optical,
        FixtureGroup::Materials,
        FixtureGroup::Microstructure,
    ];

    pub fn board_code(self) -> char {
        match self {
            Self::Fields => 'G',
            Self::Depth => 'H',
            Self::Optical => 'I',
            Self::Materials => 'J',
            Self::Microstructure => 'K',
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Fields => "Fields",
            Self::Depth => "Depth",
            Self::Optical => "Optical",
            Self::Materials => "Materials",
            Self::Microstructure => "Microstructure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureDeterminism {
    Deterministic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureProvenance {
    pub board: char,
    pub cell_index: u8,
    pub source_label: Option<&'static str>,
    pub source_artifact: &'static str,
    pub source_sha256: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureDef {
    pub id: FixtureId,
    pub group: FixtureGroup,
    pub seed: u64,
    pub description: &'static str,
    pub provenance: FixtureProvenance,
    pub capability: CapabilityClass,
    pub determinism: FixtureDeterminism,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_id_valid() {
        assert!(FixtureId::new("field.zero-positive-scalar").is_ok());
        assert!(FixtureId::new("micro.k01").is_ok());
        assert!(FixtureId::new("depth.contact-shadow").is_ok());
    }

    #[test]
    fn test_fixture_id_invalid() {
        assert!(FixtureId::new("").is_err());
        assert!(FixtureId::new(".field").is_err());
        assert!(FixtureId::new("field.").is_err());
        assert!(FixtureId::new("Field.Upper").is_err());
        assert!(FixtureId::new("field with spaces").is_err());
        assert!(FixtureId::new("field_underscore").is_err());
    }

    #[test]
    fn test_fixture_group_board_code() {
        assert_eq!(FixtureGroup::Fields.board_code(), 'G');
        assert_eq!(FixtureGroup::Depth.board_code(), 'H');
        assert_eq!(FixtureGroup::Optical.board_code(), 'I');
        assert_eq!(FixtureGroup::Materials.board_code(), 'J');
        assert_eq!(FixtureGroup::Microstructure.board_code(), 'K');
    }
}
