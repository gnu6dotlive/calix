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
                            .with_easing(crate::state::motion::ease_out),
                        |el, delta| {
                            // La surface tombe de 4 px dans son ancrage :
                            // l'inset déplace l'élément en flux sans
                            // pousser ce qui l'entoure.
                            el.opacity(delta).top(px(-4.0 * (1.0 - delta)))
                        },
                    )
                    .into_any_element(),
                MenuLifecycle::Open => base.opacity(1.0).into_any_element(),
                MenuLifecycle::Closing => base
                    .with_animation(
                        ("menu_close_anim", props.anim_epoch),
                        Animation::new(std::time::Duration::from_millis(100))
                            .with_easing(crate::state::motion::ease_out),
                        |el, delta| {
                            // Elle repart par où elle est venue : elle se
                            // soulève en s'éteignant.
                            el.opacity(1.0 - delta).top(px(-2.0 * delta))
                        },
                    )
                    .into_any_element(),
                MenuLifecycle::Closed => div().into_any_element(),
            }
        }
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

    /// L'ouverture d'un menu, mesurée sur l'image : la surface tombe de
    /// quatre pixels dans son ancrage.
    ///
    /// La première image est prise dans l'image même où l'animation naît —
    /// l'état de l'animation est créé au premier placement — et la seconde
    /// après le voyage, quand le menu est posé.
    #[gpui::test]
    fn the_menu_drops_by_four_pixels_and_lands(cx: &mut gpui::TestAppContext) {
        struct Fixture {
            theme: Theme,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                let focus = _cx.focus_handle();
                div()
                    .size_full()
                    .child(MenuSurface::render(MenuSurfaceProps {
                        id: "menu_surface_probe".into(),
                        theme: &theme,
                        min_width: px(180.0),
                        reduce_motion: false,
                        lifecycle: MenuLifecycle::Opening,
                        anim_epoch: 3,
                        focus_handle: Some(focus),
                        visual_style: VisualStyleId::default(),
                        on_close: std::rc::Rc::new(|_, _| {}),
                        on_key_navigate: None,
                        children: vec![div()
                            .debug_selector(|| "menu_item_probe".to_string())
                            .child("All states")
                            .into_any_element()],
                    }))
            }
        }

        let cx = cx.add_empty_window();
        let fixture = cx.new(|_| Fixture {
            theme: Theme::dark(),
        });

        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(600.0), px(400.0)),
                |_window, _cx| fixture.clone(),
            );
        };

        draw(cx);
        let departure = cx
            .debug_bounds("menu_item_probe")
            .expect("le menu est dessiné");

        // L'ouverture dure 150 ms ; on attend plus, et on dessine encore.
        std::thread::sleep(std::time::Duration::from_millis(260));
        draw(cx);
        let arrival = cx
            .debug_bounds("menu_item_probe")
            .expect("le menu est dessiné");

        assert_eq!(
            arrival.origin.y - departure.origin.y,
            px(4.0),
            "le menu est tombé de quatre pixels, et il s'est posé"
        );
    }
}
