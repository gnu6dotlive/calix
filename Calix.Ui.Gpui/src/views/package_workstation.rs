use crate::backend::models::UnifiedPackage;
use crate::components::menu::MenuLifecycle;
use crate::components::package_card::{PackageCard, PackageCardProps};
use crate::components::package_table::PackageTable;
use crate::components::query_workbench::{
    ActiveMenuState, QueryWorkbench, QueryWorkbenchProps, ScopeAction, WorkbenchMenu,
};
use crate::components::scope_dial::ScopeDialView;
use crate::components::search_input::SearchInputView;
use crate::components::source_bank::{SourceBank, SourceBankProps};
use crate::components::unified_search::UnifiedSearch;
use crate::state::console::{ConsoleEvent, ConsoleModel};
use crate::state::query::{PackageStateFilter, SortMode, SourceScope, WorkbenchBreakpoint};
use crate::state::{
    also_in, canonical_install_command, source_axis, AlsoIn, AppSession, NavScope, PackageKey,
    PackageSourceKind, PackageStore, PackageViewMode, SessionEvent, ToastCenter, ToastKind,
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

/// The drag of the nameplate's edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NameplateDragState {
    pub start_height: f32,
    pub start_pointer_y: f32,
}

/// The nameplate's bounds: it never swallows the readout, never disappears.
///
/// The plate is read from the top down in full-width bands. The nameplate is
/// the caption of the readout: below it, always, between a floor and a share.
pub fn nameplate_bounds(content_height: f32) -> (f32, f32) {
    let min = UiMetrics::NAMEPLATE_MIN_HEIGHT;
    let max = (content_height * UiMetrics::NAMEPLATE_MAX_RATIO).max(min);
    (min, max)
}

/// The height of the nameplate, bounded.
pub fn nameplate_height(content_height: f32, requested: f32) -> f32 {
    let (min, max) = nameplate_bounds(content_height);
    requested.clamp(min, max)
}

/// The height of the nameplate after a drag: pulling up makes it grow.
pub fn nameplate_height_after_drag(
    drag_start_height: f32,
    drag_start_pointer_y: f32,
    current_pointer_y: f32,
    content_height: f32,
) -> f32 {
    let delta_y = current_pointer_y - drag_start_pointer_y;
    nameplate_height(content_height, drag_start_height - delta_y)
}

/// Calcul pur et robuste de la largeur du panneau dérivé des deltas de pointeur
/// Garantit les invariants stricts du contrat UX :
/// Bornes légales strictes de la largeur de liste dans le domaine horizontal
/// La largeur de la zone de contenu : la largeur *mappée* de la fenêtre, moins le rail.
///
/// La largeur que l'application a *demandée* (`window.window_bounds()`) n'est pas
/// celle que le compositeur lui accorde : un compositeur à tuiles redimensionne la
/// fenêtre quand le focus change, et la demande reste celle du lancement. Mesurer
/// sur la demande faisait tenir l'inspecteur à $137px$ — sous son plancher de
/// $320px$ — dans une fenêtre de $1056px$, puis à $1622px$ — au-dessus de son
/// plafond de $520px$ — dans une fenêtre de $2542px$. C'est la taille mappée,
/// `viewport_size()`, qui fait foi.
pub fn content_width(viewport_width: f32, sidebar_width: f32) -> f32 {
    (viewport_width - sidebar_width).max(0.0)
}

/// How far the wheel's distance may wait before the list is given it.
pub const MAX_SCROLL_PENDING: f32 = 240.0;

/// The distance that waits after a wheel notch.
///
/// The distance is bounded: a long scroll must not leave the list behind by a
/// mile, and the list must never be asked for more than it can give.
pub fn scroll_pending_after(pending: f32, delta: f32) -> f32 {
    (pending + delta).clamp(-MAX_SCROLL_PENDING, MAX_SCROLL_PENDING)
}

/// The wheel, taken back from the list.
///
/// The list applies a notch in one jump. We take it back: the distance waits,
/// to be handed over eased, frame by frame.
///
/// The two cancel out whichever order the handlers run in — the list adds the
/// distance and we take it back, or we take it back and the list adds it. The
/// list is where it was, and the distance waits.
pub fn take_wheel_back(offset: &mut Point<Pixels>, delta: f32) {
    offset.y -= px(delta);
}

/// The frame's movement: the part of the waiting distance given back now.
pub fn apply_scroll_step(offset: &mut Point<Pixels>, step: f32) {
    offset.y += px(step);
}

/// The eased step of the smooth scroll, and what is left of the distance.
///
/// A frame gives back a fifth of what waits. The notch is therefore a movement
/// spread over a dozen frames rather than a jump, and the last frame gives the
/// remainder back whole, so the list lands exactly where the wheel asked.
pub fn scroll_step(pending: f32) -> (f32, f32) {
    const SHARE: f32 = 0.22;
    let step = pending * SHARE;
    let rest = pending - step;
    if rest.abs() < 0.5 {
        (pending, 0.0)
    } else {
        (step, rest)
    }
}

pub struct PackageWorkstationView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
    pub console: Entity<ConsoleModel>,
    pub toast_center: Entity<ToastCenter>,
    /// The height of the nameplate: the caption of the readout, under it.
    pub nameplate_height: f32,
    pub nameplate_drag: Option<NameplateDragState>,
    pub scroll_handle: UniformListScrollHandle,
    /// The distance the wheel asked for and the list has not been given yet.
    ///
    /// The list applies a wheel notch in one jump. We take the wheel back,
    /// keep the distance here, and give it back eased, frame by frame: a notch
    /// becomes a movement, not a step.
    pub scroll_pending: f32,
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
    /// The dial of the scope: one object with three positions.
    pub scope_dial: Entity<ScopeDialView>,
    /// The dial's ear to the session: the needle follows the scope.
    _scope_dial_sub: Subscription,
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

        // Le cadran du périmètre. Il est une entité à part entière : l'aiguille
        // a son propre état, et elle se jette sur son cran quand le
        // périmètre change, d'où qu'il vienne.
        let dial_entity = cx.entity();
        let initial_scope = session.read(cx).scope;
        let scope_dial = cx.new(|_cx| {
            ScopeDialView::new(
                initial_scope,
                config.theme,
                config.reduce_motion,
                Rc::new(move |scope, _window, cx| {
                    dial_entity.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.set_scope(scope, cx);
                        });
                    });
                }),
            )
        });

        // Le cadran suit le périmètre, d'où qu'il vienne.
        let dial = scope_dial.clone();
        let dial_subscription = cx.subscribe(&session, move |_this, _emitter, event, cx| {
            if let SessionEvent::ScopeChanged(scope) = event {
                dial.update(cx, |view, cx| {
                    view.set_scope(*scope, cx);
                });
            }
        });

        Self {
            session,
            store,
            console,
            toast_center,
            nameplate_height: 260.0,
            nameplate_drag: None,
            scroll_handle: UniformListScrollHandle::new(),
            scroll_pending: 0.0,
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
            scope_dial,
            _scope_dial_sub: dial_subscription,
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
        content_height: f32,
        cx: &mut Context<Self>,
    ) {
        if let Some(drag) = self.nameplate_drag {
            let current_y = event.position.y.to_f64() as f32;
            let new_height = nameplate_height_after_drag(
                drag.start_height,
                drag.start_pointer_y,
                current_y,
                content_height,
            );
            if (new_height - self.nameplate_height).abs() >= 1.0 {
                self.nameplate_height = new_height;
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
        if self.nameplate_drag.is_some() {
            self.nameplate_drag = None;
            cx.notify();
        }
    }
}

impl Render for PackageWorkstationView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let entity = cx.entity().clone();
        let is_busy = self.console.read(cx).is_running();

        // The smooth scroll gives back what waits, a fifth at a time, until
        // nothing waits any more. Each frame is one step, and the view asks
        // for the next one.
        if self.scroll_pending.abs() > 0.0 {
            let (step, rest) = scroll_step(self.scroll_pending);
            self.scroll_pending = rest;
            let base = self.scroll_handle.0.borrow().base_handle.clone();
            let mut offset = base.offset();
            apply_scroll_step(&mut offset, step);
            base.set_offset(offset);
            // The view asks for the next frame itself: a window animation frame
            // would notify the enclosing view instead of this one, and panics
            // when the view is laid out on its own.
            cx.notify();
        }

        let sidebar_collapsed = self.session.read(cx).sidebar_collapsed;
        let sidebar_width = if sidebar_collapsed {
            UiMetrics::SIDEBAR_COLLAPSED
        } else {
            UiMetrics::SIDEBAR_EXPANDED
        };
        let window_width = window.viewport_size().width / px(1.0);
        let content_width = content_width(window_width, sidebar_width);
        let window_height = window.viewport_size().height / px(1.0);

        // The plate is read from the top down: the band of the header, the
        // bank, the readout, the nameplate. The content's height is what the
        // nameplate is bounded against.
        let content_height = (window_height - UiMetrics::NAMEPLATE_MIN_HEIGHT).max(0.0);
        let effective_nameplate_height = nameplate_height(content_height, self.nameplate_height);
        self.nameplate_height = effective_nameplate_height;

        let active_list_width = content_width;

        let (
            scope,
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
                s.scope,
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
            match scope {
                NavScope::Results => std::sync::Arc::clone(&store.active_results),
                NavScope::Installed => std::sync::Arc::clone(&store.installed_packages),
                NavScope::Updates => std::sync::Arc::clone(&store.updates_packages),
            }
        };

        // La requête réduit le périmètre courant. Dans Résultats, les lignes
        // sont déjà celles de la recherche ; dans Installés et Mises à jour, les
        // lignes sont là et la requête les réduit.
        let query_narrows = scope != NavScope::Results && !search_query.trim().is_empty();

        let packages: std::sync::Arc<[UnifiedPackage]> = {
            if source_scope.is_all()
                && state_filter == PackageStateFilter::All
                && sort_mode == SortMode::Relevance
                && !query_narrows
            {
                raw_packages
            } else {
                let mut filtered = crate::state::query::visible_packages(
                    &raw_packages,
                    &source_scope,
                    state_filter,
                    sort_mode,
                );
                if query_narrows {
                    filtered.retain(|package| {
                        crate::state::query::matches_query(package, &search_query)
                    });
                }
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


        // The query surface is in every scope: the filters act on every
        // scope, so the controls that change them are everywhere. The scope
        // decides what the controls offer, not whether they are there.
        let on_toggle_source = entity.clone();
        let on_state = entity.clone();
        let on_sort = entity.clone();
        let on_mode = entity_mode.clone();
        let on_menu = entity.clone();
        let on_nav_menu = entity.clone();
        let on_retry = entity.clone();
        let aur_enabled = self.aur_enabled;
        let flatpak_enabled = self.flatpak_enabled;
        let appimage_enabled = self.appimage_enabled;
        let breakpoint = WorkbenchBreakpoint::from_width(active_list_width);
        let store = self.store.read(cx);

        // L'axe des sources : ce que chaque source tient, et combien de
        // lignes elle porte dans le périmètre courant.
        let source_axis = source_axis(scope, &packages, &source_scope);

        // La banque : sept ouvertures, une par source. Les sources sont un
        // axe, pas un filtre : la banque est toujours là.
        let source_bank = SourceBank::render(SourceBankProps {
            axis: &source_axis,
            source_health: &store.source_health,
            theme: &theme,
            epoch: scope.index(),
            reduce_motion: self.reduce_motion,
            on_toggle: {
                let entity = on_toggle_source.clone();
                Rc::new(move |kind, _w, cx| {
                    entity.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.toggle_source(kind, cx);
                        });
                    });
                })
            },
            on_retry: {
                let entity = on_retry.clone();
                Rc::new(move |kind, w, cx| {
                    let cb = entity.read(cx).on_retry_source.clone();
                    if let Some(cb) = cb {
                        cb(kind, w, cx);
                    }
                })
            },
        });

        let top_bar = QueryWorkbench::render(&QueryWorkbenchProps {
            search_input: self.search_input.clone(),
            scope_dial: self.scope_dial.clone(),
            list_pane_width: active_list_width,
            scope,
            source_scope,
            state_filter,
            sort_mode,
            view_mode,
            active_menu_state: self.active_menu,
            reduce_motion: self.reduce_motion,
            aur_enabled,
            flatpak_enabled,
            appimage_enabled,
            source_health: &store.source_health,
            source_axis: &source_axis,
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
            scope_action: match scope {
                NavScope::Updates => self.on_upgrade_all.clone().map(|handler| ScopeAction {
                    label: "Upgrade all",
                    handler,
                }),
                _ => None,
            },
        })
        .into_any_element();

        // ── 2. Corps de la liste (Cards vs Table) ───────────────────────────
        let (installed_err, updates_err) = {
            let st = self.store.read(cx);
            (st.installed_error.clone(), st.updates_error.clone())
        };

        let is_browse_empty =
            scope == NavScope::Results && search_query.trim().is_empty();

        let list_body = if let (NavScope::Installed, Some(err)) = (scope, installed_err)
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
        } else if let (NavScope::Updates, Some(err)) = (scope, updates_err) {
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
                    .child(PackageTable::render_header(
                        &theme,
                        PackageTable::scale_max(&packages),
                    ))
                    .child(
                        div().flex_1().h_full().overflow_hidden().child(
                            uniform_list("package-list-table", package_count, {
                                let packages = packages.clone();
                                let selected_name = selected_name.clone();
                                // The scale is shared: the bars of the readout
                                // are read against each other.
                                let scale_max = PackageTable::scale_max(&packages);
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
                                                    scale_max,
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

        // The wheel belongs to the workbench, not to the list. The list would
        // apply a notch in one jump; we take the wheel before it does, and the
        // distance waits to be given back eased, frame by frame.
        let list_body = {
            let entity = cx.entity().clone();
            let scroll_base = self.scroll_handle.0.borrow().base_handle.clone();
            div()
                .flex_1()
                .overflow_hidden()
                .on_scroll_wheel(move |event, window, cx| {
                    let delta: f32 = event.delta.pixel_delta(window.line_height()).y.into();
                    if delta == 0.0 {
                        return;
                    }
                    // The list has already taken this notch in one jump: give it
                    // back, and keep the distance here. It is handed over eased,
                    // a fifth at a time, frame by frame.
                    let mut offset = scroll_base.offset();
                    take_wheel_back(&mut offset, delta);
                    scroll_base.set_offset(offset);
                    entity.update(cx, |view, cx| {
                        view.scroll_pending = scroll_pending_after(view.scroll_pending, delta);
                        cx.notify();
                    });
                })
                .child(list_body)
        };

        // The readout: the band where the work happens, in full width.
        let readout = div()
            .flex()
            .flex_col()
            .w_full()
            .flex_1()
            .overflow_hidden()
            .child(top_bar)
            .child(list_body);

        // ── 3. The nameplate's edge, drawn between the bands ────────────────
        let is_resizing = self.nameplate_drag.is_some();
        let splitter = div()
            .id("workstation_nameplate_edge")
            .w_full()
            .h(px(UiMetrics::SPLITTER_THICKNESS))
            .flex_shrink_0()
            .bg(if is_resizing {
                theme.accent
            } else {
                theme.border
            })
            .cursor_row_resize()
            .hover({
                let h = theme.accent;
                move |s| s.bg(h)
            })
            .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                let pointer_y = event.position.y.to_f64() as f32;
                entity_split.update(cx, |view, cx| {
                    view.nameplate_drag = Some(NameplateDragState {
                        start_height: view.nameplate_height,
                        start_pointer_y: pointer_y,
                    });
                    cx.notify();
                });
            });

        // ── 4. Panneau d'inspection sémantique ──────────────────────────────
        let alpm_details = selected_key
            .as_ref()
            .and_then(|k| self.store.read(cx).get_cached_details(k).cloned());

        // Le même nom, dans les autres sources. La fiche le dit : un nom
        // n'est pas un paquet, et les sources ne sont pas interchangeables.
        let also_in: Vec<AlsoIn> = match selected_pkg.as_ref() {
            Some(package) => {
                let store = self.store.read(cx);
                also_in(package, &store.active_results)
            }
            None => Vec::new(),
        };
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
                also_in: &also_in,
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
                            s.set_scope(NavScope::Results, cx);
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

        // The nameplate: the caption of the readout, in full width, under it.
        let nameplate = div()
            .w_full()
            .h(px(effective_nameplate_height))
            .flex_shrink_0()
            .overflow_hidden()
            .child(inspector_element);

        // ── 5. Conteneur racine de la station de travail absorbant les moves ──
        div()
            .id("package_workstation_root")
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .on_mouse_move(move |event, window, cx| {
                let window_height = window.window_bounds().get_bounds().size.height / px(1.0);
                entity_move.update(cx, |view, cx| {
                    view.on_pointer_move(event, window_height, cx);
                });
            })
            .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                entity_up.update(cx, |view, cx| {
                    view.on_pointer_up(cx);
                });
            })
            .child(source_bank)
            .child(readout)
            .child(splitter)
            .child(nameplate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::prelude::StatefulInteractiveElement;
    use gpui::ScrollHandle;

    /// Le défilement fluide, proof : a wheel notch does not make the list jump,
    /// the distance is handed over in steps, and it lands exactly where the
    /// wheel asked.
    ///
    /// The harness cannot lay out the application's own uniform list — its
    /// measured layout never resolves there, and a list of no size does not
    /// scroll — so the proof is made on the same structure the application
    /// uses: an outer container that takes the wheel from the scroller, and
    /// the scroller itself, which is gpui's own scroll machinery.
    #[gpui::test]
    fn the_wheel_moves_the_list_by_steps(cx: &mut gpui::TestAppContext) {
        struct ScrollFixture {
            handle: ScrollHandle,
            pending: f32,
        }

        impl Render for ScrollFixture {
            fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                // The frame: the waiting distance is given back, a fifth at a time.
                if self.pending.abs() > 0.0 {
                    let (step, rest) = scroll_step(self.pending);
                    self.pending = rest;
                    let mut offset = self.handle.offset();
                    apply_scroll_step(&mut offset, step);
                    self.handle.set_offset(offset);
                    cx.notify();
                }

                // The wheel: the container takes it back from the scroller.
                let handle = self.handle.clone();
                let entity = cx.entity();
                div()
                    .size_full()
                    .on_scroll_wheel(move |event, window, cx| {
                        let delta: f32 = event.delta.pixel_delta(window.line_height()).y.into();
                        if delta == 0.0 {
                            return;
                        }
                        let mut offset = handle.offset();
                        take_wheel_back(&mut offset, delta);
                        handle.set_offset(offset);
                        entity.update(cx, |view, cx| {
                            view.pending = scroll_pending_after(view.pending, delta);
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .id("scroll-fixture-inner")
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(&self.handle)
                            .child(div().h(px(2000.)).w_full()),
                    )
            }
        }

        let cx = cx.add_empty_window();
        let fixture = cx.new(|_| ScrollFixture {
            handle: ScrollHandle::new(),
            pending: 0.0,
        });

        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.), px(0.)),
                size(px(800.), px(600.)),
                |_window, _cx| fixture.clone(),
            );
        };
        let offset = |cx: &mut gpui::VisualTestContext| {
            cx.update(|_window, app| {
                let handle = fixture.read(app).handle.clone();
                let offset = handle.offset().y;
                offset
            })
        };

        // The scroller is there, and it has room to travel.
        draw(cx);
        assert_eq!(offset(cx), px(0.), "the list starts at the top");
        let waiting = cx.update(|_window, app| fixture.read(app).pending);
        assert_eq!(waiting, 0.0, "nothing waits before the wheel");

        // The mouse goes over the list, then the wheel turns.
        cx.simulate_event(MouseMoveEvent {
            position: point(px(400.), px(300.)),
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        cx.simulate_event(ScrollWheelEvent {
            position: point(px(400.), px(300.)),
            delta: ScrollDelta::Pixels(point(px(0.), px(-200.))),
            modifiers: Modifiers::default(),
            touch_phase: TouchPhase::default(),
        });

        // The list was played back: it did not jump.
        assert_eq!(
            offset(cx),
            px(0.),
            "the wheel does not make the list jump"
        );

        // One frame: a fifth of the distance.
        draw(cx);
        let first = offset(cx);
        assert!(
            first < px(-30.) && first > px(-60.),
            "the first frame gives a fraction of the distance, not all of it: {first:?}"
        );

        // A few frames: the whole distance, and not a pixel more.
        for _ in 0..40 {
            draw(cx);
        }
        assert!(
            (offset(cx) - px(-200.)).abs() < px(1.),
            "the list lands exactly where the wheel asked: {:?}",
            offset(cx)
        );
    }

    #[test]
    fn the_smooth_scroll_gives_the_distance_back_in_several_frames() {
        // Une molette : la distance revient en une vingtaine d'images, pas en
        // un saut, et la liste finit exactement où la molette l'avait demandé.
        let mut pending = 120.0f32;
        let mut total = 0.0f32;
        let mut frames = 0u32;
        while pending.abs() > 0.0 {
            let (step, rest) = scroll_step(pending);
            total += step;
            pending = rest;
            frames += 1;
            assert!(frames < 60, "le mouvement se termine");
        }
        assert!(frames >= 6, "la distance est rendue en plusieurs images");
        assert!(
            (total - 120.0).abs() < 0.01,
            "toute la distance est rendue"
        );
        assert_eq!(pending, 0.0);
    }

    #[test]
    fn the_pending_distance_is_bounded() {
        // Une longue course ne doit pas laisser la liste en retard d'un mile.
        let pending = scroll_pending_after(0.0, 10_000.0);
        assert_eq!(pending, MAX_SCROLL_PENDING);
        let pending = scroll_pending_after(0.0, -10_000.0);
        assert_eq!(pending, -MAX_SCROLL_PENDING);
        assert_eq!(scroll_pending_after(10.0, 5.0), 15.0);
    }

    #[test]
    fn the_content_width_follows_the_mapped_window() {
        // La largeur mappée moins le rail : c'est elle qui décide du partage
        // entre la liste et l'inspecteur.
        assert_eq!(
            content_width(1056.0, UiMetrics::SIDEBAR_EXPANDED),
            866.0
        );
        assert_eq!(
            content_width(1056.0, UiMetrics::SIDEBAR_COLLAPSED),
            1000.0
        );

        // Un rail plus large que la fenêtre ne donne pas une largeur négative.
        assert_eq!(content_width(100.0, UiMetrics::SIDEBAR_EXPANDED), 0.0);
    }

    #[test]
    fn the_nameplate_stays_inside_its_bounds_at_the_heights_a_tiling_compositor_uses() {
        // Les hauteurs que le compositeur à tuiles a données pendant la
        // mesure : 720px et 1440px. Dans les deux, la plaque de nom reste
        // entre son plancher et sa part — elle n'avale jamais la lecture.
        for window_height in [720.0_f32, 1440.0] {
            let content = (window_height - UiMetrics::NAMEPLATE_MIN_HEIGHT).max(0.0);
            let nameplate = nameplate_height(content, 320.0);

            assert!(
                nameplate >= UiMetrics::NAMEPLATE_MIN_HEIGHT - 0.01,
                "la plaque de nom tombe sous son plancher : {nameplate}px dans une fenêtre de {window_height}px"
            );
            assert!(
                nameplate <= content * UiMetrics::NAMEPLATE_MAX_RATIO + 0.01,
                "la plaque de nom dépasse sa part : {nameplate}px dans une fenêtre de {window_height}px"
            );
            // Et la lecture garde le reste.
            assert!(
                content - nameplate > 0.0,
                "la lecture garde de la place : {content}px de contenu, {nameplate}px de plaque"
            );
        }
    }

    #[test]
    fn the_nameplate_is_pulled_up_and_grows() {
        // On tire vers le haut : la plaque grandit.
        let grown = nameplate_height_after_drag(300.0, 500.0, 450.0, 900.0);
        assert_eq!(grown, 350.0);
        // On tire vers le bas : elle rétrécit.
        let shrunk = nameplate_height_after_drag(300.0, 500.0, 550.0, 900.0);
        assert_eq!(shrunk, 250.0);
    }

    #[test]
    fn the_nameplate_never_swallows_the_readout() {
        // Tirée vers le haut sans fin, elle s'arrête à sa part.
        let content = 900.0;
        let pulled = nameplate_height_after_drag(300.0, 800.0, 0.0, content);
        assert_eq!(pulled, content * UiMetrics::NAMEPLATE_MAX_RATIO);
        // Tirée vers le bas sans fin, elle s'arrête à son plancher.
        let pushed = nameplate_height_after_drag(300.0, 0.0, 2000.0, content);
        assert_eq!(pushed, UiMetrics::NAMEPLATE_MIN_HEIGHT);
    }

    #[test]
    fn the_nameplate_is_bounded_in_a_short_window() {
        // Une fenêtre basse : la part est calculée sur le contenu, pas sur la
        // fenêtre entière, et le plancher tient toujours.
        let content = 200.0;
        let nameplate = nameplate_height(content, 1000.0);
        assert_eq!(nameplate, UiMetrics::NAMEPLATE_MIN_HEIGHT);
        let (min, max) = nameplate_bounds(content);
        assert_eq!(min, UiMetrics::NAMEPLATE_MIN_HEIGHT);
        assert_eq!(max, UiMetrics::NAMEPLATE_MIN_HEIGHT);
    }
}
