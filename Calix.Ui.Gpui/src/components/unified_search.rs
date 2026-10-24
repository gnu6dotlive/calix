use crate::icons::AppIcon;
use crate::theme::Theme;
use gpui::*;

pub struct UnifiedSearch;

impl UnifiedSearch {
    /// Rendu de l'état de découverte lorsque la barre de recherche est vide
    pub fn render_empty_discovery(theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .p_8()
            .text_center()
            .child(
                svg()
                    .path(AppIcon::Search.path())
                    .size_8()
                    .text_color(theme.accent_text)
                    .mb_3(),
            )
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text_primary)
                    .mb_2()
                    .child("Search every source at once"),
            )
            .child(
                div()
                    .text_sm()
                    .line_height(relative(1.5))
                    .text_color(theme.text_secondary)
                    .max_w(px(440.0))
                    .child(
                        "The Arch repositories, the AUR, Flatpak and AppImages answer together. Type a name; the arrows walk the results, and Ctrl+K runs any command.",
                    ),
            )
    }
}
