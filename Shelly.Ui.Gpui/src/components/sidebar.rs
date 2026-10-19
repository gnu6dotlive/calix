use crate::components::status_pill::StatusPill;
use crate::icons::AppIcon;
use crate::state::NavDestination;
use crate::theme::Theme;
use crate::visual_style::paint::apply_surface_projection;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub type NavDestinationHandler = Rc<dyn Fn(NavDestination, &mut Window, &mut App) + 'static>;
pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct SidebarProps<'a> {
    pub active_destination: NavDestination,
    pub updates_count: usize,
    pub is_collapsed: bool,
    pub current_width: f32,
    pub theme: &'a Theme,
    pub visual_style: VisualStyleId,
    pub on_select_destination: NavDestinationHandler,
    pub on_toggle_collapse: WindowActionHandler,
}

pub struct Sidebar;

struct DestinationTooltip {
    label: &'static str,
    theme: Theme,
}

impl Render for DestinationTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(self.theme.bg_surface)
            .text_color(self.theme.text_primary)
            .border_1()
            .border_color(self.theme.border)
            .rounded_md()
            .px(px(8.0))
            .py(px(4.0))
            .text_xs()
            .shadow_md()
            .child(self.label)
    }
}

impl Sidebar {
    fn render_nav_item(
        dest: NavDestination,
        active_dest: NavDestination,
        badge_count: usize,
        is_collapsed: bool,
        theme: &Theme,
        on_select: NavDestinationHandler,
    ) -> impl IntoElement {
        let is_active = active_dest == dest;
        let icon = dest.icon();
        let label = dest.label();

        let element_id = match dest {
            NavDestination::Browse => "nav_dest_browse",
            NavDestination::Installed => "nav_dest_installed",
            NavDestination::Updates => "nav_dest_updates",
            NavDestination::News => "nav_dest_news",
            NavDestination::Settings => "nav_dest_settings",
            NavDestination::RenderLab => unreachable!("RenderLab is excluded from sidebar"),
        };

        let focus_border = theme.border_focus;
        let on_select_key = on_select.clone();

        let base = div()
            .id(element_id)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_select_key(dest, window, cx);
                }
            })
            .flex()
            .items_center()
            .gap(px(10.0))
            .px(if is_collapsed { px(12.0) } else { px(14.0) })
            .py(px(8.0))
            .mx(px(6.0))
            .rounded_md()
            .cursor_pointer()
            .overflow_hidden()
            .text_sm();

        let styled = if is_active {
            base.bg(theme.bg_surface_active)
                .text_color(theme.text_primary)
                .font_weight(FontWeight::SEMIBOLD)
                .border_l_2()
                .border_color(theme.accent)
        } else {
            let hover_bg = theme.bg_surface_hover;
            base.bg(theme.bg_sidebar)
                .text_color(theme.text_secondary)
                .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
        };

        let icon_color = if is_active {
            theme.accent
        } else {
            theme.text_secondary
        };

        let mut item = styled.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .child(svg().path(icon.path()).size_4().text_color(icon_color)),
        );

        if !is_collapsed {
            item = item.child(div().flex_1().overflow_hidden().child(label));

            if badge_count > 0 {
                item = item.child(StatusPill::badge_count(badge_count, theme));
            }
        } else {
            let t = *theme;
            item = item.tooltip(move |_window, cx| {
                cx.new(|_cx| DestinationTooltip { label, theme: t }).into()
            });
        }

        item.on_mouse_down(MouseButton::Left, move |_e, window, cx| {
            on_select(dest, window, cx);
        })
    }

    pub fn render(props: SidebarProps) -> impl IntoElement {
        let theme = props.theme;
        let is_collapsed = props.is_collapsed;
        let width = px(props.current_width);

        let destinations = [
            NavDestination::Browse,
            NavDestination::Installed,
            NavDestination::Updates,
            NavDestination::News,
            NavDestination::Settings,
        ];

        let mut nav_items = div().flex().flex_col().gap(px(2.0)).py(px(8.0));

        for dest in destinations {
            let badge = if dest == NavDestination::Updates {
                props.updates_count
            } else {
                0
            };

            nav_items = nav_items.child(Self::render_nav_item(
                dest,
                props.active_destination,
                badge,
                is_collapsed,
                theme,
                props.on_select_destination.clone(),
            ));
        }

        let collapse_toggle = {
            let on_toggle = props.on_toggle_collapse.clone();
            let on_toggle_key = props.on_toggle_collapse.clone();
            let focus_border = theme.border_focus;
            let hover_bg = theme.bg_surface_hover;
            let icon = if is_collapsed {
                AppIcon::Expand
            } else {
                AppIcon::Collapse
            };
            let toggle_label = if is_collapsed { "Expand" } else { "Collapse" };
            let t = *theme;
            div()
                .id("sidebar_collapse_toggle")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(focus_border))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        on_toggle_key(window, cx);
                    }
                })
                .flex()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .py(px(8.0))
                .mx(px(6.0))
                .rounded_md()
                .cursor_pointer()
                .overflow_hidden()
                .text_xs()
                .text_color(theme.text_muted)
                .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
                .child(
                    svg()
                        .path(icon.path())
                        .size_3()
                        .text_color(theme.text_muted),
                )
                .when(!is_collapsed, |el| el.child("Collapse"))
                .when(is_collapsed, |el| {
                    el.tooltip(move |_window, cx| {
                        cx.new(|_cx| DestinationTooltip {
                            label: toggle_label,
                            theme: t,
                        })
                        .into()
                    })
                })
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    on_toggle(window, cx);
                })
        };

        let base_container = div()
            .flex()
            .flex_col()
            .justify_between()
            .w(width)
            .h_full()
            .overflow_hidden()
            .border_r_1()
            .border_color(theme.border)
            .py(px(6.0));

        apply_surface_projection(
            base_container,
            SurfaceRole::NavigationRail,
            props.visual_style,
            theme.bg_sidebar,
            theme,
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                // Header de la barre latérale avec logo Calix SVG
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .px(if is_collapsed { px(14.0) } else { px(16.0) })
                        .py(px(10.0))
                        .border_b_1()
                        .border_color(theme.border)
                        .child(
                            svg()
                                .path(AppIcon::Calix.path())
                                .size_5()
                                .text_color(theme.accent),
                        )
                        .child(if !is_collapsed {
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("CALIX GPUI")
                        } else {
                            div()
                        }),
                )
                .child(nav_items),
        )
        .child(collapse_toggle)
    }
}
