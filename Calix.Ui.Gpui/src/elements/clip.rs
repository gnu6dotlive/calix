//! Un texte coupé pour tenir se dit en entier dans une infobulle, seulement
//! tant qu'il est coupé.
//!
//! Adapté de Skwir (`crates/ui/src/elements/clip.rs`, Apache-2.0). La boîte
//! où le texte est posé dit comment il est coupé (`text_ellipsis`,
//! `line_clamp`) ; qu'il l'ait été ne se sait qu'une fois sa largeur connue.
//! L'élément de texte garde ce qu'il a dessiné : l'infobulle le compare au
//! texte entier quand le pointeur s'est posé dessus, jamais dans `render`.
//! Un nom de paquet long, une version à rallonge, une description de trois
//! lignes ne sont plus perdus derrière une ellipse.

use crate::components::tooltip;
use gpui::{Div, SharedString, Stateful, StatefulInteractiveElement, StyledText, TextLayout};
use gpui::prelude::*;

/// Ce qui dit si un texte a été coupé, et ce qu'il disait en entier.
#[derive(Clone)]
struct Clip {
    whole: SharedString,
    layout: TextLayout,
}

impl Clip {
    /// Si le texte, tel que dessiné la dernière fois, n'est pas tout le texte.
    /// Demandé seulement après le dessin : l'infobulle naît au survol.
    fn cut(&self) -> bool {
        // `text()` joint les lignes par des retours : un texte replié sans
        // être coupé se compare sans eux, et l'ellipse le distingue.
        self.layout.text().replace('\n', " ").trim_end()
            != self.whole.replace('\n', " ").trim_end()
    }

    fn says(&self) -> Option<SharedString> {
        self.cut().then(|| self.whole.clone())
    }
}

/// `el` montrant `text`, coupé comme le dit le style de `el`, et le disant en
/// entier dans une infobulle tant qu'il est coupé.
pub fn clipped(el: Stateful<Div>, text: impl Into<SharedString>) -> Stateful<Div> {
    let whole: SharedString = text.into();
    let styled = StyledText::new(whole.clone());
    let clip = Clip {
        whole,
        layout: styled.layout().clone(),
    };
    el.tooltip(move |_, cx| tooltip::tip(clip.says(), cx))
        .child(styled)
}

/// L'id d'un élément connu par ce qu'il montre, jamais par sa place.
///
/// GPUI garde l'infobulle, l'appui et le nœud d'accessibilité d'un élément
/// sous son id : un élément connu par sa position les passe à ce qui vient se
/// mettre là (Skwir, `elements::known_by`). Une rangée du catalogue est
/// connue par son paquet.
pub fn known_by(name: &'static str, what: impl std::hash::Hash) -> gpui::ElementId {
    use std::hash::BuildHasher;
    let key = std::hash::BuildHasherDefault::<std::hash::DefaultHasher>::default().hash_one(what);
    gpui::ElementId::NamedInteger(name.into(), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::{div, px, Window};

    struct Cell {
        text: &'static str,
        width: f32,
        clip: Option<Clip>,
    }

    impl Render for Cell {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let styled = StyledText::new(self.text);
            self.clip = Some(Clip {
                whole: self.text.into(),
                layout: styled.layout().clone(),
            });
            div()
                .w(px(self.width))
                .font_family("monospace")
                .overflow_hidden()
                .text_ellipsis()
                .child(styled)
        }
    }

    fn drawn(text: &'static str, width: f32, cx: &mut gpui::TestAppContext) -> Option<SharedString> {
        let (view, cx) = cx.add_window_view(|_, _| Cell {
            text,
            width,
            clip: None,
        });
        cx.run_until_parked();
        view.read_with(cx, |cell, _| cell.clip.as_ref().unwrap().says())
    }

    #[gpui::test]
    fn a_name_that_fits_says_nothing_more(cx: &mut gpui::TestAppContext) {
        assert_eq!(drawn("vim", 400., cx), None);
    }

    #[gpui::test]
    fn a_name_cut_short_says_itself_whole(cx: &mut gpui::TestAppContext) {
        let long = "python-very-long-package-name-that-cannot-fit-anywhere";
        assert_eq!(drawn(long, 60., cx), Some(long.into()));
    }

    #[test]
    fn an_element_is_known_by_what_it_shows() {
        assert_eq!(known_by("row", "vim"), known_by("row", "vim"));
        assert_ne!(known_by("row", "vim"), known_by("row", "emacs"));
        assert_ne!(known_by("row", "vim"), known_by("card", "vim"));
    }
}
