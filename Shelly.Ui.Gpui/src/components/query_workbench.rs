use crate::components::menu::{
    MenuActionHandler, MenuCheckItem, MenuCheckItemProps, MenuCheckmarkItem, MenuDivider,
    MenuKeyHandler, MenuLifecycle, MenuSection, MenuSurface, MenuSurfaceProps,
};
use crate::components::search_input::SearchInputView;
use crate::components::view_mode_switcher::{ViewModeSwitcher, ViewModeSwitcherProps};
use crate::icons::AppIcon;
use crate::state::package_store::{SourceHealthMap, SourceHealthStatus};
use crate::state::query::{PackageStateFilter, SortMode, SourceScope, WorkbenchBreakpoint};
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
    pub list_pane_width: f32,
    pub source_scope: SourceScope,
    pub state_filter: PackageStateFilter,
    pub sort_mode: SortMode,
    pub view_mode: PackageViewMode,
    pub active_menu_state: Option<ActiveMenuState>,
    pub is_searching: bool,
    pub total_count: usize,
    pub reduce_motion: bool,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    pub source_health: &'a SourceHealthMap,
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
    pub on_clear_filters: ActionHandler,
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

/// Formats integers with standard grouping separators (e.g. 1,808 or 16).
pub fn format_count(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Arguments pour le calcul de la ligne de statut permanente.
pub struct FormatStatusRailArgs<'a> {
    pub total_count: usize,
    pub is_searching: bool,
    pub source_scope: &'a SourceScope,
    pub state_filter: PackageStateFilter,
    pub source_health: &'a SourceHealthMap,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
}

/// Etat de contenu de la ligne de statut permanente (Row 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRailState {
    pub left_text: String,
    pub has_clear_filters: bool,
    pub retry_source: Option<PackageSourceKind>,
}

/// Formats the single permanent status rail content (guaranteeing zero layout shift).
pub fn format_status_rail(args: FormatStatusRailArgs<'_>) -> StatusRailState {
    let total_count = args.total_count;
    let is_searching = args.is_searching;
    let source_scope = args.source_scope;
    let state_filter = args.state_filter;
    let source_health = args.source_health;
    let aur_enabled = args.aur_enabled;
    let flatpak_enabled = args.flatpak_enabled;
    let appimage_enabled = args.appimage_enabled;

    // 1. Check for partial failures first
    let failed_source = [
        PackageSourceKind::Alpm,
        PackageSourceKind::Aur,
        PackageSourceKind::Flatpak,
        PackageSourceKind::AppImage,
    ]
    .into_iter()
    .find(|kind| {
        let is_enabled = match kind {
            PackageSourceKind::Alpm => true,
            PackageSourceKind::Aur => aur_enabled,
            PackageSourceKind::Flatpak => flatpak_enabled,
            PackageSourceKind::AppImage => appimage_enabled,
            // Les écosystèmes Calix ne sont pas des sources de recherche :
            // ils n'ont pas de rail de santé dans la station de recherche.
            PackageSourceKind::Cargo | PackageSourceKind::Npm | PackageSourceKind::Release => {
                false
            }
        };
        is_enabled
            && source_health
                .get(kind)
                .is_some_and(|h| h.status == SourceHealthStatus::Failed)
    });

    let is_all_sources =
        source_scope.is_all_enabled(aur_enabled, flatpak_enabled, appimage_enabled);
    let is_all_states = state_filter == PackageStateFilter::All;
    let has_clear_filters = !is_all_sources || !is_all_states;

    if let Some(failed) = failed_source {
        let count_str = format_count(total_count);
        let plural = if total_count == 1 {
            "package"
        } else {
            "packages"
        };
        let failed_label = match failed {
            PackageSourceKind::Alpm => "Arch ALPM",
            PackageSourceKind::Aur => "AUR",
            PackageSourceKind::Flatpak => "Flatpak",
            PackageSourceKind::AppImage => "AppImage",
            PackageSourceKind::Cargo => "Cargo",
            PackageSourceKind::Npm => "NPM",
            PackageSourceKind::Release => "Release",
        };
        return StatusRailState {
            left_text: format!("{count_str} {plural}  •  {failed_label} unavailable"),
            has_clear_filters,
            retry_source: Some(failed),
        };
    }

    if is_searching {
        let search_sources = if is_all_sources {
            "all sources".to_string()
        } else {
            let mut names = Vec::new();
            if source_scope.alpm {
                names.push("Official");
            }
            if source_scope.aur && aur_enabled {
                names.push("AUR");
            }
            if source_scope.flatpak && flatpak_enabled {
                names.push("Flatpak");
            }
            if source_scope.appimage && appimage_enabled {
                names.push("AppImage");
            }
            names.join(" + ")
        };
        return StatusRailState {
            left_text: format!("Searching {search_sources}…"),
            has_clear_filters,
            retry_source: None,
        };
    }

    if has_clear_filters {
        let count_str = format_count(total_count);
        let plural = if total_count == 1 {
            "package"
        } else {
            "packages"
        };
        let mut parts = vec![format!("{count_str} {plural}")];

        if !is_all_sources {
            let mut names = Vec::new();
            if source_scope.alpm {
                names.push("Official");
            }
            if source_scope.aur && aur_enabled {
                names.push("AUR");
            }
            if source_scope.flatpak && flatpak_enabled {
                names.push("Flatpak");
            }
            if source_scope.appimage && appimage_enabled {
                names.push("AppImage");
            }
            if !names.is_empty() {
                parts.push(names.join(" + "));
            }
        }

        if !is_all_states {
            parts.push(state_filter.label().to_string());
        }

        return StatusRailState {
            left_text: parts.join("  •  "),
            has_clear_filters: true,
            retry_source: None,
        };
    }

    // Default clean state
    let count_str = format_count(total_count);
    let plural = if total_count == 1 {
        "package"
    } else {
        "packages"
    };
    StatusRailState {
        left_text: format!("{count_str} {plural}"),
        has_clear_filters: false,
        retry_source: None,
    }
}

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

        // ALPM source check item
        {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::Alpm);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::Alpm, w, a);
                }) as MenuActionHandler)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_alpm".into(),
                    retry_id: "menu_retry_alpm".into(),
                    label: "Official / ALPM".into(),
                    is_checked: props.source_scope.alpm,
                    is_highlighted: is_filters_open && filters_highlighted == item_index,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::Alpm, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
            item_index += 1;
        }

        // AUR source check item
        if props.aur_enabled {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::Aur);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::Aur, w, a);
                }) as MenuActionHandler)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_aur".into(),
                    retry_id: "menu_retry_aur".into(),
                    label: "AUR".into(),
                    is_checked: props.source_scope.aur,
                    is_highlighted: is_filters_open && filters_highlighted == item_index,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::Aur, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
            item_index += 1;
        }

        // Flatpak source check item
        if props.flatpak_enabled {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::Flatpak);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::Flatpak, w, a);
                }) as MenuActionHandler)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_flatpak".into(),
                    retry_id: "menu_retry_flatpak".into(),
                    label: "Flatpak".into(),
                    is_checked: props.source_scope.flatpak,
                    is_highlighted: is_filters_open && filters_highlighted == item_index,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::Flatpak, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
            item_index += 1;
        }

        // AppImage source check item
        if props.appimage_enabled {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::AppImage);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::AppImage, w, a);
                }) as MenuActionHandler)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_appimage".into(),
                    retry_id: "menu_retry_appimage".into(),
                    label: "AppImage".into(),
                    is_checked: props.source_scope.appimage,
                    is_highlighted: is_filters_open && filters_highlighted == item_index,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::AppImage, w, a)),
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
            for state in [
                PackageStateFilter::All,
                PackageStateFilter::Installed,
                PackageStateFilter::NotInstalled,
                PackageStateFilter::UpdatesAvailable,
            ] {
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

            for (idx, state) in [
                PackageStateFilter::All,
                PackageStateFilter::Installed,
                PackageStateFilter::NotInstalled,
                PackageStateFilter::UpdatesAvailable,
            ]
            .into_iter()
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

        // ── 5. Unboxed Minimalist View Mode Switcher (▦ ≡) ───────────────────
        let view_switcher = ViewModeSwitcher::render(ViewModeSwitcherProps {
            current_mode: props.view_mode,
            theme,
            on_select_mode: props.on_select_view_mode.clone(),
        });

        // ── 6. Row 2 Composition: Pure Native Controls (Zero Count Crowding) ─
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
                    .child(view_switcher),
            );

        // ── 7. Row 3: Permanent Status Rail (Fixed Height 24px) ───────────────
        let rail_state = format_status_rail(FormatStatusRailArgs {
            total_count: props.total_count,
            is_searching: props.is_searching,
            source_scope: &props.source_scope,
            state_filter: props.state_filter,
            source_health: props.source_health,
            aur_enabled: props.aur_enabled,
            flatpak_enabled: props.flatpak_enabled,
            appimage_enabled: props.appimage_enabled,
        });

        let on_clear = props.on_clear_filters.clone();
        let on_clear_key = on_clear.clone();
        let on_retry = props.on_retry_source.clone();

        let clear_action = if rail_state.has_clear_filters {
            Some(
                div()
                    .id("workbench_clear_filters_btn")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_clear_key(window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .text_color(theme.accent)
                    .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.accent_hover))
                    .child(
                        svg()
                            .path(AppIcon::Close.path())
                            .size(px(10.0))
                            .text_color(theme.accent),
                    )
                    .child("Clear filters")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_clear(window, cx);
                    }),
            )
        } else {
            None
        };

        let retry_action = if let Some(failed_kind) = rail_state.retry_source {
            let cb = on_retry.clone();
            Some(
                div()
                    .id(ElementId::Name(format!("rail_retry_{failed_kind}").into()))
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down({
                        let cb_key = cb.clone();
                        move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                cb_key(failed_kind, window, cx);
                            }
                        }
                    })
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.danger)
                    .hover(|s| s.bg(theme.bg_surface_hover))
                    .child("Retry")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        cb(failed_kind, window, cx);
                    }),
            )
        } else {
            None
        };

        let status_rail = div()
            .id("workbench_row3_status_rail")
            .h(px(24.0))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.0))
            .w_full()
            .px(px(4.0))
            .child(
                div()
                    .id("workbench_status_text")
                    .text_xs()
                    .text_color(if props.is_searching {
                        theme.accent
                    } else {
                        theme.text_secondary
                    })
                    .child(rail_state.left_text),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .children(clear_action)
                    .children(retry_action),
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
        .child(status_rail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_format_count() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(16), "16");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1000), "1,000");
        assert_eq!(format_count(1808), "1,808");
        assert_eq!(format_count(1234567), "1,234,567");
    }

    #[test]
    fn test_format_status_rail_default() {
        let scope = SourceScope::all();
        let health = SourceHealthMap::default();
        let rail = format_status_rail(FormatStatusRailArgs {
            total_count: 1808,
            is_searching: false,
            source_scope: &scope,
            state_filter: PackageStateFilter::All,
            source_health: &health,
            aur_enabled: true,
            flatpak_enabled: true,
            appimage_enabled: true,
        });
        assert_eq!(rail.left_text, "1,808 packages");
        assert!(!rail.has_clear_filters);
        assert!(rail.retry_source.is_none());
    }

    #[test]
    fn test_format_status_rail_filtered() {
        let mut scope = SourceScope::all();
        scope.flatpak = false;
        scope.appimage = false;
        let health = SourceHealthMap::default();
        let rail = format_status_rail(FormatStatusRailArgs {
            total_count: 16,
            is_searching: false,
            source_scope: &scope,
            state_filter: PackageStateFilter::Installed,
            source_health: &health,
            aur_enabled: true,
            flatpak_enabled: true,
            appimage_enabled: true,
        });
        assert_eq!(
            rail.left_text,
            "16 packages  •  Official + AUR  •  Installed"
        );
        assert!(rail.has_clear_filters);
        assert!(rail.retry_source.is_none());
    }

    #[test]
    fn test_format_status_rail_searching() {
        let mut scope = SourceScope::all();
        scope.appimage = false;
        let health = SourceHealthMap::default();
        let rail = format_status_rail(FormatStatusRailArgs {
            total_count: 0,
            is_searching: true,
            source_scope: &scope,
            state_filter: PackageStateFilter::All,
            source_health: &health,
            aur_enabled: true,
            flatpak_enabled: true,
            appimage_enabled: true,
        });
        assert_eq!(rail.left_text, "Searching Official + AUR + Flatpak…");
        assert!(rail.has_clear_filters);
        assert!(rail.retry_source.is_none());
    }

    #[test]
    fn test_compute_filter_badge_count() {
        let all_scope = SourceScope::all();
        assert_eq!(
            compute_filter_badge_count(&all_scope, PackageStateFilter::All, true, true, true),
            0
        );

        let mut subset_scope = SourceScope::all();
        subset_scope.aur = false;
        subset_scope.flatpak = false;
        subset_scope.appimage = false;
        assert_eq!(
            compute_filter_badge_count(&subset_scope, PackageStateFilter::All, true, true, true),
            1
        );
        assert_eq!(
            compute_filter_badge_count(&all_scope, PackageStateFilter::Installed, true, true, true),
            1
        );
        assert_eq!(
            compute_filter_badge_count(
                &subset_scope,
                PackageStateFilter::Installed,
                true,
                true,
                true
            ),
            2
        );
    }

    #[test]
    fn test_format_status_rail_failure() {
        let scope = SourceScope::all();
        let mut health = SourceHealthMap::default();
        health.aur = crate::state::package_store::SourceHealth::failed("network timeout");
        let rail = format_status_rail(FormatStatusRailArgs {
            total_count: 500,
            is_searching: false,
            source_scope: &scope,
            state_filter: PackageStateFilter::All,
            source_health: &health,
            aur_enabled: true,
            flatpak_enabled: true,
            appimage_enabled: true,
        });
        assert_eq!(rail.left_text, "500 packages  •  AUR unavailable");
        assert_eq!(rail.retry_source, Some(PackageSourceKind::Aur));
    }
}
