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
        }

        impl Render for SheetFixture {
            fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
                Cheatsheet::render(CheatsheetProps { theme: &self.theme }).into_any_element()
            }
        }

        let cx = cx.add_empty_window();
        cx.draw(
            point(px(0.), px(0.)),
            size(px(1200.), px(900.)),
            |_window, cx| {
                cx.new(|_| SheetFixture {
                    theme: Theme::dark(),
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

    /// One line: what the action does, and the key that does it.
    fn render_action(action: &keys::KeyAction, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.0))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .child(action.label),
            )
            .child(Self::render_keycap(&action.binding, theme))
    }

    /// One group: its name, and its actions.
    fn render_group(group: &str, theme: &Theme) -> impl IntoElement {
        let actions = keys::actions_in(group);
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
            .children(actions.into_iter().map(|action| Self::render_action(action, theme)))
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
                    .id("cheatsheet_panel")
                    .w(px(520.0))
                    .max_h(px(620.0))
                    .overflow_y_scroll()
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
    }
}
