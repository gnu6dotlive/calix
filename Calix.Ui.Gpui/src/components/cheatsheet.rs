use crate::elements::scrollbar;
use crate::state::keys;
use crate::theme::Theme;
use gpui::prelude::StatefulInteractiveElement;
use gpui::*;

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    /// The sheet is drawn for real: a layout that panics would take the
    /// application with it, and the sheet is only drawn when the user asks
    /// for it — so the proof has to draw it on its own.
    #[gpui::test]
    fn the_sheet_draws_without_panicking(cx: &mut gpui::TestAppContext) {
        // The sheet is drawn inside a view: a scrollable region asks the
        // window which view it belongs to, and the harness's window has none.
        struct SheetFixture {
            theme: Theme,
            scroll: ScrollHandle,
        }

        impl Render for SheetFixture {
            fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
                Cheatsheet::render(CheatsheetProps {
                    theme: &self.theme,
                    scroll: &self.scroll,
                })
                .into_any_element()
            }
        }

        let cx = cx.add_empty_window();
        cx.draw(
            point(px(0.), px(0.)),
            size(px(1200.), px(900.)),
            |_window, cx| {
                cx.new(|_| SheetFixture {
                    theme: Theme::dark(),
                    scroll: ScrollHandle::new(),
                })
                .into_element()
            },
        );
    }
}

/// The shortcut sheet: a translucent overlay, derived from the registry.
///
/// The surface carries no key glyph: the sheet appears when the user holds
/// the modifier alone, and it lists the bindings by group. It is derived from
/// the registry, so it cannot lie about what the keys do.
pub struct CheatsheetProps<'a> {
    pub theme: &'a Theme,
    /// The sheet's scroll, kept by the window: the bar reads it.
    pub scroll: &'a ScrollHandle,
}

pub struct Cheatsheet;

impl Cheatsheet {
    /// One keycap: the binding, written the way it is on the key.
    pub(crate) fn render_keycap(binding: &keys::KeyBinding, theme: &Theme) -> impl IntoElement {
        div()
            .rounded_sm()
            .border_1()
            .border_color(theme.border)
            .bg(theme.bg_surface)
            .px_1p5()
            .py(px(1.0))
            .text_xs()
            .font_family("monospace")
            .text_color(theme.text_primary)
            .child(binding.label())
    }

    /// One line: what the action does, and every key that does it (Ctrl+F
    /// and / focus the search: one line, two keys).
    fn render_action(label: &'static str, group: &str, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.0))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .child(label),
            )
            .child(
                div().flex().gap_1().children(
                    keys::actions_in(group)
                        .into_iter()
                        .filter(|action| action.label == label)
                        .map(|action| Self::render_keycap(&action.binding, theme)),
                ),
            )
    }

    /// One group: its name, and its actions, one line per thing done.
    fn render_group(group: &'static str, theme: &Theme) -> impl IntoElement {
        let mut labels: Vec<&'static str> = Vec::new();
        for action in keys::actions_in(group) {
            if !labels.contains(&action.label) {
                labels.push(action.label);
            }
        }
        div()
            .flex()
            .flex_col()
            .gap(px(3.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child(group.to_uppercase()),
            )
            .children(labels.into_iter().map(|label| Self::render_action(label, group, theme)))
    }

    pub fn render(props: CheatsheetProps) -> impl IntoElement {
        let theme = props.theme;

        let mut groups = Vec::new();
        for group in keys::groups() {
            if keys::actions_in(group).is_empty() {
                continue;
            }
            groups.push(Self::render_group(group, theme));
        }

        // The overlay: translucent, and it does not block the surface — the
        // user can keep working while it is up.
        div()
            .id("cheatsheet_overlay")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .relative()
                    .w(px(520.0))
                    .max_h(relative(0.85))
                    .flex()
                    .flex_col()
                    .child(
                div()
                    .id("cheatsheet_panel")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(props.scroll)
                    .rounded_lg()
                    .shadow_lg()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.bg_app)
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_primary)
                                    .child("KEYBOARD"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child("esc closes"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .children(groups),
                    ),
                    )
                    .child(scrollbar::vertical("cheatsheet_scrollbar", props.scroll)),
            )
    }
}
