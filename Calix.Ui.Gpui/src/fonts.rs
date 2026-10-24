//! Les polices de Calix, embarquées : la fenêtre ne dépend pas de ce que le
//! système a installé, et ne paie jamais la résolution d'une pile par défaut
//! (Skwir mesure 35 à 50 ms par image quand elle échoue).
//!
//! - `UI` — Instrument Sans : la voix de l'interface, des titres aux chiffres.
//! - `LABEL` — Instrument Sans SemiCondensed : les petits libellés (en-têtes de
//!   colonne, noms de section). La largeur porte la hiérarchie à la place des
//!   capitales.
//! - `CODE` — JetBrains Mono : ce qui est vraiment du texte de machine, à
//!   copier tel quel : le journal, un PKGBUILD, une commande.
//!
//! Toutes sous licence SIL OFL 1.1 (`assets/fonts/OFL-*.txt`).

use std::borrow::Cow;

pub const UI: &str = "Instrument Sans";
pub const LABEL: &str = "Instrument Sans SemiCondensed";
pub const CODE: &str = "JetBrains Mono";

const FILES: &[&[u8]] = &[
    include_bytes!("../assets/fonts/InstrumentSans-Regular.ttf"),
    include_bytes!("../assets/fonts/InstrumentSans-Medium.ttf"),
    include_bytes!("../assets/fonts/InstrumentSans-SemiBold.ttf"),
    include_bytes!("../assets/fonts/InstrumentSans-Bold.ttf"),
    include_bytes!("../assets/fonts/InstrumentSansSemiCondensed-Medium.ttf"),
    include_bytes!("../assets/fonts/InstrumentSansSemiCondensed-SemiBold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf"),
];

/// Les chiffres d'une colonne : tabulaires, pour qu'une version ou une taille
/// se lise contre celle de la ligne d'au-dessus.
pub fn figures() -> gpui::FontFeatures {
    gpui::FontFeatures(std::sync::Arc::new(vec![("tnum".into(), 1)]))
}

/// Donne les polices au système de texte de GPUI, avant la première fenêtre.
pub fn install(cx: &mut gpui::App) {
    let fonts = FILES.iter().map(|bytes| Cow::Borrowed(*bytes)).collect();
    if let Err(error) = cx.text_system().add_fonts(fonts) {
        log::warn!("Les polices de Calix n'ont pas pu être chargées : {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn every_embedded_face_is_a_truetype_font() {
        // A file replaced by a pointer or a page of HTML would load as nothing
        // and the window would fall back to the system's stack, silently.
        for face in FILES {
            assert_eq!(&face[..4], &[0, 1, 0, 0], "a TrueType outline font");
            assert!(face.len() > 50_000);
        }
    }
}
