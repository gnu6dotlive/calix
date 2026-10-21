use crate::components::status_pill::StatusPill;
use crate::icons::AppIcon;
use crate::state::motion::{carrier_travel, ease_in_out, SelectionMarker};
use crate::state::{MotionDurations, NavScope, Place, RailEntry};
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use crate::visual_style::paint::apply_surface_projection;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub type RailSelectHandler = Rc<dyn Fn(RailEntry, &mut Window, &mut App) + 'static>;
pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct SidebarProps<'a> {
    pub active_place: Place,
    pub active_scope: NavScope,
    pub updates_count: usize,
    pub is_collapsed: bool,
    pub current_width: f32,
    pub theme: &'a Theme,
    pub visual_style: VisualStyleId,
    pub on_select: RailSelectHandler,
    pub on_toggle_collapse: WindowActionHandler,
    /// Le trait de la sélection : la rangée qu'il marque, celle qu'il quitte,
    /// et le numéro du voyage.
    pub marker: Option<SelectionMarker>,
    pub reduce_motion: bool,
}

pub struct Sidebar;

/// Le haut du trait de la sélection, dans le rail.
///
/// Les entrées du rail ont toutes la même hauteur et le même écart : la
/// position du trait est une multiplication, pas une mesure. C'est ce qui
/// permet au trait de voyager entre deux rangées sans mesurer la mise en page.
pub fn rail_mark_top(index: f32) -> f32 {
    UiMetrics::RAIL_TOP_PAD + index * (UiMetrics::RAIL_ROW_HEIGHT + UiMetrics::RAIL_ROW_GAP)
}

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
        entry: RailEntry,
        active_place: Place,
        active_scope: NavScope,
        badge_count: usize,
        is_collapsed: bool,
        theme: &Theme,
        on_select: RailSelectHandler,
    ) -> impl IntoElement {
        let is_active = entry.is_active(active_place, active_scope);
        let icon = entry.icon;
        let label = entry.label;
        let element_id = entry.id;

        let focus_border = theme.border_focus;
        let on_select_key = on_select.clone();

        let base = div()
            .id(element_id)
            // The handle the interaction gate clicks: the harness reads the
            // bounds of a rail entry under this name. A noop outside the
            // tests.
            .debug_selector(|| element_id.to_string())
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_select_key(entry, window, cx);
                }
            })
            .flex()
            .items_center()
            .gap(px(10.0))
            .px(if is_collapsed { px(12.0) } else { px(14.0) })
            .h(px(UiMetrics::RAIL_ROW_HEIGHT))
            .mx(px(6.0))
            .rounded_md()
            .cursor_pointer()
            .overflow_hidden()
            .text_sm();

        let styled = if is_active {
            base.bg(theme.bg_surface_active)
                .text_color(theme.text_primary)
                .font_weight(FontWeight::SEMIBOLD)
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
            on_select(entry, window, cx);
        })
    }

    pub fn render(props: SidebarProps) -> impl IntoElement {
        let theme = props.theme;
        let is_collapsed = props.is_collapsed;
        let width = px(props.current_width);

        let mut nav_rows = div()
            .flex()
            .flex_col()
            .gap(px(UiMetrics::RAIL_ROW_GAP))
            .py(px(UiMetrics::RAIL_TOP_PAD));

        for entry in RailEntry::ALL {
            let badge = if entry.scope == Some(NavScope::Updates) {
                props.updates_count
            } else {
                0
            };

            nav_rows = nav_rows.child(Self::render_nav_item(
                *entry,
                props.active_place,
                props.active_scope,
                badge,
                is_collapsed,
                theme,
                props.on_select.clone(),
            ));
        }

        // Le trait de la sélection : il voyage d'une entrée à l'autre.
        //
        // La sélection n'est pas un allumage, c'est un déplacement : le trait
        // part de l'entrée quittée et se pose sur l'entrée choisie. Un voyage
        // prend la courbe de l'aller-retour, pas celle de l'entrée : la sortie
        // partirait si vite que le trait paraîtrait sauter. Les rangées sont
        // égales, donc la position du trait se calcule.
        let mark = props.marker.map(|marker| {
            let row = marker.index as f32;
            let from = marker.from.map(|index| index as f32).unwrap_or(row);
            let bar = div()
                .id("sidebar_mark")
                .absolute()
                .left(px(6.0))
                .top(px(rail_mark_top(row)))
                .w(px(2.0))
                .h(px(UiMetrics::RAIL_ROW_HEIGHT))
                .rounded_full()
                .bg(theme.accent);
            if props.reduce_motion {
                bar.into_any_element()
            } else {
                bar.with_animation(
                    ("sidebar_mark", marker.epoch),
                    Animation::new(MotionDurations::EMPHASIS).with_easing(ease_in_out),
                    move |bar, delta| {
                        bar.top(px(rail_mark_top(carrier_travel(from, row, delta))))
                    },
                )
                .into_any_element()
            }
        });

        let nav_items = div().relative().child(nav_rows).children(mark);

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
                            // The shell mark, painted by the accent: the mark
                            // and the interface speak the same colour. It is
                            // the product's voice, not an icon in a corner.
                            svg()
                                .path(AppIcon::Calix.path())
                                .size_8()
                                .text_color(theme.accent),
                        )
                        .child(if !is_collapsed {
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("CALIX")
                        } else {
                            div()
                        }),
                )
                .child(nav_items),
        )
        .child(collapse_toggle)
    }
}

#[cfg(test)]
mod tests {
    use super::{rail_mark_top, Sidebar, SidebarProps};
    use crate::state::session::RailEntry;
    use crate::state::{NavScope, Place};
    use crate::theme::Theme;
    use crate::ui_metrics::UiMetrics;
    use crate::visual_style::VisualStyleId;
    use core::prelude::v1::test;

    #[test]
    fn the_rail_mark_lands_on_the_row_it_marks() {
        // Les entrées du rail sont égales : le trait avance d'une rangée et
        // d'un écart, exactement.
        let pitch = UiMetrics::RAIL_ROW_HEIGHT + UiMetrics::RAIL_ROW_GAP;
        assert_eq!(rail_mark_top(0.0), UiMetrics::RAIL_TOP_PAD);
        assert_eq!(rail_mark_top(1.0), UiMetrics::RAIL_TOP_PAD + pitch);
        assert_eq!(rail_mark_top(3.0), UiMetrics::RAIL_TOP_PAD + 3.0 * pitch);
        // Entre deux rangées, quand il voyage.
        assert_eq!(rail_mark_top(1.5), UiMetrics::RAIL_TOP_PAD + 1.5 * pitch);
        // La rangée suivante commence sous celle-ci : le trait ne chevauche
        // jamais la mise en page.
        assert!(rail_mark_top(1.0) > rail_mark_top(0.0) + UiMetrics::RAIL_ROW_HEIGHT);
    }

    /// La porte de l'interaction : un clic sur une entrée du rail change le
    /// lieu ou le périmètre.
    ///
    /// C'est la première porte de la série, et elle porte la navigation
    /// elle-même : le rail est la surface de priorité 1, et un clic qui ne
    /// change rien est un rail mort.
    ///
    /// Le protocole du banc : un clic est une suite d'événements séparés par
    /// des images — comme pour un utilisateur. Le premier clic déplace le
    /// focus, le focus change le style, la fenêtre se redessine, et le clic
    /// suivant est dispatché contre les hitboxes de ce redessin. Sans
    /// l'image entre les deux, le second clic ne trouve rien.
    #[gpui::test]
    fn a_click_on_a_rail_entry_changes_the_place_or_the_scope(
        cx: &mut gpui::TestAppContext,
    ) {
        use gpui::prelude::*;
        use gpui::{div, point, px, size, Context, Render, Window};
        use std::cell::RefCell;
        use std::rc::Rc;

        struct Fixture {
            theme: Theme,
            chosen: Rc<RefCell<Option<RailEntry>>>,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                let chosen = self.chosen.clone();
                div().size_full().child(Sidebar::render(SidebarProps {
                    active_place: Place::Packages,
                    active_scope: NavScope::Installed,
                    updates_count: 14,
                    is_collapsed: false,
                    current_width: UiMetrics::SIDEBAR_EXPANDED,
                    theme: &theme,
                    visual_style: VisualStyleId::default(),
                    on_select: Rc::new(move |entry, _w, _cx| {
                        *chosen.borrow_mut() = Some(entry);
                    }),
                    on_toggle_collapse: Rc::new(|_w, _cx| {}),
                    marker: None,
                    reduce_motion: false,
                }))
            }
        }

        let cx = cx.add_empty_window();
        let chosen = Rc::new(RefCell::new(None));
        let fixture = cx.new(|_| Fixture {
            theme: Theme::dark(),
            chosen: chosen.clone(),
        });

        let frame = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(240.0), px(400.0)),
                |_window, _cx| fixture.clone(),
            );
        };
        // The frame comes *before* the lookup: the bounds are the ones the
        // last frame drew. Then the click is dispatched against them.
        let click = |cx: &mut gpui::VisualTestContext, selector: &'static str| {
            frame(cx);
            let bounds = cx
                .debug_bounds(selector)
                .unwrap_or_else(|| panic!("{selector} is drawn"));
            cx.simulate_click(
                point(
                    bounds.origin.x + bounds.size.width / 2.0,
                    bounds.origin.y + bounds.size.height / 2.0,
                ),
                gpui::Modifiers::default(),
            );
        };

        // The three scopes, the two places: every entry answers.
        for entry in RailEntry::ALL {
            *chosen.borrow_mut() = None;
            click(cx, entry.id);
            assert_eq!(
                *chosen.borrow(),
                Some(*entry),
                "le clic sur {} ({} l'a choisie",
                entry.label,
                entry.id
            );
        }
    }
}
