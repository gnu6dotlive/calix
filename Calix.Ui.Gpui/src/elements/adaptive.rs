//! Choisir la variante la plus riche qui tient, par mesure intrinsèque, dans
//! le `prepaint` — là où les bornes sont réelles.
//!
//! Adapté de Skwir (`crates/ui/src/elements/adaptive.rs`, Apache-2.0) : une
//! mise en page qui dépend d'une taille ne se décide jamais dans `render`, et
//! jamais par des hauteurs mesurées une fois et recopiées en constantes. Les
//! variantes se mesurent elles-mêmes, à chaque image, avec la police et
//! l'espacement réellement utilisés.
//!
//! Les variantes partagent leur identité : un bouton présent dans plusieurs
//! variantes porte le même chemin d'id dans chacune, pour que son focus (gardé
//! par GPUI sous ce chemin) survive au changement de variante.
use gpui::{
    point, px, relative, size, AnyElement, App, AvailableSpace, Bounds, Element, ElementId,
    GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Style, Window,
};

/// La première de `count` variantes dont la mesure naturelle tient dans
/// `available`, ou la dernière (la plus compacte) quand aucune ne tient.
/// `natural` mesure une variante ; il est appelé paresseusement, dans l'ordre,
/// jusqu'à ce qu'une variante tienne. Rien à choisir, rien de choisi.
fn select(
    count: usize,
    available: Pixels,
    mut natural: impl FnMut(usize) -> Pixels,
) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(
        (0..last)
            .find(|&index| natural(index) <= available)
            .unwrap_or(last),
    )
}

/// Une rangée qui garde la plus riche de ses variantes qui tient en largeur.
///
/// Remplace les seuils en pixels (« à 640 px, la forme large ») : chaque
/// variante est mesurée à sa largeur naturelle dans `prepaint`, avec la police
/// et la langue réelles, et la première qui tient est dessinée. La rangée a
/// sa propre hauteur (un jeton d'échelle) et prend toute la largeur.
pub struct AdaptiveRow {
    /// Les variantes, de la plus riche à la plus compacte.
    pub variants: Vec<AnyElement>,
    pub height: Pixels,
}

impl AdaptiveRow {
    pub fn new(variants: Vec<AnyElement>, height: Pixels) -> Self {
        Self { variants, height }
    }
}

impl IntoElement for AdaptiveRow {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for AdaptiveRow {
    type RequestLayoutState = ();
    type PrepaintState = Option<usize>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = self.height.into();
        style.flex_shrink = 0.;
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Option<usize> {
        let height = AvailableSpace::Definite(bounds.size.height);
        let variants = &mut self.variants;
        // Un demi-pixel d'arrondi n'est pas une raison de rendre les mots.
        let selected = select(variants.len(), bounds.size.width + px(0.5), |index| {
            variants[index]
                .layout_as_root(size(AvailableSpace::MaxContent, height), window, cx)
                .width
        })?;
        let row = &mut self.variants[selected];
        row.layout_as_root(
            size(AvailableSpace::Definite(bounds.size.width), height),
            window,
            cx,
        );
        row.prepaint_at(bounds.origin, window, cx);
        Some(selected)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        selected: &mut Option<usize>,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(index) = *selected {
            self.variants[index].paint(window, cx);
        }
    }
}

/// Une pile verticale qui cède partie par partie : un chrome choisi parmi des
/// variantes, de la plus riche à la plus compacte, puis un corps qui prend ce
/// qui reste.
///
/// L'élément remplit la place qu'on lui donne. Dans `prepaint`, il mesure la
/// hauteur naturelle de chaque variante du chrome à la largeur réelle, garde la
/// première qui tient dans la hauteur réelle, et pose le corps dessous. Une
/// plaque d'inspecteur garde ainsi son identité toujours, ses onglets puis ses
/// actions quand la place les porte — sans aucune hauteur codée en dur.
pub struct AdaptiveStack {
    /// Les versions du chrome, de la plus riche à la plus compacte.
    pub chrome: Vec<AnyElement>,
    /// Ce qui vient sous le chrome et prend la hauteur restante.
    pub body: AnyElement,
}

impl AdaptiveStack {
    pub fn new(chrome: Vec<AnyElement>, body: impl IntoElement) -> Self {
        Self {
            chrome,
            body: body.into_any_element(),
        }
    }
}

impl IntoElement for AdaptiveStack {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for AdaptiveStack {
    type RequestLayoutState = ();
    type PrepaintState = Option<usize>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        style.min_size.height = Pixels::ZERO.into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Option<usize> {
        let width = AvailableSpace::Definite(bounds.size.width);
        let chrome = &mut self.chrome;
        let mut naturals = vec![None; chrome.len()];
        // Un demi-pixel d'arrondi n'est pas une raison de rendre une rangée.
        let selected = select(chrome.len(), bounds.size.height + px(0.5), |index| {
            let height = chrome[index]
                .layout_as_root(size(width, AvailableSpace::MinContent), window, cx)
                .height;
            naturals[index] = Some(height);
            height
        });
        let chrome_height = match selected {
            Some(index) => {
                let variant = &mut self.chrome[index];
                let natural = naturals[index]
                    .unwrap_or_else(|| {
                        variant
                            .layout_as_root(size(width, AvailableSpace::MinContent), window, cx)
                            .height
                    })
                    .min(bounds.size.height);
                variant.layout_as_root(size(width, AvailableSpace::Definite(natural)), window, cx);
                variant.prepaint_at(bounds.origin, window, cx);
                natural
            }
            None => Pixels::ZERO,
        };
        let rest = (bounds.size.height - chrome_height).max(Pixels::ZERO);
        self.body.prepaint_as_root(
            bounds.origin + point(Pixels::ZERO, chrome_height),
            size(width, AvailableSpace::Definite(rest)),
            window,
            cx,
        );
        selected
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        selected: &mut Option<usize>,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(index) = *selected {
            self.chrome[index].paint(window, cx);
        }
        self.body.paint(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::select;
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn nothing_to_choose_chooses_nothing() {
        assert_eq!(select(0, px(500.), |_| unreachable!()), None);
    }

    #[test]
    fn picks_the_first_variant_that_fits_or_the_last_one() {
        let heights = [px(130.), px(82.), px(48.)];
        assert_eq!(select(3, px(200.), |i| heights[i]), Some(0));
        assert_eq!(select(3, px(100.), |i| heights[i]), Some(1));
        assert_eq!(select(3, px(82.), |i| heights[i]), Some(1));
        assert_eq!(select(3, px(20.), |i| heights[i]), Some(2));
        // Une seule variante est choisie sans être mesurée.
        assert_eq!(select(1, px(10.), |_| unreachable!()), Some(0));
    }

    #[test]
    fn measures_only_until_a_variant_fits() {
        let mut measured = Vec::new();
        let heights = [px(130.), px(82.), px(48.)];
        let selected = select(3, px(100.), |i| {
            measured.push(i);
            heights[i]
        });
        assert_eq!(selected, Some(1));
        assert_eq!(measured, vec![0, 1]);
    }
}
