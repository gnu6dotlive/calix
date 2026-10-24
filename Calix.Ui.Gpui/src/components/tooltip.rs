//! The one tooltip: every tooltip of the window looks the same.
//!
//! A tooltip with nothing to say draws nothing: a text that fits needs no
//! tooltip, and GPUI 0.3 has no tooltip that can decline once asked
//! (Skwir patches gpui for `optional_tooltip`; here the view stays empty).

use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct Tip {
    says: Option<SharedString>,
}

impl Render for Tip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx);
        div().when_some(self.says.clone(), |el, says| {
            el.max_w(px(420.))
                .bg(theme.bg_surface)
                .text_color(theme.text_primary)
                .border_1()
                .border_color(theme.border)
                .rounded_md()
                .px(px(8.0))
                .py(px(4.0))
                .text_xs()
                .shadow_md()
                .child(says)
        })
    }
}

/// A tooltip saying `says`, or nothing at all.
pub fn tip(says: Option<SharedString>, cx: &mut App) -> AnyView {
    cx.new(|_| Tip { says }).into()
}
