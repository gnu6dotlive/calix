//! Les jetons de l'interface : une seule source, lue là où l'on dessine.
//!
//! La leçon de Skwir : le thème est un `Global` de GPUI (`Theme::of(cx)`), pas
//! une copie passée de vue en vue et rafraîchie par des `set_theme` en
//! cascade. Un changement de thème est un `cx.set_global` ; chaque vue le lit
//! au prochain rendu.
//!
//! - surfaces : `bg_sidebar` est le châssis (l'aluminium, le rail, l'en-tête),
//!   `bg_app` les plans posés dessus où l'on lit, `bg_surface` les contrôles et
//!   les champs. Voir « The interface: the workshop » dans `docs/brand.md`.
//! - états : `bg_surface_hover` sous le pointeur, `selection_bg` /
//!   `selection_text` pour ce qui est choisi, `on_accent` pour le texte et les glyphes posés sur une couleur pleine
//!   (l'accent, un badge, un succès). Un état n'est jamais dérivé localement.
//! - rayons : `rounded_sm` dans les contrôles (segments, rangées de menu),
//!   `rounded_md` pour les contrôles et les rangées, `rounded_lg` pour les
//!   panneaux (menus, toasts, cartes, dialogues), `rounded_full` pour les
//!   pastilles. Rien d'autre.
//! - élévation : `shadow_xs` sous un contrôle posé sur le châssis (la bande
//!   des sources), `shadow_sm` sous un plan (catalogue, inspecteur),
//!   `shadow_md` pour les menus, les popovers et le cadran, `shadow_lg` pour
//!   les toasts et dialogues ; rien d'autre ne porte d'ombre.
//!
//! Les jetons d'état sont dérivés (`Theme::derive_states`) de la palette et de
//! l'accent, puis leur contraste est garanti : un accent choisi par
//! l'utilisateur ne peut pas rendre un texte illisible.
use crate::brand::Brand;
use gpui::{rgb, App, Global, Rgba};

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
    /// Ce qui est choisi : une rangée, une cellule. Dérivé de l'accent.
    pub selection_bg: Rgba,
    /// Le texte posé sur `selection_bg`, à contraste garanti.
    pub selection_text: Rgba,
    /// Le texte et les glyphes posés sur l'accent : blanc, ou sombre sur un
    /// accent clair — celui des deux qui se lit.
    pub on_accent: Rgba,
    /// L'accent écrit en texte sur un plan : l'orange de la marque se lit mal
    /// sur l'aluminium clair, il fonce vers l'encre jusqu'à 4,5:1.
    pub accent_text: Rgba,
}

impl Global for Theme {}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

/// Le texte doit se lire à 4,5:1 sur ce qui le porte (WCAG 2.1, 1.4.3).
const TEXT_CONTRAST: f64 = 4.5;

fn lerp(a: Rgba, b: Rgba, t: f32) -> Rgba {
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn luminance(c: Rgba) -> f64 {
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

fn contrast(a: Rgba, b: Rgba) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Celui des deux textes qui se lit le mieux sur `ground`.
fn readable_on(ground: Rgba, light: Rgba, dark: Rgba) -> Rgba {
    if contrast(light, ground) >= contrast(dark, ground) {
        light
    } else {
        dark
    }
}

/// Un fond sous du texte qui cède : il s'éclaircit ou s'assombrit vers
/// `toward` jusqu'à ce que le meilleur des deux textes s'y lise.
fn guarded_fill(fill: Rgba, toward: Rgba, light: Rgba, dark: Rgba) -> (Rgba, Rgba) {
    let mut ground = fill;
    for step in 0..=10 {
        let text = readable_on(ground, light, dark);
        if contrast(text, ground) >= TEXT_CONTRAST {
            return (ground, text);
        }
        ground = lerp(fill, toward, step as f32 / 10.0);
    }
    (ground, readable_on(ground, light, dark))
}

/// Une couleur écrite en texte sur `ground` : elle va vers `toward` (l'encre)
/// jusqu'à se lire à 4,5:1.
fn guarded_text(color: Rgba, ground: Rgba, toward: Rgba) -> Rgba {
    (0..=20)
        .map(|step| lerp(color, toward, step as f32 / 20.0))
        .find(|candidate| contrast(*candidate, ground) >= TEXT_CONTRAST)
        .unwrap_or(toward)
}

impl Theme {
    /// Le thème de la fenêtre. Hors d'une fenêtre qui l'a posé (un banc de
    /// test), l'atelier de jour par défaut.
    pub fn of(cx: &App) -> Self {
        cx.try_global::<Theme>().copied().unwrap_or_default()
    }

    /// Le texte qui se lit sur une couleur pleine (un badge, un état) : blanc,
    /// ou sombre sur une couleur claire.
    pub fn on(&self, fill: Rgba) -> Rgba {
        readable_on(fill, rgb(0xffffff), rgb(0x0b0b10))
    }

    /// Le thème d'un look et d'une marque.
    pub fn look(dark: bool, brand: Brand) -> Self {
        if dark { Self::dark() } else { Self::light() }.with_brand(brand)
    }

    /// Pose le thème de l'application et redessine les fenêtres : la seule
    /// façon de changer de thème. Aucune vue ne garde de copie à rafraîchir.
    pub fn install(theme: Theme, cx: &mut App) {
        cx.set_global(theme);
        cx.refresh_windows();
    }

    /// Dérive les jetons d'état de la palette et de l'accent, contraste
    /// garanti. Appelé par chaque constructeur et après chaque changement
    /// d'accent : un état n'est jamais choisi à la main.
    fn derive_states(self) -> Self {
        let white = rgb(0xffffff);
        let ink = rgb(0x0b0b10);
        let dark = self.is_dark();
        // La sélection est l'accent, adouci vers le fond : elle se voit sans
        // crier, et son texte reste lisible.
        // L'atelier : une sélection est un plan teinté, pas un aplat ; c'est la
        // barre d'accent de la rangée qui la désigne.
        let tint = lerp(self.bg_app, self.accent, if dark { 0.24 } else { 0.14 });
        let toward = if dark { self.bg_app } else { white };
        let (light_text, dark_text) = if dark {
            (self.text_primary, ink)
        } else {
            (white, self.text_primary)
        };
        let (selection_bg, selection_text) = guarded_fill(tint, toward, light_text, dark_text);
        let on_accent = self.on(self.accent);
        let accent_text = guarded_text(self.accent, self.bg_app, self.text_primary);
        Self {
            selection_bg,
            selection_text,
            on_accent,
            accent_text,
            ..self
        }
    }

    /// L'atelier de nuit : la tuile anthracite de la marque (`#111418`) pour le
    /// châssis, des plans d'aluminium anodisé foncé pour ce qui se lit.
    pub fn dark() -> Self {
        Self {
            bg_sidebar: rgb(0x111418),        // Le châssis : la tuile anthracite
            bg_app: rgb(0x171b21),            // Les plans posés dessus
            bg_surface: rgb(0x1e232b),        // Les contrôles et les champs
            bg_surface_hover: rgb(0x242a33),
            bg_surface_active: rgb(0x2c333d),
            border: rgb(0x2b313a),            // L'arête d'un plan
            border_focus: rgb(0xff6a00),
            text_primary: rgb(0xeceef1),
            text_secondary: rgb(0xb3b9c2),
            text_muted: rgb(0x8f97a2),
            accent: rgb(0xff6a00),
            accent_hover: rgb(0xff8e40),
            success: rgb(0x3fb971),
            success_text: rgb(0x62cf90),
            warning: rgb(0xe9a23b),
            warning_text: rgb(0xf2b961),
            danger: rgb(0xf05a63),
            danger_hover: rgb(0xf57c83),
            // Les badges disent un écosystème, pas la marque : chacun sa
            // couleur, celle de son projet quand il en a une.
            badge_alpm: rgb(0x1793d1),        // Le bleu d'Arch
            badge_aur: rgb(0xa182e6),
            badge_flatpak: rgb(0x5c9de0),
            badge_appimage: rgb(0x8fb3c9),
            badge_cargo: rgb(0xe0763f),
            badge_npm: rgb(0xe0565a),
            badge_release: rgb(0x3fb971),
            // Dérivés plus bas (`derive_states`).
            selection_bg: rgb(0x000000),
            selection_text: rgb(0x000000),
            on_accent: rgb(0x000000),
            accent_text: rgb(0x000000),
        }
        .derive_states()
    }

    /// L'atelier de jour, le look par défaut : un instrument d'aluminium clair,
    /// à l'encre anthracite. Le châssis est l'aluminium brossé (`bg_sidebar`),
    /// les plans où l'on lit sont plus clairs et posés dessus, l'orange ne sert
    /// qu'à signaler : le curseur, le focus, l'action.
    pub fn light() -> Self {
        Self {
            bg_sidebar: rgb(0xe7eaee),        // Le châssis : aluminium brossé
            bg_app: rgb(0xf7f8fa),            // Les plans : ce qui se lit
            bg_surface: rgb(0xffffff),        // Les contrôles et les champs
            bg_surface_hover: rgb(0xeef0f3),
            bg_surface_active: rgb(0xe2e6eb),
            border: rgb(0xd5dae0),            // L'arête d'un plan
            border_focus: rgb(0xff6a00),
            text_primary: rgb(0x111418),      // L'encre : l'anthracite de la marque
            text_secondary: rgb(0x3d444f),
            text_muted: rgb(0x5d6571),
            accent: rgb(0xff6a00),
            accent_hover: rgb(0xff8e40),
            success: rgb(0x1f8a4c),
            success_text: rgb(0x16703c),
            warning: rgb(0xc26a00),
            warning_text: rgb(0x8c4a00),
            danger: rgb(0xc62839),
            danger_hover: rgb(0xa91f30),
            badge_alpm: rgb(0x1793d1),
            badge_aur: rgb(0x7a4cc2),
            badge_flatpak: rgb(0x3c78c3),
            badge_appimage: rgb(0x52707f),
            badge_cargo: rgb(0xb7410e),
            badge_npm: rgb(0xcb3837),
            badge_release: rgb(0x1f8a4c),
            // Dérivés plus bas (`derive_states`).
            selection_bg: rgb(0x000000),
            selection_text: rgb(0x000000),
            on_accent: rgb(0x000000),
            accent_text: rgb(0x000000),
        }
        .derive_states()
    }

    /// Applique la couleur de marque choisie par l'utilisateur.
    ///
    /// La marque est une coupe en écalille : elle est peinte par l'accent, donc
    /// l'accent et son survol portent la couleur de la marque. L'anneau de
    /// focus suit le même accent — un focus d'une autre couleur que l'accent
    /// ressemblerait à une erreur.
    pub fn with_brand(self, brand: Brand) -> Self {
        Self {
            accent: brand.accent(),
            accent_hover: brand.accent_hover(),
            border_focus: brand.accent(),
            ..self
        }
        .derive_states()
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
    fn the_brand_colour_paints_the_accent_and_the_focus_ring() {
        let dark = Theme::dark();
        let royal = dark.with_brand(Brand::Royal);
        assert_eq!(royal.accent, Brand::Royal.accent());
        assert_eq!(royal.accent_hover, Brand::Royal.accent_hover());
        assert_eq!(royal.border_focus, Brand::Royal.accent());

        // The rest of the palette is the brand's business, not the theme's.
        assert_eq!(royal.bg_app, dark.bg_app);
        assert_eq!(royal.success, dark.success);
        assert_eq!(royal.badge_aur, dark.badge_aur);
    }

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

        // The default is the day workshop.
        let def = Theme::default();
        assert_eq!(def.bg_app, light.bg_app);
        assert_eq!(def.accent, light.accent);
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

    #[test]
    fn the_derived_states_read_under_every_brand_and_look() {
        for base in [Theme::dark(), Theme::light()] {
            for &brand in Brand::ALL {
                let theme = base.with_brand(brand);
                let on_accent = super::contrast(theme.on_accent, theme.accent);
                assert!(
                    on_accent >= 4.5,
                    "{brand:?}: le texte sur l'accent se lit à {on_accent:.2}:1"
                );
                let selection = super::contrast(theme.selection_text, theme.selection_bg);
                assert!(
                    selection >= 4.5,
                    "{brand:?}: le texte sélectionné se lit à {selection:.2}:1"
                );
                assert_ne!(
                    theme.selection_bg, theme.bg_app,
                    "{brand:?}: la sélection se voit sur le fond"
                );
            }
        }
    }

    #[test]
    fn the_accent_written_as_text_reads_on_the_planes() {
        // Signal orange is 2.8:1 on the day workshop's planes: as text it
        // darkens toward the ink until it reads, in every look and brand.
        for brand in Brand::ALL {
            for theme in [Theme::light().with_brand(*brand), Theme::dark().with_brand(*brand)] {
                let ratio = Theme::contrast_ratio(theme.accent_text, theme.bg_app);
                assert!(ratio >= 4.5, "{brand:?}: accent text at {ratio:.2}:1");
            }
        }
        // In the night workshop the brand's own colour already reads.
        assert_eq!(Theme::dark().accent_text, Theme::dark().accent);
    }

    #[gpui::test]
    fn the_theme_is_read_from_the_global(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            assert_eq!(Theme::of(cx).bg_app, Theme::light().bg_app, "l'atelier de jour par défaut");
            cx.set_global(Theme::dark());
            assert_eq!(Theme::of(cx).bg_app, Theme::dark().bg_app);
        });
    }
}
