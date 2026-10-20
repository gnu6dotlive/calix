use gpui::{rgb, Rgba};
use serde::{Deserialize, Serialize};

/// Les deux colorways de la marque Calix.
///
/// La marque est une coupe en écalille dans une coupe à vin : la géométrie est
/// unique, la couleur est un choix de l'utilisateur. Les teintes sont celles du
/// design system (jetons Gnosix DS), reprises telles quelles — ni re-light, ni
/// re-teinte : la base donne l'accent de l'interface, la teinte 300 donne son
/// survol.
///
/// - `Orange` — signal orange : base `#FF6A00`, survol `#FF8E40`.
/// - `Royal` — bleu royal : base `#4169E1`, survol `#87A0F4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Brand {
    #[default]
    Orange,
    Royal,
}

impl Brand {
    /// Les deux colorways, dans l'ordre où les réglages les présentent.
    pub const ALL: &'static [Brand] = &[Brand::Orange, Brand::Royal];

    /// La clé du réglage : ce qui est écrit dans `gpui-ui.json`.
    pub fn as_str(self) -> &'static str {
        match self {
            Brand::Orange => "orange",
            Brand::Royal => "royal",
        }
    }

    /// Le nom lisible, dans les réglages.
    pub fn label(self) -> &'static str {
        match self {
            Brand::Orange => "Signal Orange",
            Brand::Royal => "Royal Blue",
        }
    }

    /// La description, dans les réglages.
    pub fn description(self) -> &'static str {
        match self {
            Brand::Orange => "The shell in warm light, on the signal orange of the design system",
            Brand::Royal => "The shell in cool light, on the royal blue of the design system",
        }
    }

    /// Lit une valeur de réglage. Toute valeur inconnue est refusée.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "orange" | "signal-orange" | "signalorange" => Ok(Brand::Orange),
            "royal" | "royal-blue" | "royalblue" | "blue" => Ok(Brand::Royal),
            other => Err(format!(
                "Invalid brand colour '{other}', expected 'orange' or 'royal'"
            )),
        }
    }

    /// La couleur de base : l'accent de l'interface, et la couleur de la marque.
    pub fn accent(self) -> Rgba {
        match self {
            Brand::Orange => rgb(0xFF6A00),
            Brand::Royal => rgb(0x4169E1),
        }
    }

    /// La teinte de survol (jeton 300).
    pub fn accent_hover(self) -> Rgba {
        match self {
            Brand::Orange => rgb(0xFF8E40),
            Brand::Royal => rgb(0x87A0F4),
        }
    }
}

impl std::fmt::Display for Brand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_colourways_parse_from_their_keys() {
        assert_eq!(Brand::parse("orange").unwrap(), Brand::Orange);
        assert_eq!(Brand::parse("Royal").unwrap(), Brand::Royal);
        assert_eq!(Brand::parse("  ROYAL  ").unwrap(), Brand::Royal);
        assert!(Brand::parse("cyan").is_err());
    }

    #[test]
    fn the_colourway_carries_the_design_system_tokens() {
        assert_eq!(Brand::Orange.accent(), rgb(0xFF6A00));
        assert_eq!(Brand::Orange.accent_hover(), rgb(0xFF8E40));
        assert_eq!(Brand::Royal.accent(), rgb(0x4169E1));
        assert_eq!(Brand::Royal.accent_hover(), rgb(0x87A0F4));
    }

    #[test]
    fn the_colourway_is_written_as_its_key() {
        assert_eq!(serde_json::to_string(&Brand::Orange).unwrap(), "\"orange\"");
        assert_eq!(
            serde_json::from_str::<Brand>("\"royal\"").unwrap(),
            Brand::Royal
        );
    }
}
