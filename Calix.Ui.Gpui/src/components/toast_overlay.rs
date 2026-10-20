use crate::icons::AppIcon;
use crate::state::{Toast, ToastAction, ToastKind, ToastLifecycle};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type ToastDismissHandler = Rc<dyn Fn(u64, &mut Window, &mut App) + 'static>;
pub type ToastActionHandler = Rc<dyn Fn(ToastAction, &mut Window, &mut App) + 'static>;

pub struct ToastOverlayProps<'a> {
    pub toasts: &'a [Toast],
    pub reduce_motion: bool,
    pub theme: &'a Theme,
    pub on_dismiss: ToastDismissHandler,
    pub on_action: ToastActionHandler,
}

pub struct ToastOverlay;

impl ToastOverlay {
    pub fn render(props: ToastOverlayProps) -> impl IntoElement {
        let theme = props.theme;
        let mut container = div()
            .id("toast_overlay_container")
            .absolute()
            .bottom(px(16.0))
            .right(px(16.0))
            .flex()
            .flex_col()
            .gap(px(8.0));

        for toast in props.toasts {
            let toast_id = toast.id;
            let on_dismiss = props.on_dismiss.clone();
            let on_action = props.on_action.clone();

            let border_color = match toast.kind {
                ToastKind::Info => theme.accent,
                ToastKind::Success => theme.success,
                ToastKind::Warning => theme.warning,
                ToastKind::Error => theme.danger,
            };

            let mut toast_el = div()
                .id(ElementId::NamedInteger("toast-item".into(), toast.id))
                .flex()
                .flex_col()
                .w(px(320.0))
                .p_3()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(border_color);

            // En-tête du toast (Icône, Titre, Bouton fermeture)
            let header = div()
                .flex()
                .items_center()
                .justify_between()
                .mb_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            svg()
                                .path(toast.kind.icon().path())
                                .size_3_5()
                                .text_color(border_color),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(toast.title.clone()),
                        ),
                )
                .child({
                    let on_dismiss_key = on_dismiss.clone();
                    let border_focus = theme.border_focus;
                    div()
                        .id(ElementId::NamedInteger("toast_dismiss".into(), toast_id))
                        .focusable()
                        .tab_stop(true)
                        .focus(move |s| s.border_1().border_color(border_focus))
                        .on_key_down(move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                on_dismiss_key(toast_id, window, cx);
                            }
                        })
                        .cursor_pointer()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .hover(move |s| s.text_color(theme.text_primary))
                        .child(
                            svg()
                                .path(AppIcon::Close.path())
                                .size_3()
                                .text_color(theme.text_muted),
                        )
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            on_dismiss(toast_id, window, cx);
                        })
                });

            toast_el = toast_el.child(header);

            // Message du toast
            toast_el = toast_el.child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .child(toast.message.clone()),
            );

            // Action optionnelle (ex: Ouvrir les logs)
            if let Some(action) = &toast.action {
                let act = action.clone();
                let act_key = action.clone();
                let on_action_key = on_action.clone();
                let border_focus = theme.border_focus;
                let action_btn = div().flex().justify_end().mt_2().child(
                    div()
                        .id(ElementId::NamedInteger("toast_action".into(), toast_id))
                        .focusable()
                        .tab_stop(true)
                        .focus(move |s| s.border_1().border_color(border_focus))
                        .on_key_down(move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                on_action_key(act_key.clone(), window, cx);
                            }
                        })
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(theme.bg_surface_active)
                        .border_1()
                        .border_color(theme.border)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.accent)
                        .cursor_pointer()
                        .hover(move |s| s.bg(theme.bg_surface_hover))
                        .child("Open Logs")
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            on_action(act.clone(), window, cx);
                        }),
                );
                toast_el = toast_el.child(action_btn);
            }

            if props.reduce_motion {
                container = container.child(toast_el.opacity(1.0));
            } else {
                match toast.lifecycle {
                    ToastLifecycle::Entering => {
                        let animated = toast_el.with_animation(
                            ElementId::NamedInteger("toast-enter".into(), toast_id),
                            Animation::new(std::time::Duration::from_millis(120))
                                .with_easing(gpui::ease_out_quint()),
                            |el, delta| el.opacity(delta),
                        );
                        container = container.child(animated);
                    }
                    ToastLifecycle::Visible => {
                        container = container.child(toast_el.opacity(1.0));
                    }
                    ToastLifecycle::Exiting => {
                        let animated = toast_el.with_animation(
                            ElementId::NamedInteger("toast-exit".into(), toast_id),
                            Animation::new(std::time::Duration::from_millis(120))
                                .with_easing(gpui::ease_out_quint()),
                            |el, delta| el.opacity(1.0 - delta),
                        );
                        container = container.child(animated);
                    }
                }
            }
        }

        container
    }
}
