use crate::icons::AppIcon;
use crate::theme::Theme;
use crate::visual_style::paint::apply_surface_projection;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::*;
use std::rc::Rc;

pub type MenuActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type MenuKeyHandler = Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>;

/// Cycle de vie cinématique d'un menu déroulant de station de travail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuLifecycle {
    Opening,
    Open,
    Closing,
    Closed,
}

pub struct MenuSurfaceProps<'a> {
    pub id: ElementId,
    pub theme: &'a Theme,
    pub min_width: Pixels,
    pub reduce_motion: bool,
    pub lifecycle: MenuLifecycle,
    pub anim_epoch: usize,
    pub focus_handle: Option<FocusHandle>,
    pub visual_style: VisualStyleId,
    pub on_close: MenuActionHandler,
    pub on_key_navigate: Option<MenuKeyHandler>,
    pub children: Vec<AnyElement>,
}

/// Conteneur de surface de menu desktop natif (ancré, animé, focalisable avec navigation clavier roving)
pub struct MenuSurface;

impl MenuSurface {
    pub fn render(props: MenuSurfaceProps) -> impl IntoElement {
        let theme = props.theme;
        let on_close_out = props.on_close.clone();
        let on_close_key = props.on_close.clone();
        let on_nav = props.on_key_navigate.clone();

        let mut base = div()
            .id(props.id)
            .relative()
            .occlude()
            .key_context("MenuSurface")
            .focusable()
            .tab_stop(true)
            .on_mouse_down_out(move |_ev, window, cx| {
                on_close_out(window, cx);
            })
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "escape" => on_close_key(window, cx),
                    "up" | "down" | "home" | "end" | "enter" | "space" => {
                        if let Some(ref nav) = on_nav {
                            nav(key, window, cx);
                        }
                    }
                    _ => {}
                }
            })
            .flex()
            .flex_col()
            .min_w(props.min_width)
            .max_w(px(340.0))
            .p_1p5()
            .border_1()
            .border_color(theme.border)
            .rounded_md();

        base = apply_surface_projection(
            base,
            SurfaceRole::Menu,
            props.visual_style,
            theme.bg_surface,
            theme,
        );

        base = base.children(props.children);

        if let Some(ref fh) = props.focus_handle {
            base = base.track_focus(fh);
        }

        if props.reduce_motion {
            match props.lifecycle {
                MenuLifecycle::Closing | MenuLifecycle::Closed => div().into_any_element(),
                _ => base.opacity(1.0).into_any_element(),
            }
        } else {
            match props.lifecycle {
                MenuLifecycle::Opening => base
                    .with_animation(
                        ("menu_open_anim", props.anim_epoch),
                        Animation::new(std::time::Duration::from_millis(150))
                            .with_easing(gpui::ease_out_quint()),
                        |el, delta| {
                            let offset_y = px(-4.0 * (1.0 - delta));
                            el.opacity(delta).top(offset_y)
                        },
                    )
                    .into_any_element(),
                MenuLifecycle::Open => base.opacity(1.0).into_any_element(),
                MenuLifecycle::Closing => base
                    .with_animation(
                        ("menu_close_anim", props.anim_epoch),
                        Animation::new(std::time::Duration::from_millis(100))
                            .with_easing(gpui::ease_out_quint()),
                        |el, delta| {
                            let offset_y = px(-2.0 * delta);
                            el.opacity(1.0 - delta).top(offset_y)
                        },
                    )
                    .into_any_element(),
                MenuLifecycle::Closed => div().into_any_element(),
            }
        }
    }
}

/// Séparateur horizontal au sein d'un menu
pub struct MenuDivider;

impl MenuDivider {
    pub fn render(theme: &Theme) -> impl IntoElement {
        div().h(px(1.0)).my(px(4.0)).bg(theme.border)
    }
}

/// En-tête de section au sein d'un menu complexe
pub struct MenuSection;

impl MenuSection {
    pub fn render(title: &'static str, theme: &Theme) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.text_muted)
            .child(title)
    }
}

/// Élément de sélection exclusive avec checkmark desktop natif (pas de radio circle)
pub struct MenuCheckmarkItem;

impl MenuCheckmarkItem {
    pub fn render(
        id: ElementId,
        label: impl Into<SharedString>,
        is_selected: bool,
        is_highlighted: bool,
        theme: &Theme,
        on_select: MenuActionHandler,
    ) -> impl IntoElement {
        let hover_bg = theme.bg_surface_hover;
        let border_focus = theme.border_focus;
        let on_key_select = on_select.clone();

        div()
            .id(id)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.bg(hover_bg).border_color(border_focus))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_key_select(window, cx);
                }
            })
            .flex()
            .items_center()
            .gap(px(8.0))
            .px_2p5()
            .py_1p5()
            .rounded_sm()
            .cursor_pointer()
            .text_xs()
            .bg(if is_highlighted {
                theme.bg_surface_hover
            } else {
                gpui::rgba(0x00000000)
            })
            .text_color(if is_selected {
                theme.accent
            } else {
                theme.text_secondary
            })
            .font_weight(if is_selected {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(14.0))
                    .child(if is_selected {
                        svg()
                            .path(AppIcon::Check.path())
                            .size(px(12.0))
                            .text_color(theme.accent)
                            .into_any_element()
                    } else {
                        div().size(px(12.0)).into_any_element()
                    }),
            )
            .child(label.into())
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_select(window, cx);
            })
    }
}

/// Propriétés d'un élément de sélection multiple de menu
pub struct MenuCheckItemProps<'a> {
    pub id: ElementId,
    pub retry_id: ElementId,
    pub label: SharedString,
    pub is_checked: bool,
    pub is_highlighted: bool,
    pub health_label: Option<&'static str>,
    pub is_failed: bool,
    pub theme: &'a Theme,
    pub on_toggle: MenuActionHandler,
    pub on_retry: Option<MenuActionHandler>,
}

/// Élément de sélection multiple type case à cocher avec état de santé et action Retry accessible au clavier
pub struct MenuCheckItem;

impl MenuCheckItem {
    pub fn render(props: MenuCheckItemProps) -> impl IntoElement {
        let theme = props.theme;
        let hover_bg = theme.bg_surface_hover;
        let border_focus = theme.border_focus;
        let on_key_toggle = props.on_toggle.clone();

        let retry_btn = if let Some(ref retry_cb) = props.on_retry {
            let cb = retry_cb.clone();
            let retry_hover = theme.accent_hover;
            Some(
                div()
                    .id(props.retry_id)
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(border_focus))
                    .on_key_down({
                        let cb_key = cb.clone();
                        move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                cb_key(window, cx);
                            }
                        }
                    })
                    .px_1p5()
                    .py(px(2.0))
                    .rounded_sm()
                    .bg(Rgba {
                        a: 0.15,
                        ..theme.accent
                    })
                    .border_1()
                    .border_color(theme.accent)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .cursor_pointer()
                    .hover(move |s| s.bg(retry_hover).text_color(theme.bg_app))
                    .child("Retry")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        cb(window, cx);
                    }),
            )
        } else {
            None
        };

        div()
            .id(props.id)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.bg(hover_bg).border_color(border_focus))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_key_toggle(window, cx);
                }
            })
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_sm()
            .cursor_pointer()
            .text_xs()
            .bg(if props.is_highlighted {
                theme.bg_surface_hover
            } else {
                gpui::rgba(0x00000000)
            })
            .text_color(theme.text_primary)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_3p5()
                            .rounded_xs()
                            .border_1()
                            .border_color(if props.is_checked {
                                theme.accent
                            } else {
                                theme.border
                            })
                            .bg(if props.is_checked {
                                theme.accent
                            } else {
                                theme.bg_surface
                            })
                            .child(if props.is_checked {
                                svg()
                                    .path(AppIcon::Check.path())
                                    .size(px(10.0))
                                    .text_color(theme.bg_app)
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            }),
                    )
                    .child(props.label),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .children(props.health_label.map(|lbl| {
                        div()
                            .text_xs()
                            .text_color(if props.is_failed {
                                theme.danger
                            } else {
                                theme.text_muted
                            })
                            .child(lbl)
                    }))
                    .children(retry_btn),
            )
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                (props.on_toggle)(window, cx);
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_menu_lifecycle_transitions() {
        assert_ne!(MenuLifecycle::Opening, MenuLifecycle::Open);
        assert_ne!(MenuLifecycle::Open, MenuLifecycle::Closing);
        assert_ne!(MenuLifecycle::Closing, MenuLifecycle::Closed);
    }
}
