use serde::{Deserialize, Serialize};

/// Semantic surface role authority.
///
/// Components request a role; the style authority decides its visual
/// projection. No widget may hardcode transparency, blur, or shader
/// selection for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceRole {
    AppChrome,
    NavigationRail,
    QueryChrome,
    ResultSurface,
    InspectorChrome,
    InspectorContent,
    ConsoleChrome,
    Popover,
    Menu,
    Dialog,
    ControlSurface,
    ContentSurface,
    SelectionSurface,
}

impl SurfaceRole {
    pub const ALL: &[SurfaceRole] = &[
        SurfaceRole::AppChrome,
        SurfaceRole::NavigationRail,
        SurfaceRole::QueryChrome,
        SurfaceRole::ResultSurface,
        SurfaceRole::InspectorChrome,
        SurfaceRole::InspectorContent,
        SurfaceRole::ConsoleChrome,
        SurfaceRole::Popover,
        SurfaceRole::Menu,
        SurfaceRole::Dialog,
        SurfaceRole::ControlSurface,
        SurfaceRole::ContentSurface,
        SurfaceRole::SelectionSurface,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppChrome => "app-chrome",
            Self::NavigationRail => "navigation-rail",
            Self::QueryChrome => "query-chrome",
            Self::ResultSurface => "result-surface",
            Self::InspectorChrome => "inspector-chrome",
            Self::InspectorContent => "inspector-content",
            Self::ConsoleChrome => "console-chrome",
            Self::Popover => "popover",
            Self::Menu => "menu",
            Self::Dialog => "dialog",
            Self::ControlSurface => "control-surface",
            Self::ContentSurface => "content-surface",
            Self::SelectionSurface => "selection-surface",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AppChrome => "App Chrome",
            Self::NavigationRail => "Navigation Rail",
            Self::QueryChrome => "Query Chrome",
            Self::ResultSurface => "Result Surface",
            Self::InspectorChrome => "Inspector Chrome",
            Self::InspectorContent => "Inspector Content",
            Self::ConsoleChrome => "Console Chrome",
            Self::Popover => "Popover",
            Self::Menu => "Menu",
            Self::Dialog => "Dialog",
            Self::ControlSurface => "Control Surface",
            Self::ContentSurface => "Content Surface",
            Self::SelectionSurface => "Selection Surface",
        }
    }

    /// Roles whose readable content must never depend on an unknown
    /// wallpaper for contrast. Under Transparency they own a predictable
    /// style-owned content plane (scrim).
    ///
    /// Only [`SurfaceRole::AppChrome`] — the base canvas owning no text of
    /// its own — resolves unprotected. Every other role carries labels,
    /// data, controls, or actions in real Calix surfaces (rail labels and
    /// counts, query text and filters, inspector metadata and tabs, console
    /// status and controls), so all resolve protected.
    pub fn requires_content_protection(self) -> bool {
        !matches!(self, Self::AppChrome)
    }

    /// Semantic treatment class governing how strongly a style may act on
    /// this role. Treatment carries no visual magnitudes in STYLE-00A; it
    /// gives `style.resolve(role)` a real projection authority whose values
    /// arrive with STYLE-00B evidence.
    pub fn treatment(self) -> SurfaceTreatment {
        match self {
            Self::AppChrome | Self::NavigationRail => SurfaceTreatment::AmbientChrome,
            Self::QueryChrome
            | Self::InspectorChrome
            | Self::ConsoleChrome
            | Self::ControlSurface => SurfaceTreatment::InteractiveChrome,
            Self::ResultSurface
            | Self::InspectorContent
            | Self::ContentSurface
            | Self::SelectionSurface => SurfaceTreatment::ContentPlane,
            Self::Popover | Self::Menu | Self::Dialog => SurfaceTreatment::ElevatedSurface,
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        for role in Self::ALL {
            if role.as_str() == value.to_ascii_lowercase() {
                return Ok(*role);
            }
        }
        let valid: Vec<&str> = Self::ALL.iter().map(|r| r.as_str()).collect();
        Err(format!(
            "Invalid surface role '{value}', expected one of: {}",
            valid.join(", ")
        ))
    }
}

impl std::fmt::Display for SurfaceRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Semantic treatment class: how strongly a visual style may act on a
/// surface role. Distinct from content protection: an ambient role may
/// still require a content plane (e.g. navigation labels over a
/// translucent rail).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceTreatment {
    /// Decorative / navigation shell; most atmospheric treatment allowed.
    AmbientChrome,
    /// Controls, toolbars, chrome carrying actions; moderate treatment.
    InteractiveChrome,
    /// Text and data planes; strongest opacity floor, calmest treatment.
    ContentPlane,
    /// Menus, popovers, dialogs; floating transient treatment with a
    /// stable readable material regardless of backdrop.
    ElevatedSurface,
}

impl SurfaceTreatment {
    pub const ALL: &[SurfaceTreatment] = &[
        SurfaceTreatment::AmbientChrome,
        SurfaceTreatment::InteractiveChrome,
        SurfaceTreatment::ContentPlane,
        SurfaceTreatment::ElevatedSurface,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AmbientChrome => "ambient-chrome",
            Self::InteractiveChrome => "interactive-chrome",
            Self::ContentPlane => "content-plane",
            Self::ElevatedSurface => "elevated-surface",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AmbientChrome => "Ambient Chrome",
            Self::InteractiveChrome => "Interactive Chrome",
            Self::ContentPlane => "Content Plane",
            Self::ElevatedSurface => "Elevated Surface",
        }
    }
}

impl std::fmt::Display for SurfaceTreatment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_surface_role_count_and_kebab_serde() {
        assert_eq!(SurfaceRole::ALL.len(), 13);
        for role in SurfaceRole::ALL {
            let json = serde_json::to_string(role).expect("Serialize role");
            assert_eq!(json, format!("\"{}\"", role.as_str()));
            let parsed: SurfaceRole = serde_json::from_str(&json).expect("Deserialize role");
            assert_eq!(*role, parsed);
            assert_eq!(role.to_string(), role.as_str());
            assert_eq!(
                *role,
                SurfaceRole::parse(role.as_str()).expect("parse role")
            );
        }
        assert!(SurfaceRole::parse("glass-card").is_err());
        assert!(SurfaceRole::parse("").is_err());
    }

    #[test]
    fn test_content_protection_policy_covers_all_roles() {
        let protected: Vec<SurfaceRole> = SurfaceRole::ALL
            .iter()
            .copied()
            .filter(|r| r.requires_content_protection())
            .collect();
        let unprotected: Vec<SurfaceRole> = SurfaceRole::ALL
            .iter()
            .copied()
            .filter(|r| !r.requires_content_protection())
            .collect();
        assert_eq!(protected.len(), 12);
        assert_eq!(unprotected, vec![SurfaceRole::AppChrome]);
        // The four roles misclassified before STYLE-00A-R must resolve protected.
        assert!(SurfaceRole::NavigationRail.requires_content_protection());
        assert!(SurfaceRole::QueryChrome.requires_content_protection());
        assert!(SurfaceRole::InspectorChrome.requires_content_protection());
        assert!(SurfaceRole::ConsoleChrome.requires_content_protection());
        assert!(SurfaceRole::ContentSurface.requires_content_protection());
        assert!(SurfaceRole::Menu.requires_content_protection());
    }

    #[test]
    fn test_surface_treatment_mapping_full_coverage() {
        use SurfaceTreatment::*;
        let expected = [
            (SurfaceRole::AppChrome, AmbientChrome),
            (SurfaceRole::NavigationRail, AmbientChrome),
            (SurfaceRole::QueryChrome, InteractiveChrome),
            (SurfaceRole::ResultSurface, ContentPlane),
            (SurfaceRole::InspectorChrome, InteractiveChrome),
            (SurfaceRole::InspectorContent, ContentPlane),
            (SurfaceRole::ConsoleChrome, InteractiveChrome),
            (SurfaceRole::Popover, ElevatedSurface),
            (SurfaceRole::Menu, ElevatedSurface),
            (SurfaceRole::Dialog, ElevatedSurface),
            (SurfaceRole::ControlSurface, InteractiveChrome),
            (SurfaceRole::ContentSurface, ContentPlane),
            (SurfaceRole::SelectionSurface, ContentPlane),
        ];
        assert_eq!(expected.len(), SurfaceRole::ALL.len());
        for (role, treatment) in expected {
            assert_eq!(role.treatment(), treatment, "treatment of {role}");
        }
        assert_eq!(SurfaceTreatment::ALL.len(), 4);
        for treatment in SurfaceTreatment::ALL {
            let json = serde_json::to_string(treatment).expect("Serialize treatment");
            assert_eq!(json, format!("\"{}\"", treatment.as_str()));
            let parsed: SurfaceTreatment =
                serde_json::from_str(&json).expect("Deserialize treatment");
            assert_eq!(*treatment, parsed);
            assert_eq!(treatment.to_string(), treatment.as_str());
        }
        assert_eq!(SurfaceTreatment::AmbientChrome.label(), "Ambient Chrome");
        assert_eq!(
            SurfaceTreatment::InteractiveChrome.label(),
            "Interactive Chrome"
        );
        assert_eq!(SurfaceTreatment::ContentPlane.label(), "Content Plane");
        assert_eq!(
            SurfaceTreatment::ElevatedSurface.label(),
            "Elevated Surface"
        );
    }
}
