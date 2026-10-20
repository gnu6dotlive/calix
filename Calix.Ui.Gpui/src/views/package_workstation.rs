use crate::backend::models::UnifiedPackage;
use crate::components::menu::MenuLifecycle;
use crate::components::package_card::{PackageCard, PackageCardProps};
use crate::components::package_table::PackageTable;
use crate::components::query_workbench::{
    ActiveMenuState, QueryWorkbench, QueryWorkbenchProps, WorkbenchMenu,
};
use crate::components::search_input::SearchInputView;
use crate::components::unified_search::UnifiedSearch;
use crate::state::console::{ConsoleEvent, ConsoleModel};
use crate::state::query::{PackageStateFilter, SortMode, SourceScope, WorkbenchBreakpoint};
use crate::state::{
    canonical_install_command, AppSession, NavDestination, PackageKey, PackageSourceKind,
    PackageStore, PackageViewMode, ToastCenter, ToastKind,
};
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use crate::views::inspector::{PackageInspectorProps, PackageInspectorView};
use crate::visual_style::VisualStyleId;
use gpui::*;
use gpui::{uniform_list, UniformListScrollHandle};
use std::rc::Rc;
use std::time::Duration;

pub type PackageMutationHandler =
    Rc<dyn Fn(&UnifiedPackage, bool /* is_install */, &mut Window, &mut App) + 'static>;
pub type SystemUpgradeHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SourceRetryHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;
pub type ActionReloadHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DetailsRetryHandler = Rc<dyn Fn(&PackageKey, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitterDragState {
    pub start_width: f32,
    pub start_pointer_x: f32,
}

/// Mode de disposition adaptatif du workbench
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkstationLayoutMode {
    /// Mode scindé horizontal : liste à gauche, splitter, inspecteur docké à droite
    Horizontal,
    /// Mode empilé : liste au-dessus, inspecteur en-dessous (zéro overlay)
    Stacked,
}

impl WorkstationLayoutMode {
    pub fn from_content_width(content_width: f32) -> Self {
        if content_width >= UiMetrics::HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH {
            WorkstationLayoutMode::Horizontal
        } else {
            WorkstationLayoutMode::Stacked
        }
    }
}

/// Calcul pur et robuste de la largeur du panneau dérivé des deltas de pointeur
/// Garantit les invariants stricts du contrat UX :
/// 1. inspector_width <= min(520.0, content_width * 3/7)
/// 2. inspector_width >= 320.0 (INSPECTOR_MIN_USABLE)
/// 3. list_width >= LIST_MIN_USABLE (340.0)
pub fn compute_splitter_width(
    drag_start_width: f32,
    drag_start_pointer_x: f32,
    current_pointer_x: f32,
    content_width: f32,
) -> f32 {
    let delta_x = current_pointer_x - drag_start_pointer_x;
    let requested_width = drag_start_width + delta_x;
    let (list_min, list_max) = compute_splitter_bounds(content_width);
    if list_min <= list_max {
        requested_width.clamp(list_min, list_max)
    } else {
        list_min
    }
}

/// Bornes légales strictes de la largeur de liste dans le domaine horizontal
pub fn compute_splitter_bounds(content_width: f32) -> (f32, f32) {
    let inspector_max =
        (content_width * UiMetrics::INSPECTOR_MAX_RATIO).min(UiMetrics::INSPECTOR_MAX_WIDTH);
    let inspector_min = UiMetrics::INSPECTOR_MIN_USABLE;
    let list_min =
        (content_width - UiMetrics::SPLITTER_WIDTH - inspector_max).max(UiMetrics::LIST_MIN_USABLE);
    let list_max = (content_width - UiMetrics::SPLITTER_WIDTH - inspector_min).max(list_min);
    (list_min, list_max)
}

pub struct PackageWorkstationView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
    pub console: Entity<ConsoleModel>,
    pub toast_center: Entity<ToastCenter>,
    pub list_pane_width: f32,
    pub drag_state: Option<SplitterDragState>,
    pub scroll_handle: UniformListScrollHandle,
    pub on_mutation: Option<PackageMutationHandler>,
    pub on_upgrade_all: Option<SystemUpgradeHandler>,
    pub on_retry_source: Option<SourceRetryHandler>,
    pub on_reload_installed: Option<ActionReloadHandler>,
    pub on_reload_updates: Option<ActionReloadHandler>,
    pub on_retry_details: Option<DetailsRetryHandler>,
    pub copy_cmd_feedback: bool,
    pub is_loading_pkgbuild: bool,
    pub reduce_motion: bool,
    pub theme: Theme,
    pub visual_style: VisualStyleId,
    pub compact: bool,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    pub search_input: Entity<SearchInputView>,
    pub active_menu: Option<ActiveMenuState>,
    pub menu_epoch: usize,
    pub filters_btn_focus: FocusHandle,
    pub state_btn_focus: FocusHandle,
    pub sort_btn_focus: FocusHandle,
    pub menu_surface_focus: FocusHandle,
}

#[derive(Clone, Copy, Debug)]
pub struct PackageWorkstationConfig {
    pub theme: Theme,
    pub reduce_motion: bool,
    pub compact: bool,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    pub visual_style: VisualStyleId,
}

impl PackageWorkstationView {
    pub fn new(
        session: Entity<AppSession>,
        store: Entity<PackageStore>,
        console: Entity<ConsoleModel>,
        toast_center: Entity<ToastCenter>,
        config: PackageWorkstationConfig,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.subscribe(&session, |_this, _emitter, _event, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(&store, |_this, _emitter, _event, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(&console, |_this, _emitter, event, cx| match event {
            ConsoleEvent::OperationStarted(_) | ConsoleEvent::OperationFinished(_) => {
                cx.notify();
            }
            _ => {}
        })
        .detach();

        let search_input = cx.new(|cx| {
            SearchInputView::new(
                "Search packages and apps...",
                config.theme,
                config.reduce_motion,
                cx,
            )
        });

        Self {
            session,
            store,
            console,
            toast_center,
            list_pane_width: 640.0,
            drag_state: None,
            scroll_handle: UniformListScrollHandle::new(),
            on_mutation: None,
            on_upgrade_all: None,
            on_retry_source: None,
            on_reload_installed: None,
            on_reload_updates: None,
            on_retry_details: None,
            copy_cmd_feedback: false,
            is_loading_pkgbuild: false,
            reduce_motion: config.reduce_motion,
            theme: config.theme,
            visual_style: config.visual_style,
            compact: config.compact,
            aur_enabled: config.aur_enabled,
            flatpak_enabled: config.flatpak_enabled,
            appimage_enabled: config.appimage_enabled,
            search_input,
            active_menu: None,
            menu_epoch: 0,
            filters_btn_focus: cx.focus_handle(),
            state_btn_focus: cx.focus_handle(),
            sort_btn_focus: cx.focus_handle(),
            menu_surface_focus: cx.focus_handle(),
        }
    }

    pub fn toggle_menu(
        &mut self,
        target: Option<WorkbenchMenu>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target {
            None => self.close_menu(window, cx),
            Some(menu) => {
                if let Some(ref current) = self.active_menu {
                    if current.menu == menu
                        && current.lifecycle != MenuLifecycle::Closing
                        && current.lifecycle != MenuLifecycle::Closed
                    {
                        self.close_menu(window, cx);
                        return;
                    }
                }
                self.open_menu(menu, window, cx);
            }
        }
    }

    pub fn open_menu(&mut self, menu: WorkbenchMenu, window: &mut Window, cx: &mut Context<Self>) {
        self.menu_epoch = self.menu_epoch.wrapping_add(1);
        let epoch = self.menu_epoch;

        if self.reduce_motion {
            self.active_menu = Some(ActiveMenuState {
                menu,
                lifecycle: MenuLifecycle::Open,
                anim_epoch: epoch,
                highlighted_index: 0,
            });
            window.focus(&self.menu_surface_focus);
            cx.notify();
        } else {
            self.active_menu = Some(ActiveMenuState {
                menu,
                lifecycle: MenuLifecycle::Opening,
                anim_epoch: epoch,
                highlighted_index: 0,
            });
            window.focus(&self.menu_surface_focus);
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(150))
                    .await;
                let _ = this.update(cx, |view, cx| {
                    if let Some(ref mut state) = view.active_menu {
                        if state.lifecycle == MenuLifecycle::Opening && state.anim_epoch == epoch {
                            state.lifecycle = MenuLifecycle::Open;
                            cx.notify();
                        }
                    }
                });
            })
            .detach();
        }
    }

    pub fn close_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(current) = self.active_menu else {
            return;
        };

        if current.lifecycle == MenuLifecycle::Closing || current.lifecycle == MenuLifecycle::Closed
        {
            return;
        }

        match current.menu {
            WorkbenchMenu::Filters => window.focus(&self.filters_btn_focus),
            WorkbenchMenu::State => window.focus(&self.state_btn_focus),
            WorkbenchMenu::Sort => window.focus(&self.sort_btn_focus),
        }

        if self.reduce_motion {
            self.active_menu = None;
            cx.notify();
        } else {
            self.menu_epoch = self.menu_epoch.wrapping_add(1);
            let epoch = self.menu_epoch;

            self.active_menu = Some(ActiveMenuState {
                menu: current.menu,
                lifecycle: MenuLifecycle::Closing,
                anim_epoch: epoch,
                highlighted_index: current.highlighted_index,
            });
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let _ = this.update(cx, |view, cx| {
                    if let Some(ref state) = view.active_menu {
                        if state.lifecycle == MenuLifecycle::Closing && state.anim_epoch == epoch {
                            view.active_menu = None;
                            cx.notify();
                        }
                    }
                });
            })
            .detach();
        }
    }

    pub fn menu_item_count(&self, menu: WorkbenchMenu, breakpoint: WorkbenchBreakpoint) -> usize {
        match menu {
            WorkbenchMenu::Filters => {
                let mut count = 1; // ALPM
                if self.aur_enabled {
                    count += 1;
                }
                if self.flatpak_enabled {
                    count += 1;
                }
                if self.appimage_enabled {
                    count += 1;
                }
                if breakpoint == WorkbenchBreakpoint::Narrow {
                    count += 4; // All, Installed, NotInstalled, UpdatesAvailable
                }
                count
            }
            WorkbenchMenu::State => 4,
            WorkbenchMenu::Sort => 6,
        }
    }

    pub fn navigate_menu(
        &mut self,
        key: &str,
        breakpoint: WorkbenchBreakpoint,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(current) = self.active_menu else {
            return;
        };

        let total = self.menu_item_count(current.menu, breakpoint);
        if total == 0 {
            return;
        }

        match key {
            "up" => {
                let next_idx = if current.highlighted_index == 0 {
                    total - 1
                } else {
                    current.highlighted_index - 1
                };
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = next_idx;
                }
                cx.notify();
            }
            "down" => {
                let next_idx = (current.highlighted_index + 1) % total;
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = next_idx;
                }
                cx.notify();
            }
            "home" => {
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = 0;
                }
                cx.notify();
            }
            "end" => {
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = total - 1;
                }
                cx.notify();
            }
            "enter" | "space" => {
                self.activate_menu_item(
                    current.menu,
                    current.highlighted_index,
                    breakpoint,
                    window,
                    cx,
                );
            }
            _ => {}
        }
    }

    pub fn activate_menu_item(
        &mut self,
        menu: WorkbenchMenu,
        index: usize,
        breakpoint: WorkbenchBreakpoint,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match menu {
            WorkbenchMenu::Filters => {
                let mut sources = vec![PackageSourceKind::Alpm];
                if self.aur_enabled {
                    sources.push(PackageSourceKind::Aur);
                }
                if self.flatpak_enabled {
                    sources.push(PackageSourceKind::Flatpak);
                }
                if self.appimage_enabled {
                    sources.push(PackageSourceKind::AppImage);
                }

                if index < sources.len() {
                    let source = sources[index];
                    self.session.update(cx, |s, cx| {
                        s.toggle_source(source, cx);
                    });
                } else if breakpoint == WorkbenchBreakpoint::Narrow {
                    let state_idx = index - sources.len();
                    let states = [
                        PackageStateFilter::All,
                        PackageStateFilter::Installed,
                        PackageStateFilter::NotInstalled,
                        PackageStateFilter::UpdatesAvailable,
                    ];
                    if let Some(&state) = states.get(state_idx) {
                        self.session.update(cx, |s, cx| {
                            s.set_state_filter(state, cx);
                        });
                        self.close_menu(window, cx);
                    }
                }
            }
            WorkbenchMenu::State => {
                let states = [
                    PackageStateFilter::All,
                    PackageStateFilter::Installed,
                    PackageStateFilter::NotInstalled,
                    PackageStateFilter::UpdatesAvailable,
                ];
                if let Some(&state) = states.get(index) {
                    self.session.update(cx, |s, cx| {
                        s.set_state_filter(state, cx);
                    });
                    self.close_menu(window, cx);
                }
            }
            WorkbenchMenu::Sort => {
                let sorts = [
                    SortMode::Relevance,
                    SortMode::NameAsc,
                    SortMode::NameDesc,
                    SortMode::Source,
                    SortMode::InstalledFirst,
                    SortMode::UpdatesFirst,
                ];
                if let Some(&sort) = sorts.get(index) {
                    self.session.update(cx, |s, cx| {
                        s.set_sort_mode(sort, cx);
                    });
                    self.close_menu(window, cx);
                }
            }
        }
    }

    pub fn set_sources_enabled(
        &mut self,
        aur: bool,
        flatpak: bool,
        appimage: bool,
        cx: &mut Context<Self>,
    ) {
        self.aur_enabled = aur;
        self.flatpak_enabled = flatpak;
        self.appimage_enabled = appimage;
        cx.notify();
    }

    pub fn set_on_mutation(&mut self, handler: PackageMutationHandler) {
        self.on_mutation = Some(handler);
    }

    pub fn set_on_upgrade_all(&mut self, handler: SystemUpgradeHandler) {
        self.on_upgrade_all = Some(handler);
    }

    pub fn on_pointer_move(
        &mut self,
        event: &MouseMoveEvent,
        content_width: f32,
        cx: &mut Context<Self>,
    ) {
        if let Some(drag) = self.drag_state {
            let current_x = event.position.x.to_f64() as f32;
            let new_width = compute_splitter_width(
                drag.start_width,
                drag.start_pointer_x,
                current_x,
                content_width,
            );
            if (new_width - self.list_pane_width).abs() >= 1.0 {
                self.list_pane_width = new_width;
                cx.notify();
            }
        }
    }

    pub fn set_compact(&mut self, compact: bool, cx: &mut Context<Self>) {
        if self.compact != compact {
            self.compact = compact;
            cx.notify();
        }
    }

    pub fn set_reduce_motion(&mut self, reduce_motion: bool, cx: &mut Context<Self>) {
        if self.reduce_motion != reduce_motion {
            self.reduce_motion = reduce_motion;
            cx.notify();
        }
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        self.search_input
            .update(cx, |si, cx| si.set_theme(theme, cx));
        cx.notify();
    }

    pub fn set_visual_style(&mut self, style: VisualStyleId, cx: &mut Context<Self>) {
        self.visual_style = style;
        cx.notify();
    }

    pub fn on_pointer_up(&mut self, cx: &mut Context<Self>) {
        if self.drag_state.is_some() {
            self.drag_state = None;
            cx.notify();
        }
    }
}

impl Render for PackageWorkstationView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let entity = cx.entity().clone();
        let is_busy = self.console.read(cx).is_running();

        let sidebar_collapsed = self.session.read(cx).sidebar_collapsed;
        let sidebar_width = if sidebar_collapsed {
            UiMetrics::SIDEBAR_COLLAPSED
        } else {
            UiMetrics::SIDEBAR_EXPANDED
        };
        let window_width = window.window_bounds().get_bounds().size.width / px(1.0);
        let content_width = (window_width - sidebar_width).max(0.0);
        let layout_mode = WorkstationLayoutMode::from_content_width(content_width);

        let (list_min, list_max) = compute_splitter_bounds(content_width);
        let effective_list_width = self.list_pane_width.clamp(list_min, list_max);
        self.list_pane_width = effective_list_width;

        let active_list_width = if layout_mode == WorkstationLayoutMode::Horizontal {
            effective_list_width
        } else {
            content_width
        };

        let (
            destination,
            is_searching,
            selected_key,
            view_mode,
            active_tab,
            search_query,
            reduce_motion,
            inspector_tab_epoch,
        ) = {
            let s = self.session.read(cx);
            (
                s.destination,
                s.is_searching,
                s.selected_package_key.clone(),
                s.view_mode,
                s.inspector_tab,
                s.search_query.clone(),
                self.reduce_motion,
                s.inspector_tab_epoch,
            )
        };

        let (source_scope, state_filter, sort_mode) = {
            let s = self.session.read(cx);
            (s.source_scope, s.state_filter, s.sort_mode)
        };

        let raw_packages: std::sync::Arc<[UnifiedPackage]> = {
            let store = self.store.read(cx);
            match destination {
                NavDestination::Browse => std::sync::Arc::clone(&store.active_results),
                NavDestination::Installed => std::sync::Arc::clone(&store.installed_packages),
                NavDestination::Updates => std::sync::Arc::clone(&store.updates_packages),
                _ => std::sync::Arc::from([]),
            }
        };

        let packages: std::sync::Arc<[UnifiedPackage]> = {
            if source_scope.is_all()
                && state_filter == PackageStateFilter::All
                && sort_mode == SortMode::Relevance
            {
                raw_packages
            } else {
                let mut filtered: Vec<UnifiedPackage> = raw_packages
                    .iter()
                    .filter(|p| {
                        source_scope.contains_str(&p.source_type)
                            && state_filter.matches(p.is_installed, p.has_update)
                    })
                    .cloned()
                    .collect();
                sort_mode.sort_packages(&mut filtered);
                std::sync::Arc::from(filtered)
            }
        };

        let selected_pkg = selected_key
            .as_ref()
            .and_then(|key| packages.iter().find(|p| &p.key() == key).cloned());

        let entity_mode = entity.clone();
        let entity_select = entity.clone();
        let entity_split = entity.clone();
        let entity_move = entity.clone();
        let entity_up = entity.clone();

        let mode_switcher = {
            let on_mode = entity_mode.clone();
            crate::components::view_mode_switcher::ViewModeSwitcher::render(
                crate::components::view_mode_switcher::ViewModeSwitcherProps {
                    current_mode: view_mode,
                    theme: &theme,
                    on_select_mode: Rc::new(move |mode, _w, cx| {
                        on_mode.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_view_mode(mode, cx);
                            });
                        });
                    }),
                },
            )
        };

        let top_bar = match destination {
            crate::state::NavDestination::Browse => {
                let on_toggle_source = entity.clone();
                let on_state = entity.clone();
                let on_sort = entity.clone();
                let on_mode = entity_mode.clone();
                let on_menu = entity.clone();
                let on_nav_menu = entity.clone();
                let on_retry = entity.clone();
                let on_clear = entity.clone();
                let aur_enabled = self.aur_enabled;
                let flatpak_enabled = self.flatpak_enabled;
                let appimage_enabled = self.appimage_enabled;
                let breakpoint = WorkbenchBreakpoint::from_width(active_list_width);
                let store = self.store.read(cx);

                QueryWorkbench::render(&QueryWorkbenchProps {
                    search_input: self.search_input.clone(),
                    list_pane_width: active_list_width,
                    source_scope,
                    state_filter,
                    sort_mode,
                    view_mode,
                    active_menu_state: self.active_menu,
                    is_searching,
                    total_count: packages.len(),
                    reduce_motion: self.reduce_motion,
                    aur_enabled: self.aur_enabled,
                    flatpak_enabled: self.flatpak_enabled,
                    appimage_enabled: self.appimage_enabled,
                    source_health: &store.source_health,
                    theme: &theme,
                    visual_style: self.visual_style,
                    filters_btn_focus: self.filters_btn_focus.clone(),
                    state_btn_focus: self.state_btn_focus.clone(),
                    sort_btn_focus: self.sort_btn_focus.clone(),
                    menu_surface_focus: self.menu_surface_focus.clone(),
                    on_toggle_source: Rc::new(move |kind, _w, cx| {
                        on_toggle_source.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.toggle_source(kind, cx);
                            });
                        });
                    }),
                    on_select_state: Rc::new(move |state, _w, cx| {
                        on_state.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_state_filter(state, cx);
                            });
                        });
                    }),
                    on_select_sort: Rc::new(move |sort, _w, cx| {
                        on_sort.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_sort_mode(sort, cx);
                            });
                        });
                    }),
                    on_select_view_mode: Rc::new(move |mode, _w, cx| {
                        on_mode.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_view_mode(mode, cx);
                            });
                        });
                    }),
                    on_toggle_menu: Rc::new(move |menu, window, cx| {
                        on_menu.update(cx, |view, cx| {
                            view.toggle_menu(menu, window, cx);
                        });
                    }),
                    on_navigate_menu: Rc::new(move |key, window, cx| {
                        on_nav_menu.update(cx, |view, cx| {
                            view.navigate_menu(key, breakpoint, window, cx);
                        });
                    }),
                    on_retry_source: Rc::new(move |kind, w, cx| {
                        let cb = on_retry.read(cx).on_retry_source.clone();
                        if let Some(cb) = cb {
                            cb(kind, w, cx);
                        }
                    }),
                    on_clear_filters: Rc::new(move |_w, cx| {
                        on_clear.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.clear_filters(aur_enabled, flatpak_enabled, appimage_enabled, cx);
                            });
                        });
                    }),
                })
                .into_any_element()
            }
            crate::state::NavDestination::Installed => div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .bg(theme.bg_surface)
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .text_color(theme.text_primary)
                        .child(
                            svg()
                                .path(crate::icons::AppIcon::Installed.path())
                                .size_4()
                                .text_color(theme.accent),
                        )
                        .child("Locally Installed Packages"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(format!("{} packages", packages.len())),
                        )
                        .child(mode_switcher),
                )
                .into_any_element(),
            crate::state::NavDestination::Updates => {
                let on_upgrade_cb = self.on_upgrade_all.clone();
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .bg(theme.bg_surface)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .font_weight(FontWeight::BOLD)
                            .text_sm()
                            .text_color(theme.text_primary)
                            .child(
                                svg()
                                    .path(crate::icons::AppIcon::Updates.path())
                                    .size_4()
                                    .text_color(theme.accent),
                            )
                            .child("Available System Updates"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(if is_busy {
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme.bg_surface_active)
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_muted)
                                    .child("Upgrade All")
                                    .into_any_element()
                            } else {
                                let focus_border = theme.border_focus;
                                let on_upgrade_key = on_upgrade_cb.clone();
                                div()
                                    .id("upgrade_all_btn")
                                    .focusable()
                                    .tab_stop(true)
                                    .focus(move |s| s.border_1().border_color(focus_border))
                                    .on_key_down(move |event, window, cx| {
                                        let key = event.keystroke.key.as_str();
                                        if key == "enter" || key == "space" {
                                            if let Some(ref cb) = on_upgrade_key {
                                                cb(window, cx);
                                            }
                                        }
                                    })
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme.accent)
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.bg_app)
                                    .cursor_pointer()
                                    .hover({
                                        let h = theme.accent_hover;
                                        move |s| s.bg(h)
                                    })
                                    .child("Upgrade All")
                                    .on_mouse_down(MouseButton::Left, move |_e, w, cx| {
                                        if let Some(ref cb) = on_upgrade_cb {
                                            cb(w, cx);
                                        }
                                    })
                                    .into_any_element()
                            })
                            .child(mode_switcher),
                    )
                    .into_any_element()
            }
            _ => div().into_any_element(),
        };

        // ── 2. Corps de la liste (Cards vs Table) ───────────────────────────
        let (installed_err, updates_err) = {
            let st = self.store.read(cx);
            (st.installed_error.clone(), st.updates_error.clone())
        };

        let is_browse_empty =
            destination == crate::state::NavDestination::Browse && search_query.trim().is_empty();

        let list_body = if let (NavDestination::Installed, Some(err)) = (destination, installed_err)
        {
            let on_reload_installed_cb = self.on_reload_installed.clone();
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .p_8()
                .gap_4()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.danger)
                        .child(format!("Failed to load installed packages: {err}")),
                )
                .child(
                    div()
                        .id("retry_load_installed_btn")
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.accent)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.bg_app)
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.accent_hover))
                        .child("Retry")
                        .on_mouse_down(MouseButton::Left, move |_ev, w, cx| {
                            if let Some(ref cb) = on_reload_installed_cb {
                                cb(w, cx);
                            }
                        }),
                )
                .into_any_element()
        } else if let (NavDestination::Updates, Some(err)) = (destination, updates_err) {
            let on_reload_updates_cb = self.on_reload_updates.clone();
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .p_8()
                .gap_4()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.danger)
                        .child(format!("Failed to check for updates: {err}")),
                )
                .child(
                    div()
                        .id("retry_load_updates_btn")
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.accent)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.bg_app)
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.accent_hover))
                        .child("Retry")
                        .on_mouse_down(MouseButton::Left, move |_ev, w, cx| {
                            if let Some(ref cb) = on_reload_updates_cb {
                                cb(w, cx);
                            }
                        }),
                )
                .into_any_element()
        } else if is_browse_empty {
            div()
                .flex_1()
                .child(UnifiedSearch::render_empty_discovery(&theme))
                .into_any_element()
        } else if packages.is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .p_8()
                .text_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text_muted)
                        .child(if is_searching {
                            "Searching..."
                        } else {
                            "No matching packages found."
                        }),
                )
                .into_any_element()
        } else {
            let package_count = packages.len();
            let selected_name = selected_pkg.as_ref().map(|p| p.name.clone());
            let list_theme = theme;

            if view_mode == PackageViewMode::Table {
                let row_h = if self.compact {
                    UiMetrics::ROW_HEIGHT_COMPACT
                } else {
                    UiMetrics::ROW_HEIGHT_NORMAL
                };
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(PackageTable::render_header(&theme))
                    .child(
                        div().flex_1().h_full().overflow_hidden().child(
                            uniform_list("package-list-table", package_count, {
                                let packages = packages.clone();
                                let selected_name = selected_name.clone();
                                let is_compact = self.compact;

                                move |range, _window, _cx| {
                                    range
                                        .map(|idx| {
                                            let pkg = &packages[idx];
                                            let is_selected = selected_name
                                                .as_ref()
                                                .map(|n| n == &pkg.name)
                                                .unwrap_or(false);
                                            let pkg_key = pkg.key();
                                            let on_select = entity_select.clone();

                                            div()
                                                .h(px(row_h))
                                                .child(PackageTable::render_row(
                                                    pkg,
                                                    is_selected,
                                                    &list_theme,
                                                    is_compact,
                                                ))
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    move |_e, _w, cx| {
                                                        let key = pkg_key.clone();
                                                        on_select.update(cx, |view, cx| {
                                                            view.session.update(cx, |s, cx| {
                                                                s.select_package(Some(key), cx);
                                                            });
                                                        });
                                                    },
                                                )
                                        })
                                        .collect()
                                }
                            })
                            .h_full()
                            .track_scroll(self.scroll_handle.clone()),
                        ),
                    )
                    .into_any_element()
            } else {
                let card_wrapper_h = if self.compact {
                    UiMetrics::CARD_WRAPPER_COMPACT
                } else {
                    UiMetrics::CARD_WRAPPER_NORMAL
                };
                let is_compact = self.compact;
                let visual_style = self.visual_style;
                uniform_list("package-list-surface", package_count, {
                    let packages = packages.clone();
                    let selected_name = selected_name.clone();

                    move |range, _window, _cx| {
                        range
                            .map(|idx| {
                                let pkg = &packages[idx];
                                let is_selected = selected_name
                                    .as_ref()
                                    .map(|n| n == &pkg.name)
                                    .unwrap_or(false);
                                let pkg_key = pkg.key();
                                let on_select = entity_select.clone();

                                div()
                                    .h(px(card_wrapper_h))
                                    .px_3()
                                    .py_1()
                                    .child(PackageCard::render(PackageCardProps {
                                        package: pkg,
                                        is_selected,
                                        theme: &list_theme,
                                        compact: is_compact,
                                        visual_style,
                                    }))
                                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                                        let key = pkg_key.clone();
                                        on_select.update(cx, |view, cx| {
                                            view.session.update(cx, |s, cx| {
                                                s.select_package(Some(key), cx);
                                            });
                                        });
                                    })
                            })
                            .collect()
                    }
                })
                .h_full()
                .track_scroll(self.scroll_handle.clone())
                .into_any_element()
            }
        };

        let list_pane = if layout_mode == WorkstationLayoutMode::Horizontal {
            div()
                .flex()
                .flex_col()
                .w(px(effective_list_width))
                .h_full()
                .border_r_1()
                .border_color(theme.border)
                .child(top_bar)
                .child(div().flex_1().overflow_hidden().child(list_body))
        } else {
            div()
                .flex()
                .flex_col()
                .w_full()
                .flex_1()
                .border_b_1()
                .border_color(theme.border)
                .child(top_bar)
                .child(div().flex_1().overflow_hidden().child(list_body))
        };

        // ── 3. Splitter draggable isolé ─────────────────────────────────────
        let is_resizing = self.drag_state.is_some();
        let splitter = div()
            .id("workstation_splitter")
            .w(px(5.0))
            .h_full()
            .bg(if is_resizing {
                theme.accent
            } else {
                theme.border
            })
            .cursor_col_resize()
            .hover({
                let h = theme.accent;
                move |s| s.bg(h)
            })
            .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                let pointer_x = event.position.x.to_f64() as f32;
                entity_split.update(cx, |view, cx| {
                    view.drag_state = Some(SplitterDragState {
                        start_width: view.list_pane_width,
                        start_pointer_x: pointer_x,
                    });
                    cx.notify();
                });
            });

        // ── 4. Panneau d'inspection sémantique ──────────────────────────────
        let alpm_details = selected_key
            .as_ref()
            .and_then(|k| self.store.read(cx).get_cached_details(k).cloned());
        let detail_error = selected_key
            .as_ref()
            .and_then(|k| self.store.read(cx).get_detail_error(k).cloned());

        let cached_pkgbuild = selected_pkg.as_ref().and_then(|p| {
            if p.source_type == "AUR" {
                self.store.read(cx).get_cached_pkgbuild(&p.name)
            } else {
                None
            }
        });

        let on_retry_details = if detail_error.is_some() {
            if let Some(key) = selected_key.clone() {
                self.on_retry_details.clone().map(|cb| {
                    Rc::new(move |w: &mut Window, cx: &mut App| {
                        cb(&key, w, cx);
                    }) as Rc<dyn Fn(&mut Window, &mut App)>
                })
            } else {
                None
            }
        } else {
            None
        };

        let selected_pkg_clone = selected_pkg.clone();
        let on_mutation_cb = self.on_mutation.clone();
        let on_mutation_cb_rm = self.on_mutation.clone();
        let entity_tab = entity.clone();
        let entity_nav_dep = entity.clone();
        let tc_entity = self.toast_center.clone();

        let inspector_element = PackageInspectorView::render_with_motion(
            PackageInspectorProps {
                package: selected_pkg.as_ref(),
                alpm_details: alpm_details.as_ref(),
                detail_error: detail_error.as_deref(),
                pkgbuild: cached_pkgbuild,
                is_loading_pkgbuild: self.is_loading_pkgbuild,
                active_tab,
                theme: &theme,
                is_busy,
                copy_feedback: self.copy_cmd_feedback,
                on_select_tab: Rc::new(move |tab, _w, cx| {
                    entity_tab.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.set_inspector_tab(tab, cx);
                        });
                    });
                }),
                on_install: selected_pkg_clone.as_ref().and_then(|p| {
                    on_mutation_cb.as_ref().map(|cb| {
                        let p_c = p.clone();
                        let cb_c = cb.clone();
                        Rc::new(move |window: &mut Window, cx: &mut App| {
                            cb_c(&p_c, true, window, cx);
                        }) as Rc<dyn Fn(&mut Window, &mut App)>
                    })
                }),
                on_remove: selected_pkg_clone.as_ref().and_then(|p| {
                    on_mutation_cb_rm.as_ref().map(|cb| {
                        let p_c = p.clone();
                        let cb_c = cb.clone();
                        Rc::new(move |window: &mut Window, cx: &mut App| {
                            cb_c(&p_c, false, window, cx);
                        }) as Rc<dyn Fn(&mut Window, &mut App)>
                    })
                }),
                on_copy_install_cmd: selected_pkg_clone.as_ref().and_then(|p| {
                    canonical_install_command(p).map(|cmd| {
                        let tc = tc_entity.clone();
                        let rm = reduce_motion;
                        Rc::new(move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(cmd.clone()));
                            tc.update(cx, |center, cx| {
                                center.post(
                                    ToastKind::Success,
                                    "Command copied",
                                    cmd.clone(),
                                    None,
                                    rm,
                                    cx,
                                );
                            });
                        }) as Rc<dyn Fn(&mut Window, &mut App)>
                    })
                }),
                on_copy_pkgbuild: cached_pkgbuild.cloned().map(|content| {
                    let tc = tc_entity.clone();
                    let rm = reduce_motion;
                    Rc::new(move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(content.clone()));
                        tc.update(cx, |center, cx| {
                            center.post(
                                ToastKind::Success,
                                "PKGBUILD copied",
                                "The PKGBUILD recipe has been copied to your clipboard.",
                                None,
                                rm,
                                cx,
                            );
                        });
                    }) as Rc<dyn Fn(&mut Window, &mut App)>
                }),
                on_navigate_package: Some(Rc::new(move |pkg_name, _w, cx| {
                    entity_nav_dep.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.set_destination(crate::state::NavDestination::Browse, cx);
                            s.set_source_scope(SourceScope::all(), cx);
                            s.set_state_filter(PackageStateFilter::All, cx);
                            s.set_search_query(pkg_name.clone(), cx);
                            s.select_package(None, cx);
                        });
                    });
                })),
                on_retry_details,
            },
            reduce_motion,
            inspector_tab_epoch,
        );

        let details_pane = if layout_mode == WorkstationLayoutMode::Horizontal {
            div()
                .flex_1()
                .h_full()
                .overflow_hidden()
                .child(inspector_element)
        } else {
            div()
                .w_full()
                .h(px(280.0))
                .overflow_hidden()
                .child(inspector_element)
        };

        // ── 5. Conteneur racine de la station de travail absorbant les moves ──
        if layout_mode == WorkstationLayoutMode::Horizontal {
            div()
                .id("package_workstation_root")
                .flex()
                .flex_row()
                .size_full()
                .overflow_hidden()
                .on_mouse_move(move |event, window, cx| {
                    let window_width = window.window_bounds().get_bounds().size.width / px(1.0);
                    let sidebar_width = if sidebar_collapsed {
                        UiMetrics::SIDEBAR_COLLAPSED
                    } else {
                        UiMetrics::SIDEBAR_EXPANDED
                    };
                    let content_width = (window_width - sidebar_width).max(0.0);
                    entity_move.update(cx, |view, cx| {
                        view.on_pointer_move(event, content_width, cx);
                    });
                })
                .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                    entity_up.update(cx, |view, cx| {
                        view.on_pointer_up(cx);
                    });
                })
                .child(list_pane)
                .child(splitter)
                .child(details_pane)
        } else {
            div()
                .id("package_workstation_root")
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                    entity_up.update(cx, |view, cx| {
                        view.on_pointer_up(cx);
                    });
                })
                .child(list_pane)
                .child(details_pane)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_inspector_ratio_pure_math_and_bounds() {
        // 1. Boundary at HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH (~746.67px)
        let min_w = UiMetrics::HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH;
        let (list_min_boundary, list_max_boundary) = compute_splitter_bounds(min_w);
        assert!((list_min_boundary - list_max_boundary).abs() < 0.01);
        let inspector_at_boundary = min_w - UiMetrics::SPLITTER_WIDTH - list_min_boundary;
        assert!((inspector_at_boundary - UiMetrics::INSPECTOR_MIN_USABLE).abs() < 0.01);
        assert!(inspector_at_boundary <= min_w * UiMetrics::INSPECTOR_MAX_RATIO + 0.01);

        // 2. Reference 900px viewport
        let (list_min_900, list_max_900) = compute_splitter_bounds(900.0);
        let inspector_at_min_list = 900.0 - UiMetrics::SPLITTER_WIDTH - list_min_900;
        let inspector_at_max_list = 900.0 - UiMetrics::SPLITTER_WIDTH - list_max_900;
        assert!(inspector_at_min_list <= 900.0 * UiMetrics::INSPECTOR_MAX_RATIO + 0.01);
        assert!(inspector_at_max_list >= UiMetrics::INSPECTOR_MIN_USABLE - 0.01);

        // 3. Reference 1280px viewport (cap at 520px)
        let (list_min_1280, list_max_1280) = compute_splitter_bounds(1280.0);
        let inspector_max_1280 = 1280.0 - UiMetrics::SPLITTER_WIDTH - list_min_1280;
        let inspector_min_1280 = 1280.0 - UiMetrics::SPLITTER_WIDTH - list_max_1280;
        assert_eq!(inspector_max_1280, UiMetrics::INSPECTOR_MAX_WIDTH);
        assert_eq!(inspector_min_1280, UiMetrics::INSPECTOR_MIN_USABLE);
        assert_eq!(list_min_1280, 755.0);
        assert_eq!(list_max_1280, 955.0);

        // 4. Large 1600px viewport (cap at 520px)
        let (list_min_1600, list_max_1600) = compute_splitter_bounds(1600.0);
        let inspector_max_1600 = 1600.0 - UiMetrics::SPLITTER_WIDTH - list_min_1600;
        let inspector_min_1600 = 1600.0 - UiMetrics::SPLITTER_WIDTH - list_max_1600;
        assert_eq!(inspector_max_1600, UiMetrics::INSPECTOR_MAX_WIDTH);
        assert_eq!(inspector_min_1600, UiMetrics::INSPECTOR_MIN_USABLE);
        assert_eq!(list_min_1600, 1075.0);
        assert_eq!(list_max_1600, 1275.0);
    }

    #[test]
    fn test_layout_mode_adaptation_at_boundary() {
        assert_eq!(
            WorkstationLayoutMode::from_content_width(700.0),
            WorkstationLayoutMode::Stacked
        );
        assert_eq!(
            WorkstationLayoutMode::from_content_width(746.0),
            WorkstationLayoutMode::Stacked
        );
        assert_eq!(
            WorkstationLayoutMode::from_content_width(
                UiMetrics::HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH
            ),
            WorkstationLayoutMode::Horizontal
        );
        assert_eq!(
            WorkstationLayoutMode::from_content_width(800.0),
            WorkstationLayoutMode::Horizontal
        );
        assert_eq!(
            WorkstationLayoutMode::from_content_width(1280.0),
            WorkstationLayoutMode::Horizontal
        );
    }

    #[test]
    fn test_splitter_delta_forward() {
        // At 1280px content_width, bounds are [755.0, 955.0]
        let width = compute_splitter_width(800.0, 650.0, 700.0, 1280.0);
        assert_eq!(width, 850.0);
    }

    #[test]
    fn test_splitter_delta_reverse() {
        // At 1280px content_width, bounds are [755.0, 955.0]
        let width = compute_splitter_width(800.0, 650.0, 620.0, 1280.0);
        assert_eq!(width, 770.0);
    }

    #[test]
    fn test_splitter_clamp_minimum() {
        // At 1280px, minimum list width is 755.0 so inspector does not exceed 520px
        let width = compute_splitter_width(800.0, 650.0, 100.0, 1280.0);
        assert_eq!(
            width, 755.0,
            "Must clamp to 755 minimum at 1280px content width"
        );
    }

    #[test]
    fn test_splitter_clamp_maximum() {
        // At 1280px, maximum list width is 955.0 so inspector maintains at least 320px
        let width = compute_splitter_width(800.0, 650.0, 1200.0, 1280.0);
        assert_eq!(
            width, 955.0,
            "Must clamp to 955 maximum at 1280px content width"
        );
    }

    #[test]
    fn test_dual_usable_splitter_clamp_invariants() {
        let (_list_min, list_max) = compute_splitter_bounds(900.0);
        let width = compute_splitter_width(520.0, 650.0, 1000.0, 900.0);
        assert_eq!(width, list_max);
        assert!(width >= UiMetrics::LIST_MIN_USABLE);
        let inspector_remains = 900.0 - UiMetrics::SPLITTER_WIDTH - width;
        assert!(inspector_remains >= UiMetrics::INSPECTOR_MIN_USABLE);
        assert!(inspector_remains <= 900.0 * UiMetrics::INSPECTOR_MAX_RATIO + 0.01);
    }

    #[test]
    fn test_splitter_sidebar_offset_independence() {
        // Sidebar collapsed (56px) vs expanded (190px)
        let collapsed_start_x = 56.0 + 800.0;
        let collapsed_current_x = 56.0 + 850.0;
        let width_collapsed =
            compute_splitter_width(800.0, collapsed_start_x, collapsed_current_x, 1280.0);

        let expanded_start_x = 190.0 + 800.0;
        let expanded_current_x = 190.0 + 850.0;
        let width_expanded =
            compute_splitter_width(800.0, expanded_start_x, expanded_current_x, 1280.0);

        assert_eq!(width_collapsed, 850.0);
        assert_eq!(width_expanded, 850.0);
        assert_eq!(
            width_collapsed, width_expanded,
            "Splitter width calculation must be completely independent of sidebar width"
        );
    }

    #[test]
    fn test_splitter_guarantees_inspector_bounds_across_reference_viewports() {
        for content_w in [750.0, 900.0, 1024.0, 1280.0, 1600.0, 1920.0] {
            let (list_min, list_max) = compute_splitter_bounds(content_w);
            let insp_at_min = content_w - UiMetrics::SPLITTER_WIDTH - list_min;
            let insp_at_max = content_w - UiMetrics::SPLITTER_WIDTH - list_max;

            // Inspector must never exceed min(520, content_w * 3/7)
            let max_allowed =
                (content_w * UiMetrics::INSPECTOR_MAX_RATIO).min(UiMetrics::INSPECTOR_MAX_WIDTH);
            assert!(
                insp_at_min <= max_allowed + 0.01,
                "Inspector at min list ({insp_at_min}) must not exceed {max_allowed} at content_w={content_w}"
            );

            // Inspector must never drop below 320
            assert!(
                insp_at_max >= UiMetrics::INSPECTOR_MIN_USABLE - 0.01,
                "Inspector at max list ({insp_at_max}) must be at least 320 at content_w={content_w}"
            );
        }
    }
}
