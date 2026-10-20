use crate::state::session::PackageViewMode;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;

pub struct ViewModeSwitcherProps<'a> {
    pub current_mode: PackageViewMode,
    pub theme: &'a Theme,
    pub on_select_mode: ViewModeHandler,
}

pub struct ViewModeSwitcher;

impl ViewModeSwitcher {
    pub fn render(props: ViewModeSwitcherProps) -> impl IntoElement {
        let theme = props.theme;
        let modes = [PackageViewMode::Cards, PackageViewMode::Table];

        let mut container = div()
            .id("workbench_view_mode_switcher")
            .flex()
            .items_center()
            .gap(px(2.0));

        for mode in modes {
            let is_active = props.current_mode == mode;
            let on_select = props.on_select_mode.clone();
            let on_select_key = props.on_select_mode.clone();
            let element_id = match mode {
                PackageViewMode::Cards => "view_mode_cards",
                PackageViewMode::Table => "view_mode_table",
            };
            let focus_border = theme.border_focus;
            let hover_bg = theme.bg_surface_hover;
            container =
                container.child(
                    div()
                        .id(element_id)
                        .focusable()
                        .tab_stop(true)
                        .focus(move |s| s.border_1().border_color(focus_border))
                        .on_key_down(move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                on_select_key(mode, window, cx);
                            }
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(26.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .bg(if is_active {
                            theme.bg_surface_active
                        } else {
                            gpui::rgba(0x00000000)
                        })
                        .text_color(if is_active {
                            theme.accent
                        } else {
                            theme.text_muted
                        })
                        .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
                        .child(svg().path(mode.icon().path()).size(px(14.0)).text_color(
                            if is_active {
                                theme.accent
                            } else {
                                theme.text_muted
                            },
                        ))
                        .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                            on_select(mode, window, cx);
                        }),
                );
        }

        container
    }
}
