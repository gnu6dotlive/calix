use crate::components::menu::{
    MenuCheckmarkItem, MenuKeyHandler, MenuLifecycle, MenuSection, MenuSurface, MenuSurfaceProps,
};
use crate::components::search_input::SearchInputView;
use crate::components::view_mode_switcher::{ViewModeSwitcher, ViewModeSwitcherProps};
use crate::state::motion::SelectionMarker;
use crate::elements::adaptive::AdaptiveRow;
use crate::state::query::{PackageStateFilter, SortMode, WorkbenchBreakpoint};
use crate::ui_metrics::UiMetrics;
use crate::state::session::NavScope;
use crate::state::PackageViewMode;
use crate::theme::Theme;
use crate::visual_style::paint::apply_surface_projection;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbenchMenu {
    State,
    Sort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveMenuState {
    pub menu: WorkbenchMenu,
    pub lifecycle: MenuLifecycle,
    pub anim_epoch: usize,
    pub highlighted_index: usize,
}

pub type StateSelectHandler = Rc<dyn Fn(PackageStateFilter, &mut Window, &mut App) + 'static>;
pub type SortSelectHandler = Rc<dyn Fn(SortMode, &mut Window, &mut App) + 'static>;
pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;
pub type MenuToggleHandler = Rc<dyn Fn(Option<WorkbenchMenu>, &mut Window, &mut App) + 'static>;
pub type ActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct QueryWorkbenchProps<'a> {
    pub search_input: Entity<SearchInputView>,
    /// Le périmètre : il décide de ce que la banque et le filtre d'état
    /// proposent.
    pub scope: NavScope,
    pub state_filter: PackageStateFilter,
    pub sort_mode: SortMode,
    pub view_mode: PackageViewMode,
    /// Le porteur du commutateur de vue : la marque qui voyage d'une cellule
    /// à l'autre.
    pub view_marker: Option<SelectionMarker>,
    pub active_menu_state: Option<ActiveMenuState>,
    pub reduce_motion: bool,
    pub theme: &'a Theme,
    pub visual_style: VisualStyleId,
    pub state_btn_focus: FocusHandle,
    pub sort_btn_focus: FocusHandle,
    pub menu_surface_focus: FocusHandle,
    pub on_select_state: StateSelectHandler,
    pub on_select_sort: SortSelectHandler,
    pub on_select_view_mode: ViewModeHandler,
    pub on_toggle_menu: MenuToggleHandler,
    pub on_navigate_menu: MenuKeyHandler,
    /// L'action propre au périmètre, quand il en a une.
    pub scope_action: Option<ScopeAction>,
}

/// L'action d'un périmètre : dans Mises à jour, la mise à niveau de tout.
#[derive(Clone)]
pub struct ScopeAction {
    pub label: &'static str,
    pub handler: ActionHandler,
}

/// La barre de requête : la recherche, l'état, le tri et la vue.
pub struct QueryWorkbench;

impl QueryWorkbench {
    pub fn render(props: &QueryWorkbenchProps) -> impl IntoElement {
        let theme = props.theme;

        // ── 1. Row 1: Dominant Full-Width Search Input ────────────────────────
        let row1 = div()
            .id("workbench_row1_search")
            // The handle the bench measures: the harness reads the bounds of
            // the row under this name. A noop outside the tests.
            .debug_selector(|| "workbench_row1_search".to_string())
            .w_full()
            .h(px(40.0))
            .child(props.search_input.clone());

        // ── 2. Row 2: the toolbar, in the richest form that fits ─────────────
        //
        // The three forms are drawn, and the element that lays them out keeps
        // the first whose natural width fits the room the row really has —
        // measured in its `prepaint`, with the real labels and font. No pixel
        // breakpoint: a longer translation or a larger font picks the next
        // form by itself.
        let row2 = AdaptiveRow::new(
            WorkbenchBreakpoint::RICHEST_FIRST
                .iter()
                .map(|&form| Self::toolbar(props, form))
                .collect(),
            px(UiMetrics::QUERY_CONTROLS_HEIGHT),
        );

        // ── 8. Final Container Assembly (Fixed Invariant Geometry) ────────────
        let base_container = div()
            .id("query_workbench_container")
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .p(px(8.0))
            .border_b_1()
            .border_color(theme.border);

        apply_surface_projection(
            base_container,
            SurfaceRole::QueryChrome,
            props.visual_style,
            theme.bg_sidebar,
            theme,
        )
        .child(row1)
        .child(row2)
    }

    /// One form of the toolbar: the state, the sort, the view and the scope's
    /// action. The labels say what the commands do when there is room, and
    /// what they are when there is not.
    fn toolbar(props: &QueryWorkbenchProps, breakpoint: WorkbenchBreakpoint) -> AnyElement {
        let theme = props.theme;
        // ── 2. The menus' shared state ────────────────────────────────────────
        let on_toggle_menu = props.on_toggle_menu.clone();
        let active_state = props.active_menu_state;
        // L'anneau de focus des deux menus qui restent.
        let focus_border = theme.border_focus;

        // ── 3. State Dropdown ─────────────────────────────────────────────────
        let is_state_open = active_state.is_some_and(|s| {
            s.menu == WorkbenchMenu::State && s.lifecycle != MenuLifecycle::Closed
        });
        let state_lifecycle = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::State).then_some(s.lifecycle))
            .unwrap_or(MenuLifecycle::Closed);
        let state_epoch = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::State).then_some(s.anim_epoch))
            .unwrap_or(0);
        let state_highlighted = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::State).then_some(s.highlighted_index))
            .unwrap_or(0);

        let on_close_state = {
            let cb = on_toggle_menu.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                cb(None, window, cx);
            })
        };

        // Le bouton d'état est là à toute largeur : c'est le seul contrôle
        // de cet axe, et une fenêtre ne descend pas sous 1024 px.
        let state_btn = {
            let mut state_items = Vec::new();
            state_items.push(MenuSection::render("Filter by State", theme).into_any_element());

            for (idx, state) in PackageStateFilter::options_for(props.scope)
                .iter()
                .copied()
                .enumerate()
            {
                let on_select = props.on_select_state.clone();
                let close_cb = on_close_state.clone();
                let state_val = state;
                let id_str = match state {
                    PackageStateFilter::All => "state_dropdown_all",
                    PackageStateFilter::Installed => "state_dropdown_installed",
                    PackageStateFilter::NotInstalled => "state_dropdown_not_installed",
                    PackageStateFilter::UpdatesAvailable => "state_dropdown_updates",
                };
                state_items.push(
                    MenuCheckmarkItem::render(
                        id_str.into(),
                        state.label(),
                        props.state_filter == state,
                        is_state_open && state_highlighted == idx,
                        theme,
                        Rc::new(move |w, a| {
                            on_select(state_val, w, a);
                            close_cb(w, a);
                        }),
                    )
                    .into_any_element(),
                );
            }

            let on_nav_state = props.on_navigate_menu.clone();

            let state_dropdown = if is_state_open {
                Some(
                    deferred(
                        anchored()
                            .anchor(Anchor::TopLeft)
                            .offset(point(px(0.0), px(30.0)))
                            .snap_to_window()
                            .child(MenuSurface::render(MenuSurfaceProps {
                                id: "state_menu_surface".into(),
                                theme,
                                min_width: px(180.0),
                                reduce_motion: props.reduce_motion,
                                lifecycle: state_lifecycle,
                                anim_epoch: state_epoch,
                                focus_handle: Some(props.menu_surface_focus.clone()),
                                visual_style: props.visual_style,
                                on_close: on_close_state,
                                on_key_navigate: Some(on_nav_state),
                                children: state_items,
                            })),
                    )
                    .into_any_element(),
                )
            } else {
                None
            };

            let on_toggle_state_btn = on_toggle_menu.clone();
            let on_toggle_state_key = on_toggle_menu.clone();
            // L'état garde son mot tant qu'il y a la place ; à la forme la
            // plus étroite, il ne dit plus que « State ».
            let state_label = if breakpoint == WorkbenchBreakpoint::Narrow {
                "State ▾".to_string()
            } else {
                format!("State: {} ▾", props.state_filter.label())
            };

            Some(
                div()
                    .id("workbench_state_btn")
                    // The handle the bench measures: the harness reads the
                    // bounds of the button under this name. A noop outside
                    // the tests.
                    .debug_selector(|| "workbench_state_btn".to_string())
                    .relative()
                    .track_focus(&props.state_btn_focus)
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            let next = if is_state_open {
                                None
                            } else {
                                Some(WorkbenchMenu::State)
                            };
                            on_toggle_state_key(next, window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .h(px(28.0))
                    .gap(px(5.0))
                    .px(px(8.0))
                    .rounded_md()
                    .bg(if is_state_open {
                        theme.bg_surface_active
                    } else {
                        gpui::rgba(0x00000000)
                    })
                    .border_1()
                    .border_color(if is_state_open {
                        theme.border
                    } else {
                        gpui::rgba(0x00000000)
                    })
                    .text_xs()
                    .text_color(
                        if props.state_filter != PackageStateFilter::All || is_state_open {
                            theme.accent
                        } else {
                            theme.text_secondary
                        },
                    )
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
                    .child(state_label)
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        let next = if is_state_open {
                            None
                        } else {
                            Some(WorkbenchMenu::State)
                        };
                        on_toggle_state_btn(next, window, cx);
                    })
                    .children(state_dropdown),
            )
        };

        // ── 4. Sort Dropdown ──────────────────────────────────────────────────
        let is_sort_open = active_state
            .is_some_and(|s| s.menu == WorkbenchMenu::Sort && s.lifecycle != MenuLifecycle::Closed);
        let sort_lifecycle = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::Sort).then_some(s.lifecycle))
            .unwrap_or(MenuLifecycle::Closed);
        let sort_epoch = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::Sort).then_some(s.anim_epoch))
            .unwrap_or(0);
        let sort_highlighted = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::Sort).then_some(s.highlighted_index))
            .unwrap_or(0);

        let on_close_sort = {
            let cb = on_toggle_menu.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                cb(None, window, cx);
            })
        };

        let mut sort_items = Vec::new();
        sort_items.push(MenuSection::render("Sort by", theme).into_any_element());

        for (idx, sort) in [
            SortMode::Relevance,
            SortMode::NameAsc,
            SortMode::NameDesc,
            SortMode::Source,
            SortMode::InstalledFirst,
            SortMode::UpdatesFirst,
        ]
        .into_iter()
        .enumerate()
        {
            let on_select = props.on_select_sort.clone();
            let close_cb = on_close_sort.clone();
            let sort_val = sort;
            let id_str = match sort {
                SortMode::Relevance => "sort_dropdown_relevance",
                SortMode::NameAsc => "sort_dropdown_name_asc",
                SortMode::NameDesc => "sort_dropdown_name_desc",
                SortMode::Source => "sort_dropdown_source",
                SortMode::InstalledFirst => "sort_dropdown_installed",
                SortMode::UpdatesFirst => "sort_dropdown_updates",
            };
            sort_items.push(
                MenuCheckmarkItem::render(
                    id_str.into(),
                    sort.label(),
                    props.sort_mode == sort,
                    is_sort_open && sort_highlighted == idx,
                    theme,
                    Rc::new(move |w, a| {
                        on_select(sort_val, w, a);
                        close_cb(w, a);
                    }),
                )
                .into_any_element(),
            );
        }

        let on_nav_sort = props.on_navigate_menu.clone();

        let sort_dropdown = if is_sort_open {
            Some(
                deferred(
                    anchored()
                        .anchor(Anchor::TopLeft)
                        .offset(point(px(0.0), px(30.0)))
                        .snap_to_window()
                        .child(MenuSurface::render(MenuSurfaceProps {
                            id: "sort_menu_surface".into(),
                            theme,
                            min_width: px(180.0),
                            reduce_motion: props.reduce_motion,
                            lifecycle: sort_lifecycle,
                            anim_epoch: sort_epoch,
                            focus_handle: Some(props.menu_surface_focus.clone()),
                            visual_style: props.visual_style,
                            on_close: on_close_sort,
                            on_key_navigate: Some(on_nav_sort),
                            children: sort_items,
                        })),
                )
                .into_any_element(),
            )
        } else {
            None
        };

        // Les trois formes de la barre, selon la place qui reste.
        //
        // À la forme large, les commandes disent ce qu'elles font. Quand la
        // place manque, elles disent ce qu'elles *sont* : le tri se réduit à
        // son mot, l'état à son mot. La recherche, elle, garde la sienne :
        // c'est la commande qui porte le plus de fois.
        let sort_label = match breakpoint {
            WorkbenchBreakpoint::Wide => match props.sort_mode {
                SortMode::Relevance => "Sort: Relevance ▾",
                SortMode::NameAsc => "Sort: Name (A–Z) ▾",
                SortMode::NameDesc => "Sort: Name (Z–A) ▾",
                SortMode::Source => "Sort: Source ▾",
                SortMode::InstalledFirst => "Sort: Installed First ▾",
                SortMode::UpdatesFirst => "Sort: Updates First ▾",
            },
            WorkbenchBreakpoint::Medium => match props.sort_mode {
                SortMode::Relevance => "Sort: Rel ▾",
                SortMode::NameAsc => "Sort: A–Z ▾",
                SortMode::NameDesc => "Sort: Z–A ▾",
                SortMode::Source => "Sort: Src ▾",
                SortMode::InstalledFirst => "Sort: Inst ▾",
                SortMode::UpdatesFirst => "Sort: Upd ▾",
            },
            WorkbenchBreakpoint::Narrow => match props.sort_mode {
                SortMode::Relevance => "Rel ▾",
                SortMode::NameAsc => "A–Z ▾",
                SortMode::NameDesc => "Z–A ▾",
                SortMode::Source => "Src ▾",
                SortMode::InstalledFirst => "Inst ▾",
                SortMode::UpdatesFirst => "Upd ▾",
            },
        };

        let on_toggle_sort_btn = on_toggle_menu.clone();
        let on_toggle_sort_key = on_toggle_menu.clone();

        // Unboxed command trigger for Sort
        let sort_btn = div()
            .id("workbench_sort_btn")
            // The handle the bench measures: the harness reads the bounds of
            // the button under this name. A noop outside the tests.
            .debug_selector(|| "workbench_sort_btn".to_string())
            .relative()
            .track_focus(&props.sort_btn_focus)
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    let next = if is_sort_open {
                        None
                    } else {
                        Some(WorkbenchMenu::Sort)
                    };
                    on_toggle_sort_key(next, window, cx);
                }
            })
            .flex()
            .items_center()
            .h(px(28.0))
            .gap(px(5.0))
            .px(px(8.0))
            .rounded_md()
            .bg(if is_sort_open {
                theme.bg_surface_active
            } else {
                gpui::rgba(0x00000000)
            })
            .border_1()
            .border_color(if is_sort_open {
                theme.border
            } else {
                gpui::rgba(0x00000000)
            })
            .text_xs()
            .text_color(if props.sort_mode != SortMode::Relevance || is_sort_open {
                theme.accent
            } else {
                theme.text_secondary
            })
            .cursor_pointer()
            .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
            .child(sort_label)
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                let next = if is_sort_open {
                    None
                } else {
                    Some(WorkbenchMenu::Sort)
                };
                on_toggle_sort_btn(next, window, cx);
            })
            .children(sort_dropdown);

        // ── 5. Le commutateur de vue ─────────────────────────────────────────
        //
        // Le cadran a quitté le bandeau : un objet de la taille d'un cadran
        // ne peut pas tenir dans une rangée d'outils sans creuser un vide
        // au-dessus et au-dessous. Il est monté dans l'en-tête, à côté du
        // nom du périmètre, qui est son étiquette.
        let view_switcher = ViewModeSwitcher::render(ViewModeSwitcherProps {
            current_mode: props.view_mode,
            theme,
            reduce_motion: props.reduce_motion,
            marker: props.view_marker,
            on_select_mode: props.on_select_view_mode.clone(),
        });

        // ── 6. Row 2 Composition: Pure Native Controls (Zero Count Crowding) ─
        // L'action du périmètre : dans Mises à jour, la mise à niveau de tout.
        let scope_action = props.scope_action.as_ref().map(|action| {
            let handler = action.handler.clone();
            div()
                .id("workbench_scope_action")
                .px(px(8.0))
                .py(px(2.0))
                .rounded_sm()
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.accent)
                .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.accent_hover))
                .child(action.label)
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    handler(window, cx);
                })
        });

        let row2 = div()
            .id("workbench_row2_toolbar")
            .flex()
            .items_center()
            .justify_between()
            .gap(px(6.0))
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .children(state_btn)
                    .child(sort_btn),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(view_switcher)
                    .children(scope_action),
            );
        row2.into_any_element()
    }
}


#[cfg(test)]
mod tests {
    use super::{QueryWorkbench, QueryWorkbenchProps};
    use crate::components::search_input::SearchInputView;
    use crate::state::query::{PackageStateFilter, SortMode};
    use crate::state::session::PackageViewMode;
    use crate::state::NavScope;
    use crate::theme::Theme;
    use crate::visual_style::VisualStyleId;
    use gpui::prelude::*;
    use gpui::{div, point, px, size, Context, Entity, FocusHandle, Render, Window};
    use std::rc::Rc;

    /// Les trois formes de la barre, mesurées.
    ///
    /// La forme large dit ce que les commandes font. Quand la place manque,
    /// elles disent ce qu'elles *sont*. Ce test mesure les trois : il dessine
    /// la barre aux largeurs qui séparent les formes et il lit les bornes
    /// que la fenêtre a dessinées.
    ///
    /// C'est ce qui rend les branches *atteignables* : la règle du point de
    /// rupture existe, elle a trois formes distinctes, et chacune est
    /// exercée.
    #[gpui::test]
    fn the_three_forms_of_the_toolbar_are_measured(cx: &mut gpui::TestAppContext) {
        struct Fixture {
            theme: Theme,
            search_input: Entity<SearchInputView>,
            state_focus: FocusHandle,
            sort_focus: FocusHandle,
            menu_focus: FocusHandle,
            width: f32,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                div()
                    .w(px(self.width))
                    .h(px(120.0))
                    .child(QueryWorkbench::render(&QueryWorkbenchProps {
                        search_input: self.search_input.clone(),
                        scope: NavScope::Installed,
                        state_filter: PackageStateFilter::All,
                        sort_mode: SortMode::Relevance,
                        view_mode: PackageViewMode::Table,
                        view_marker: None,
                        active_menu_state: None,
                        reduce_motion: false,
                        theme: &theme,
                        visual_style: VisualStyleId::default(),
                        state_btn_focus: self.state_focus.clone(),
                        sort_btn_focus: self.sort_focus.clone(),
                        menu_surface_focus: self.menu_focus.clone(),
                        on_select_state: Rc::new(|_, _, _| {}),
                        on_select_sort: Rc::new(|_, _, _| {}),
                        on_select_view_mode: Rc::new(|_, _, _| {}),
                        on_toggle_menu: Rc::new(|_, _, _| {}),
                        on_navigate_menu: Rc::new(|_, _, _| {}),
                        scope_action: None,
                    }))
            }
        }

        let cx = cx.add_empty_window();
        let theme = Theme::dark();
        let search_input = cx.new(|cx| {
            SearchInputView::new(
                "Search packages and apps...",
                false,
                cx,
            )
        });

        let mut measure = |width: f32| -> (f32, f32, f32) {
            let fixture = cx.new(|cx| Fixture {
                theme: theme.clone(),
                search_input: search_input.clone(),
                state_focus: cx.focus_handle(),
                sort_focus: cx.focus_handle(),
                menu_focus: cx.focus_handle(),
                width,
            });
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(width), px(120.0)),
                |_window, _cx| fixture.clone().into_element(),
            );
            let state = cx
                .debug_bounds("workbench_state_btn")
                .expect("the state button is drawn");
            let sort = cx
                .debug_bounds("workbench_sort_btn")
                .expect("the sort button is drawn");
            let search = cx
                .debug_bounds("search_input_container")
                .expect("the search field is drawn");
            (
                state.size.width.into(),
                sort.size.width.into(),
                search.size.width.into(),
            )
        };

        // Drawn from wide to narrow: the toolbar keeps the richest form that
        // fits, so the buttons only ever get shorter as the room shrinks.
        let forms: Vec<(f32, f32, f32, f32)> = [1400.0, 900.0, 700.0, 600.0, 520.0, 460.0, 400.0, 340.0, 280.0, 240.0]
            .iter()
            .map(|&width| {
                let (state, sort, search) = measure(width);
                (width, state, sort, search)
            })
            .collect();
        for pair in forms.windows(2) {
            let (wide, narrow) = (pair[0], pair[1]);
            assert!(
                narrow.1 <= wide.1 + 0.5 && narrow.2 <= wide.2 + 0.5,
                "à {} px les boutons ne grandissent pas : état {} -> {}, tri {} -> {}",
                narrow.0, wide.1, narrow.1, wide.2, narrow.2
            );
        }
        let widest = forms.first().unwrap();
        let narrowest = forms.last().unwrap();
        assert!(
            narrowest.2 < widest.2,
            "la forme étroite réduit le tri : {} puis {}",
            widest.2,
            narrowest.2
        );
        assert!(
            narrowest.1 < widest.1,
            "et l'état se réduit à son mot : {} puis {}",
            widest.1,
            narrowest.1
        );
        // The search keeps its room in every form: it is the command that
        // carries the most, and it is the last to be cut.
        for (width, _, _, search) in &forms {
            assert!(
                *search > 150.0,
                "la recherche garde sa place à {width} px : {search}"
            );
        }
    }
}
