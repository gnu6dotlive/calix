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
                    .text_color(theme.accent)
                    .mb_3(),
            )
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .mb_2()
                    .child("Unified Search in Calix"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .max_w(px(460.0))
                    .mb_4()
                    .child(
                        "Type a package or application name to search across Arch repositories, the AUR, Flatpak, and AppImages.",
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_3()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("• No network requests are dispatched on empty input")
                    .child("• Filters menu refines results by distribution backend and state")
                    .child("• Use Up / Down arrows to navigate through the virtualized results"),
            )
    }
}
