use gpui::{rgb, Rgba};

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub bg_app: Rgba,
    pub bg_sidebar: Rgba,
    pub bg_surface: Rgba,
    pub bg_surface_hover: Rgba,
    pub bg_surface_active: Rgba,
    pub border: Rgba,
    pub border_focus: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub success: Rgba,
    pub success_text: Rgba,
    pub warning: Rgba,
    pub warning_text: Rgba,
    pub danger: Rgba,
    pub danger_hover: Rgba,
    pub badge_alpm: Rgba,
    pub badge_aur: Rgba,
    pub badge_flatpak: Rgba,
    pub badge_appimage: Rgba,
    pub badge_cargo: Rgba,
    pub badge_npm: Rgba,
    pub badge_release: Rgba,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg_app: rgb(0x0f0f14),            // Ultra-deep charcoal base
            bg_sidebar: rgb(0x0a0a0e),        // Dark contrast header/nav
            bg_surface: rgb(0x181822),        // Layered elevated card surface
            bg_surface_hover: rgb(0x232332),  // Immediate reactive hover state
            bg_surface_active: rgb(0x2c2d40), // Active item surface
            border: rgb(0x262638),            // Subtle 1px boundary frame
            border_focus: rgb(0x38bdf8),      // Electric cyan focus glow
            text_primary: rgb(0xf1f5f9),      // Crisp high-contrast slate-50
            text_secondary: rgb(0x94a3b8),    // Balanced legible secondary text
            text_muted: rgb(0x94a3b8),        // Legible WCAG AA hints and captions (6.87:1)
            accent: rgb(0x38bdf8),            // Electric cyan
            accent_hover: rgb(0x7dd3fc),      // Luminous cyan highlight
            success: rgb(0x10b981),           // Vibrant emerald accent dot
            success_text: rgb(0x34d399),      // High-contrast emerald text label (9.16:1)
            warning: rgb(0xf59e0b),           // Bright amber accent dot
            warning_text: rgb(0xfbbf24),      // High-contrast amber text label (10.55:1)
            danger: rgb(0xf43f5e),            // Vivid ruby rose
            danger_hover: rgb(0xfb7185),      // Luminous ruby highlight
            badge_alpm: rgb(0x38bdf8),        // Electric Cyan (Official ALPM)
            badge_aur: rgb(0xa855f7),         // Vivid Violet (AUR)
            badge_flatpak: rgb(0x06b6d4),     // Cyan (Flatpak)
            badge_appimage: rgb(0xf97316),    // Warm Orange (AppImage)
            badge_cargo: rgb(0xd97757),       // Burnt Sienna (Cargo)
            badge_npm: rgb(0xcb3837),         // NPM Red
            badge_release: rgb(0x10b981),     // Emerald (Release)
        }
    }

    pub fn light() -> Self {
        Self {
            bg_app: rgb(0xf8fafc),
            bg_sidebar: rgb(0xf1f5f9),
            bg_surface: rgb(0xffffff),
            bg_surface_hover: rgb(0xe2e8f0),
            bg_surface_active: rgb(0xcbd5e1),
            border: rgb(0xe2e8f0),
            border_focus: rgb(0x0284c7),
            text_primary: rgb(0x0f172a),
            text_secondary: rgb(0x475569),
            text_muted: rgb(0x64748b), // WCAG AA compliant muted text (4.76:1)
            accent: rgb(0x0284c7),
            accent_hover: rgb(0x0369a1),
            success: rgb(0x059669),      // Emerald accent dot
            success_text: rgb(0x047857), // WCAG AA compliant emerald text (5.48:1)
            warning: rgb(0xd97706),      // Amber accent dot
            warning_text: rgb(0xb45309), // WCAG AA compliant amber text (5.02:1)
            danger: rgb(0xe11d48),
            danger_hover: rgb(0xbe123c),
            badge_alpm: rgb(0x0284c7),
            badge_aur: rgb(0x7c3aed),
            badge_flatpak: rgb(0x0891b2),
            badge_appimage: rgb(0xea580c),
            badge_cargo: rgb(0xb45309),
            badge_npm: rgb(0x9f1239),
            badge_release: rgb(0x047857),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.bg_app.r < 0.5
    }

    /// Calcule la luminance relative standard sRGB (WCAG 2.1)
    #[cfg(test)]
    pub fn relative_luminance(c: Rgba) -> f64 {
        let channel = |v: f32| -> f64 {
            let v = v as f64;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
    }

    /// Calcule le ratio de contraste WCAG 2.1 entre deux couleurs
    #[cfg(test)]
    pub fn contrast_ratio(c1: Rgba, c2: Rgba) -> f64 {
        let l1 = Self::relative_luminance(c1);
        let l2 = Self::relative_luminance(c2);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_theme_dark_and_light_parity_and_contrast() {
        let dark = Theme::dark();
        let light = Theme::light();

        // Ensure surface is distinct from app background in both themes
        assert_ne!(dark.bg_app, dark.bg_surface);
        assert_ne!(light.bg_app, light.bg_surface);

        // Ensure primary text differs from background in both themes
        assert_ne!(dark.text_primary, dark.bg_app);
        assert_ne!(light.text_primary, light.bg_app);

        // Ensure border focus differs from base border
        assert_ne!(dark.border, dark.border_focus);
        assert_ne!(light.border, light.border_focus);

        // Ensure default is dark theme
        let def = Theme::default();
        assert_eq!(def.bg_app, dark.bg_app);
        assert_eq!(def.accent, dark.accent);
    }

    #[test]
    fn test_wcag_21_aa_contrast_ratios_mathematical_guarantee() {
        let themes = [("dark", Theme::dark()), ("light", Theme::light())];

        for (name, theme) in themes {
            // Text tokens must achieve >= 4.5:1 against both surface and app background
            let surfaces = [("surface", theme.bg_surface), ("app", theme.bg_app)];

            for (surf_name, bg) in surfaces {
                let primary_ratio = Theme::contrast_ratio(theme.text_primary, bg);
                assert!(
                    primary_ratio >= 4.5,
                    "{} theme text_primary on {} ratio {} < 4.5:1",
                    name,
                    surf_name,
                    primary_ratio
                );

                let secondary_ratio = Theme::contrast_ratio(theme.text_secondary, bg);
                assert!(
                    secondary_ratio >= 4.5,
                    "{} theme text_secondary on {} ratio {} < 4.5:1",
                    name,
                    surf_name,
                    secondary_ratio
                );

                let muted_ratio = Theme::contrast_ratio(theme.text_muted, bg);
                assert!(
                    muted_ratio >= 4.5,
                    "{} theme text_muted on {} ratio {} < 4.5:1",
                    name,
                    surf_name,
                    muted_ratio
                );

                let success_text_ratio = Theme::contrast_ratio(theme.success_text, bg);
                assert!(
                    success_text_ratio >= 4.5,
                    "{} theme success_text on {} ratio {} < 4.5:1",
                    name,
                    surf_name,
                    success_text_ratio
                );

                let warning_text_ratio = Theme::contrast_ratio(theme.warning_text, bg);
                assert!(
                    warning_text_ratio >= 4.5,
                    "{} theme warning_text on {} ratio {} < 4.5:1",
                    name,
                    surf_name,
                    warning_text_ratio
                );
            }
        }
    }
}
