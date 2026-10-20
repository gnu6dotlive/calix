use crate::theme::Theme;
use gpui::*;

pub struct StatusPill;

impl StatusPill {
    pub fn badge_count(count: usize, theme: &Theme) -> impl IntoElement {
        div()
            .px_1p5()
            .py_0p5()
            .rounded_full()
            .bg(theme.accent)
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.bg_app)
            .child(format!("{}", count))
    }
}
