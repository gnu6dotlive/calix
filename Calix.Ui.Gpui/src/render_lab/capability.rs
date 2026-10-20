use serde::{Deserialize, Serialize};

/// Classification of what rendering capability a fixture requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityClass {
    /// Reproducible with GPUI 0.2.2 native primitives at full fidelity.
    Native,
    /// Composed from multiple native primitives; no single-primitive equivalent.
    Composable,
    /// Requires painting onto a texture (Canvas API or pixel buffer), but no custom shader.
    TextureProof,
    /// Cannot be reproduced with sufficient fidelity without a custom WGSL shader.
    ShaderRequired,
    /// Capability not yet classified; awaiting evaluation.
    Unknown,
}

impl CapabilityClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "NATIVE",
            Self::Composable => "COMPOSABLE",
            Self::TextureProof => "TEXTURE_PROOF",
            Self::ShaderRequired => "SHADER_REQUIRED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for CapabilityClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_class_round_trip() {
        let cases = [
            (CapabilityClass::Native, "\"NATIVE\""),
            (CapabilityClass::Composable, "\"COMPOSABLE\""),
            (CapabilityClass::TextureProof, "\"TEXTURE_PROOF\""),
            (CapabilityClass::ShaderRequired, "\"SHADER_REQUIRED\""),
            (CapabilityClass::Unknown, "\"UNKNOWN\""),
        ];

        for (variant, json_str) in cases {
            let serialized = serde_json::to_string(&variant).unwrap();
            assert_eq!(serialized, json_str);
            let deserialized: CapabilityClass = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, variant);
            assert_eq!(variant.as_str(), variant.to_string());
        }
    }
}
