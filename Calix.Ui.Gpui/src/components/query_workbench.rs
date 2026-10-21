use crate::components::menu::{
    MenuActionHandler, MenuCheckItem, MenuCheckItemProps, MenuCheckmarkItem, MenuDivider,
    MenuKeyHandler, MenuLifecycle, MenuSection, MenuSurface, MenuSurfaceProps,
};
use crate::components::search_input::SearchInputView;
use crate::components::scope_dial::ScopeDialView;
use crate::components::view_mode_switcher::{ViewModeSwitcher, ViewModeSwitcherProps};
use crate::state::package_store::{SourceHealthMap, SourceHealthStatus};
use crate::state::query::{PackageStateFilter, SortMode, SourceScope, WorkbenchBreakpoint};
use crate::state::SourceAxisEntry;
use crate::state::session::NavScope;
use crate::state::{PackageSourceKind, PackageViewMode};
use crate::theme::Theme;
use crate::visual_style::paint::apply_surface_projection;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbenchMenu {
    Filters,
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

pub type SourceToggleHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;
pub type StateSelectHandler = Rc<dyn Fn(PackageStateFilter, &mut Window, &mut App) + 'static>;
pub type SortSelectHandler = Rc<dyn Fn(SortMode, &mut Window, &mut App) + 'static>;
pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;
pub type MenuToggleHandler = Rc<dyn Fn(Option<WorkbenchMenu>, &mut Window, &mut App) + 'static>;
pub type SourceRetryHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;
pub type ActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct QueryWorkbenchProps<'a> {
    pub search_input: Entity<SearchInputView>,
    /// The dial of the scope: one object with three positions.
    pub scope_dial: Entity<ScopeDialView>,
    pub list_pane_width: f32,
    /// Le périmètre : il décide de ce que le contrôle des sources et de l'état
    /// proposent.
    pub scope: NavScope,
    pub source_scope: SourceScope,
    pub state_filter: PackageStateFilter,
    pub sort_mode: SortMode,
    pub view_mode: PackageViewMode,
    pub active_menu_state: Option<ActiveMenuState>,
    pub reduce_motion: bool,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    pub source_health: &'a SourceHealthMap,
    /// L'axe des sources : ce que chaque source tient, et combien de lignes.
    pub source_axis: &'a [SourceAxisEntry],
    pub theme: &'a Theme,
    pub visual_style: VisualStyleId,
    pub filters_btn_focus: FocusHandle,
    pub state_btn_focus: FocusHandle,
    pub sort_btn_focus: FocusHandle,
    pub menu_surface_focus: FocusHandle,
    pub on_toggle_source: SourceToggleHandler,
    pub on_select_state: StateSelectHandler,
    pub on_select_sort: SortSelectHandler,
    pub on_select_view_mode: ViewModeHandler,
    pub on_toggle_menu: MenuToggleHandler,
    pub on_navigate_menu: MenuKeyHandler,
    pub on_retry_source: SourceRetryHandler,
    /// L'action propre au périmètre, quand il en a une.
    pub scope_action: Option<ScopeAction>,
}

/// L'action d'un périmètre : dans Mises à jour, la mise à niveau de tout.
#[derive(Clone)]
pub struct ScopeAction {
    pub label: &'static str,
    pub handler: ActionHandler,
}

/// Computes the active filter badge count (non-default sources or state).
pub fn compute_filter_badge_count(
    source_scope: &SourceScope,
    state_filter: PackageStateFilter,
    aur_enabled: bool,
    flatpak_enabled: bool,
    appimage_enabled: bool,
) -> usize {
    let mut count = 0;
    if !source_scope.is_all_enabled(aur_enabled, flatpak_enabled, appimage_enabled) {
        count += 1;
    }
    if state_filter != PackageStateFilter::All {
        count += 1;
    }
    count
}


/// Arguments pour le calcul de la ligne de statut permanente.

pub struct QueryWorkbench;

impl QueryWorkbench {
    pub fn render(props: &QueryWorkbenchProps) -> impl IntoElement {
        let theme = props.theme;
        let breakpoint = WorkbenchBreakpoint::from_width(props.list_pane_width);

        // ── 1. Row 1: Dominant Full-Width Search Input ────────────────────────
        let row1 = div()
            .id("workbench_row1_search")
            .w_full()
            .h(px(40.0))
            .child(props.search_input.clone());

        // ── 2. Filters Popover Setup ──────────────────────────────────────────
        let on_toggle_menu = props.on_toggle_menu.clone();
        let active_state = props.active_menu_state;
        let is_filters_open = active_state.is_some_and(|s| {
            s.menu == WorkbenchMenu::Filters && s.lifecycle != MenuLifecycle::Closed
        });
        let filters_lifecycle = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::Filters).then_some(s.lifecycle))
            .unwrap_or(MenuLifecycle::Closed);
        let filters_epoch = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::Filters).then_some(s.anim_epoch))
            .unwrap_or(0);
        let filters_highlighted = active_state
            .and_then(|s| (s.menu == WorkbenchMenu::Filters).then_some(s.highlighted_index))
            .unwrap_or(0);

        let on_close_filters = {
            let cb = on_toggle_menu.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                cb(None, window, cx);
            })
        };

        let filter_badge_count = compute_filter_badge_count(
            &props.source_scope,
            props.state_filter,
            props.aur_enabled,
            props.flatpak_enabled,
            props.appimage_enabled,
        );

        let filters_btn_text = if filter_badge_count > 0 {
            format!("Filters  {filter_badge_count}")
        } else {
            "Filters ▾".to_string()
        };

        let focus_border = theme.border_focus;
        let on_toggle_filters_btn = on_toggle_menu.clone();
        let on_toggle_filters_key = on_toggle_menu.clone();

        // Build items for Filters popover
        let mut filter_menu_items = Vec::new();
        filter_menu_items.push(MenuSection::render("Sources", theme).into_any_element());

        let build_health_badge = |kind: PackageSourceKind| -> (Option<&'static str>, bool) {
            if let Some(h) = props.source_health.get(&kind) {
                match h.status {
                    SourceHealthStatus::Failed => (Some("Failed"), true),
                    SourceHealthStatus::Disabled => (Some("Disabled"), false),
                    SourceHealthStatus::Loading => (Some("Loading..."), false),
                    SourceHealthStatus::Ready => (None, false),
                }
            } else {
                (None, false)
            }
        };

        let mut item_index = 0;

        // Les sources du périmètre : celles qu'il contient, et pas d'autres.
        for entry in props.source_axis.iter() {
            let is_enabled = match entry.kind {
                PackageSourceKind::Alpm => true,
                PackageSourceKind::Aur => props.aur_enabled,
                PackageSourceKind::Flatpak => props.flatpak_enabled,
                PackageSourceKind::AppImage => props.appimage_enabled,
                // Les écosystèmes d'inventaire n'ont pas de dépôt à interroger :
                // ils ne se conjuguent pas.
                PackageSourceKind::Cargo | PackageSourceKind::Npm | PackageSourceKind::Release => {
                    true
                }
            };
            if !is_enabled {
                continue;
            }
            let (health_lbl, is_failed) = if entry.inventory_only {
                (None, false)
            } else {
                build_health_badge(entry.kind)
            };
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                let kind = entry.kind;
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(kind, w, a);
                }) as MenuActionHandler)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            let kind = entry.kind;
            let id: &'static str = Box::leak(
                format!("menu_source_{}", kind.as_str()).into_boxed_str(),
            );
            let retry_id: &'static str = Box::leak(
                format!("menu_retry_{}", kind.as_str()).into_boxed_str(),
            );
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: id.into(),
                    retry_id: retry_id.into(),
                    label: entry.label.into(),
                    note: Some(entry.holds),
                    count: Some(entry.count),
                    is_checked: props.source_scope.is_selected(kind),
                    is_highlighted: is_filters_open && filters_highlighted == item_index,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(kind, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
            item_index += 1;
        }




        // In Narrow mode ONLY: dynamically include Package State in Filters menu
        if breakpoint == WorkbenchBreakpoint::Narrow {
            filter_menu_items.push(MenuDivider::render(theme).into_any_element());
            filter_menu_items.push(MenuSection::render("Package State", theme).into_any_element());
            for state in PackageStateFilter::options_for(props.scope) {
                let state = *state;
                let on_select = props.on_select_state.clone();
                let close_cb = on_close_filters.clone();
                let state_val = state;
                let id_str = match state {
                    PackageStateFilter::All => "menu_state_all",
                    PackageStateFilter::Installed => "menu_state_installed",
                    PackageStateFilter::NotInstalled => "menu_state_not_installed",
                    PackageStateFilter::UpdatesAvailable => "menu_state_updates",
                };
                filter_menu_items.push(
                    MenuCheckmarkItem::render(
                        id_str.into(),
                        state.label(),
                        props.state_filter == state,
                        is_filters_open && filters_highlighted == item_index,
                        theme,
                        Rc::new(move |w, a| {
                            on_select(state_val, w, a);
                            close_cb(w, a);
                        }),
                    )
                    .into_any_element(),
                );
                item_index += 1;
            }
        }

        let on_nav_filters = props.on_navigate_menu.clone();

        let filters_dropdown = if is_filters_open {
            Some(
                deferred(
                    anchored()
                        .anchor(Corner::TopLeft)
                        .offset(point(px(0.0), px(30.0)))
                        .snap_to_window()
                        .child(MenuSurface::render(MenuSurfaceProps {
                            id: "filters_menu_surface".into(),
                            theme,
                            min_width: px(240.0),
                            reduce_motion: props.reduce_motion,
                            lifecycle: filters_lifecycle,
                            anim_epoch: filters_epoch,
                            focus_handle: Some(props.menu_surface_focus.clone()),
                            visual_style: props.visual_style,
                            on_close: on_close_filters,
                            on_key_navigate: Some(on_nav_filters),
                            children: filter_menu_items,
                        })),
                )
                .into_any_element(),
            )
        } else {
            None
        };

        // Unboxed command trigger for Filters
        let filters_trigger = div()
            .id("workbench_filters_btn")
            .relative()
            .track_focus(&props.filters_btn_focus)
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    let next = if is_filters_open {
                        None
                    } else {
                        Some(WorkbenchMenu::Filters)
                    };
                    on_toggle_filters_key(next, window, cx);
                }
            })
            .flex()
            .items_center()
            .h(px(28.0))
            .gap(px(5.0))
            .px(px(8.0))
            .rounded_md()
            .bg(if is_filters_open {
                theme.bg_surface_active
            } else {
                gpui::rgba(0x00000000)
            })
            .border_1()
            .border_color(if is_filters_open {
                theme.border
            } else {
                gpui::rgba(0x00000000)
            })
            .text_xs()
            .font_weight(if filter_badge_count > 0 {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(if filter_badge_count > 0 || is_filters_open {
                theme.accent
            } else {
                theme.text_secondary
            })
            .cursor_pointer()
            .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
            .child(filters_btn_text)
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                let next = if is_filters_open {
                    None
                } else {
                    Some(WorkbenchMenu::Filters)
                };
                on_toggle_filters_btn(next, window, cx);
            })
            .children(filters_dropdown);

        // ── 3. State Dropdown (Wide and Medium Breakpoints ONLY) ───────────────
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

        let state_btn = if breakpoint != WorkbenchBreakpoint::Narrow {
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
                            .anchor(Corner::TopLeft)
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
            let state_label = format!("State: {} ▾", props.state_filter.label());

            Some(
                div()
                    .id("workbench_state_btn")
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
        } else {
            None
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
                        .anchor(Corner::TopLeft)
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

        let sort_label = if breakpoint == WorkbenchBreakpoint::Narrow {
            match props.sort_mode {
                SortMode::Relevance => "Sort: Rel ▾",
                SortMode::NameAsc => "Sort: A–Z ▾",
                SortMode::NameDesc => "Sort: Z–A ▾",
                SortMode::Source => "Sort: Src ▾",
                SortMode::InstalledFirst => "Sort: Inst ▾",
                SortMode::UpdatesFirst => "Sort: Upd ▾",
            }
        } else {
            match props.sort_mode {
                SortMode::Relevance => "Sort: Relevance ▾",
                SortMode::NameAsc => "Sort: Name (A–Z) ▾",
                SortMode::NameDesc => "Sort: Name (Z–A) ▾",
                SortMode::Source => "Sort: Source ▾",
                SortMode::InstalledFirst => "Sort: Installed First ▾",
                SortMode::UpdatesFirst => "Sort: Updates First ▾",
            }
        };

        let on_toggle_sort_btn = on_toggle_menu.clone();
        let on_toggle_sort_key = on_toggle_menu.clone();

        // Unboxed command trigger for Sort
        let sort_btn = div()
            .id("workbench_sort_btn")
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

        // ── 5. Le cadran du périmètre, et le commutateur de vue ─────────────
        //
        // Le cadran est un objet d'une autre nature que la banque : il a ses
        // propres positions, son propre geste et son propre mouvement. Les
        // deux axes ne se ressemblent pas, et ne se partagent pas un contrôle.
        let scope_dial = props.scope_dial.clone();

        let view_switcher = ViewModeSwitcher::render(ViewModeSwitcherProps {
            current_mode: props.view_mode,
            theme,
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
                    .gap(px(4.0))
                    .child(filters_trigger),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .children(state_btn)
                    .child(sort_btn)
                    .child(view_switcher)
                    .children(scope_action),
            )
            .child(
                // Le cadran, à sa place : un objet seul, dans son coin, qui
                // ne ressemble à rien d'autre sur la plaque.
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .w_full()
                    .child(scope_dial.clone()),
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
}


