//! What a package is, said as a list of facts: a label in the narrow cut, its
//! value beside it, a hairline between two facts.
//!
//! The facts were a grid of boxed cards with capitalised captions, two per
//! row: a dashboard's kit, and a long URL broke out of its card. A list of
//! definitions reads down one edge, gives the value all the room the label
//! leaves, and wraps a long value instead of cutting into its neighbour.

use crate::theme::Theme;
use gpui::*;

/// The width of the labels' column: the longest label in the narrow cut
/// ("Download size") and its gap.
const LABEL_COLUMN: f32 = 112.0;

/// A section's name: the narrow cut, sentence case, muted.
pub fn section_label(text: &'static str, theme: &Theme) -> Div {
    div()
        .font_family(crate::fonts::LABEL)
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text_muted)
        .child(text)
}

/// One fact: what it is, and its value.
pub fn fact(label: &'static str, value: impl IntoElement, theme: &Theme) -> Div {
    div()
        .flex()
        .items_baseline()
        .gap_3()
        .py(px(7.0))
        .border_b_1()
        .border_color(theme.border)
        .child(
            section_label(label, theme)
                .flex_none()
                .w(px(LABEL_COLUMN)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .text_color(theme.text_primary)
                .font_features(crate::fonts::figures())
                .child(value),
        )
}

/// The facts, one under the other.
pub fn list(facts: impl IntoIterator<Item = Div>) -> Div {
    div().flex().flex_col().children(facts)
}
